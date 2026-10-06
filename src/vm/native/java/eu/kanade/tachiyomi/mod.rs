use crate::vm::native::NativeEntry;

pub(crate) mod app_info;

pub(crate) use app_info::lazy_app_info_instance;

pub(crate) const TABLE: &[NativeEntry] = &[];
