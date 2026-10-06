# RESOLUTION ERRORS FIX PLAN

## Overview
**Total Resolution Errors:** 1,566 across 182 extensions
**Target:** Fix all resolution errors to unblock 182 extensions

---

## PRIORITY 1: Critical Missing Classes (High Impact)

### 1. Primitive Type Wrappers (Java/Kotlin)
**Impact:** ~300 errors | **Priority:** CRITICAL

| Class | Descriptor | Extensions Affected | Fix Location |
|-------|------------|---------------------|--------------|
| `I` | `I` (Integer) | ~30 | `src/vm/class.rs` |
| `F` | `F` (Float) | ~30 | `src/vm/class.rs` |
| `B` | `B` (Byte) | ~30 | `src/vm/class.rs` |
| `C` | `C` (Char) | ~10 | `src/vm/class.rs` |
| `D` | `D` (Double) | ~10 | `src/vm/class.rs` |
| `J` | `J` (Long) | ~10 | `src/vm/class.rs` |

**Fix:** Add shim classes for primitive wrappers in `src/vm/class.rs`
```rust
// Primitive wrappers
shim!("Ljava/lang/Integer;", Some("Ljava/lang/Number;"), &["Ljava/io/Serializable;", "Ljava/lang/Comparable;"], ACC_PUBLIC),
shim!("Ljava/lang/Float;", Some("Ljava/lang/Number;"), &["Ljava/io/Serializable;", "Ljava/lang/Comparable;"], ACC_PUBLIC),
shim!("Ljava/lang/Double;", Some("Ljava/lang/Number;"), &["Ljava/io/Serializable;", "Ljava/lang/Comparable;"], ACC_PUBLIC),
shim!("Ljava/lang/Long;", Some("Ljava/lang/Number;"), &["Ljava/io/Serializable;", "Ljava/lang/Comparable;"], ACC_PUBLIC),
shim!("Ljava/lang/Short;", Some("Ljava/lang/Number;"), &["Ljava/io/Serializable;", "Ljava/lang/Comparable;"], ACC_PUBLIC),
shim!("Ljava/lang/Byte;", Some("Ljava/lang/Number;"), &["Ljava/io/Serializable;", "Ljava/lang/Comparable;"], ACC_PUBLIC),
shim!("Ljava/lang/Character;", Some("Ljava/lang/Number;"), &["Ljava/io/Serializable;", "Ljava/lang/Comparable;"], ACC_PUBLIC),
shim!("Ljava/lang/Boolean;", Some("Ljava/lang/Number;"), &["Ljava/io/Serializable;", "Ljava/lang/Comparable;"], ACC_PUBLIC),
```

---

### 2. Android Core Classes

#### A. `android.os.Build$VERSION` (68 errors)
**Shim needed:**
```rust
shim!(
    "Landroid/os/Build$VERSION;",
    Some("Ljava/lang/Object;"),
    &[],
    0,
    [
        sdef!("SDK_INT", "I", ShimValue::Const(JValue::Int(33))), // API 33
        sdef!("RELEASE", "Ljava/lang/String;", ShimValue::Const(JValue::Null)),
        sdef!("CODENAME", "Ljava/lang/String;", ShimValue::Const(JValue::Null)),
        sdef!("INCREMENTAL", "Ljava/lang/String;", ShimValue::Const(JValue::Null)),
        sdef!("SDK", "Ljava/lang/String;", ShimValue::Const(JValue::Null)),
    ]
),
```

#### B. `androidx.preference.PreferenceManager` (61 errors)
```rust
#[cfg(feature = "android")]
shim!(
    "Landroidx/preference/PreferenceManager;",
    Some("Ljava/lang/Object;"),
    &[],
    0,
    [
        sdef!(
            "INSTANCE",
            "Landroidx/preference/PreferenceManager;",
            ShimValue::Lazy(native::android::preference::lazy_preference_manager)
        ),
    ]
),
```

#### C. `java.time.ZoneOffset` (62 errors)
```rust
shim!(
    "Ljava/time/ZoneOffset;",
    Some("Ljava/lang/Object;"),
    &[],
    0,
    [
        sdef!("UTC", "Ljava/time/ZoneOffset;", ShimValue::Lazy(native::java::time::zone_offset_utc)),
        sdef!("of", "(Ljava/lang/String;)Ljava/time/ZoneOffset;", ShimValue::Native(native::java::time::zone_offset_of)),
        sdef!("ofHours", "(I)Ljava/time/ZoneOffset;", ShimValue::Native(native::java::time::zone_offset_of_hours)),
        sdef!("ofHoursMinutes", "(II)Ljava/time/ZoneOffset;", ShimValue::Native(native::java::time::zone_offset_of_hours_minutes)),
    ]
),
```

#### D. `java.time.format.DateTimeFormatterBuilder.parseCaseInsensitive` (62 errors)
```rust
// In java/time/format.rs
pub(crate) fn date_time_formatter_builder_parse_case_insensitive(vm: &mut Vm, args: &[JValue]) -> R {
    // Return self for chaining
    Ok(args[0])
}
```

---

### 3. Java Time Classes

| Class | Errors | Fix |
|-------|--------|-----|
| `java.time.temporal.ChronoField` | 25 | Add shim with static fields |
| `java.time.temporal.WeekFields` | 1 | Add shim |
| `java.time.temporal.ChronoUnit` | - | Add shim |

