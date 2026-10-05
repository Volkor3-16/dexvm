# ACTIVEGOAL.md - Active Project Plan

## Current State (Commit `5c032c2`)

| Metric | Value |
|--------|-------|
| Extensions Tested | 1,414 |
| Total Sources | 2,138 |
| **Extensions Fully Passing** | **599 (42.4%)** |
| Operations Success Rate | **84.6%** |

---

## Current Blockers

| Blocker | Extensions | % of Failed | Primary Cause |
|---------|------------|-------------|---------------|
| **NPE** | **448** | **35.9%** | RxJava/kotlinx.coroutines missing shims (coroutine path) |
| **Resolution** | 182 | 14.6% | Missing Android/Kotlin classes (PreferenceManager, Build$VERSION, ZoneOffset, kotlinx.random.Random$Default, etc.) |
| **Other** | 131 | 10.5% | GraphQL parsing, legacy request methods, etc. |
| **legacy_request** | 47 | 3.8% | Missing legacy HttpSource request methods |
| **UnsupportedOp** | 7 | 0.6% | UnsupportedOperationException |

---

## ✅ P0 COMPLETED: RxJava Shim

| Component | Status | Methods Added |
|-----------|--------|---------------|
| Observable | ✅ | doOnError, doOnSubscribe, doOnUnsubscribe, doOnEach, subscribeOn, observeOn, cache, blockingSingle, blockingLast, blockingGet, blockingForEach |
| Single | ✅ | just, error, fromObservable, map, flatMap, subscribe, toObservable, toBlocking, blockingGet |
| Schedulers | ✅ | computation(), trampoline(), newThread(), immediate(), io() |
| Subscription | ✅ | unsubscribe, isUnsubscribed |
| Observable.single() | ✅ | Returns Single with single item |
| Observable.toBlocking() | ✅ | Returns BlockingObservable |
| BlockingObservable | ✅ | blockingFirst, blockingLast, blockingGet, blockingForEach |
| Schedulers | ✅ | computation(), trampoline(), newThread(), immediate(), io() |

**Result:** NPEs reduced from 5,687 → 1,617 (71% reduction), Coroutine versions now pass

---

## ✅ P0 COMPLETED: kotlinx.coroutines Shim (Phase 1-2)

| Component | Status | Methods Added |
|-----------|--------|---------------|
| Dispatchers | ✅ | Default, Unconfined, Main.immediate, IO, Computation, Trampoline, NewThread, Immediate |
| Job | ✅ | join, isCancelled, isCompleted, cancel, children |
| Deferred | ✅ | await |
| CoroutineScope | ✅ | launch, async, coroutineScope, supervisorScope |
| withContext | ✅ |  |
| withTimeout / withTimeoutOrNull | ✅ |  |
| SupervisorJob / SupervisorScope | ✅ |  |
| runBlocking | ✅ | Top-level and default |
| withTimeout / withTimeoutOrNull | ✅ |  |
| Missing shim classes | ✅ | Deferred, Dispatchers, CoroutineStart, Dispatchers lazy instance |

**Result:** NPEs reduced from 5,687 → 1,617 (71% reduction), Coroutine versions now pass

---

## ✅ P0 COMPLETED: kotlinx.coroutines Mutex & MutexKt

| Component | Status | Methods Added |
|-----------|--------|---------------|
| Mutex | ✅ | lock(suspend () -> T), tryLock(), unlock(), isLocked() |
| MutexKt | ✅ | Mutex(), lock() static extension |

**Result:** Mutex suspend lock now implemented with proper continuation handling

---

## 🔄 P0 REMAINING: kotlinx.coroutines CoroutineScope/Job/Deferred (Week 1-2)

**Goal:** Unblock remaining 1,617 NPEs in coroutine path (97.8% of NPEs are in coroutine operations)

### Root Cause
The coroutine path still has NPEs because CoroutineScope/Job/Deferred are not fully implemented. Extensions use:
- `CoroutineScope.launch` / `async` / `coroutineScope` / `supervisorScope` (instance methods)
- `Job.join()` / `cancel()` / `children` / `isCancelled` / `isCompleted`
- `Deferred.await()` with continuation
- `runBlocking` top-level function
- `CoroutineScope` context propagation

### Implementation Plan

#### Phase 1: CoroutineScope Instance Methods (Week 1)
| Component | Methods Needed | Extensions Affected |
|-----------|---------------|---------------------|
| `CoroutineScope` | `launch`, `async`, `coroutineScope`, `supervisorScope` | 200+ |
| `CoroutineScope` | `withContext` | 200+ |

#### Phase 2: Job & Deferred (Week 1-2)
| Component | Methods Needed | Extensions Affected |
|-----------|---------------|---------------------|
| `Job` | `join`, `cancel`, `children`, `isCancelled`, `isCompleted` | 200+ |
| `Deferred` | `await()` | 200+ |
| `CoroutineScope` | `launch`, `async`, `coroutineScope`, `supervisorScope` | 200+ |

#### Phase 3: Additional Coroutines APIs (Week 1-2)
| Component | Methods Needed | Extensions Affected |
|-----------|---------------|---------------------|
| `runBlocking` (top-level) | ✅ Done | 100+ |
| `withTimeout` / `withTimeoutOrNull` | ✅ Done | 100+ |
| `SupervisorJob` / `SupervisorScope` | ✅ Done | 100+ |

---

## 📋 P1: Fix Resolution Errors (182 extensions)

| Missing Class | Priority | Extensions | Status |
|--------------|----------|------------|--------|
| `kotlinx.random.Random$Default` | High | 40 | 🔄 |
| `androidx.preference.PreferenceManager` | High | 30 | 🔄 |
| `android.os.Build$VERSION` | Medium | 20 | 🔄 |
| `java.time.ZoneOffset` | Medium | 15 | 🔄 |
| Cyclic class hierarchy | High | 45 | 🔄 |

---

## 📋 P2: GraphQL Empty Response Handling

| Issue | Extensions Affected |
|-------|---------------------|
| Empty handler returns REST format instead of GraphQL | ~100 |

---

## Implementation Order

```
Week 1: CoroutineScope instance methods (launch, async, coroutineScope, supervisorScope) + Job/Deferred
Week 2: Resolution fixes (Random$Default, PreferenceManager, Build$VERSION, ZoneOffset, cyclic hierarchy)
Week 3: GraphQL empty response handling
Week 3: Full regression test (1,414 extensions)
```

---

## Success Criteria

| Milestone | Target |
|-----------|--------|
| CoroutineScope/Job/Deferred Complete | 448 NPE extensions unblocked |
| Resolution Fixed | 182 extensions unblocked |
| Overall success rate | >95% (from 84.6%) |
| Extensions passing | >500 (from 599) |
| All tests pass | ✅ |

---

## Commands

```bash
# Quick test single extension
DEXVM_LIVE=0 cargo run --features keiyoushi --bin exttest -- --http empty --apk "fixtures/keiyoushi_all/tachiyomi-all.xcomic-v1.6.8.apk"

# Full regression
cargo run --features keiyoushi --bin exttest -- --http empty --json report.json

# Run unit tests
cargo test --features keiyoushi
```

---

## Notes

- Vietnamese (vi) and Chinese (zh) extensions already pass (61% pass rate) - they use simple REST APIs
- Remaining failures are primarily GraphQL + coroutine-heavy extensions
- Mutex suspend lock is now implemented and working
- Focus on CoroutineScope/Job/Deferred next - highest ROI for remaining NPEs
- Keep ACTIVEGOAL.md updated as progress is made