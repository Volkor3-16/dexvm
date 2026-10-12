//! java.lang.Iterable host shim.

use crate::vm::native::*;

pub(crate) fn iterable_iterator(vm: &mut Vm, args: &[JValue]) -> R {
    let obj = match args[0] {
        JValue::Obj(obj) => obj,
        _ => return Err(npe(vm)),
    };
    let Some(Native::List(_items)) = payload(vm, JValue::Obj(obj)) else {
        return Err(npe(vm));
    };
    let iterator_id = vm.arena.alloc(
        0, // class will be set by caller
        Vec::new(),
        Some(Native::Iter(IterKind::List { list: obj, idx: 0 })),
    );
    Ok(JValue::Obj(iterator_id))
}

pub(crate) const ITERABLE_TABLE: &[NativeEntry] = &[ne!(
    "Ljava/lang/Iterable;",
    "iterator",
    "()Ljava/util/Iterator;",
    true,
    iterable_iterator
)];
