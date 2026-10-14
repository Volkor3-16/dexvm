//! java.time.format.DateTimeFormatter host shims: DateTimeFormatter.ofPattern
//! + Locale.ROOT.

use crate::vm::native::*;

pub(crate) fn dtf_of_pattern(vm: &mut Vm, args: &[JValue]) -> R {
    let pattern = jstr(vm, args[0])?;
    alloc(
        vm,
        "Ljava/time/format/DateTimeFormatter;",
        Native::DateFormatter {
            pattern,
            zone: String::new(),
        },
    )
}

/// `ofPattern(String)`: the overload extension code actually calls (only the
/// locale-carrying one was here). Locale is ignored by this ASCII/fixed
/// pattern formatter.
pub(crate) fn dtf_of_pattern_single(vm: &mut Vm, args: &[JValue]) -> R {
    dtf_of_pattern(vm, args)
}

fn zone_name(vm: &mut Vm, v: JValue) -> Result<String, NatErr> {
    match payload(vm, v) {
        Some(Native::ZoneId(z)) => Ok(z.clone()),
        _ => {
            let s = jstr(vm, v)?;
            Ok(if s.is_empty() { "Z".to_string() } else { s })
        }
    }
}

/// `withZone`/`getZone`: the formatter carries a zone id string, which is all
/// the fixed-offset `java.time` model needs.
pub(crate) fn dtf_with_zone(vm: &mut Vm, args: &[JValue]) -> R {
    let zone = zone_name(vm, args[1])?;
    let Some(Native::DateFormatter { zone: old, .. }) = payload_mut(vm, args[0]) else {
        return Err(npe(vm));
    };
    *old = zone;
    Ok(args[0])
}

pub(crate) fn dtf_get_zone(vm: &mut Vm, args: &[JValue]) -> R {
    let zone = match payload(vm, args[0]) {
        Some(Native::DateFormatter { zone, .. }) if !zone.is_empty() => zone.clone(),
        Some(Native::DateFormatter { .. }) => "Z".to_string(),
        _ => return Err(npe(vm)),
    };
    alloc(vm, "Ljava/time/ZoneId;", Native::ZoneId(zone))
}

macro_rules! dtf_iso_static {
    ($name:ident, $pattern:expr) => {
        /// `DateTimeFormatter.ISO_*` / `BASIC_ISO_DATE` constant.
        pub(crate) fn $name(vm: &mut Vm) -> JValue {
            let Ok(class) = vm.ensure_class_by_desc("Ljava/time/format/DateTimeFormatter;") else {
                return JValue::Null;
            };
            JValue::Obj(vm.arena.alloc(
                class,
                Vec::new(),
                Some(Native::DateFormatter {
                    pattern: $pattern.to_string(),
                    zone: String::new(),
                }),
            ))
        }
    };
}

dtf_iso_static!(lazy_dtf_iso_instant, "yyyy-MM-dd'T'HH:mm:ss'Z'");
dtf_iso_static!(lazy_dtf_iso_date_time, "yyyy-MM-dd'T'HH:mm:ss");
dtf_iso_static!(lazy_dtf_iso_local_date_time, "yyyy-MM-dd'T'HH:mm:ss");
dtf_iso_static!(lazy_dtf_iso_local_date, "yyyy-MM-dd");
dtf_iso_static!(lazy_dtf_iso_offset_date_time, "yyyy-MM-dd'T'HH:mm:ss");
dtf_iso_static!(lazy_dtf_iso_date, "yyyy-MM-dd");
dtf_iso_static!(lazy_dtf_basic_iso_date, "yyyyMMdd");

pub(crate) fn dtf_tostring(vm: &mut Vm, args: &[JValue]) -> R {
    let pattern = match payload(vm, args[0]) {
        Some(Native::DateFormatter { pattern, .. }) => pattern.clone(),
        _ => return Err(npe(vm)),
    };
    Ok(new_str(vm, &pattern))
}

/// `withLocale` doesn't affect this simplified (ASCII, fixed-pattern)
/// formatter — returns `this` for chaining.
pub(crate) fn dtf_with_locale(_vm: &mut Vm, args: &[JValue]) -> R {
    Ok(args[0])
}

