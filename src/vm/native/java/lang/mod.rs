#[allow(unused_imports)]
use crate::vm::native::{iae, int_of, new_str, npe, payload, NativeEntry};

mod boolean;
mod byte;
mod character;
mod charsequence;
mod class;
mod classloader;
mod classvalue;
mod double;
mod r#enum;
mod float;
mod integer;
mod iterable;
mod long;
mod math;
mod number;
mod object;
mod reflect;
mod short;
mod soft_reference;
mod string;
mod stringbuilder;
mod system;
mod thread;
mod throwable;

pub(crate) use boolean::*;
pub(crate) use class::*;
pub(crate) use integer::*;
pub(crate) use string::*;
pub(crate) use throwable::*;

pub(crate) use charsequence::CHARSEQUENCE_TABLE;
pub(crate) use classvalue::CLASSVALUE_TABLE;
pub(crate) use iterable::ITERABLE_TABLE;

/// All java.lang native tables, grouped for `register`.
pub(crate) const LANG_TABLE: &[&[NativeEntry]] = &[
    object::TABLE,
    string::TABLE,
    stringbuilder::TABLE,
    class::TABLE,
    classloader::TABLE,
    reflect::TABLE,
    r#enum::TABLE,
    throwable::TABLE,
    system::TABLE,
    thread::TABLE,
    math::TABLE,
    number::TABLE,
    integer::TABLE,
    long::TABLE,
    short::TABLE,
    byte::TABLE,
    character::TABLE,
    boolean::TABLE,
    float::TABLE,
    double::TABLE,
    soft_reference::TABLE,
    CHARSEQUENCE_TABLE,
    ITERABLE_TABLE,
    CLASSVALUE_TABLE,
];
