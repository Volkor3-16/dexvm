# IMPLEMENTATION PLAN: P1 Tasks

## Overview
Two parallel P1 tasks to unblock the remaining 1,215 extensions (815 failing):
1. **Resolution Errors** (182 extensions) - Missing Android/Kotlin classes
2. **CoroutineScope/Job/Deferred Instance Methods** (448 extensions with NPEs)

---

## P1 Task 1: Resolution Errors (182 extensions)

### Target Classes

| Class | Priority | Extensions | Location |
|-------|----------|------------|----------|
| `kotlinx.random.Random$Default` | HIGH | 40 | `src/vm/class.rs` + `src/vm/native/kotlin/` |
| `androidx.preference.PreferenceManager` | HIGH | 30 | `src/vm/class.rs` + `src/vm/native/android/` |
| `android.os.Build$VERSION` | MEDIUM | 20 | `src/vm/class.rs` + `src/vm/native/android/` |
| `java.time.ZoneOffset` | MEDIUM | 15 | `src/vm/class.rs` + `src/vm/native/java/time/` |
| Cyclic class hierarchy | HIGH | 45 | `src/vm/class.rs` |

---

### Implementation Plan: Resolution Errors

#### Week 1: Kotlin Random & Android Preferences

**Day 1-2: `kotlinx.random.Random$Default`**
```bash
# Files to modify:
# - src/vm/class.rs: Add shim for Lkotlinx/random/Random$Default;
# - src/vm/native/kotlin/random.rs (new file): Implement Random$Default
```

**Shim Definition** (`src/vm/class.rs`):
```rust
shim!(
    "Lkotlinx/random/Random$Default;",
    Some("Ljava/lang/Object;"),
    &[],
    0,
    [sdef!(
        "INSTANCE",
        "Lkotlinx/random/Random$Default;",
        ShimValue::Lazy(native::lazy_random_default)
    ),]
),
```

**Implementation** (`src/vm/native/kotlin/random.rs`):
```rust
pub(crate) fn lazy_random_default(vm: &mut Vm) -> JValue {
    let class = vm.ensure_class_by_desc("Lkotlinx/random/Random$Default;").unwrap();
    let obj = vm.arena.alloc(class, Vec::new(), Some(Native::Opaque));
    JValue::Obj(obj)
}

// Register in kotlin/mod.rs
ne!("Lkotlinx/random/Random$Default;", "INSTANCE", "Lkotlinx/random/Random$Default;", false, lazy_random_default),
```

**Day 3-4: `androidx.preference.PreferenceManager`**
```bash
# Files to modify:
# - src/vm/class.rs: Add shim for Landroidx/preference/PreferenceManager;
# - src/vm/native/android/preference.rs (new file)
```

**Shim Definition**:
```rust
#[cfg(feature = "android")]
shim!(
    "Landroidx/preference/PreferenceManager;",
    Some("Ljava/lang/Object;"),
    &[],
    0,
    [sdef!(
        "INSTANCE",
        "Landroidx/preference/PreferenceManager;",
        ShimValue::Lazy(native::android::preference::lazy_preference_manager)
    ),]
),
```

**Implementation** (`src/vm/native/android/preference.rs`):
```rust
pub(crate) fn lazy_preference_manager(vm: &mut Vm) -> JValue {
    let class = vm.ensure_class_by_desc("Landroidx/preference/PreferenceManager;").unwrap();
    let obj = vm.arena.alloc(class, Vec::new(), Some(Native::Opaque));
    JValue::Obj(obj)
}
```

---

#### Week 2: Android Build & Java Time

**Day 1-2: `android.os.Build$VERSION`**
```rust
shim!(
    "Landroid/os/Build$VERSION;",
    Some("Ljava/lang/Object;"),
    &[],
    0,
    [
        sdef!("SDK_INT", "I", ShimValue::Const(JValue::Int(33))), // API 33
        sdef!("RELEASE", "Ljava/lang/String;", ShimValue::Lazy(lazy_build_version_release)),
        sdef!("CODENAME", "Ljava/lang/String;", ShimValue::Const(JValue::Null)),
    ]
),
```

**Day 3-4: `java.time.ZoneOffset`**
```rust
shim!(
    "Ljava/time/ZoneOffset;",
    Some("Ljava/lang/Object;"),
    &[],
    0,
    [
        sdef!("UTC", "Ljava/time/ZoneOffset;", ShimValue::Lazy(lazy_zone_offset_utc)),
        sdef!("of", "(Ljava/lang/String;)Ljava/time/ZoneOffset;", ShimValue::Native(native::java::time::zone_offset_of)),
        sdef!("ofHours", "(I)Ljava/time/ZoneOffset;", ShimValue::Native(native::java::time::zone_offset_of_hours)),
    ]
),
```

