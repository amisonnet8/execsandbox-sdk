// Command receiver is a minimal ExecSandbox guest: it blocks until one
// sandbox-to-sandbox message arrives, prints it, and exits. Run it with a
// host launched as `execsandbox -n <this-name> -s out ...` so it has an
// identity to receive on and its stdout is visible.
package main

import (
	"fmt"

	"github.com/amisonnet8/execsandbox-sdk/execsandbox"
)

func main() {
	msg, ok := execsandbox.Recv(-1)
	if !ok {
		fmt.Println("timed out waiting for a message")
		return
	}
	fmt.Printf("kind=%d data=%s\n", msg.Kind, msg.Data)
}
