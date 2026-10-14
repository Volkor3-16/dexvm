//! Compatibility contract tests: the guarantees a run owes the host, independent
//! of any particular site.
//!
//! Everything here is offline and deterministic — the HTTP layer is canned, so a
//! failure is always a code failure, never a network flake.
//!
//! 1. The suspend/coroutine bridge returns parsed data instead of failing the
//!    call with a resolution error, for both extension generations.
//! 2. A native argument that is a DEX null reference becomes a catchable
//!    `java.lang.NullPointerException`, not a Rust panic in the host.
//! 3. A host native that panics is absorbed, named, and counted: the current
//!    source fails, the process and the VM keep running.
//! 4. Shims read the argument the platform API actually passes them.

use std::sync::Once;

use dexvm::context::{Context, SandboxOptions};
use dexvm::keiyoushi::{HttpData, HttpResp, Keiyoushi};
use dexvm::vm::error::JvmError;
use dexvm::vm::native::{native_panic_count, register_global, want_arg_obj};
use dexvm::vm::object::Native;
use dexvm::vm::value::JValue;
use dexvm::vm::{NatErr, NativeEntry, Vm};

const AKUMA: &str = "fixtures/tachiyomi-all.akuma-v1.4.10.apk";
const MOETRUYEN: &str = "fixtures/tachiyomi-vi.moetruyen-v1.6.8.apk";

/// Classic (pre-1.6) generation response shape.
const AKUMA_POPULAR_HTML: &str = r#"<html><body>
<div class="post-loop">
<ul>
<li>
  <a href="/manga/one-piece-2"><div class="cover"><img src="/img/one.jpg"></div></a>
  <div class="overlay-title">One Piece</div>
</li>
<li>
  <a href="/manga/boruto-ng"><div class="cover"><img src="/img/boruto.jpg"></div></a>
  <div class="overlay-title">Boruto Next Generations</div>
</li>
</ul>
<nav class="page-nav"><a rel="prev">1</a></nav>
</div>
</body></html>"#;

/// Suspend (1.6+) generation response shape.
const MOETRUYEN_POPULAR_HTML: &str = r#"<html><body>
<ol class="homepage-ranking-list" data-ranking-period="total">
<li>
  <a class="homepage-ranking-item__link" href="/truyen/one-piece">
    <div class="homepage-ranking-item__title">One Piece</div>
    <img src="/img/one.png"/>
  </a>
</li>
</ol>
</body></html>"#;

fn canned(body: &str) -> impl Fn(&HttpData) -> HttpResp + 'static {
    let body = body.to_string();
    move |_| HttpResp::ok(body.clone())
}

fn init_logger() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("off")).init();
    });
}

/// Load an extension APK into a plain [`Context`] (no keiyoushi bridge).
fn context(apk: &str) -> Context {
    let data = std::fs::read(apk).expect("fixture apk");
    Context::new_with(&data, SandboxOptions::default()).expect("context")
}

/// The id of a fresh java.lang.String instance.
fn string_id(ctx: &mut Context, s: &str) -> u32 {
    match ctx.vm().alloc_string(s) {
        JValue::Obj(id) => id,
        other => panic!("alloc_string must yield an object, got {other:?}"),
    }
}

/// The class descriptor of a thrown throwable object.
fn throwable_class(ctx: &mut Context, id: u32) -> String {
    let class = {
        let vm = ctx.vm();
        vm.arena
            .objects
            .get(id as usize)
            .unwrap_or_else(|| panic!("throwable {id} is not in the arena"))
            .class
    };
    ctx.vm().class_desc_str(class).to_string()
}

// ---------------------------------------------------------------------------
// 1. the suspend bridge resolves to data, not to a resolution error
// ---------------------------------------------------------------------------

