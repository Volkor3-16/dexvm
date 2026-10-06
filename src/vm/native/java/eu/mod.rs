use crate::vm::native::NativeEntry;

pub(crate) mod kanade;

pub(crate) const TABLE: &[&[NativeEntry]] = &[kanade::TACHIYOMI_TABLE];
