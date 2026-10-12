//! java.lang.CharSequence host shim.

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
        return Err(iae(
            vm,
            format!("index {} out of bounds for length {}", index, s.len()),
        ));
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