#[test]
fn coro_bridge_returns_data_for_the_classic_generation() {
    init_logger();
    let mut ext = Keiyoushi::open(AKUMA).expect("open akuma fixture");
    ext.set_http(canned(AKUMA_POPULAR_HTML));
    let srcs = ext.sources().expect("sources");
    let src = &srcs[0];

    let popular = ext
        .popular_coro(src, 1)
        .unwrap_or_else(|e| panic!("popular_coro: {}", ext.describe_error(&e)));
    assert_eq!(popular.mangas.len(), 2, "two <li> in canned html");
    assert_eq!(popular.mangas[0].title, "One Piece");

    let manga = &popular.mangas[0];
    // The 1.6-era suspend entry points must complete for a classic extension too
    // (they resolve through the host-default shim) instead of failing resolution.
    let found = ext
        .search_coro(src, 1, "one piece", &[])
        .unwrap_or_else(|e| panic!("search_coro: {}", ext.describe_error(&e)));
    assert_eq!(found.mangas.len(), 2, "search replays the same canned page");

    let chapters = ext
        .chapters(src, manga)
        .unwrap_or_else(|e| panic!("chapters: {}", ext.describe_error(&e)));
    eprintln!("compat: akuma parsed {} chapters", chapters.len());
    if let Some(ch) = chapters.first() {
        let pages = ext
            .pages_coro(src, ch)
            .unwrap_or_else(|e| panic!("pages_coro: {}", ext.describe_error(&e)));
        eprintln!("compat: akuma pages_coro parsed {} pages", pages.len());
    }
}

#[test]
fn coro_bridge_returns_data_for_the_suspend_generation() {
    init_logger();
    let mut ext = Keiyoushi::open(MOETRUYEN).expect("open moetruyen fixture");
    ext.set_http(canned(MOETRUYEN_POPULAR_HTML));
    let srcs = ext.sources().expect("sources");
    let src = &srcs[0];

    let popular = ext
        .popular_coro(src, 1)
        .unwrap_or_else(|e| panic!("popular_coro: {}", ext.describe_error(&e)));
    assert_eq!(popular.mangas.len(), 1, "one ranking entry in canned html");
    assert_eq!(popular.mangas[0].title, "One Piece");
}

// ---------------------------------------------------------------------------
// host natives used by the argument/panic contract tests
// ---------------------------------------------------------------------------

/// A host native that reads its argument the sanctioned way: a DEX null
/// reference (`Null`, or the `Int(0)` a `const 0` leaves behind) becomes a
/// catchable `java.lang.NullPointerException`.
fn probe_null_arg(vm: &mut Vm, args: &[JValue]) -> Result<JValue, NatErr> {
    let id = want_arg_obj(vm, args, 1)?;
    Ok(vm.alloc_string(&format!("got {id}")))
}

/// A host native that panics. It must not take the process down.
fn probe_boom(_vm: &mut Vm, _args: &[JValue]) -> Result<JValue, NatErr> {
    panic!("boom from a compat host native");
}

/// A host native that indexes an argument list that is too short.
fn probe_short_args(vm: &mut Vm, args: &[JValue]) -> Result<JValue, NatErr> {
    let id = want_arg_obj(vm, args, 3)?;
    Ok(vm.alloc_string(&format!("got {id}")))
}

static COMPAT_TABLE: &[NativeEntry] = &[
    NativeEntry {
        class: "Ljava/lang/String;",
        name: "compatNullArg",
        sig: "(Ljava/lang/Object;)Ljava/lang/String;",
        instance: true,
        f: probe_null_arg,
    },
    NativeEntry {
        class: "Ljava/lang/String;",
        name: "compatBoom",
        sig: "()Ljava/lang/String;",
        instance: true,
        f: probe_boom,
    },
    NativeEntry {
        class: "Ljava/lang/String;",
        name: "compatShortArgs",
        sig: "()Ljava/lang/String;",
        instance: true,
        f: probe_short_args,
    },
];

fn register_compat_natives() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| register_global(COMPAT_TABLE));
}

// ---------------------------------------------------------------------------
// 2. a null argument is a Java exception, not a host panic
// ---------------------------------------------------------------------------

#[test]
fn null_argument_becomes_a_catchable_null_pointer_exception() {
    init_logger();
    register_compat_natives();
    let mut ctx = context(AKUMA);
    let id = string_id(&mut ctx, "compat");

    // `const 0` in an object register: the value a DEX null argument carries.
    let err = ctx
        .invoke_on(
            id,
            "compatNullArg",
            "(Ljava/lang/Object;)Ljava/lang/String;",
            &[JValue::Int(0)],
        )
        .expect_err("a null argument must not be accepted");
    let JvmError::Uncaught(thrown) = err else {
        panic!("expected an uncaught java throwable, got {err:?}");
    };
    assert_eq!(
        throwable_class(&mut ctx, thrown),
        "java.lang.NullPointerException",
        "a null argument must raise the exception real Java raises"
    );
}

