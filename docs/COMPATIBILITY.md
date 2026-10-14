# Compatibility contract

What a dexvm run owes the host, and how each guarantee is checked. Everything
below is enforced by offline, deterministic tests (`tests/compat.rs`,
`src/vm/native/**/tests`) with a canned HTTP layer: a failure here is a code
failure, never a network flake.

## Guarantees

| Guarantee | Mechanism | Checked by |
|---|---|---|
| A host native never aborts the process | every native runs behind a panic boundary in `Vm::call_native_guarded` | `panicking_native_is_absorbed_named_and_counted` |
| A failure names the native that caused it | `native_label` resolves built-in, global and per-context natives; the error text is `native panic in <class>::<name><sig>: <panic message>` | same test |
| Absorbed panics are visible, not silent | `dexvm::vm::native::native_panic_count()` | same test |
| A DEX null reference is a Java exception, not a Rust panic | `want_obj` / `want_arg_obj` raise `java.lang.NullPointerException` | `null_argument_becomes_a_catchable_null_pointer_exception` |
| A short argument list is a Java exception | `want_arg_obj` raises `java.lang.ArrayIndexOutOfBoundsException` | `short_argument_list_becomes_a_catchable_index_out_of_bounds_exception` |
| The suspend/coroutine bridge returns data instead of failing with a resolution error | host-default shim on missing virtual methods + `suspend_cont` continuation replay | `coro_bridge_returns_data_for_the_classic_generation`, `coro_bridge_returns_data_for_the_suspend_generation` |
| Shims read the arguments the platform API actually passes | argument index conventions + registered overloads | `url_encoder_and_decoder_encode_the_value_not_the_charset`, `url::tests::*` |

## Safe argument access

Host natives read arguments through the `want_*` helpers in
`dexvm::vm::native`. Each returns `Err(NatErr::Throw(id))` carrying a real Java
throwable, which guest bytecode can catch, instead of unwrapping a `JValue`:

```rust
use dexvm::vm::native::{want_arg_obj, want_int};
use dexvm::vm::{NatErr, NativeEntry, Vm};
use dexvm::vm::value::JValue;

fn my_native(vm: &mut Vm, args: &[JValue]) -> Result<JValue, NatErr> {
    // args[0] is the receiver for an instance native.
    let receiver = want_arg_obj(vm, args, 0)?;          // NPE for a null reference
    let index = want_int(vm, args.get(1).copied().unwrap_or(JValue::Null))?;
    Ok(vm.field_i32(receiver, index as usize))
}

static TABLE: &[NativeEntry] = &[NativeEntry {
    class: "Lcom/example/Host;",
    name: "read",
    sig: "(I)I",
    instance: true,
    f: my_native,
}];
```

| Helper | Rejects with |
|---|---|
| `want_obj(vm, v)` | `NullPointerException` (null reference), `IllegalArgumentException` (wrong type) |
| `want_arg_obj(vm, args, i)` | `ArrayIndexOutOfBoundsException` (missing argument), otherwise as `want_obj` |
| `want_int(vm, v)` / `want_long(vm, v)` | `IllegalArgumentException`; `Null` keeps the existing zero convention |

`JValue::as_obj_checked` / `as_int_checked` (`src/vm/value.rs`) are the
panic-free primitives underneath them.

## What changed in this increment

* `java.net.URLEncoder.encode` / `URLDecoder.decode` take the text from
  `args[0]`. They previously read `args[1]`, which encoded the *charset* string
  and indexed past the end of the argument list; `encode(s, charset)` from
  bytecode aborted the process with
  `index out of bounds: the len is 2 but the index is 2`. Both platform
  overloads are registered: `encode(s)`, `encode(s, charset)`, `decode(s)`,
  `decode(s, charset)` (`src/vm/native/java/net/url.rs`).
* `normalize_charset` accepts the canonical `UTF-8`. `URLEncoder.encode(s,
  "UTF-8")` — the call most extensions actually make — previously failed with
  `Unsupported charset: UTF-8` (`java.lang.IllegalArgumentException`). Only
  spellings like `UTF8` / `UTF_8` were recognized
  (`src/vm/native/java/nio/charset.rs`).
* A charset argument that is absent, or not a string value, falls back to
  UTF-8 instead of indexing a value that is not a string.
* The panic boundary now names natives the host registered at runtime
  (`register_global`, per-context natives), not only built-in ones.
* Shim classes report whether a host native exists for a descriptor
  (`Vm::has_native_for`), so a host can check a gap before a run instead of
  reading a panic back out of an error string.
* Tests: `tests/compat.rs` (six offline contract tests) and the argument
  contract unit tests in `src/vm/native/java/net/url.rs`.

## Known limits

* An absorbed native panic fails the **current** source (`JvmError::Fatal`) and
  keeps the VM and process alive. It is a backstop for shims that are still
  wrong, not a substitute for raising throwables through `want_*`.
* A native that cannot be identified is reported as `?` in the panic label.
* Era boundaries are not gaps: a pre-1.6 fixture (for example Akuma 1.4.10)
  declares no `getMangaUpdate`, so `manga_update_coro` legitimately fails
  resolution for it. The 1.6-era entry points are exercised against the
  moetruyen fixture.
* Assertions that depend on a live site stay gated on `DEXVM_LIVE=1`; the
  offline contract suite replays canned responses.
