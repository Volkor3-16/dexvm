//! android.os.Build and Build.VERSION host shims.

use crate::vm::native::*;

// Build.VERSION constants
macro_rules! version_const {
    ($name:ident, $value:expr) => {
        pub fn $name(_vm: &mut Vm, _args: &[JValue]) -> R {
            Ok(new_str(_vm, $value))
        }
    };
}
version_const!(lazy_version_codename, "REL");
version_const!(lazy_version_incremental, "1");
version_const!(lazy_version_release, "14");
version_const!(lazy_version_sdk, "34");
version_const!(lazy_version_sdk_int, "34");
version_const!(lazy_version_security_patch, "2024-01-01");
version_const!(lazy_version_base_os, "");
version_const!(lazy_version_preview_sdk_int, "0");

// Build constants
macro_rules! build_const {
    ($name:ident, $value:expr) => {
        pub fn $name(_vm: &mut Vm, _args: &[JValue]) -> R {
            Ok(new_str(_vm, $value))
        }
    };
}
build_const!(lazy_build_board, "");
build_const!(lazy_build_bootloader, "");
build_const!(lazy_build_brand, "generic");
build_const!(lazy_build_device, "generic");
build_const!(lazy_build_display, "generic");
build_const!(lazy_build_fingerprint, "generic/generic/generic:14/UP1A.231005.007/12345678:user/release-keys");
build_const!(lazy_build_hardware, "generic");
build_const!(lazy_build_host, "build-host");
build_const!(lazy_build_id, "UP1A.231005.007");
build_const!(lazy_build_manufacturer, "generic");
build_const!(lazy_build_model, "generic");
build_const!(lazy_build_product, "generic");
build_const!(lazy_build_serial, "");
build_const!(lazy_build_tags, "release-keys");
build_const!(lazy_build_type, "user");
build_const!(lazy_build_user, "builder");

// ShimLazy wrapper functions (fn(&mut Vm) -> JValue) for static field access
macro_rules! build_const_lazy {
    ($name:ident, $value:expr) => {
        pub fn $name(vm: &mut Vm) -> JValue {
            new_str(vm, $value)
        }
    };
}
build_const_lazy!(lazy_build_board_lazy, "");
build_const_lazy!(lazy_build_bootloader_lazy, "");
build_const_lazy!(lazy_build_brand_lazy, "generic");
build_const_lazy!(lazy_build_device_lazy, "generic");
build_const_lazy!(lazy_build_display_lazy, "generic");
build_const_lazy!(lazy_build_fingerprint_lazy, "generic/generic/generic:14/UP1A.231005.007/12345678:user/release-keys");
build_const_lazy!(lazy_build_hardware_lazy, "generic");
build_const_lazy!(lazy_build_host_lazy, "build-host");
build_const_lazy!(lazy_build_id_lazy, "UP1A.231005.007");
build_const_lazy!(lazy_build_manufacturer_lazy, "generic");
build_const_lazy!(lazy_build_model_lazy, "generic");
build_const_lazy!(lazy_build_product_lazy, "generic");
build_const_lazy!(lazy_build_serial_lazy, "");
build_const_lazy!(lazy_build_tags_lazy, "release-keys");
build_const_lazy!(lazy_build_type_lazy, "user");
build_const_lazy!(lazy_build_user_lazy, "builder");

// ShimLazy wrapper functions for Build.VERSION
macro_rules! version_const_lazy {
    ($name:ident, $value:expr) => {
        pub fn $name(vm: &mut Vm) -> JValue {
            new_str(vm, $value)
        }
    };
}
version_const_lazy!(lazy_version_codename_lazy, "REL");
version_const_lazy!(lazy_version_incremental_lazy, "1");
version_const_lazy!(lazy_version_release_lazy, "14");
version_const_lazy!(lazy_version_sdk_lazy, "34");
version_const_lazy!(lazy_version_sdk_int_lazy, "34");
version_const_lazy!(lazy_version_security_patch_lazy, "2024-01-01");
version_const_lazy!(lazy_version_base_os_lazy, "");
version_const_lazy!(lazy_version_preview_sdk_int_lazy, "0");


