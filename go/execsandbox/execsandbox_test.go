package execsandbox

import (
	"encoding/binary"
	"testing"
	"time"
	"unsafe"
)

// resetHostFakes restores hostSend/hostRecv/hostConnWrite/hostMaxFrame and
// recvBuf to a clean state before each test, since they are package-level
// state shared across tests.
func resetHostFakes(t *testing.T) {
	t.Helper()
	hostSend = nil
	hostRecv = nil
	hostConnWrite = nil
	hostMaxFrame = nil
	recvBuf = nil
}

func TestSend(t *testing.T) {
	resetHostFakes(t)

	var gotDest int32
	var gotLen int32
	var gotPtr unsafe.Pointer
	hostSend = func(dest int32, ptr unsafe.Pointer, length int32) {
		gotDest, gotPtr, gotLen = dest, ptr, length
	}

	Send(3, []byte("hi"))
	if gotDest != 3 || gotLen != 2 || gotPtr == nil {
		t.Fatalf("Send(3, \"hi\") called host with dest=%d ptr=%v len=%d, want dest=3 ptr!=nil len=2", gotDest, gotPtr, gotLen)
	}
}

func TestSendEmptyData(t *testing.T) {
	resetHostFakes(t)

	var gotPtr unsafe.Pointer
	var gotLen int32
	called := false
	hostSend = func(dest int32, ptr unsafe.Pointer, length int32) {
		called = true
		gotPtr, gotLen = ptr, length
	}

	Send(1, nil)
	if !called {
		t.Fatal("Send(1, nil) did not call the host")
	}
	if gotPtr != nil || gotLen != 0 {
		t.Fatalf("Send(1, nil) called host with ptr=%v len=%d, want ptr=nil len=0", gotPtr, gotLen)
	}
}

// fakeRecv builds a hostRecv fake that returns the given (kind, connID,
// payload) on the first call and then reports a timeout on every
// subsequent call, so a test's Recv call cannot spin forever if the
// production loop logic has a bug.
func fakeRecvOnce(t *testing.T, kind Kind, connID uint32, payload []byte) func(metaPtr, bufPtr unsafe.Pointer, bufCap, timeoutMs int32) int32 {
	t.Helper()
	served := false
	return func(metaPtr, bufPtr unsafe.Pointer, bufCap, timeoutMs int32) int32 {
		if served {
			return -1
		}
		served = true
		if int32(len(payload)) > bufCap {
			t.Fatalf("fakeRecvOnce: payload (%d bytes) does not fit in the buffer given by Recv (%d bytes)", len(payload), bufCap)
		}
		meta := (*[8]byte)(metaPtr)
		binary.LittleEndian.PutUint32(meta[0:4], uint32(kind))
		binary.LittleEndian.PutUint32(meta[4:8], connID)
		buf := unsafe.Slice((*byte)(bufPtr), bufCap)
		copy(buf, payload)
		return int32(len(payload))
	}
}

func TestRecvSuccess(t *testing.T) {
	resetHostFakes(t)
	hostMaxFrame = func() int32 { return 4096 }
	hostRecv = fakeRecvOnce(t, KindConnData, 7, []byte("payload"))

	msg, ok := Recv(time.Second)
	if !ok {
		t.Fatal("Recv reported no message, want a message")
	}
	if msg.Kind != KindConnData || msg.ConnID != 7 || string(msg.Data) != "payload" {
		t.Fatalf("Recv = %+v, want Kind=%d ConnID=7 Data=\"payload\"", msg, KindConnData)
	}
}

func TestRecvTimeout(t *testing.T) {
	resetHostFakes(t)
	hostMaxFrame = func() int32 { return 4096 }
	hostRecv = func(metaPtr, bufPtr unsafe.Pointer, bufCap, timeoutMs int32) int32 { return -1 }

	msg, ok := Recv(100 * time.Millisecond)
	if ok {
		t.Fatalf("Recv reported a message (%+v), want a timeout", msg)
	}
}

func TestRecvTimeoutConversion(t *testing.T) {
	resetHostFakes(t)
	hostMaxFrame = func() int32 { return 4096 }

	cases := []struct {
		name    string
		timeout time.Duration
		want    int32
	}{
		{"negative blocks forever", -1, -1},
		{"zero returns immediately", 0, 0},
		{"positive converts to milliseconds", 250 * time.Millisecond, 250},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			var got int32
			hostRecv = func(metaPtr, bufPtr unsafe.Pointer, bufCap, timeoutMs int32) int32 {
				got = timeoutMs
				return -1
			}
			Recv(tc.timeout)
			if got != tc.want {
				t.Errorf("Recv(%s) passed timeoutMs=%d to host, want %d", tc.timeout, got, tc.want)
			}
		})
	}
}

