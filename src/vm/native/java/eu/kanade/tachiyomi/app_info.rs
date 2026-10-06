//! eu.kanade.tachiyomi.AppInfo host shim.

use crate::vm::native::*;

/// `AppInfo.INSTANCE` - the singleton instance of AppInfo.
pub(crate) fn lazy_app_info_instance(vm: &mut Vm) -> JValue {
    let Ok(class_id) = vm.ensure_class_by_desc("Leu/kanade/tachiyomi/AppInfo;") else {
        return JValue::Null;
    };

    // Return a singleton instance. For our purposes, we can just allocate
    // an opaque object and cache it as the INSTANCE.
    // We'll use a static to ensure we return the same instance.
    static INSTANCE: std::sync::OnceLock<JValue> = std::sync::OnceLock::new();
    *INSTANCE.get_or_init(|| {
        let obj_id = vm.arena.alloc(class_id, Vec::new(), Some(Native::Opaque));
        JValue::Obj(obj_id)
    })
}
