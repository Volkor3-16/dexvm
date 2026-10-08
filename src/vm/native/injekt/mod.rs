//! injekt (kohesive) DI host shims.

use super::*;

// ---------------------------------------------------------------------------
// injekt DI (kohesive)
// ---------------------------------------------------------------------------

// injekt DI (kohesive)
// ---------------------------------------------------------------------------

pub(crate) fn injekt_get_injekt(vm: &mut Vm, _args: &[JValue]) -> R {
    alloc(vm, "Luy/kohesive/injekt/api/InjektScope;", Native::Opaque)
}

/// `InjektFactory.getInstance(Type)` — allocates an instance of the concrete
/// type carried by the `java.lang.reflect.Type` argument (which
/// `FullTypeReference.getType()` fills with the receiver's generic
/// signature). Falls back to an Application opaque when the type is unknown.
pub(crate) fn injekt_get_instance(vm: &mut Vm, args: &[JValue]) -> R {
    let (desc, source_class) = match args.get(1).and_then(|t| payload(vm, *t)) {
        Some(Native::Type { desc, source_class }) => (desc.clone(), source_class.clone()),
        _ => ("Landroid/app/Application;".to_string(), String::new()),
    };
    eprintln!("DEBUG injekt_get_instance: desc = '{}', source_class = '{}'", desc, source_class);
    // Special handling for known types that need proper instances
    if desc == "Lkotlinx/serialization/json/Json;" {
        // Return the default Json instance
        return Ok(super::serialization::lazy_json_default_instance(vm));
    }
    // General workaround: if desc is Application but the injekt registry has ANY
    // FullTypeReference subclass that maps to Json, return Json.
    // This handles cases where getType() on a FullTypeReference subclass incorrectly
    // returns Application instead of the actual generic type (Json).
    if desc == "Landroid/app/Application;" {
        eprintln!("DEBUG injekt_get_instance: checking registry with {} entries", vm.injekt_type_by_subclass.len());
        for (sub_id, t_id) in &vm.injekt_type_by_subclass {
            let sub = vm.str_of(*sub_id);
            let t = vm.str_of(*t_id);
            eprintln!("DEBUG injekt_get_instance: registry entry {} -> {}", sub, t);
            if t == "Lkotlinx/serialization/json/Json;" {
                eprintln!("DEBUG injekt_get_instance: FullTypeReference workaround, returning Json for {}", sub);
                return Ok(super::serialization::lazy_json_default_instance(vm));
            }
        }
    }
    alloc(vm, &desc, Native::Opaque)
}

pub(crate) fn injekt_full_type_init(_vm: &mut Vm, _args: &[JValue]) -> R {
    Ok(JValue::Null)
}

