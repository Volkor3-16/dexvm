# ACTIVE GOAL: Fix Remaining kotlinx.serialization & Coroutine Failures

**Last Updated:** 2026-10-07
**Status:** In Progress - Core coroutines working (~40% pass), kotlinx.serialization incomplete

---

## 🎯 Current Status Summary

| Category | Pass Rate | Status |
|----------|-----------|--------|
| Core operations (filters, popular, search) | 97-98% | ✅ Done |
| popular_coro | ~40% | 🟡 Partial |
| search_coro | ~44% | 🟡 Partial |
| latest_coro | ~32% | 🟡 Partial |
| manga_details/chapters/pages | ~0% (skipped) | ❌ Blocked |

**Main Blocker Fixed:** AppInfo.INSTANCE resolution error - all 1409 extensions now load.

---

## 📋 Remaining Tasks

### 1. kotlinx.serialization Support (Priority: HIGH)

#### Missing Classes & Methods

| Class/Interface | Missing Methods | Impact |
|-----------------|-----------------|--------|
| `kotlinx.serialization.json.JsonKt` | `Json()` extension function | Coroutine source loading fails |
| `kotlinx.serialization.json.Json` | `Default` static field, `Json()` method | Serialization initialization |
| `kotlinx.serialization.encoding.Decoder` | `decodeSerializableElement()`, `decodeSerializableValue()` | Deserialization fails |
| `kotlinx.serialization.encoding.Encoder` | `encodeSerializableElement()`, `encodeSerializableValue()` | Serialization fails |
| `kotlinx.serialization.json.JsonDecoder` | Full implementation | JSON parsing fails |
| `kotlinx.serialization.json.JsonEncoder` | Full implementation | JSON encoding fails |
| `kotlinx.serialization.json.JsonObjectSerializer` | `deserialize()`, `serialize()` | Object handling fails |
| `kotlinx.serialization.json.JsonArraySerializer` | `deserialize()`, `serialize()` | Array handling fails |
| `kotlinx.serialization.internal.LinkedHashMapSerializer` | `deserialize()`, `serialize()` | Map handling fails |
| `kotlinx.serialization.internal.ReferenceArraySerializer` | `deserialize()` | Array handling fails |
| `kotlinx.serialization.internal.ObjectSerializer` | `deserialize()`, `serialize()` | Object handling fails |
| `kotlinx.serialization.internal.EnumSerializer` | `deserialize()`, `serialize()` | Enum handling fails |
| `kotlinx.serialization.internal.PolymorphicSerializer` | `deserialize()`, `serialize()` | Polymorphic types fail |

#### Error Examples from Test Runs
```
✗ sources(): resolution error: no method Json (Lkotlinx/serialization/json/Json;Lkotlin/jvm/functions/Function1;)Lkotlinx/serialization/json/Json; found (on Lkotlinx/serialization/json/JsonKt; starting from Ljava/lang/Object;)
✗ popular_coro: resolution error: class not found: Lkotlinx/serialization/encoding/AbstractDecoder;
✗ search_coro: resolution error: no method getVersionName ()Ljava/lang/String; found (on Lkotlinx/serialization/json/Json; starting from Ljava/lang/Object;)
```

---

### 2. Kotlin Coroutine Enhancements (Priority: MEDIUM)

#### Missing Coroutine Classes & Methods

