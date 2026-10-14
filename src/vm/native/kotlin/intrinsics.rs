//! Kotlin JVM intrinsic helpers.
use crate::vm::native::*;

fn are_equal(vm: &mut Vm, args: &[JValue]) -> R {
    Ok(JValue::Int(i32::from(java_equals(vm, args[0], args[1])?)))
}
fn check_not_null_parameter(vm: &mut Vm, args: &[JValue]) -> R {
    if args[0].is_null_ref() {
        let name = jstr(vm, args[1]).unwrap_or_else(|_| "parameter".into());
        // The name names the caller's parameter, so the caller is what needs
        // diagnosing: record which bytecode handed us the null.
        log::warn!(
            "INV checkNotNullParameter(\"{name}\") got null from {}",
            vm.frame_chain(4)
        );
        return Err(NatErr::Throw(vm.throwable_of(
            "Ljava/lang/NullPointerException;",
            format!("{name} must not be null"),
        )));
    }
    Ok(JValue::Null)
}
fn check_not_null_expression_value(vm: &mut Vm, args: &[JValue]) -> R {
    if args[0].is_null_ref() {
        let expression = jstr(vm, args[1]).unwrap_or_else(|_| "expression".into());
        return Err(NatErr::Throw(vm.throwable_of(
            "Ljava/lang/NullPointerException;",
            format!("Expression \"{expression}\" is null"),
        )));
    }
    Ok(JValue::Null)
}

/// `Intrinsics.checkNotNull(value)` — the single-argument form the compiler
/// emits for a platform-type dereference.
fn check_not_null(vm: &mut Vm, _args: &[JValue]) -> R {
    if _args[0].is_null_ref() {
        return Err(NatErr::Throw(vm.throwable_of(
            "Ljava/lang/NullPointerException;",
            "Value should not be null".to_string(),
        )));
    }
    Ok(JValue::Null)
}

pub(super) fn throw_uninitialized(vm: &mut Vm, args: &[JValue]) -> R {
    let name = jstr(vm, args[0]).unwrap_or_else(|_| "property".into());
    Err(NatErr::Throw(vm.throwable_of(
        "Lkotlin/UninitializedPropertyAccessException;",
        format!("lateinit property {name} has not been initialized"),
    )))
}
pub(crate) const TABLE: &[NativeEntry] = &[
    ne!(
        "Lkotlin/jvm/internal/Intrinsics;",
        "areEqual",
        "(Ljava/lang/Object;Ljava/lang/Object;)Z",
        false,
        are_equal
    ),
    ne!(
        "Lkotlin/jvm/internal/Intrinsics;",
        "checkNotNullParameter",
        "(Ljava/lang/Object;Ljava/lang/String;)V",
        false,
        check_not_null_parameter
    ),
    ne!(
        "Lkotlin/jvm/internal/Intrinsics;",
        "checkNotNullExpressionValue",
        "(Ljava/lang/Object;Ljava/lang/String;)V",
        false,
        check_not_null_expression_value
    ),
    ne!(
        "Lkotlin/jvm/internal/Intrinsics;",
        "checkNotNull",
        "(Ljava/lang/Object;)V",
        false,
        check_not_null
    ),
    ne!(
        "Lkotlin/jvm/internal/Intrinsics;",
        "throwUninitializedPropertyAccessException",
        "(Ljava/lang/String;)V",
        false,
        throw_uninitialized
    ),
];
