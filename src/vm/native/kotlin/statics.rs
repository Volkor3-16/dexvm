//! Lazy materializers for Kotlin singleton and companion fields.
use super::support::opaque_inst;
use crate::vm::native::*;

pub(crate) fn duration_companion(vm: &mut Vm) -> JValue {
    opaque_inst(vm, "Lkotlin/time/Duration$Companion;")
}
pub(crate) fn string_companion(vm: &mut Vm) -> JValue {
    opaque_inst(vm, "Lkotlin/jvm/internal/StringCompanionObject;")
}
pub(crate) fn duration_unit_seconds(vm: &mut Vm) -> JValue {
    opaque_inst(vm, "Lkotlin/time/DurationUnit;")
}
pub(crate) fn clock_system_instance(vm: &mut Vm) -> JValue {
    opaque_inst(vm, "Lkotlin/time/Clock$System;")
}
pub(crate) fn duration_unit_hours(vm: &mut Vm) -> JValue {
    opaque_inst(vm, "Lkotlin/time/DurationUnit;")
}
pub(crate) fn duration_unit_days(vm: &mut Vm) -> JValue {
    opaque_inst(vm, "Lkotlin/time/DurationUnit;")
}
pub(crate) fn duration_unit_millis(vm: &mut Vm) -> JValue {
    opaque_inst(vm, "Lkotlin/time/DurationUnit;")
}
pub(crate) fn unit_instance(vm: &mut Vm) -> JValue {
    opaque_inst(vm, "Lkotlin/Unit;")
}
pub(crate) fn global_scope(vm: &mut Vm) -> JValue {
    opaque_inst(vm, "Lkotlinx/coroutines/GlobalScope;")
}
pub(crate) fn result_companion(vm: &mut Vm) -> JValue {
    opaque_inst(vm, "Lkotlin/Result$Companion;")
}
/// `kotlin.random.Random.Default` (the `companion object Default : Random()`).
pub(crate) fn random_default(vm: &mut Vm) -> JValue {
    opaque_inst(vm, "Lkotlin/random/Random$Default;")
}
/// `kotlin.time.Instant.Companion`.
pub(crate) fn kotlin_instant_companion(vm: &mut Vm) -> JValue {
    opaque_inst(vm, "Lkotlin/time/Instant$Companion;")
}
