# Examples

Minimal ExecSandbox guest modules built with the
[`execsandbox`](../execsandbox) package, one per ABI capability:

| Example | Demonstrates |
|---|---|
| [`sender`](sender) | `Send` — sends one message to destination 1 and exits |
| [`receiver`](receiver) | `Recv` — blocks for one message, prints it, and exits |
| [`echo`](echo) | `ConnWrite` — echoes data back on an external connection |
| [`worker`](worker) | A positive `Recv` timeout for periodic work between messages, dispatching every `Kind` explicitly |

## Building

Each example is its own Go module. Build one with TinyGo, targeting WASI:

```
cd sender
tinygo build -target=wasip1 -o sender.wasm .
```

## Running

The resulting `.wasm` is a guest module, not something you run directly —
stamp it into an executable with ExecSandbox's builder
(`execsandbox-build`, from the [host project](https://github.com/amisonnet8/execsandbox)),
then launch it like any other ExecSandbox instance. See that project's
`docs/usage/execsandbox.md` for the full CLI, or
[`../tests/e2e_send_recv.sh`](../tests/e2e_send_recv.sh) and
[`../tests/e2e_conn.sh`](../tests/e2e_conn.sh) in this repository for
complete, working examples of building, stamping, and running these.