---

#### Week 2-3: Cyclic Class Hierarchy

**Root Cause Analysis**:
```bash
# Find cyclic dependencies
cd /home/volkor/git/dexvm && grep -r "cyclic class hierarchy" src/
```

**Common Patterns**:
1. A → B → A (direct cycle)
2. A → B → C → A (indirect cycle)
3. Self-referencing classes

**Fix Strategy**:
1. Identify all cyclic classes from error logs
2. Add forward declarations in class.rs
4. Use lazy initialization for circular dependencies

---

## P1 Task 2: CoroutineScope/Job/Deferred Instance Methods

### Missing Methods (Week 1-2)

| Class | Missing Methods | Priority |
|-------|----------------|----------|
| `CoroutineScope` | `launch`, `async`, `coroutineScope`, `supervisorScope`, `withContext` | HIGH |
| `Job` | `join`, `cancel`, `children`, `isCancelled`, `isCompleted` | HIGH |
| `Deferred` | `await()` | HIGH |

---

### Implementation Plan: CoroutineScope/Job/Deferred

#### Week 1: CoroutineScope Instance Methods

**File**: `src/vm/native/kotlinx/coroutines.rs`

**Add Instance Methods**:
```rust
// CoroutineScope.launch
pub(crate) fn coroutines_launch(vm: &mut Vm, args: &[JValue]) -> R {
    let scope = args[0];  // receiver
    let block = args[1];
    let _ = vm.invoke_virtual_args(block, "invoke", "()Ljava/lang/Object;", vec![]);
    alloc(vm, "Lkotlinx/coroutines/Job;", Native::Opaque)
}

// CoroutineScope.async
pub(crate) fn coroutines_async(vm: &mut Vm, args: &[JValue]) -> R {
    let scope = args[0];
    let block = args[1];
    let value = vm.invoke_virtual_args(block, "invoke", "()Ljava/lang/Object;", vec![]).unwrap_or(JValue::Null);
    alloc(vm, "Lkotlinx/coroutines/Deferred;", Native::Deferred { value, error: JValue::Null })
}

// CoroutineScope.coroutineScope
pub(crate) fn coroutines_coroutine_scope(vm: &mut Vm, args: &[JValue]) -> R {
    let scope = alloc(vm, "Lkotlinx/coroutines/CoroutineScope;", Native::Opaque)?;
    inv_virt(vm, args[0], "invoke", "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;", &[scope, args[1]])
}

// CoroutineScope.supervisorScope
pub(crate) fn coroutines_supervisor_scope(vm: &mut Vm, args: &[JValue]) -> R {
    let scope = alloc(vm, "Lkotlinx/coroutines/CoroutineScope;", Native::Opaque)?;
    inv_virt(vm, args[0], "invoke", "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;", &[scope, args[1]])
}

// CoroutineScope.withContext
pub(crate) fn coroutines_with_context(vm: &mut Vm, args: &[JValue]) -> R {
    let scope = alloc(vm, "Lkotlinx/coroutines/CoroutineScope;", Native::Opaque)?;
    inv_virt(vm, args[1], "invoke", "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;", &[scope, args[2]])
}
```

**Register in TABLE**:
```rust
ne!("Lkotlinx/coroutines/CoroutineScope;", "launch", "(Lkotlin/jvm/functions/Function2;)Lkotlinx/coroutines/Job;", true, coroutines_launch),
ne!("Lkotlinx/coroutines/CoroutineScope;", "async", "(Lkotlin/jvm/functions/Function2;)Lkotlinx/coroutines/Deferred;", true, coroutines_async),
ne!("Lkotlinx/coroutines/CoroutineScope;", "coroutineScope", "(Lkotlin/jvm/functions/Function2;Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", true, coroutines_coroutine_scope),
ne!("Lkotlinx/coroutines/CoroutineScope;", "supervisorScope", "(Lkotlin/jvm/functions/Function2;Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", true, coroutines_supervisor_scope),
ne!("Lkotlinx/coroutines/CoroutineScope;", "withContext", "(Lkotlin/coroutines/CoroutineContext;Lkotlin/jvm/functions/Function2;Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", true, coroutines_with_context),
```

