mod build;

pub(crate) use build::BUILD_TABLE;
pub(crate) use build::{
    lazy_build_board, lazy_build_bootloader, lazy_build_brand, lazy_build_device,
    lazy_build_display, lazy_build_fingerprint, lazy_build_hardware, lazy_build_host,
    lazy_build_id, lazy_build_manufacturer, lazy_build_model, lazy_build_product,
    lazy_build_serial, lazy_build_tags, lazy_build_type, lazy_build_user,
    lazy_version_codename, lazy_version_incremental, lazy_version_release,
    lazy_version_sdk, lazy_version_sdk_int, lazy_version_security_patch,
    lazy_version_base_os, lazy_version_preview_sdk_int,
    // Lazy functions for shim static fields
    lazy_build_board_lazy, lazy_build_bootloader_lazy, lazy_build_brand_lazy,
    lazy_build_device_lazy, lazy_build_display_lazy, lazy_build_fingerprint_lazy,
    lazy_build_hardware_lazy, lazy_build_host_lazy, lazy_build_id_lazy,
    lazy_build_manufacturer_lazy, lazy_build_model_lazy, lazy_build_product_lazy,
    lazy_build_serial_lazy, lazy_build_tags_lazy, lazy_build_type_lazy,
    lazy_build_user_lazy,
    lazy_version_codename_lazy, lazy_version_incremental_lazy, lazy_version_release_lazy,
    lazy_version_sdk_lazy, lazy_version_sdk_int_lazy, lazy_version_security_patch_lazy,
    lazy_version_base_os_lazy, lazy_version_preview_sdk_int_lazy,
};