/// A missing argument is `ArrayIndexOutOfBoundsException`, not an out-of-bounds
/// read of the host's argument slice.
#[test]
fn short_argument_list_becomes_a_catchable_index_out_of_bounds_exception() {
    init_logger();
    register_compat_natives();
    let mut ctx = context(AKUMA);
    let id = string_id(&mut ctx, "compat");

    let err = ctx
        .invoke_on(id, "compatShortArgs", "()Ljava/lang/String;", &[])
        .expect_err("a missing argument must not be accepted");
    let JvmError::Uncaught(thrown) = err else {
        panic!("expected an uncaught java throwable, got {err:?}");
    };
    assert_eq!(
        throwable_class(&mut ctx, thrown),
        "java.lang.ArrayIndexOutOfBoundsException",
        "a short argument list must raise the exception real Java raises"
    );
}

// ---------------------------------------------------------------------------
// 3. a panicking native is absorbed, named, counted — the run survives
// ---------------------------------------------------------------------------

#[test]
fn panicking_native_is_absorbed_named_and_counted() {
    init_logger();
    register_compat_natives();
    let mut ctx = context(AKUMA);
    let id = string_id(&mut ctx, "compat");

    let before = native_panic_count();
    let err = ctx
        .invoke_on(id, "compatBoom", "()Ljava/lang/String;", &[])
        .expect_err("a panicking native must be reported, not swallowed silently");
    let JvmError::Fatal(msg) = &err else {
        panic!("expected a fatal error naming the native, got {err:?}");
    };
    assert!(msg.contains("native panic"), "{msg}");
    assert!(msg.contains("compatBoom"), "{msg}");
    assert!(
        msg.contains("boom from a compat host native"),
        "the panic message must survive the boundary: {msg}"
    );
    assert!(
        native_panic_count() > before,
        "absorbed panics must be visible to the host"
    );

    // The VM and the process kept running: the next call works normally.
    let len = ctx
        .invoke_on(id, "length", "()I", &[])
        .expect("the VM must keep running after an absorbed native panic");
    assert!(matches!(len, JValue::Int(n) if n > 0), "{len:?}");
}

// ---------------------------------------------------------------------------
// 4. shims read the arguments the platform API passes them
// ---------------------------------------------------------------------------

#[test]
fn url_encoder_and_decoder_encode_the_value_not_the_charset() {
    init_logger();
    let mut ctx = context(AKUMA);
    let utf8 = string_id(&mut ctx, "UTF-8");
    let value = string_id(&mut ctx, "a b&c=d/e");

    let encoded = ctx
        .call(
            "Ljava/net/URLEncoder;",
            "encode",
            &[JValue::Obj(value), JValue::Obj(utf8)],
        )
        .expect("URLEncoder.encode(String, String)");
    let JValue::Obj(encoded_id) = encoded else {
        panic!("encode must return a String, got {encoded:?}");
    };
    let encoded = ctx.string_of(encoded_id).expect("string payload");
    assert_eq!(encoded, "a+b%26c%3Dd%2Fe", "encode the *value* argument");

    let roundtrip_arg = string_id(&mut ctx, &encoded);
    let decoded = ctx
        .call(
            "Ljava/net/URLDecoder;",
            "decode",
            &[JValue::Obj(roundtrip_arg), JValue::Obj(utf8)],
        )
        .expect("URLDecoder.decode(String, String)");
    let JValue::Obj(decoded_id) = decoded else {
        panic!("decode must return a String, got {decoded:?}");
    };
    assert_eq!(
        ctx.string_of(decoded_id).expect("string payload"),
        "a b&c=d/e",
        "decode must round-trip the value"
    );

    // The single-argument (legacy) overloads work too.
    let legacy = ctx
        .call("Ljava/net/URLEncoder;", "encode", &[JValue::Obj(value)])
        .expect("URLEncoder.encode(String)");
    let JValue::Obj(legacy_id) = legacy else {
        panic!("encode must return a String, got {legacy:?}");
    };
    assert_eq!(ctx.string_of(legacy_id).expect("payload"), encoded);
}

