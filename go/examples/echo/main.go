// Command echo is a minimal ExecSandbox guest: it bridges an external
// connection back to itself, writing back whatever data it receives on
// one. Run it with a host launched as `execsandbox -l <address> ...`.
//
// Sandbox-to-sandbox messages and connection lifecycle events
// (KindConnEstablished, KindConnClosed) are received but ignored, per
// spec section 5.3's guidance to skip message kinds a guest doesn't act
// on rather than treating them as errors.
package main

import "github.com/amisonnet8/execsandbox-sdk/go/execsandbox"

func main() {
	for {
		msg, ok := execsandbox.Recv(-1)
		if !ok {
			continue
		}
		if msg.Kind == execsandbox.KindConnData {
			execsandbox.ConnWrite(msg.ConnID, msg.Data)
		}
	}
}
