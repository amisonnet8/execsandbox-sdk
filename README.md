# ExecSandbox SDK

A collection of language-native SDK libraries for writing WASM guest
modules for [ExecSandbox](https://github.com/amisonnet8/execsandbox).

## What is ExecSandbox

A portable, setup-free sandboxed execution tool that bundles a WASM
runtime and a WASM module into a single executable (written in Go, built
on [wazero](https://github.com/tetratelabs/wazero) at its core). Multiple
ExecSandbox instances can be wired together into a system via
mailbox-style messaging. See the
[execsandbox repository](https://github.com/amisonnet8/execsandbox)'s
`docs/spec/execsandbox_spec_ja.md` for details on the host project.

## This repository's role

An ExecSandbox guest module (the WASM side) exchanges messages with other
ExecSandbox instances by importing a small set of host functions the
host provides (`send`/`recv`/`conn_write`/`max_frame`). This calling
convention (ABI) is a low-level contract: used directly, it requires the
caller to handle pointers, lengths, and buffer allocation itself.

**This repository provides SDK libraries that wrap that ABI in a
language-native skin.** Usability improvements such as hiding buffer
allocation, turning message kinds into an enum, and taking timeouts as a
language-native type (`time.Duration`, etc.) belong here.

## Relationship with the ABI

- **The ABI itself is not defined in this repository.** The source of
  truth is section 5 of the
  [execsandbox repository](https://github.com/amisonnet8/execsandbox)'s
  spec document; this repository only references it, never copies it.
- The ABI only guarantees backward compatibility. Even if the host
  project's and this SDK's versions drift apart, things keep working as
  long as the host is newer (only if the host stays old while the SDK
  starts using a newer host function does the user need to update the
  host first).
- Zero dependency on the host project's code. Each language's SDK hits
  the ABI directly through that language's own extern function
  declarations (Go's `//go:wasmimport`, Rust's `extern "C"`, etc.) and
  never imports the execsandbox host project's own packages.

## Supported languages

Implemented in order, TinyGo (Go) then Rust — both are underway. Each
language gets its own top-level directory (`go/` for the TinyGo version,
`rust/` for the Rust version). See `PLAN.md` (Japanese) for details.

- TinyGo version: [`go/execsandbox/`](go/execsandbox/) (package-user
  documentation in
  [`go/execsandbox/README.md`](go/execsandbox/README.md), examples in
  [`go/examples/`](go/examples/)).
  [![Go Reference](https://pkg.go.dev/badge/github.com/amisonnet8/execsandbox-sdk/go/execsandbox.svg)](https://pkg.go.dev/github.com/amisonnet8/execsandbox-sdk/go/execsandbox)
- Rust version: [`rust/execsandbox/`](rust/execsandbox/) (crate-user
  documentation in
  [`rust/execsandbox/README.md`](rust/execsandbox/README.md), examples
  in [`rust/execsandbox/examples/`](rust/execsandbox/examples/) — inside
  the crate itself, unlike the Go version, since it uses Cargo's
  built-in examples feature).
  [![Crates.io](https://img.shields.io/crates/v/execsandbox.svg)](https://crates.io/crates/execsandbox)
  [![docs.rs](https://docs.rs/execsandbox/badge.svg)](https://docs.rs/execsandbox)

## Using a built WASM module

This SDK only lets you write the code for a WASM module (the guest). To
actually run it, you need to embed it into the ExecSandbox host with
[execsandbox-build](https://github.com/amisonnet8/execsandbox) (the
builder) to produce a single executable.

## Current status

Both the TinyGo and Rust versions have a thin ABI wrapper
implementation, unit tests, examples, end-to-end tests against the
execsandbox host project, and GitHub Actions CI in place. See `PLAN.md`
(Japanese) for details and progress.

## Testing

Both the TinyGo and Rust versions follow the same three-tier structure:
unit tests (host architecture), examples (wasm build verification), and
end-to-end tests (real interop verified against a checkout of the
execsandbox host project's source).

- TinyGo version: `cd go/execsandbox && go test ./...`,
  `tinygo build -target=wasip1 -o out.wasm .` (in each example
  directory), `EXECSANDBOX_HOST_REPO=/path/to/execsandbox
  go/tests/e2e_send_recv.sh`.
- Rust version: `cd rust/execsandbox && cargo test`,
  `cargo build --target wasm32-wasip1 --examples`,
  `EXECSANDBOX_HOST_REPO=/path/to/execsandbox
  rust/tests/e2e_send_recv.sh`.

## License

[MIT](LICENSE)

---

日本語版は [README_ja.md](README_ja.md) を参照してください。
