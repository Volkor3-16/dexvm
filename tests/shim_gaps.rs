//! Tests for the named shim gaps: each class and member the corpus replay
//! could not resolve now resolves, and reading or calling it through the VM
//! gives the value the bytecode expects.

#![cfg(all(feature = "android", feature = "okhttp"))]

use dexvm::vm::object::Native;
use dexvm::{vm::FieldRef, Context, JValue, SandboxOptions, Vm};

fn with_vm<T>(f: impl FnOnce(&mut Vm) -> T) -> T {
    let data = std::fs::read("fixtures/classes.dex").unwrap();
    let mut ctx = Context::new_with(&data, SandboxOptions::allow_all()).unwrap();
    f(ctx.vm())
}

/// Read a static field the way `sget-object` does: by declaring descriptor,
/// name and type.
fn static_of(vm: &mut Vm, class: &str, name: &str, ty: &str) -> JValue {
    let fr = FieldRef {
        name: vm.intern(name),
        ty: vm.intern(ty),
        class_desc: vm.intern(class),
    };
    vm.static_field_get(fr)
        .unwrap_or_else(|e| panic!("read static field {name} on {class}: {e}"))
}

fn obj_of(v: JValue, what: &str) -> u32 {
    match v {
        JValue::Obj(id) => id,
        other => panic!("expected an object for {what}, got {other:?}"),
    }
}

fn str_payload(vm: &Vm, v: JValue, what: &str) -> String {
    match vm.payload_of(v) {
        Some(Native::Str(s)) => s,
        _ => panic!("{what} must carry a java.lang.String payload"),
    }
}

/// `MultipartBody.FORM` / `STRICT_FORM` are okhttp3's `MediaType` constants,
/// read with `sget-object`.
#[test]
fn multipart_body_media_type_constants_are_static_fields() {
    with_vm(|vm| {
        for (name, want) in [
            ("FORM", "multipart/form-data"),
            ("STRICT_FORM", "application/x-www-form-urlencoded"),
        ] {
            let value = static_of(vm, "Lokhttp3/MultipartBody;", name, "Lokhttp3/MediaType;");
            let dotted = vm.class_desc_str(vm.object_class(value).unwrap());
            assert_eq!(dotted, "okhttp3.MediaType", "{name} must be a MediaType");
            let got = str_payload(vm, value, name);
            assert_eq!(got, want);
            // The type also answers through the method the extensions call.
            let media = vm
                .invoke_virtual(value, "mediaType", "()Ljava/lang/String;")
                .expect("MediaType.mediaType()");
            let got = str_payload(vm, media, "mediaType()");
            assert_eq!(got, want);
        }
    });
}

/// `android.os.Build` and `android.os.Build.VERSION` fields, including the
/// nested class the replay could not even name.
#[test]
fn build_and_build_version_fields_read() {
    with_vm(|vm| {
        let model = static_of(vm, "Landroid/os/Build;", "MODEL", "Ljava/lang/String;");
        assert_eq!(str_payload(vm, model, "Build.MODEL"), "generic");
        let sdk = static_of(vm, "Landroid/os/Build$VERSION;", "SDK_INT", "I");
        assert_eq!(sdk, JValue::Int(34), "Build.VERSION.SDK_INT");
        let release = static_of(
            vm,
            "Landroid/os/Build$VERSION;",
            "RELEASE",
            "Ljava/lang/String;",
        );
        assert_eq!(str_payload(vm, release, "Build.VERSION.RELEASE"), "14");
    });
}

/// A read/write lock and its two views: taking a view, locking it, and
/// releasing it all resolve and run.
#[test]
fn read_write_lock_views_lock_and_unlock() {
    with_vm(|vm| {
        let rw_id = vm
            .ensure_class_by_desc("Ljava/util/concurrent/locks/ReentrantReadWriteLock;")
            .expect("register ReentrantReadWriteLock");
        let read_id = vm
            .ensure_class_by_desc("Ljava/util/concurrent/locks/ReentrantReadWriteLock$ReadLock;")
            .expect("register ReadLock view");
        let write_id = vm
            .ensure_class_by_desc("Ljava/util/concurrent/locks/ReentrantReadWriteLock$WriteLock;")
            .expect("register WriteLock view");
        assert_eq!(
            vm.class_desc_str(read_id),
            "java.util.concurrent.locks.ReentrantReadWriteLock$ReadLock"
        );
        assert_eq!(
            vm.class_desc_str(write_id),
            "java.util.concurrent.locks.ReentrantReadWriteLock$WriteLock"
        );

        let rw = JValue::Obj(vm.alloc_object(rw_id, Some(Native::Opaque)));
        let read = vm
            .invoke_virtual(
                rw,
                "readLock",
                "()Ljava/util/concurrent/locks/ReentrantReadWriteLock$ReadLock;",
            )
            .expect("readLock()");
        assert_eq!(
            vm.class_desc_str(vm.object_class(read).unwrap()),
            "java.util.concurrent.locks.ReentrantReadWriteLock$ReadLock"
        );
        vm.invoke_virtual(read, "lock", "()V")
            .expect("readLock.lock()");
        let held = vm
            .invoke_virtual(read, "tryLock", "()Z")
            .expect("readLock.tryLock()");
        assert_eq!(held, JValue::Int(1), "a view never contends in this VM");
        vm.invoke_virtual(read, "unlock", "()V")
            .expect("readLock.unlock()");
        let write = vm
            .invoke_virtual(
                rw,
                "writeLock",
                "()Ljava/util/concurrent/locks/ReentrantReadWriteLock$WriteLock;",
            )
            .expect("writeLock()");
        vm.invoke_virtual(write, "lock", "()V")
            .expect("writeLock.lock()");
        let held = vm
            .invoke_virtual(rw, "getReadHoldCount", "()I")
            .expect("getReadHoldCount()");
        assert_eq!(held, JValue::Int(0), "the write view holds no read count");
    });
}