pub(crate) fn dtf_of_localized_date(vm: &mut Vm, _args: &[JValue]) -> R {
    alloc(
        vm,
        "Ljava/time/format/DateTimeFormatter;",
        Native::DateFormatter {
            pattern: "yyyy-MM-dd".to_string(),
            zone: String::new(),
        },
    )
}

pub(crate) fn dtf_format(vm: &mut Vm, args: &[JValue]) -> R {
    let pattern = match payload(vm, args[0]) {
        Some(Native::DateFormatter { pattern, .. }) => pattern.clone(),
        _ => return Err(npe(vm)),
    };
    let millis = match payload(vm, args[1]) {
        Some(Native::EpochMillis(m)) => *m,
        Some(Native::LocalDay(d)) => i64::from(*d) * super::super::civil::DAY_MS,
        Some(Native::Date(m)) => *m,
        _ => return Err(npe(vm)),
    };
    let (year, month, day, hour, minute, second, _) = super::super::civil::components(millis);
    let mut out = String::new();
    let pb = pattern.as_bytes();
    let mut pi = 0;
    while pi < pb.len() {
        let c = pb[pi];
        if c == b'\'' {
            pi += 1;
            while pi < pb.len() && pb[pi] != b'\'' {
                out.push(pb[pi] as char);
                pi += 1;
            }
            pi += 1;
            continue;
        }
        let mut run = 1;
        while pi + run < pb.len() && pb[pi + run] == c {
            run += 1;
        }
        match c {
            b'y' | b'Y' => out.push_str(&format!("{:0width$}", year, width = run.max(4))),
            b'M' => out.push_str(&format!("{:0width$}", month, width = run)),
            b'd' => out.push_str(&format!("{:0width$}", day, width = run)),
            b'H' => out.push_str(&format!("{:0width$}", hour, width = run)),
            b'm' => out.push_str(&format!("{:0width$}", minute, width = run)),
            b's' => out.push_str(&format!("{:0width$}", second, width = run)),
            _ => {
                for _ in 0..run {
                    out.push(c as char);
                }
            }
        }
        pi += run;
    }
    Ok(new_str(vm, &out))
}

/// Native methods for Ljava/time/format/DateTimeFormatter;
pub(crate) const TABLE: &[NativeEntry] = &[
    ne!(
        "Ljava/time/format/DateTimeFormatter;",
        "ofPattern",
        "(Ljava/lang/String;Ljava/util/Locale;)Ljava/time/format/DateTimeFormatter;",
        false,
        dtf_of_pattern
    ),
    ne!(
        "Ljava/time/format/DateTimeFormatter;",
        "ofPattern",
        "(Ljava/lang/String;)Ljava/time/format/DateTimeFormatter;",
        false,
        dtf_of_pattern_single
    ),
    ne!(
        "Ljava/time/format/DateTimeFormatter;",
        "withZone",
        "(Ljava/time/ZoneId;)Ljava/time/format/DateTimeFormatter;",
        true,
        dtf_with_zone
    ),
    ne!(
        "Ljava/time/format/DateTimeFormatter;",
        "getZone",
        "()Ljava/time/ZoneId;",
        true,
        dtf_get_zone
    ),
    ne!(
        "Ljava/time/format/DateTimeFormatter;",
        "toString",
        "()Ljava/lang/String;",
        true,
        dtf_tostring
    ),
    ne!(
        "Ljava/time/format/DateTimeFormatter;",
        "withLocale",
        "(Ljava/util/Locale;)Ljava/time/format/DateTimeFormatter;",
        true,
        dtf_with_locale
    ),
    ne!(
        "Ljava/time/format/DateTimeFormatter;",
        "ofLocalizedDate",
        "(Ljava/time/format/FormatStyle;)Ljava/time/format/DateTimeFormatter;",
        false,
        dtf_of_localized_date
    ),
    ne!(
        "Ljava/time/format/DateTimeFormatter;",
        "format",
        "(Ljava/time/temporal/TemporalAccessor;)Ljava/lang/String;",
        true,
        dtf_format
    ),
];
