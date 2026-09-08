//go:build wasm

package execsandbox

import "unsafe"

// These declare the four host functions of the "execsandbox" WASM import
// module. Signatures and semantics are fixed by the host's ABI (spec
// section 5) and must not be changed here; see .claude/rules/abi-boundary.md.
//
// This file is only built for a wasm architecture, since the wasmimport
// directive is rejected on any other target. execsandbox.go binds these to
// the package-level hostSend etc. variables at init time so the rest of
// the package can be built and unit tested on any host architecture, with
// execsandbox_test.go substituting fakes for these variables instead.

//go:wasmimport execsandbox send
func abiSend(dest int32, ptr unsafe.Pointer, length int32)

//go:wasmimport execsandbox recv
func abiRecv(metaPtr unsafe.Pointer, bufPtr unsafe.Pointer, bufCap int32, timeoutMs int32) int32

//go:wasmimport execsandbox conn_write
func abiConnWrite(connID int32, ptr unsafe.Pointer, length int32) int32

//go:wasmimport execsandbox max_frame
func abiMaxFrame() int32

func init() {
	// TinyGo rejects using a //go:wasmimport function as a value directly
	// (only calling it is allowed), so each is wrapped in a closure here.
	hostSend = func(dest int32, ptr unsafe.Pointer, length int32) {
		abiSend(dest, ptr, length)
	}
	hostRecv = func(metaPtr, bufPtr unsafe.Pointer, bufCap, timeoutMs int32) int32 {
		return abiRecv(metaPtr, bufPtr, bufCap, timeoutMs)
	}
	hostConnWrite = func(connID int32, ptr unsafe.Pointer, length int32) int32 {
		return abiConnWrite(connID, ptr, length)
	}
	hostMaxFrame = func() int32 {
		return abiMaxFrame()
	}
}
