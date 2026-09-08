package execsandbox

import (
	"encoding/binary"
	"fmt"
	"time"
	"unsafe"
)

// Kind identifies what a received Message represents.
type Kind uint32

const (
	// KindMessage is a sandbox-to-sandbox message sent by a peer's Send.
	KindMessage Kind = 0
	// KindConnEstablished announces a new external connection. ConnID is
	// valid; Data is empty.
	KindConnEstablished Kind = 1
	// KindConnData carries data received on an external connection.
	KindConnData Kind = 2
	// KindConnClosed announces that an external connection has closed.
	// ConnID is valid; Data is empty. Unlike other kinds, this one is
	// always delivered even if the mailbox is otherwise full.
	KindConnClosed Kind = 3
)

// Message is one item taken off this sandbox's mailbox by Recv.
type Message struct {
	// Kind identifies what this message represents.
	Kind Kind
	// ConnID identifies the external connection this message relates to.
	// It is only meaningful when Kind is not KindMessage.
	ConnID uint32
	// Data is the message payload. It is empty for KindConnEstablished
	// and KindConnClosed.
	Data []byte
}

// hostSend, hostRecv, hostConnWrite and hostMaxFrame are bound to the real
// ABI (abi.go's init) when built for a wasm target. execsandbox_test.go
// substitutes fakes for these so the rest of this file's logic can be
// tested on any host architecture.
var (
	hostSend      func(dest int32, ptr unsafe.Pointer, length int32)
	hostRecv      func(metaPtr, bufPtr unsafe.Pointer, bufCap, timeoutMs int32) int32
	hostConnWrite func(connID int32, ptr unsafe.Pointer, length int32) int32
	hostMaxFrame  func() int32
)

// recvBuf is reused across calls to Recv and grown on demand. Starting it
// at the host's configured max frame size means, per the ABI's own design,
// it should never need to grow in practice.
var recvBuf []byte

func ensureRecvBuf() {
	if recvBuf != nil {
		return
	}
	size := int(hostMaxFrame())
	if size <= 0 {
		size = 4096
	}
	recvBuf = make([]byte, size)
}

func dataPtr(b []byte) unsafe.Pointer {
	if len(b) == 0 {
		return nil
	}
	return unsafe.Pointer(&b[0])
}

// Send delivers data to the destination numbered dest, as wired up for
// this sandbox instance at launch time (the host's -d flag).
//
// Send never blocks and never reports failure: an unconfigured
// destination, an unreachable peer, or a full mailbox on the receiving
// side all drop the message silently. This is ExecSandbox's own design,
// not a limitation of this wrapper (spec section 3.4).
func Send(dest uint32, data []byte) {
	hostSend(int32(dest), dataPtr(data), int32(len(data)))
}

// Recv takes one message off this sandbox's mailbox.
//
// The timeout follows time.Duration's sign convention, extended with
// ExecSandbox's own meaning for zero and negative values: a negative
// duration waits indefinitely for a message, zero returns immediately,
// and a positive duration waits up to that long (spec section 5.3). The
// second return value reports whether a message arrived before the
// timeout elapsed.
func Recv(timeout time.Duration) (Message, bool) {
	ensureRecvBuf()

	timeoutMs := int32(-1)
	if timeout >= 0 {
		timeoutMs = int32(timeout.Milliseconds())
	}

	var meta [8]byte
	for {
		n := hostRecv(unsafe.Pointer(&meta[0]), dataPtr(recvBuf), int32(len(recvBuf)), timeoutMs)
		switch {
		case n == -1:
			return Message{}, false
		case n < -1:
			recvBuf = make([]byte, -(n + 1))
			continue
		}

		kind := Kind(binary.LittleEndian.Uint32(meta[0:4]))
		if kind > KindConnClosed {
			// Forward compatibility: the host may add kinds this version
			// of the wrapper doesn't know about. Per spec section 5.3,
			// unknown kinds are skipped rather than surfaced. Note this
			// re-issues the same timeout rather than tracking a
			// remaining budget, so a stream of unknown kinds can make
			// Recv wait longer than timeout in total.
			continue
		}

		data := make([]byte, n)
		copy(data, recvBuf[:n])
		return Message{
			Kind:   kind,
			ConnID: binary.LittleEndian.Uint32(meta[4:8]),
			Data:   data,
		}, true
	}
}

// ConnWrite writes data to the external connection identified by connID.
// It reports an error if connID does not name a currently open connection
// (spec section 5.4).
func ConnWrite(connID uint32, data []byte) error {
	if hostConnWrite(int32(connID), dataPtr(data), int32(len(data))) == -1 {
		return fmt.Errorf("execsandbox: unknown connection id %d", connID)
	}
	return nil
}

// MaxFrame reports the maximum frame size, in bytes, that this
// ExecSandbox instance was configured with (the host's --max-frame flag).
func MaxFrame() int {
	return int(hostMaxFrame())
}
