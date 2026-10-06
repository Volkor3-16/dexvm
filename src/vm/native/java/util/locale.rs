use crate::vm::native::*;

pub fn lazy_opaque_locale(vm: &mut Vm) -> JValue {
    let Ok(class) = vm.ensure_class_by_desc("Ljava/util/Locale;") else {
        return JValue::Null;
    };
    JValue::Obj(vm.arena.alloc(class, Vec::new(), Some(Native::Opaque)))
}

// Locale constants (e.g. Locale.US) used by `String.lowercase(Locale)` etc.
macro_rules! locale_const {
    ($name:ident, $tag:expr) => {
        pub fn $name(_vm: &mut Vm, _args: &[JValue]) -> R {
            let Ok(class) = _vm.ensure_class_by_desc("Ljava/util/Locale;") else {
                return Err(npe(_vm));
            };
            Ok(JValue::Obj(
                _vm.arena
                    .alloc(class, Vec::new(), Some(Native::Str($tag.into()))),
            ))
        }
    };
}
locale_const!(lazy_locale_us, "en-US");
locale_const!(lazy_locale_uk, "en-GB");
locale_const!(lazy_locale_canada, "en-CA");
locale_const!(lazy_locale_japan, "ja-JP");
locale_const!(lazy_locale_korea, "ko-KR");
locale_const!(lazy_locale_china, "zh-CN");
locale_const!(lazy_locale_france, "fr-FR");
locale_const!(lazy_locale_germany, "de-DE");
locale_const!(lazy_locale_italy, "it-IT");

// Language constants (Locale.ENGLISH, Locale.FRENCH, etc.)
locale_const!(lazy_locale_english, "en");
locale_const!(lazy_locale_french, "fr");
locale_const!(lazy_locale_german, "de");
locale_const!(lazy_locale_italian, "it");
locale_const!(lazy_locale_japanese, "ja");
locale_const!(lazy_locale_korean, "ko");
locale_const!(lazy_locale_chinese, "zh");
locale_const!(lazy_locale_simplified_chinese, "zh-CN");
locale_const!(lazy_locale_traditional_chinese, "zh-TW");

// Wrapper functions with old signature for class registration (ShimValue::Lazy)
pub fn lazy_locale_us_vm(vm: &mut Vm) -> JValue {
    lazy_locale_us(vm, &[]).unwrap_or_else(|_| JValue::Null)
}
pub fn lazy_locale_uk_vm(vm: &mut Vm) -> JValue {
    lazy_locale_uk(vm, &[]).unwrap_or_else(|_| JValue::Null)
}
pub fn lazy_locale_canada_vm(vm: &mut Vm) -> JValue {
    lazy_locale_canada(vm, &[]).unwrap_or_else(|_| JValue::Null)
}
pub fn lazy_locale_japan_vm(vm: &mut Vm) -> JValue {
    lazy_locale_japan(vm, &[]).unwrap_or_else(|_| JValue::Null)
}
pub fn lazy_locale_korea_vm(vm: &mut Vm) -> JValue {
    lazy_locale_korea(vm, &[]).unwrap_or_else(|_| JValue::Null)
}
pub fn lazy_locale_china_vm(vm: &mut Vm) -> JValue {
    lazy_locale_china(vm, &[]).unwrap_or_else(|_| JValue::Null)
}
pub fn lazy_locale_france_vm(vm: &mut Vm) -> JValue {
    lazy_locale_france(vm, &[]).unwrap_or_else(|_| JValue::Null)
}
pub fn lazy_locale_germany_vm(vm: &mut Vm) -> JValue {
    lazy_locale_germany(vm, &[]).unwrap_or_else(|_| JValue::Null)
}
pub fn lazy_locale_italy_vm(vm: &mut Vm) -> JValue {
    lazy_locale_italy(vm, &[]).unwrap_or_else(|_| JValue::Null)
}

// java.util.Locale host shims.

// java.util.Locale
// ---------------------------------------------------------------------------

pub(crate) fn locale_get_default(vm: &mut Vm, _args: &[JValue]) -> R {
    Ok(lazy_opaque_locale(vm))
}

pub(crate) fn locale_init(vm: &mut Vm, args: &[JValue]) -> R {
    let tag = if args.len() > 1 {
        jstr(vm, args[1])?
    } else {
        String::new()
    };
    let Some(n) = payload_mut(vm, args[0]) else {
        return Err(npe(vm));
    };
    match n {
        Native::Str(dst) => *dst = tag,
        _ => *n = Native::Str(tag),
    }
    Ok(JValue::Null)
}

pub(crate) fn locale_tag(vm: &mut Vm, v: JValue) -> String {
    match payload(vm, v) {
        Some(Native::Str(s)) => s.clone(),
        _ => String::new(),
    }
}

pub(crate) fn locale_to_string(vm: &mut Vm, args: &[JValue]) -> R {
    let tag = locale_tag(vm, args[0]);
    Ok(new_str(vm, &tag))
}

pub(crate) fn locale_get_language(vm: &mut Vm, args: &[JValue]) -> R {
    let lang = locale_tag(vm, args[0])
        .split(['_', '-'])
        .next()
        .unwrap_or("")
        .to_string();
    Ok(new_str(vm, &lang))
}