func TestRecvGrowsBufferOnUndersize(t *testing.T) {
	resetHostFakes(t)
	hostMaxFrame = func() int32 { return 4 } // deliberately smaller than the payload below

	payload := []byte("this does not fit in 4 bytes")
	calls := 0
	hostRecv = func(metaPtr, bufPtr unsafe.Pointer, bufCap, timeoutMs int32) int32 {
		calls++
		if int32(len(payload)) > bufCap {
			return -(int32(len(payload)) + 1)
		}
		meta := (*[8]byte)(metaPtr)
		binary.LittleEndian.PutUint32(meta[0:4], uint32(KindMessage))
		binary.LittleEndian.PutUint32(meta[4:8], 0)
		buf := unsafe.Slice((*byte)(bufPtr), bufCap)
		copy(buf, payload)
		return int32(len(payload))
	}

	msg, ok := Recv(-1)
	if !ok {
		t.Fatal("Recv reported no message, want a message after the buffer grew")
	}
	if string(msg.Data) != string(payload) {
		t.Fatalf("Recv.Data = %q, want %q", msg.Data, payload)
	}
	if calls != 2 {
		t.Fatalf("hostRecv was called %d times, want 2 (undersized, then resized)", calls)
	}
	if len(recvBuf) < len(payload) {
		t.Fatalf("recvBuf did not grow: len=%d, want >= %d", len(recvBuf), len(payload))
	}
}

func TestRecvSkipsUnknownKind(t *testing.T) {
	resetHostFakes(t)
	hostMaxFrame = func() int32 { return 4096 }

	calls := 0
	hostRecv = func(metaPtr, bufPtr unsafe.Pointer, bufCap, timeoutMs int32) int32 {
		calls++
		meta := (*[8]byte)(metaPtr)
		buf := unsafe.Slice((*byte)(bufPtr), bufCap)
		if calls == 1 {
			// A kind the host may add in the future, unknown to this
			// wrapper: spec section 5.3 says to skip it, not surface it.
			binary.LittleEndian.PutUint32(meta[0:4], 99)
			binary.LittleEndian.PutUint32(meta[4:8], 0)
			copy(buf, "ignored")
			return int32(len("ignored"))
		}
		binary.LittleEndian.PutUint32(meta[0:4], uint32(KindMessage))
		binary.LittleEndian.PutUint32(meta[4:8], 0)
		copy(buf, "real")
		return int32(len("real"))
	}

	msg, ok := Recv(-1)
	if !ok {
		t.Fatal("Recv reported no message, want the second, known-kind message")
	}
	if calls != 2 {
		t.Fatalf("hostRecv was called %d times, want 2 (unknown kind skipped, then the real message)", calls)
	}
	if msg.Kind != KindMessage || string(msg.Data) != "real" {
		t.Fatalf("Recv = %+v, want Kind=%d Data=\"real\"", msg, KindMessage)
	}
}

func TestConnWriteSuccess(t *testing.T) {
	resetHostFakes(t)

	var gotConnID int32
	hostConnWrite = func(connID int32, ptr unsafe.Pointer, length int32) int32 {
		gotConnID = connID
		return 0
	}

	if err := ConnWrite(42, []byte("data")); err != nil {
		t.Fatalf("ConnWrite returned error %v, want nil", err)
	}
	if gotConnID != 42 {
		t.Fatalf("ConnWrite called host with connID=%d, want 42", gotConnID)
	}
}

func TestConnWriteUnknownConn(t *testing.T) {
	resetHostFakes(t)
	hostConnWrite = func(connID int32, ptr unsafe.Pointer, length int32) int32 { return -1 }

	err := ConnWrite(42, []byte("data"))
	if err == nil {
		t.Fatal("ConnWrite returned nil error, want an error for an unknown connection id")
	}
}

func TestMaxFrame(t *testing.T) {
	resetHostFakes(t)
	hostMaxFrame = func() int32 { return 1048576 }

	if got := MaxFrame(); got != 1048576 {
		t.Fatalf("MaxFrame() = %d, want 1048576", got)
	}
}