---

#### Week 2: Job & Deferred Methods

**Job Methods**:
```rust
pub(crate) fn coroutines_job_join(vm: &mut Vm, args: &[JValue]) -> R {
    // Job.join() - for sync VM, job is already complete
    Ok(JValue::Null)
}

pub(crate) fn coroutines_job_cancel(vm: &mut Vm, args: &[JValue]) -> R {
    Ok(JValue::Null)
}

pub(crate) fn coroutines_job_children(vm: &mut Vm, args: &[JValue]) -> R {
    list_alloc(vm, vec![])
}

pub(crate) fn coroutines_job_is_cancelled(vm: &mut Vm, args: &[JValue]) -> R {
    Ok(JValue::Int(0))
}

pub(crate) fn coroutines_job_is_completed(vm: &mut Vm, args: &[JValue]) -> R {
    Ok(JValue::Int(1))
}

pub(crate) fn coroutines_job_cancel(vm: &mut Vm, args: &[JValue]) -> R {
    Ok(JValue::Null)
}

pub(crate) fn coroutines_job_children(vm: &mut Vm, args: &[JValue]) -> R {
    list_alloc(vm, vec![])
}
```

**Deferred.await**:
```rust
pub(crate) fn coroutines_deferred_await(vm: &mut Vm, args: &[JValue]) -> R {
    match payload(vm, args[0]) {
        Some(Native::Deferred { value, .. }) => Ok(*value),
        _ => Err(npe(vm)),
    }
}
```

---

## Implementation Checklist

### Week 1 Checklist
- [ ] Add `CoroutineScope.launch` (instance method)
- [ ] Add `CoroutineScope.async` (instance method)
- [ ] Add `CoroutineScope.coroutineScope` (instance method)
- [ ] Add `CoroutineScope.supervisorScope` (instance method)
- [ ] Add `CoroutineScope.withContext` (instance method)
- [ ] Add `Job.join` (instance method)
- [ ] Add `Job.cancel` (instance method)
- [ ] Add `Job.children` (instance method)
- [ ] Add `Job.isCancelled` (instance method)
- [ ] Add `Job.isCompleted` (instance method)
- [ ] Add `Job.cancel` (instance method)
- [ ] Add `Job.children` (instance method)
- [ ] Add `Deferred.await` (instance method)
- [ ] Register all in TABLE constant

### Week 2 Checklist
- [ ] `kotlinx.random.Random$Default` shim
- [ ] `androidx.preference.PreferenceManager` shim
- [ ] `android.os.Build$VERSION` shim
- [ ] `java.time.ZoneOffset` shim
- [ ] Fix cyclic class hierarchy (45 extensions)

---

## Testing Strategy

### Phase 1: Unit Tests
```bash
cargo test --features keiyoushi
```

### Phase 2: Integration Tests
```bash
# Test specific extensions with coroutine NPEs
DEXVM_LIVE=0 cargo run --features keiyoushi --bin exttest -- --http empty --apk "fixtures/keiyoushi_all/tachiyomi-all.globalcomix-v1.6.0.apk"
```

### Phase 3: Full Regression
```bash
cargo run --features keiyoushi --bin exttest -- --http empty --json report.json
python3 analyze_report.py
```

---

## Success Criteria

| Metric | Current | Target |
|--------|---------|--------|
| Extensions Passing | 599 (42.4%) | >500 (35%+) |
| Success Rate | 84.6% | >95% |
| NPE Errors | 1,617 | <100 |
| Resolution Errors | 1,035 | <50 |
| Coroutine Lock | 358 | <50 |

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Breaking existing tests | Run full test suite after each change |
| Breaking coroutine path | Test with GlobalComix (known coroutine user) |
| Performance regression | Profile before/after |

---

## Next Steps

1. **Immediate**: Implement CoroutineScope instance methods (launch, async, coroutineScope, supervisorScope, withContext)
2. **Then**: Job methods (join, cancel, children, isCancelled, isCompleted)
3. **Then**: Resolution fixes (Random$Default, PreferenceManager, etc.)
4. **Finally**: Full regression test

---

## Commands

```bash
# Quick test
DEXVM_LIVE=0 cargo run --features keiyoushi --bin exttest -- --http empty --apk "fixtures/keiyoushi_all/tachiyomi-all.xcomic-v1.6.8.apk"

# Full regression
cargo run --features keiyoushi --bin exttest -- --http empty --json report.json

# Run tests
cargo test --features keiyoushi
```