// ---------------------------------------------------------------------------
// 5. an API gap is reportable before a run, not only through a failure
// ---------------------------------------------------------------------------

#[test]
fn shim_coverage_is_reportable() {
    init_logger();
    let mut ctx = context(AKUMA);
    assert!(
        ctx.vm().has_native_for("Ljava/net/URLEncoder;"),
        "the form encoder must be implemented, not left as a hole"
    );
    assert!(
        !ctx.vm().has_native_for("Lcom/example/NotShimmed;"),
        "an unimplemented class must report as a gap"
    );
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// 6. a body that is not JSON fails as the Java serializer exception, and says
//    when the site answered with an HTML page instead of its own data
// ---------------------------------------------------------------------------

/// The shape a blocked site returns: a challenge page, not the source's JSON.
const HTML_BLOCK_PAGE: &str = "<!DOCTYPE html>\n<html><head><title>DDoS-Guard</title></head>\n\
<body>Check your browser before accessing</body></html>";

/// Message of an uncaught `Native::Throwable`.
fn throwable_message(ctx: &mut Context, id: u32) -> String {
    ctx.vm()
        .payload_of(JValue::Obj(id))
        .and_then(|p| match p {
            Native::Throwable { message, .. } => message.clone(),
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn html_response_body_fails_as_serialization_exception() {
    init_logger();
    let mut ctx = context(MOETRUYEN);
    let text = string_id(&mut ctx, HTML_BLOCK_PAGE);

    let before = dexvm::vm::native::blocked_response_count();
    let err = match ctx.call(
        "Lkotlinx/serialization/json/Json;",
        "decodeFromString",
        &[JValue::Null, JValue::Obj(text)],
    ) {
        Ok(v) => panic!("an html page must not decode as json, got {v:?}"),
        Err(e) => e,
    };
    let JvmError::Uncaught(thrown) = err else {
        panic!("a json decode failure must be an uncaught java throwable, got {err:?}");
    };
    let class = throwable_class(&mut ctx, thrown);
    let msg = throwable_message(&mut ctx, thrown);
    eprintln!("compat: decode error: {class}: {msg}");

    // The extension sees the exception its own `try`/`catch` catches, not a
    // VM-level resolution error that aborts the source.
    assert_eq!(
        class, "kotlinx.serialization.SerializationException",
        "a json decode failure must surface as SerializationException"
    );
    // and the message says the payload was an HTML page, which is what the
    // host uses to report the source as blocked rather than broken.
    assert!(
        msg.contains(dexvm::vm::native::HTML_RESPONSE_MARKER),
        "an html body must carry the blocked marker: {msg}"
    );
    // The host can also see it without reading the message: the blocked-payload
    // counter moves, which is how a source is reported as blocked even when the
    // extension catches the exception and fails later for another reason.
    assert!(
        dexvm::vm::native::blocked_response_count() > before,
        "an html payload must be counted as a blocked response"
    );
}

/// A body that is not JSON at all still fails as a catchable SerializationException.
#[test]
fn non_json_body_fails_as_serialization_exception() {
    init_logger();
    let mut ctx = context(MOETRUYEN);
    let text = string_id(&mut ctx, "not json at all");
    let err = match ctx.call(
        "Lkotlinx/serialization/json/Json;",
        "decodeFromString",
        &[JValue::Null, JValue::Obj(text)],
    ) {
        Ok(v) => panic!("expected a decode failure, got {v:?}"),
        Err(e) => e,
    };
    let JvmError::Uncaught(thrown) = err else {
        panic!("expected an uncaught java throwable, got {err:?}");
    };
    assert_eq!(
        throwable_class(&mut ctx, thrown),
        "kotlinx.serialization.SerializationException"
    );
    // Nothing blocked about it: no html marker.
    assert!(
        !throwable_message(&mut ctx, thrown).contains(dexvm::vm::native::HTML_RESPONSE_MARKER),
        "a plain bad payload must not be labelled blocked"
    );
}
