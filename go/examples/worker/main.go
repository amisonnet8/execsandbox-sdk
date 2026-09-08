// Command worker is a minimal ExecSandbox guest that demonstrates a
// service loop: Recv is called with a bounded, positive timeout so the
// loop can also do periodic work (here, a heartbeat sent to destination
// 1) whenever no message arrives before the timeout elapses. Every Kind
// is handled explicitly, unlike the echo example, which only acts on
// KindConnData. Run it with a host launched as
// `execsandbox -n <this-name> -s out -d 1=<peer-name> -l <address> ...`.
package main

import (
	"fmt"
	"time"

	"github.com/amisonnet8/execsandbox-sdk/go/execsandbox"
)

func main() {
	for {
		msg, ok := execsandbox.Recv(time.Second)
		if !ok {
			execsandbox.Send(1, []byte("heartbeat"))
			continue
		}

		switch msg.Kind {
		case execsandbox.KindMessage:
			fmt.Printf("message: %s\n", msg.Data)
		case execsandbox.KindConnEstablished:
			fmt.Printf("connection %d established\n", msg.ConnID)
		case execsandbox.KindConnData:
			execsandbox.ConnWrite(msg.ConnID, msg.Data)
		case execsandbox.KindConnClosed:
			fmt.Printf("connection %d closed\n", msg.ConnID)
		}
	}
}