/// `FullTypeReference.getType()` — reflects the concrete generic type of the
/// receiver subclass. Two sources, in order:
/// 1. the dex `Signature` annotation, e.g. a subclass
///    `class Lq extends FullTypeReference<Lkotlinx/serialization/json/Json;>`
///    yields `Lkotlinx/serialization/json/Json;`.
/// 2. the bytecode-derived injekt registry (`getInstance` result check-casts),
///    which works even when minification stripped the dex `Signature`
///    annotation.
pub(crate) fn injekt_full_type_get(vm: &mut Vm, args: &[JValue]) -> R {
    let class = match args.first().copied() {
        Some(JValue::Obj(o)) => vm.arena.objects[o as usize].class,
        _ => return alloc(vm, "Ljava/lang/reflect/Type;", Native::Opaque),
    };
    let class_desc = vm.classes[class as usize].descriptor;
    let class_name = vm.str_of(class_desc).to_string();
    // First try the dex generic signature (most authoritative)
    let desc = vm.generic_signature(class).unwrap_or_default();
    if !desc.is_empty() && desc != "Landroid/app/Application;" {
        eprintln!("DEBUG injekt_full_type_get: class={} generic_signature={}", class_name, desc);
        return alloc(vm, "Ljava/lang/reflect/Type;", Native::Type { desc, source_class: class_name.clone() });
    }
    // Fall back to bytecode-derived registry
    if let Some(desc) = vm.injekt_type_of(class_desc) {
        let desc = vm.str_of(desc).to_string();
        if desc != "Landroid/app/Application;" {
            eprintln!("DEBUG injekt_full_type_get: class={} registry={}", class_name, desc);
            return alloc(vm, "Ljava/lang/reflect/Type;", Native::Type { desc, source_class: class_name.clone() });
        }
    }
    // Fallback: if this class is a FullTypeReference subclass (or inherits from it),
    // check the registry for ANY FullTypeReference subclass that maps to a non-Application type
    eprintln!("DEBUG injekt_full_type_get: class={}, checking is_full_type_reference_subclass", class_name);
    if is_full_type_reference_subclass(vm, class) {
        eprintln!("DEBUG injekt_full_type_get: class={} is FullTypeReference subclass, checking registry for Json mapping", class_name);
        for (sub_id, t_id) in &vm.injekt_type_by_subclass {
            let t = vm.str_of(*t_id);
            if t == "Lkotlinx/serialization/json/Json;" {
                let sub = vm.str_of(*sub_id);
                eprintln!("DEBUG injekt_full_type_get: class={} using FullTypeReference sibling {} -> Json", class_name, vm.str_of(*sub_id));
                return alloc(vm, "Ljava/lang/reflect/Type;", Native::Type { desc: "Lkotlinx/serialization/json/Json;".to_string(), source_class: class_name });
            }
        }
    } else {
        eprintln!("DEBUG injekt_full_type_get: class={} is NOT FullTypeReference subclass", class_name);
    }
    // Fallback: return a Type with Application desc (better than Opaque)
    eprintln!("DEBUG injekt_full_type_get: class={} fallback=Application", class_name);
    alloc(vm, "Ljava/lang/reflect/Type;", Native::Type { desc: "Landroid/app/Application;".to_string(), source_class: class_name })
}

/// Checks if a class is a FullTypeReference subclass (direct or indirect).
fn is_full_type_reference_subclass(vm: &Vm, class: u32) -> bool {
    let full_type_ref_desc = "Luy/kohesive/injekt/api/FullTypeReference;";
    let full_type_ref_id = match vm.intern_map.get(full_type_ref_desc) {
        Some(id) => *id,
        None => {
            eprintln!("DEBUG is_full_type_reference_subclass: FullTypeReference not interned");
            return false;
        },
    };
    eprintln!("DEBUG is_full_type_reference_subclass: class={}, full_type_ref_id={}", class, full_type_ref_id);
    let mut current = Some(class);
    while let Some(c) = current {
        eprintln!("DEBUG is_full_type_reference_subclass: checking class={}", c);
        if c == full_type_ref_id {
            eprintln!("DEBUG is_full_type_reference_subclass: FOUND FullTypeReference");
            return true;
        }
        current = vm.classes[c as usize].superclass;
    }
    eprintln!("DEBUG is_full_type_reference_subclass: NOT a FullTypeReference subclass");
    false
}

// ---------------------------------------------------------------------------
// injekt native table
// ---------------------------------------------------------------------------

pub(crate) const INJEKT_TABLE: &[NativeEntry] = &[
    ne!(
        "Luy/kohesive/injekt/InjektKt;",
        "getInjekt",
        "()Luy/kohesive/injekt/api/InjektScope;",
        false,
        injekt_get_injekt
    ),
    ne!(
        "Luy/kohesive/injekt/api/InjektFactory;",
        "getInstance",
        "(Ljava/lang/reflect/Type;)Ljava/lang/Object;",
        true,
        injekt_get_instance
    ),
    ne!(
        "Luy/kohesive/injekt/api/FullTypeReference;",
        "<init>",
        "()V",
        true,
        injekt_full_type_init
    ),
    ne!(
        "Luy/kohesive/injekt/api/FullTypeReference;",
        "getType",
        "()Ljava/lang/reflect/Type;",
        true,
        injekt_full_type_get
    ),
];

#[cfg(test)]
mod tests;
