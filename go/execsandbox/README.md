# execsandbox

Go bindings for [ExecSandbox](https://github.com/amisonnet8/execsandbox)'s
WASM guest ABI (`send`/`recv`/`conn_write`/`max_frame`), for guest modules
compiled with [TinyGo](https://tinygo.org/).

It hides pointer/length plumbing and buffer sizing behind a small
Go-native API: `Send`/`Recv`/`ConnWrite` work with `[]byte`, a `Kind` enum
identifies what `Recv` returned, and timeouts are a `time.Duration`.

## Install

```
go get github.com/amisonnet8/execsandbox-sdk/go/execsandbox
```

## Usage

This package only makes sense inside a guest module built for a wasm
target — build it with TinyGo, not `go build`:

```
tinygo build -target=wasip1 -o guest.wasm .
```

```go
package main

import "github.com/amisonnet8/execsandbox-sdk/go/execsandbox"

func main() {
	execsandbox.Send(1, []byte("hello"))

	msg, ok := execsandbox.Recv(-1) // block until one message arrives
	if ok && msg.Kind == execsandbox.KindConnData {
		execsandbox.ConnWrite(msg.ConnID, msg.Data)
	}
}
```

See [`../examples`](../examples) for complete, runnable guest modules, and
the [package documentation](https://pkg.go.dev/github.com/amisonnet8/execsandbox-sdk/go/execsandbox)
for the full API.

## ABI compatibility

This package binds to ExecSandbox's WASM ABI, which the host project only
guarantees backward compatibility for: an ExecSandbox host at or newer
than the version this package's ABI functions were introduced in will run
guest modules built with this package. The ABI itself is defined by the
host project, not by this package — see
[`docs/spec/execsandbox_spec_ja.md`](https://github.com/amisonnet8/execsandbox/blob/main/docs/spec/execsandbox_spec_ja.md),
section 5.
