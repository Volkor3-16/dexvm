# ACTIVEGOAL.md - Project Plan for Next Phase

## Current State (Commit `fab0eec`)

| Metric | Value |
|--------|-------|
| Total Operations | 25,656 |
| Failed | 8,030 (31.3%) |
| **Primary Blocker** | **Response NPE: 5,689 (70.8%)** |

---

## Root Cause Analysis

The latest commit fixed 4,072 legacy request resolution errors but shifted the failure point to **parse-time NPE** when handling GraphQL responses. The "empty" HTTP handler returns REST format (`{"mangas":[],"hasNext":false}`) but GraphQL-based extensions expect GraphQL format (`{"data":{"popularManga":{"mangas":[],"hasNext":false}}}`).

---

## Priority Queue

### 🔴 P0 - Critical (Highest Impact)

| Task | Estimated Effort | Impact | Description |
|------|------------------|--------|-------------|
| **Fix Response NPE for GraphQL endpoints** | 2-3 days | **5,689 errors (70.8%)** | VM crashes when allocating `Lokhttp3/Response` for GraphQL responses. The empty handler returns REST format but GraphQL extensions parse expecting GraphQL structure. |
| **Fix Response allocation** | 1-2 days | **5,689 errors** | `alloc()` for `Lokhttp3/Response` fails during class loading/initialization for GraphQL-based extensions (XCOMIC, MangaDex, MangaMillion, etc.) |

### 🟠 P1 - High Impact

| Task | Estimated Effort | Impact | Description |
|------|------------------|--------|-------------|
| **Add `getMangaUpdate` shim** | 1 day | ~50 errors | `MangaUpdate` interface with `getMangaUpdate` suspend function needed by Roxinha, Mangadusleri, MangaPill |
| **Add `kotlinx.random.Random$Default` shim** | 1 day | ~40 errors | Missing static field `Default` in `kotlin.random.Random$Default` |
| **Fix cyclic class hierarchy** | 2-3 days | ~45 errors | Some extensions have circular inheritance (e.g., `GenericSource` -> `HttpSource` -> `GenericSource`) |

### 🟡 P2 - Medium Impact

| Task | Estimated Effort | Impact | Description |
|------|------------------|--------|-------------|
| **Improve empty response handling for coroutines** | 1 day | ~25 errors | Coroutine parse methods (`popular_coro`, `search_coro`, `latest_coro`) need better empty response defaults |
| **Fix kotlinx.serialization remaining 2 errors** | < 1 day | 2 errors | Minor edge cases |

---

## Technical Plan for P0: Fix Response NPE

### Problem
```
DEXVM_TRACE alloc: desc=Lokhttp3/Response;
DEXVM_TRACE load_shim_class: Lokhttp3/Response;
NPE@native
NPE@created
```

The VM crashes when:
1. Extension makes GraphQL request
2. Empty handler returns REST format `{"mangas":[],"hasNext":false}`
3. Extension's GraphQL parser tries to access `data.popularManga.mangas`
4. Returns null → NPE in parse code

### Solution Options

| Option | Pros | Cons | Effort |
|--------|------|------|--------|
| **A. Detect GraphQL in empty handler** | Fixes root cause, proper format | Need to detect GraphQL endpoints | 1 day |
| **B. Return valid GraphQL for ALL empty responses** | Simple, consistent | Wastes bandwidth for REST-only extensions | 0.5 days |
| **C. Make parse code handle both REST + GraphQL** | Already partially done | Doesn't fix VM allocation crash | 1 day |
| **D. Fix Response class loading/shim** | Fixes VM crash at source | Need to understand why `Response` alloc fails | 1-2 days |

### Recommended Approach: **A + D**

1. **Fix Response class loading** (D): Ensure `Lokhttp3/Response` shim loads correctly with all required methods (`code()`, `message()`, `headers()`, `body()`)
2. **Add GraphQL detection in empty handler** (A): Check `url.contains("graphql")` OR check request `Accept` header for `application/graphql`

---

## Execution Sequence

### Week 1: P0 - Response NPE
- **Day 1-2**: Debug Response allocation crash
  - Add trace to `load_shim_class` for `Lokhttp3/Response`
  - Verify shim has all required methods from native table
  - Check if `ResponseBody` companion removal caused regression
- **Day 2-3**: Implement GraphQL-aware empty handler
  - Detect GraphQL via URL path (`/graphql`, `/api/graphql`) or `Accept` header
  - Return proper GraphQL structure: `{"data":{"popularManga":{"mangas":[],"hasNext":false}}}`
- **Day 3**: Test against top 5 GraphQL extensions (XCOMIC, MangaDex, MangaMillion, GlobalComix, HentaiHand)

### Week 2: P1 - High Impact
- **Day 4**: Add `getMangaUpdate` shim for MangaUpdate interface
- **Day 5**: Add `kotlinx.random.Random$Default` static field shim
- **Day 5-6**: Fix cyclic class hierarchy (analyze with `cargo run --features keiyoushi --bin dexcli -- --classes`)

### Week 3: P2 - Polish
- **Day 7**: Improve empty response handling for coroutines
- **Day 7-8**: Fix remaining kotlinx.serialization edge cases
- **Day 8**: Full regression test (1,414 extensions)

---

## Success Criteria

| Metric | Target |
|--------|--------|
| Response NPE errors | < 100 (from 5,689) |
| Legacy request missing | < 50 (from 150) |
| Total failed operations | < 2,000 (from 8,030) |
| Success rate | > 92% (from 68.7%) |
| GraphQL extensions working | XCOMIC, MangaDex, MangaMillion, GlobalComix, HentaiHand |

---

## Test Commands

```bash
# Quick test single GraphQL extension
DEXVM_LIVE=0 cargo run --features keiyoushi --bin exttest -- --http empty --apk "fixtures/keiyoushi_all/tachiyomi-all.xcomic-v1.6.8.apk"

# Full regression
cargo run --features keiyoushi --bin exttest -- --http empty --json report.json

# Debug Response loading
DEXVM_TRACE=1 cargo run --features keiyoushi --bin exttest -- --http empty --apk "fixtures/keiyoushi_all/tachiyomi-all.xcomic-v1.6.8.apk" 2>&1 | grep -E "alloc.*Response|load_shim_class.*Response"
```

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| GraphQL detection misses some endpoints | Fallback: return BOTH REST + GraphQL in empty response |
| Response shim missing methods | Audit native table for `Lokhttp3/Response` methods |
| Breaking REST-only extensions | Use path-based detection (`/graphql` in URL) not blanket GraphQL |

---

## Notes

- The **Response NPE is a VM crash**, not a Java exception - it happens during class loading/allocation, not in user code
- GraphQL extensions affected: **XCOMIC (109), MangaDex (61), MangaMillion (108), GlobalComix (41), HentaiHand (34), DragonBallMultiverse (36)** = 389 sources
- Fixing this alone would reduce failures by ~5,600 (70%) and push success rate to >85%