package execsandbox_test

import (
	"fmt"
	"time"

	"github.com/amisonnet8/execsandbox-sdk/execsandbox"
)

func ExampleSend() {
	// Send is fire-and-forget: it neither blocks nor reports whether the
	// message reached its destination. See the Send docs for why.
	execsandbox.Send(1, []byte("hello"))
}

func ExampleRecv() {
	msg, ok := execsandbox.Recv(5 * time.Second)
	if !ok {
		fmt.Println("timed out waiting for a message")
		return
	}
	fmt.Println(msg.Kind, string(msg.Data))
}

func ExampleConnWrite() {
	msg, ok := execsandbox.Recv(-1)
	if ok && msg.Kind == execsandbox.KindConnData {
		execsandbox.ConnWrite(msg.ConnID, msg.Data)
	}
}