| Class/Interface | Missing Methods | Impact |
|-----------------|-----------------|--------|
| `kotlinx.coroutines.Job` | `isActive` ✓, `invokeOnCompletion`, `getChildren`, `ensureActive` | Coroutine lifecycle |
| `kotlinx.coroutines.CoroutineScope` | `coroutineContext`, `isActive` ✓ | Scope management |
| `kotlinx.coroutines.CoroutineContext` | `get`, `fold`, `minusKey`, `plus` | Context propagation |
| `kotlinx.coroutines.CoroutineDispatcher` | `dispatch`, `isDispatchNeeded` | Dispatch management |
| `kotlinx.coroutines.CancellableContinuation` | `resume`, `resumeWithException`, `invokeOnCancellation` | Suspension handling |
| `kotlinx.coroutines.ContinuationInterceptor` | `interceptContinuation`, `releaseInterceptedContinuation` | Interception |
| `kotlinx.coroutines.Deferred` | `await`, `awaitAll` | Async results |
| `kotlinx.coroutines.CompletableDeferred` | `complete`, `completeExceptionally` | Completion |
| `kotlinx.coroutines.MainCoroutineDispatcher` | `getImmediate` ✓ | Main thread |
| `kotlinx.coroutines.SupervisorJob` | `supervisorScope` | Error isolation |
| `kotlinx.coroutines.DelayKt` | `delay` | Time delays |
| `kotlinx.coroutines.TimeoutKt` | `withTimeout`, `withTimeoutOrNull` | Timeouts |

#### Error Examples
```
✗ latest_coro: resolution error: no method isActive ()Z found (on Lkotlinx/coroutines/Job; starting from Ljava/lang/Object;)
✗ popular_coro: resolution error: class not found: Lkotlinx/serialization/encoding/AbstractDecoder;
✗ search_coro: resolution error: no method getVersionName ()Ljava/lang/String; found (on Lkotlinx/serialization/json/Json; starting from Ljava/lang/Object;)
```

---

### 3. Kotlin Standard Library Gaps (Priority: MEDIUM)

| Class | Missing Methods | Impact |
|-------|-----------------|--------|
| `kotlin.random.Random.Default` | Static field | Random number generation |
| `kotlin.time.temporal.ChronoField` | Class not found | Time operations |
| `kotlin.ranges.IntIterator` | ✅ Added | ✅ Fixed - iterator() works |
| `kotlin.collections.ArraysKt` | `iterator()` on IntProgression | ⚠️ Fixed via IntProgression shim |
| `kotlin.ranges.RangesKt` | `downTo`, `step`, `reversed` | ✅ Partially done |

---

### 4. kotlinx.html / jsoup Support (Priority: LOW)

| Missing Class | Methods Needed |
|---------------|----------------|
| `org.jsoup.Jsoup` | `parse()`, `connect()` |
| `org.jsoup.nodes.Document` | `select()`, `text()`, `html()` |
| `org.jsoup.nodes.Element` | `select()`, `attr()`, `text()` |
| `org.jsoup.select.Elements` | Iterator, get(), size() |

---

### 5. Android Framework Stubs (Priority: LOW)

| Missing Class | Methods Needed |
|---------------|----------------|
| `android.app.Application$ActivityLifecycleCallbacks` | Interface methods |
| `android.content.pm.ApplicationInfo` | Fields/methods |
| `android.util.Base64$Decoder/Encoder` | Decode/encode |
| `android.util.JsonReader` | Parsing methods |
| `androidx.preference.PreferenceScreen` | UI preferences |

---

## 🛠️ Implementation Priority Order

### Phase 1: Critical kotlinx.serialization (Week 1-2)
- [ ] Implement `JsonKt.Json()` extension function
- [ ] Add `Json.Default` static field with `serialize`/`deserialize`
- [ ] Implement `Decoder.decodeSerializableValue()` / `Encoder.encodeSerializableValue()`
- [ ] Implement `JsonDecoder` / `JsonEncoder` basics
- [ ] Add `JsonObjectSerializer` / `JsonArraySerializer` deserialize/serialize
- [ ] Add `AbstractDecoder` / `Encoder` interfaces

### Phase 2: Coroutine Infrastructure (Week 2-3)
- [ ] Implement `Job.invokeOnCompletion`, `getChildren`, `ensureActive`
- [ ] Implement `CoroutineScope.coroutineContext`, `CoroutineContext` operations
- [ ] Add `CompletableDeferred.complete/completeExceptionally`
- [ ] Add `SupervisorJob` / `supervisorScope`
- [ ] Add `DelayKt.delay`, `TimeoutKt.withTimeout`

