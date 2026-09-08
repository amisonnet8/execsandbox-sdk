//! Raw bindings to the `execsandbox` WASM import module. Signatures and
//! semantics are fixed by the host's ABI (spec section 5) and must not be
//! changed here; see `.claude/rules/abi-boundary.md`.
//!
//! Only built for a wasm32 target, since `wasm_import_module` is only
//! meaningful there. `lib.rs`'s `Host` trait binds these to a safe,
//! testable wrapper.

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "execsandbox")]
extern "C" {
    pub fn send(dest: i32, ptr: *const u8, len: i32);
    pub fn recv(meta_ptr: *mut u8, buf_ptr: *mut u8, buf_cap: i32, timeout_ms: i32) -> i32;
    pub fn conn_write(conn_id: i32, ptr: *const u8, len: i32) -> i32;
    pub fn max_frame() -> i32;
}
