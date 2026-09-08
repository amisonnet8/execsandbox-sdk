// Package execsandbox is a thin, Go-native wrapper around the WASM ABI
// that an ExecSandbox host exposes to guest modules (send/recv/conn_write/
// max_frame). It lets a guest module written in Go (compiled with TinyGo)
// exchange messages with other ExecSandbox instances and bridge external
// connections, without handling raw pointers, buffer sizing, or manual
// little-endian decoding itself.
//
// This package must be built for a wasm architecture (e.g. TinyGo's
// wasip1 target) — the ABI it wraps only exists on that target. It has no
// dependency on the ExecSandbox host's own code; the ABI it binds to is
// defined by the host project at
// https://github.com/amisonnet8/execsandbox (docs/spec/execsandbox_spec_ja.md,
// section 5), which this package treats as the sole source of truth.
package execsandbox