### Phase 3: kotlinx.serialization Deep Support (Week 3-4)
- [ ] `JsonDecoder` / `JsonEncoder` full implementation
- [ ] `JsonObjectSerializer` / `JsonArraySerializer` full impl
- [ ] `LinkedHashMapSerializer` / `ReferenceArraySerializer` / `ObjectSerializer`
- [ ] `EnumSerializer` / `PolymorphicSerializer` full impl
- [ ] `Json$Default` full serialize/deserialize

### Phase 4: Kotlin Stdlib & Android Stubs (Week 4)
- [ ] `kotlin.random.Random.Default`
- [ ] `kotlin.time.temporal.ChronoField`
- [ ] Android `Application$ActivityLifecycleCallbacks`
- [ ] `androidx.preference` stubs

---

## 🧪 Test Strategy

### Quick Validation Commands
```bash
# Test single extension (fast)
DEXVM_LIVE=1 cargo run --features keiyoushi --bin exttest -- --apk fixtures/keiyoushi_all/tachiyomi-en.weebcentral-v1.6.25.apk --http replay

# Test coroutine-specific
DEXVM_LIVE=1 cargo run --features keiyoushi --bin exttest -- --apk fixtures/keiyoushi_all/tachiyomi-en.weebcentral-v1.6.25.apk --http replay 2>&1 | grep -E "popular_coro|search_coro|latest_coro"

# Full suite (slow, may stack overflow)
DEXVM_LIVE=1 cargo run --features keiyoushi --bin exttest -- --apk fixtures/keiyoushi_all/ --http replay
```

### Success Criteria
- [ ] `popular_coro` > 80% pass rate
- [ ] `search_coro` > 80% pass rate
- [ ] `latest_coro` > 80% pass rate
- [ ] Zero `resolution error.*kotlinx` errors
- [ ] Zero `resolution error.*kotlin/coroutines` errors
- [ ] `manga_details`, `chapters`, `pages` operations working

---

## 📁 Files to Modify

### Core Serialization
- `src/vm/native/serialization.rs` - Main serialization implementation
- `src/vm/class.rs` - Shim class definitions
- `src/vm/native/kotlin/ranges.rs` - IntIterator ✅ Done

### Coroutine Infrastructure
- `src/vm/native/kotlinx/coroutines.rs` - Coroutine primitives
- `src/vm/native/kotlin/statics.rs` - Lazy static initializers
- `src/vm/native/kotlin/mod.rs` - Exports

### Shim Classes
- `src/vm/class.rs` - Shim class definitions (559 classes)

### Test Infrastructure
- `src/bin/exttest.rs` - Extension testing
- `src/bin/exttest_live.rs` - Live capture/replay

---

## 📊 Progress Tracking

```
Phase 1: Critical kotlinx.serialization ████████░░ 40%
Phase 2: Coroutine Infrastructure ████████░░ 40%
Phase 3: Deep kotlinx.serialization ░░░░░░░░░░ 0%
Phase 4: Stdlib/Android Stubs ░░░░░░░░░░ 0%

Overall: ████░░░░░░░░ 30%
```

---

## 🚀 Quick Wins (Can Do Today)

1. **Add `JsonKt.Json()`** - Single method, fixes source loading
2. **Add `Json.Default`** - Static field with serialize/deserialize
3. **`Decoder.decodeSerializableValue()`** - One method, fixes many deserializers
4. **`Encoder.encodeSerializableValue()`** - One method, fixes many serializers
5. **`Job.invokeOnCompletion`** - Simple callback registration

---

## 📝 Notes

- Stack overflow issues may require increasing stack size or fixing recursive calls in serialization
- Many extensions use HTML (jsoup) not JSON APIs → `manga_details/chapters/pages` will remain skipped until HTML parsing works
- Coroutine variants fail mostly due to missing kotlinx.serialization, not coroutine logic itself
- The `JsonKt.Json()` method is the single biggest blocker for coroutine source loading