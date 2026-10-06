//! eu.kanade.tachiyomi.AppInfo host shim.

use crate::vm::native::*;

/// `AppInfo.INSTANCE` - the singleton instance of AppInfo.
pub(crate) fn lazy_app_info_instance(_vm: &mut Vm) -> JValue {
    let Ok(class) = _vm.ensure_class_by_desc("Leu/kanade/tachiyomi/AppInfo;") else {
        return JValue::Null;
    };
    // Return a singleton instance. For our purposes, we can just allocate
    // an opaque object and cache it as the INSTANCE.
    // We'll use a static to ensure we return the same instance.
    static INSTANCE: std::sync::OnceLock<JValue> = std::sync::OnceLock::new();
    *INSTANCE.get_or_init(|| {
        let class_id = _vm.ensure_class_by_desc("Leu/kanade/tachiyomi/AppInfo;").unwrap();
        _vm.arena.alloc(class_id, Vec::new(), Some(Native::Opaque)).unwrap()
    })
}

/// Native methods for Leu/kanade/tachiyomi/AppInfo;
pub(crate) const TABLE: &[NativeEntry] = &[
    ne!(
        "Leu/kanade/tachiyomi/AppInfo;",
        "INSTANCE",
        "()Leu/kanade/tachiyomi/AppInfo;",
        false,
        lazy_app_info_instance
    ),
];