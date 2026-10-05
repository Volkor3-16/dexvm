# ACTIVEGOAL.md - Active Project Plan

## Current State (Commit `c87482f`)

| Metric | Value |
|--------|-------|
| Extensions Tested | 1,414 |
| Total Sources | 2,138 |
| **Extensions Fully Passing** | **168 (11.9%)** |
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

## 🔄 P0 REMAINING: kotlinx.coroutines Shim

**Goal:** Unblock 448 extensions failing with NPE in coroutine path (`popular_coro`, `search_coro`, `latest_coro`)

### Root Cause
The coroutine path uses `kotlinx.coroutines` which is not properly shimmed. Extensions use:
- `kotlinx.coroutines.sync.Mutex.lock(suspend () -> T)` with continuation
- `kotlinx.coroutines.CoroutineScope` for `launch`/`async`
- `kotlinx.coroutines.sync.Mutex` with `isLocked()`, `tryLock()`, `unlock()`

### Implementation Plan

#### Phase 1: kotlinx.coroutines.sync.Mutex (Week 1)
| Component | Methods Needed | Extensions Affected |
|-----------|---------------|---------------------|
| `Mutex` | `lock(block: suspend () -> T)`, `tryLock()`, `unlock()`, `isLocked()` | 200+ |
| `MutexKt.Mutex()` | Default constructor | 200+ |

#### Phase 2: kotlinx.coroutines.CoroutineScope (Week 1-2)
| Component | Methods Needed | Extensions Affected |
|-----------|---------------|---------------------|
| `CoroutineScope` | `launch`, `async`, `coroutineScope`, `supervisorScope` | 100+ |
| `Dispatchers` | `IO`, `Default`, `Main`, `Unconfined` | 100+ |
| `Job` | `cancel`, `join`, `isCancelled` | 100+ |
| `Deferred` | `await()` | 100+ |
| `runBlocking` | Top-level function | 100+ |

#### Phase 3: Android/Kotlin Integration (Week 2)
| Class | Methods | Extensions |
|-------|---------|------------|
| `androidx.lifecycle.CoroutineScope` | `coroutineScope` | 50+ |
| `kotlinx.coroutines.CoroutineScope` | `coroutineScope` | 50+ |

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
Week 1: kotlinx.coroutines Mutex + CoroutineScope
Week 2: Android/Kotlin Resolution fixes
Week 3: GraphQL empty response handling
Week 3: Full regression test (1,414 extensions)
```

---

## Success Criteria

| Milestone | Target |
|-----------|--------|
| kotlinx.coroutines Complete | 448 NPE extensions unblocked |
| Resolution Fixed | 182 extensions unblocked |
| Overall success rate | >95% (from 84.6%) |
| Extensions passing | >500 (from 168) |
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
- Focus on kotlinx.coroutines shim next - highest ROI for remaining NPEs
- Keep ACTIVEGOAL.md updated as progress is made