use crate::vm::native::NativeEntry;

pub(crate) mod kanade;

pub(crate) use kanade::lazy_app_info_instance;

pub(crate) const TABLE: &[&[NativeEntry]] = &[kanade::TACHIYOMI_TABLE];