pub(crate) fn locale_get_country(vm: &mut Vm, args: &[JValue]) -> R {
    let tag = locale_tag(vm, args[0]);
    let mut parts = tag.split(['_', '-']);
    let _lang = parts.next();
    Ok(new_str(vm, parts.next().unwrap_or("")))
}

pub(crate) fn locale_for_language_tag(vm: &mut Vm, args: &[JValue]) -> R {
    let tag = jstr(vm, args[0])?;
    alloc(vm, "Ljava/util/Locale;", Native::Str(tag))
}

/// Native methods for Ljava/util/Locale;
pub(crate) const TABLE: &[NativeEntry] = &[
    ne!(
        "Ljava/util/Locale;",
        "<init>",
        "(Ljava/lang/String;)V",
        true,
        locale_init
    ),
    ne!(
        "Ljava/util/Locale;",
        "<init>",
        "(Ljava/lang/String;Ljava/lang/String;)V",
        true,
        locale_init
    ),
    ne!(
        "Ljava/util/Locale;",
        "<init>",
        "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V",
        true,
        locale_init
    ),
    ne!(
        "Ljava/util/Locale;",
        "getDefault",
        "()Ljava/util/Locale;",
        false,
        locale_get_default
    ),
    ne!(
        "Ljava/util/Locale;",
        "toString",
        "()Ljava/lang/String;",
        true,
        locale_to_string
    ),
    ne!(
        "Ljava/util/Locale;",
        "getLanguage",
        "()Ljava/lang/String;",
        true,
        locale_get_language
    ),
    ne!(
        "Ljava/util/Locale;",
        "getCountry",
        "()Ljava/lang/String;",
        true,
        locale_get_country
    ),
    ne!(
        "Ljava/util/Locale;",
        "forLanguageTag",
        "(Ljava/lang/String;)Ljava/util/Locale;",
        false,
        locale_for_language_tag
    ),
    ne!(
        "Ljava/util/Locale;",
        "getDisplayLanguage",
        "()Ljava/lang/String;",
        true,
        locale_get_language
    ),
    ne!(
        "Ljava/util/Locale;",
        "getDisplayLanguage",
        "(Ljava/util/Locale;)Ljava/lang/String;",
        true,
        locale_get_language
    ),
    ne!(
        "Ljava/util/Locale;",
        "getDisplayName",
        "(Ljava/util/Locale;)Ljava/lang/String;",
        true,
        locale_to_string
    ),
    // Static field constants (Locale.US, Locale.FRENCH, etc.)
    ne!(
        "Ljava/util/Locale;",
        "US",
        "()Ljava/util/Locale;",
        false,
        lazy_locale_us
    ),
    ne!(
        "Ljava/util/Locale;",
        "UK",
        "()Ljava/util/Locale;",
        false,
        lazy_locale_uk
    ),
    ne!(
        "Ljava/util/Locale;",
        "CANADA",
        "()Ljava/util/Locale;",
        false,
        lazy_locale_canada
    ),
    ne!(
        "Ljava/util/Locale;",
        "JAPAN",
        "()Ljava/util/Locale;",
        false,
        lazy_locale_japan
    ),
    ne!(
        "Ljava/util/Locale;",
        "KOREA",
        "()Ljava/util/Locale;",
        false,
        lazy_locale_korea
    ),
    ne!(
        "Ljava/util/Locale;",
        "CHINA",
        "()Ljava/util/Locale;",
        false,
        lazy_locale_china
    ),
    ne!(
        "Ljava/util/Locale;",
        "FRANCE",
        "()Ljava/util/Locale;",
        false,
        lazy_locale_france
    ),
    ne!(
        "Ljava/util/Locale;",
        "GERMANY",
        "()Ljava/util/Locale;",
        false,
        lazy_locale_germany
    ),
    ne!(
        "Ljava/util/Locale;",
        "ITALY",
        "()Ljava/util/Locale;",
        false,
        lazy_locale_italy
    ),
    // Language constants
    ne!(
        "Ljava/util/Locale;",
        "ENGLISH",
        "()Ljava/util/Locale;",
        false,
        lazy_locale_english
    ),
    ne!(
        "Ljava/util/Locale;",
        "FRENCH",
        "()Ljava/util/Locale;",
        false,
        lazy_locale_french
    ),
    ne!(
        "Ljava/util/Locale;",
        "GERMAN",
        "()Ljava/util/Locale;",
        false,
        lazy_locale_german
    ),
    ne!(
        "Ljava/util/Locale;",
        "ITALIAN",
        "()Ljava/util/Locale;",
        false,
        lazy_locale_italian
    ),
    ne!(
        "Ljava/util/Locale;",
        "JAPANESE",
        "()Ljava/util/Locale;",
        false,
        lazy_locale_japanese
    ),
    ne!(
        "Ljava/util/Locale;",
        "KOREAN",
        "()Ljava/util/Locale;",
        false,
        lazy_locale_korean
    ),
    ne!(
        "Ljava/util/Locale;",
        "CHINESE",
        "()Ljava/util/Locale;",
        false,
        lazy_locale_chinese
    ),
    ne!(
        "Ljava/util/Locale;",
        "SIMPLIFIED_CHINESE",
        "()Ljava/util/Locale;",
        false,
        lazy_locale_simplified_chinese
    ),
    ne!(
        "Ljava/util/Locale;",
        "TRADITIONAL_CHINESE",
        "()Ljava/util/Locale;",
        false,
        lazy_locale_traditional_chinese
    ),
];