/// `Class.getClassifier` is how Kotlin reaches a `KClassifier` from
/// `javaClass`; it answers with the same class handle.
#[test]
fn class_getclassifier_answers_with_the_class_itself() {
    with_vm(|vm| {
        let class_id = vm
            .ensure_class_by_desc("Ljava/lang/Class;")
            .expect("register java.lang.Class");
        let cls = JValue::Obj(vm.alloc_object(class_id, None));
        let k = vm
            .invoke_virtual(cls, "getClassifier", "()Lkotlin/reflect/KClassifier;")
            .expect("Class.getClassifier()");
        let k_id = obj_of(k, "getClassifier()");
        assert_eq!(
            vm.object_class(k),
            vm.object_class(cls),
            "the classifier is the class handle itself"
        );
        let _ = k_id;
    });
}

/// `Headers.Companion` as a static field, and the marker interfaces the bundled
/// kotlinx collections implement.
#[test]
fn headers_companion_field_and_kotlin_markers_resolve() {
    with_vm(|vm| {
        let companion = static_of(
            vm,
            "Lokhttp3/Headers;",
            "Companion",
            "Lokhttp3/Headers$Companion;",
        );
        assert_eq!(
            vm.class_desc_str(vm.object_class(companion).unwrap()),
            "okhttp3.Headers$Companion"
        );
        for desc in [
            "Lkotlin/jvm/internal/markers/KMappedMarker;",
            "Lkotlin/jvm/internal/markers/KMutableList;",
        ] {
            vm.ensure_class_by_desc(desc)
                .unwrap_or_else(|e| panic!("register {desc}: {e}"));
        }
    });
}

/// The class-hierarchy members the replay names once class loading stops failing
/// silently: `java.lang.annotation.Annotation` is the declared superclass of
/// every annotation class in the bundled kotlinx/protobuf tables, and the other
/// three are interfaces an extension implements or is passed. Each must resolve
/// as the interface it is, with the supertype the platform API declares.
#[test]
fn annotation_and_marker_interfaces_resolve_as_interfaces() {
    with_vm(|vm| {
        let annotation = vm
            .ensure_class_by_desc("Ljava/lang/annotation/Annotation;")
            .expect("register java.lang.annotation.Annotation");
        let element = vm
            .ensure_class_by_desc("Ljava/lang/reflect/AnnotatedElement;")
            .expect("register java.lang.reflect.AnnotatedElement");
        assert_eq!(
            vm.classes[annotation as usize].interfaces,
            vec![element],
            "Annotation declares AnnotatedElement"
        );

        for (desc, dotted) in [
            (
                "Ljava/lang/annotation/Annotation;",
                "java.lang.annotation.Annotation",
            ),
            (
                "Ljavax/net/ssl/HostnameVerifier;",
                "javax.net.ssl.HostnameVerifier",
            ),
            (
                "Leu/kanade/tachiyomi/source/UnmeteredSource;",
                "eu.kanade.tachiyomi.source.UnmeteredSource",
            ),
            (
                "Landroid/app/Application$ActivityLifecycleCallbacks;",
                "android.app.Application$ActivityLifecycleCallbacks",
            ),
        ] {
            let class = vm
                .ensure_class_by_desc(desc)
                .unwrap_or_else(|e| panic!("register {desc}: {e}"));
            assert_eq!(vm.class_desc_str(class), dotted);
            assert!(
                vm.classes[class as usize].is_interface,
                "{dotted} is an interface"
            );
        }
    });
}

/// `okhttp3.Protocol.HTTP_1_1` and friends are read with `sget-object`, so they
/// have to be static fields carrying real enum values.
#[test]
fn okhttp_protocol_constants_read_as_enum_values() {
    with_vm(|vm| {
        let protocol = vm
            .ensure_class_by_desc("Lokhttp3/Protocol;")
            .expect("register okhttp3.Protocol");
        let java_enum = vm
            .ensure_class_by_desc("Ljava/lang/Enum;")
            .expect("register java.lang.Enum");
        assert_eq!(
            vm.classes[protocol as usize].superclass,
            Some(java_enum),
            "Protocol is a java.lang.Enum"
        );

        for (name, ordinal) in [
            ("HTTP_1_0", 0),
            ("HTTP_1_1", 1),
            ("SPDY_3", 2),
            ("HTTP_2", 3),
            ("CLEARTEXT", 4),
        ] {
            let value = static_of(vm, "Lokhttp3/Protocol;", name, "Lokhttp3/Protocol;");
            assert_eq!(
                vm.class_desc_str(vm.object_class(value).unwrap()),
                "okhttp3.Protocol",
                "Protocol.{name} is a Protocol"
            );
            match vm.payload_of(value) {
                Some(Native::Enum {
                    name: n,
                    ordinal: o,
                }) => {
                    assert_eq!(n, name, "Protocol.{name} carries its own name");
                    assert_eq!(o, ordinal, "Protocol.{name} keeps its ordinal");
                }
                _ => panic!("Protocol.{name} must carry an enum payload"),
            }
        }
    });
}
