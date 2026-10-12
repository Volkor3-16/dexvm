// Additional Java native tables for extended functionality

use crate::vm::native::*;

// java.lang.CharSequence host shim.
pub(crate) mod charsequence {
    use crate::vm::native::*;

    pub(crate) fn charsequence_length(vm: &mut Vm, args: &[JValue]) -> R {
        let Some(Native::Str(s)) = payload(vm, args[0]) else {
            return Err(npe(vm));
        };
        Ok(JValue::Int(s.len() as i32))
    }

    pub(crate) fn charsequence_char_at(vm: &mut Vm, args: &[JValue]) -> R {
        let Some(Native::Str(s)) = payload(vm, args[0]) else {
            return Err(npe(vm));
        };
        let index = int_of(vm, args[1]);
        if index < 0 || index >= s.len() as i32 {
            return Err(iae(vm, format!("index {} out of bounds for length {}", index, s.len())));
        }
        let ch = s.chars().nth(index as usize).unwrap_or('\0');
        Ok(JValue::Int(ch as u32 as i32))
    }

    pub(crate) fn charsequence_subsequence(vm: &mut Vm, args: &[JValue]) -> R {
        let Some(Native::Str(s)) = payload(vm, args[0]) else {
            return Err(npe(vm));
        };
        let start = int_of(vm, args[1]);
        let end = int_of(vm, args[2]);
        if start < 0 || end > s.len() as i32 || start > end {
            return Err(iae(vm, "invalid subsequence range"));
        }
        // Extract the substring as an owned String to avoid borrowing issues
        let substring = s[start as usize..end as usize].to_string();
        Ok(new_str(vm, &substring))
    }

    pub(crate) fn charsequence_to_string(vm: &mut Vm, args: &[JValue]) -> R {
        let Some(Native::Str(s)) = payload(vm, args[0]) else {
            return Err(npe(vm));
        };
        // Clone the string data to avoid borrowing issues
        let string_data = s.clone();
        Ok(new_str(vm, &string_data))
    }

    pub(crate) const CHARSEQUENCE_TABLE: &[NativeEntry] = &[
        ne!(
            "Ljava/lang/CharSequence;",
            "length",
            "()I",
            true,
            charsequence_length
        ),
        ne!(
            "Ljava/lang/CharSequence;",
            "charAt",
            "(I)C",
            true,
            charsequence_char_at
        ),
        ne!(
            "Ljava/lang/CharSequence;",
            "subSequence",
            "(II)Ljava/lang/CharSequence;",
            true,
            charsequence_subsequence
        ),
        ne!(
            "Ljava/lang/CharSequence;",
            "toString",
            "()Ljava/lang/String;",
            true,
            charsequence_to_string
        ),
    ];
}

// java.lang.Iterable host shim.
pub(crate) mod iterable {
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
            Some(Native::Iter(IterKind::List {
                list: obj,
                idx: 0,
            })),
        );
        Ok(JValue::Obj(iterator_id))
    }

    pub(crate) const ITERABLE_TABLE: &[NativeEntry] = &[
        ne!(
            "Ljava/lang/Iterable;",
            "iterator",
            "()Ljava/util/Iterator;",
            true,
            iterable_iterator
        ),
    ];
}

// java.lang.ClassValue host shim
pub(crate) mod classvalue {
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
}
