//! java.lang.ClassValue host shim

use crate::vm::native::*;

pub(crate) fn classvalue_init(_vm: &mut Vm, _args: &[JValue]) -> R {
    // ClassValue constructor does nothing special
    Ok(JValue::Null)
}

pub(crate) fn classvalue_compute_value(_vm: &mut Vm, _args: &[JValue]) -> R {
    // In a real implementation, this would compute the value for the class
    // For now, return a default value
    Ok(JValue::Null)
}

pub(crate) const CLASSVALUE_TABLE: &[NativeEntry] = &[
    ne!(
        "Ljava/lang/ClassValue;",
        "<init>",
        "()V",
        true,
        classvalue_init
    ),
    ne!(
        "Ljava/lang/ClassValue;",
        "computeValue",
        "(Ljava/lang/Class;)Ljava/lang/Object;",
        true,
        classvalue_compute_value
    ),
];
