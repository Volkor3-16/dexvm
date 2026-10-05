# ACTIVEGOAL.md - Active Project Plan

## Current State (Commit `1047ad5`)

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

## Next Task: P0 - Full RxJava/kotlinx.coroutines Shim

**Goal:** Unblock 448 extensions failing with NPE in coroutine path

### Root Cause
The coroutine path (`popular_coro`, `search_coro`, `latest_coro`) uses RxJava + kotlinx.coroutines which are not properly shimmed. Extensions use:
- `RxJava` Observable → `awaitSingle()` / `blockingFirst()` → Single
- `kotlinx.coroutines` Mutex, suspend functions
- `kotlinx.coroutines.sync.Mutex.lock` with continuation

### Implementation Plan

#### Phase 1: RxJava Core (Week 1)
| Component | Methods Needed | Extensions Affected |
|-----------|---------------|---------------------|
| `Observable` | `just`, `error`, `fromCallable`, `map`, `flatMap`, `switchMap`, `doOnNext`, `doOnError`, `doOnTerminate`, `subscribeOn`, `observeOn`, `cache`, `toBlocking`, `toList`, `single`, `subscribe`, `blockingFirst`, `blockingSingle` | 448 |
| `Single` | `just`, `error`, `fromCallable`, `map`, `flatMap`, `subscribe`, `blockingGet` | 448 |
| `Schedulers` | `io()`, `computation()`, `trampoline()`, `newThread()`, `io()` | 448 |
| `Subscription` | `unsubscribe`, `isUnsubscribed` | 448 |

#### Phase 2: kotlinx.coroutines (Week 1-2)
| Component | Methods Needed | Extensions Affected |
|-----------|---------------|---------------------|
| `Mutex` | `lock(block: suspend () -> T)`, `tryLock()`, `unlock()`, `isLocked()` | 200+ |
| `CoroutineScope` | `launch`, `async`, `coroutineScope`, `supervisorScope` | 100+ |
| `Dispatchers` | `IO`, `Default`, `Main`, `Unconfined` | 100+ |
| `Job` | `cancel`, `join`, `isCancelled` | 100+ |
| `Deferred` | `await()` | 100+ |

#### Phase 3: Android/Kotlin Integration (Week 2)
| Class | Methods | Extensions |
|-------|---------|------------|
| `androidx.lifecycle.CoroutineScope` | `coroutineScope` | 50+ |
| `kotlinx.coroutines.CoroutineScope` | `coroutineScope` | 50+ |

---

## Secondary Tasks (After P0)

### P1: Fix Resolution Errors (182 extensions)
| Missing Class | Priority | Extensions |
|--------------|----------|------------|
| `kotlinx.random.Random$Default` | High | 40 |
| `androidx.preference.PreferenceManager` | High | 30 |
| `android.os.Build$VERSION` | Medium | 20 |
| `java.time.ZoneOffset` | Medium | 15 |
| Cyclic class hierarchy | High | 45 |

### P2: GraphQL Empty Response Handling
| Issue | Extensions Affected |
|-------|---------------------|
| Empty handler returns REST format instead of GraphQL | ~100 |

---

## Implementation Order

```
Week 1: RxJava Observable + Single + Schedulers
Week 1-2: kotlinx.coroutines Mutex + CoroutineScope
Week 2: Android lifecycle integration
Week 2: Fix Resolution errors (Random$Default, PreferenceManager, etc.)
Week 3: GraphQL empty response handling
Week 3: Full regression test (1,414 extensions)
```

---

## Success Criteria

| Milestone | Target |
|-----------|--------|
| P0 Complete | 448 NPE extensions unblocked |
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
- Focus on RxJava/kotlinx.coroutines shim first - highest ROI
- Keep ACTIVEGOAL.md updated as progress is made