```rust
shim!(
    "Ljava/time/temporal/ChronoField;",
    Some("Ljava/lang/Object;"),
    &[],
    0,
    [
        sdef!("DAY_OF_WEEK", "Ljava/time/temporal/ChronoField;", ShimValue::Lazy(native::java::time::chrono_field_day_of_week)),
        sdef!("DAY_OF_MONTH", "Ljava/time/temporal/ChronoField;", ShimValue::Lazy(native::java::time::chrono_field_day_of_month)),
        // ... more fields
    ]
),
```

---

### 4. Android Specific Classes

| Class | Errors | Priority |
|-------|--------|----------|
| `android.util.DisplayMetrics` | 27 | HIGH |
| `android.widget.Button` | 4 | MEDIUM |
| `android.webkit.ValueCallback` | 3 | MEDIUM |
| `android.webkit.WebResourceError` | 2 | LOW |
| `android.text.TextWatcher` | 1 | LOW |
| `android.widget.TextView` methods | - | MEDIUM |

```rust
shim!(
    "Landroid/util/DisplayMetrics;",
    Some("Ljava/lang/Object;"),
    &[],
    0,
    [
        sdef!("widthPixels", "I", ShimValue::Const(JValue::Int(1080))),
        sdef!("heightPixels", "I", ShimValue::Const(JValue::Int(1920))),
        sdef!("density", "F", ShimValue::Const(JValue::Float(2.0))),
        sdef!("densityDpi", "I", ShimValue::Const(JValue::Int(320))),
        sdef!("scaledDensity", "F", ShimValue::Const(JValue::Float(2.0))),
        sdef!("xdpi", "F", ShimValue::Const(JValue::Float(1.0))),
        sdef!("ydpi", "F", ShimValue::Const(JValue::Float(1.0))),
    ]
),
```

---

### 5. Kotlin Reflection/Internal Classes

| Class | Errors | Fix |
|-------|--------|-----|
| `kotlinx.serialization.encoding.AbstractDecoder` | 5 | Add shim |
| `kotlinx.serialization.encoding.AbstractEncoder` | - | Add shim |
| `kotlin.jvm.internal.markers.KMappedMarker` | 3 | Add shim |
| `kotlin.jvm.internal.markers.KMutableList` | 1 | Add shim |
| `kotlin.properties.PropertyDelegateProvider` | 1 | Add shim |
| `kotlin.reflect.KTypeProjection` | 3 | Add shim |

---

### 6. Kotlin Standard Library Methods

| Method | Errors | Fix Location |
|--------|--------|--------------|
| `iterator()` | 36 | Collections/List |
| `component1()` | 34 | Data classes/Tuples |
| `toList()` | 3 | Collections |
| `toArray()` | 7 | Arrays/Collections |
| `size()` | 5 | Collections |
| `last()` | 5 | Collections |
| `getFields()` | 6 | Reflection |
| `withDefault` | 10 | Maps |
| `toArray()` | 7 | Collections |
| `isActive` | 12 | Coroutines (DONE) |
| `getFilterList` | 19 | Filters |
| `getDayOfWeek` | 10 | Date/Time |
| `withZone` | 4 | ZoneOffset |
| `getDayOfWeek` | 10 | Date/Time |
| `joinTo$default` | 5 | Strings |
| `sequence` | 1 | Sequences |
| `toList` | 3 | Collections |
| `getFilterList` | 19 | Filters |

---

## Implementation Priority Order

### Week 1: Core Missing Classes (Days 1-3)
1. **Day 1:** Primitive type wrappers (I, F, B, C, D, J, Z)
2. **Day 2:** Android Build$VERSION, PreferenceManager, ZoneOffset
3. **Day 3:** Java Time classes (ZoneOffset, ChronoField, WeekFields)

### Week 2: Android Classes & Kotlin Stdlib (Days 4-7)
4. **Day 4-5:** Android classes (DisplayMetrics, Button, etc.)
5. **Day 6:** Kotlin reflection/internal classes
6. **Day 7:** Kotlin stdlib methods (iterator, component1, etc.)

### Week 3: Cyclic Hierarchy Resolution
6. **Day 8-10:** Analyze and fix cyclic class hierarchy (66 errors)
   - Use `ACC_INTERFACE` without `ACC_ABSTRACT` for interfaces
   - Add forward declarations
   - Ensure parent classes loaded before children

---

## Implementation Details

### File Locations
- **Shim classes:** `src/vm/class.rs`
- **Native implementations:** `src/vm/native/` (android/, java/, kotlin/)
- **Registration:** `src/vm/native/mod.rs` and `src/vm/native/kotlin/mod.rs`

### Adding a New Shim Class
1. Add to `src/vm/class.rs` in `SHIM_CLASSES` array
2. Add static field getters in `ShimValue::Lazy` functions
3. Add native function implementations in appropriate `src/vm/native/*/`
7. Register in appropriate module's TABLE

### Testing
```bash
# Quick test
cargo run --features keiyoushi --bin exttest -- --http empty --apk "fixtures/keiyoushi_all/tachiyomi-all.globalcomix-v1.6.0.apk"

# Full regression
cargo run --features keiyoushi --bin exttest -- --http empty --json report.json
```

---

## Success Criteria

| Metric | Target |
|--------|--------|
| Resolution errors | < 50 (from 1,035) |
| Extensions unblocked | > 150 |
| Resolution error rate | < 5% |

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Breaking existing tests | Run full test suite after each batch |
| Cyclic dependencies | Use forward declarations, lazy initialization |
| Performance | Profile after each batch, optimize hot paths |
| Android API differences | Test on multiple API levels if possible |