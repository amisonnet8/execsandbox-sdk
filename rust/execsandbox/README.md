# execsandbox

Rust bindings for [ExecSandbox](https://github.com/amisonnet8/execsandbox)'s
WASM guest ABI (`send`/`recv`/`conn_write`/`max_frame`), for guest modules
built for the `wasm32-wasip1` target.

It hides pointer/length plumbing and buffer sizing behind a small
Rust-native API: `send`/`recv`/`conn_write` work with `&[u8]`/`Vec<u8>`, a
`Kind` enum identifies what `recv` returned, and timeouts are an
`Option<Duration>`.

## Install

```
cargo add execsandbox
```

## Usage

This crate only makes sense inside a guest module built for a wasm
target — build it for `wasm32-wasip1`, not the host architecture:

```
cargo build --target wasm32-wasip1 --release
```

```rust
fn main() {
    execsandbox::send(1, b"hello");

    if let Some(msg) = execsandbox::recv(None) { // block until one message arrives
        if msg.kind == execsandbox::Kind::ConnData {
            let _ = execsandbox::conn_write(msg.conn_id, &msg.data);
        }
    }
}
```

See [`examples/`](examples) for complete, runnable guest modules, and the
[crate documentation](https://docs.rs/execsandbox) for the full API.

## ABI compatibility

This crate binds to ExecSandbox's WASM ABI, which the host project only
guarantees backward compatibility for: an ExecSandbox host at or newer
than the version this crate's ABI functions were introduced in will run
guest modules built with this crate. The ABI itself is defined by the
host project, not by this crate — see
[`docs/spec/execsandbox_spec_ja.md`](https://github.com/amisonnet8/execsandbox/blob/main/docs/spec/execsandbox_spec_ja.md),
section 5.
