//! eu.kanade.tachiyomi.AppInfo host shim.

use crate::vm::native::*;

/// `AppInfo.INSTANCE` - the singleton instance of AppInfo.
///
/// Object ids live in the owning `Vm` arena, so the value must not be cached
/// across VMs; the shim static slot (`statics`/`statics_lazy`) already caches
/// it per VM after the first `sget-object`.
pub(crate) fn lazy_app_info_instance(vm: &mut Vm) -> JValue {
    let Ok(class_id) = vm.ensure_class_by_desc("Leu/kanade/tachiyomi/AppInfo;") else {
        return JValue::Null;
    };
    JValue::Obj(vm.alloc_object(class_id, Some(Native::Opaque)))
}
