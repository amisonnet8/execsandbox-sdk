// Command sender is a minimal ExecSandbox guest: it sends one message to
// destination 1 and exits. Run it with a host launched as
// `execsandbox -d 1=<peer-name> ...` so destination 1 resolves to a peer
// sandbox.
package main

import "github.com/amisonnet8/execsandbox-sdk/execsandbox"

func main() {
	execsandbox.Send(1, []byte("hello from execsandbox-sdk"))
}