// android.os.Build host shims.

/// Native methods for Landroid/os/Build;
pub(crate) const BUILD_TABLE: &[&[NativeEntry]] = &[&[
    // Static field accessors for Build constants
    ne!(
        "Landroid/os/Build;",
        "BOARD",
        "()Ljava/lang/String;",
        false,
        lazy_build_board
    ),
    ne!(
        "Landroid/os/Build;",
        "BOOTLOADER",
        "()Ljava/lang/String;",
        false,
        lazy_build_bootloader
    ),
    ne!(
        "Landroid/os/Build;",
        "BRAND",
        "()Ljava/lang/String;",
        false,
        lazy_build_brand
    ),
    ne!(
        "Landroid/os/Build;",
        "DEVICE",
        "()Ljava/lang/String;",
        false,
        lazy_build_device
    ),
    ne!(
        "Landroid/os/Build;",
        "DISPLAY",
        "()Ljava/lang/String;",
        false,
        lazy_build_display
    ),
    ne!(
        "Landroid/os/Build;",
        "FINGERPRINT",
        "()Ljava/lang/String;",
        false,
        lazy_build_fingerprint
    ),
    ne!(
        "Landroid/os/Build;",
        "HARDWARE",
        "()Ljava/lang/String;",
        false,
        lazy_build_hardware
    ),
    ne!(
        "Landroid/os/Build;",
        "HOST",
        "()Ljava/lang/String;",
        false,
        lazy_build_host
    ),
    ne!(
        "Landroid/os/Build;",
        "ID",
        "()Ljava/lang/String;",
        false,
        lazy_build_id
    ),
    ne!(
        "Landroid/os/Build;",
        "MANUFACTURER",
        "()Ljava/lang/String;",
        false,
        lazy_build_manufacturer
    ),
    ne!(
        "Landroid/os/Build;",
        "MODEL",
        "()Ljava/lang/String;",
        false,
        lazy_build_model
    ),
    ne!(
        "Landroid/os/Build;",
        "PRODUCT",
        "()Ljava/lang/String;",
        false,
        lazy_build_product
    ),
    ne!(
        "Landroid/os/Build;",
        "SERIAL",
        "()Ljava/lang/String;",
        false,
        lazy_build_serial
    ),
    ne!(
        "Landroid/os/Build;",
        "TAGS",
        "()Ljava/lang/String;",
        false,
        lazy_build_tags
    ),
    ne!(
        "Landroid/os/Build;",
        "TYPE",
        "()Ljava/lang/String;",
        false,
        lazy_build_type
    ),
    ne!(
        "Landroid/os/Build;",
        "USER",
        "()Ljava/lang/String;",
        false,
        lazy_build_user
    ),
    // VERSION inner class static fields
    ne!(
        "Landroid/os/Build$VERSION;",
        "CODENAME",
        "()Ljava/lang/String;",
        false,
        lazy_version_codename
    ),
    ne!(
        "Landroid/os/Build$VERSION;",
        "INCREMENTAL",
        "()Ljava/lang/String;",
        false,
        lazy_version_incremental
    ),
    ne!(
        "Landroid/os/Build$VERSION;",
        "RELEASE",
        "()Ljava/lang/String;",
        false,
        lazy_version_release
    ),
    ne!(
        "Landroid/os/Build$VERSION;",
        "SDK",
        "()Ljava/lang/String;",
        false,
        lazy_version_sdk
    ),
    ne!(
        "Landroid/os/Build$VERSION;",
        "SDK_INT",
        "()I",
        false,
        lazy_version_sdk_int
    ),
    ne!(
        "Landroid/os/Build$VERSION;",
        "SECURITY_PATCH",
        "()Ljava/lang/String;",
        false,
        lazy_version_security_patch
    ),
    ne!(
        "Landroid/os/Build$VERSION;",
        "BASE_OS",
        "()Ljava/lang/String;",
        false,
        lazy_version_base_os
    ),
    ne!(
        "Landroid/os/Build$VERSION;",
        "PREVIEW_SDK_INT",
        "()I",
        false,
        lazy_version_preview_sdk_int
    ),
]];