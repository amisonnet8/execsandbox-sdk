//! Rust bindings for [ExecSandbox]'s WASM guest ABI
//! (`send`/`recv`/`conn_write`/`max_frame`), for guest modules built for
//! the `wasm32-wasip1` target.
//!
//! It hides pointer/length plumbing and buffer sizing behind a small
//! Rust-native API: [`send`], [`recv`] and [`conn_write`] work with
//! `&[u8]`/`Vec<u8>`, [`Kind`] identifies what [`recv`] returned, and
//! timeouts are an `Option<Duration>`.
//!
//! This crate must be built for `wasm32-wasip1` — the ABI it wraps only
//! exists on that target. It has no dependency on the ExecSandbox host's
//! own code; the ABI it binds to is defined by the host project at
//! <https://github.com/amisonnet8/execsandbox>
//! (`docs/spec/execsandbox_spec_ja.md`, section 5), which this crate
//! treats as the sole source of truth.
//!
//! [ExecSandbox]: https://github.com/amisonnet8/execsandbox

mod abi;

use std::cell::RefCell;
use std::time::Duration;

/// Identifies what a received [`Message`] represents.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A sandbox-to-sandbox message sent by a peer's [`send`].
    Message = 0,
    /// A new external connection. `conn_id` is valid; `data` is empty.
    ConnEstablished = 1,
    /// Data received on an external connection.
    ConnData = 2,
    /// An external connection has closed. `conn_id` is valid; `data` is
    /// empty. Unlike other kinds, this one is always delivered even if
    /// the mailbox is otherwise full.
    ConnClosed = 3,
}

impl Kind {
    fn from_u32(v: u32) -> Kind {
        match v {
            0 => Kind::Message,
            1 => Kind::ConnEstablished,
            2 => Kind::ConnData,
            _ => Kind::ConnClosed,
        }
    }
}

/// One item taken off this sandbox's mailbox by [`recv`].
#[derive(Debug, Clone)]
pub struct Message {
    /// What this message represents.
    pub kind: Kind,
    /// The external connection this message relates to. Only meaningful
    /// when `kind` is not [`Kind::Message`].
    pub conn_id: u32,
    /// The message payload. Empty for [`Kind::ConnEstablished`] and
    /// [`Kind::ConnClosed`].
    pub data: Vec<u8>,
}

/// Error returned by [`conn_write`] when `conn_id` does not name a
/// currently open connection (spec section 5.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownConnection(pub u32);

impl std::fmt::Display for UnknownConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "execsandbox: unknown connection id {}", self.0)
    }
}

impl std::error::Error for UnknownConnection {}

/// Seam between the safe wrapper below and the actual ABI, so the
/// wrapper's logic (timeout conversion, buffer growth, unknown-kind
/// skipping) can be unit tested on any host architecture with a fake
/// implementation, without requiring a real ExecSandbox host or a wasm32
/// build. `RealHost` is the only production implementation.
trait Host {
    fn send(&self, dest: i32, ptr: *const u8, len: i32);
    fn recv(&self, meta_ptr: *mut u8, buf_ptr: *mut u8, buf_cap: i32, timeout_ms: i32) -> i32;
    fn conn_write(&self, conn_id: i32, ptr: *const u8, len: i32) -> i32;
    fn max_frame(&self) -> i32;
}

struct RealHost;

#[cfg(target_arch = "wasm32")]
impl Host for RealHost {
    fn send(&self, dest: i32, ptr: *const u8, len: i32) {
        unsafe { abi::send(dest, ptr, len) }
    }
    fn recv(&self, meta_ptr: *mut u8, buf_ptr: *mut u8, buf_cap: i32, timeout_ms: i32) -> i32 {
        unsafe { abi::recv(meta_ptr, buf_ptr, buf_cap, timeout_ms) }
    }
    fn conn_write(&self, conn_id: i32, ptr: *const u8, len: i32) -> i32 {
        unsafe { abi::conn_write(conn_id, ptr, len) }
    }
    fn max_frame(&self) -> i32 {
        unsafe { abi::max_frame() }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Host for RealHost {
    fn send(&self, _dest: i32, _ptr: *const u8, _len: i32) {
        unreachable!("execsandbox: only usable when built for a wasm32 target")
    }
    fn recv(&self, _meta_ptr: *mut u8, _buf_ptr: *mut u8, _buf_cap: i32, _timeout_ms: i32) -> i32 {
        unreachable!("execsandbox: only usable when built for a wasm32 target")
    }
    fn conn_write(&self, _conn_id: i32, _ptr: *const u8, _len: i32) -> i32 {
        unreachable!("execsandbox: only usable when built for a wasm32 target")
    }
    fn max_frame(&self) -> i32 {
        unreachable!("execsandbox: only usable when built for a wasm32 target")
    }
}

thread_local! {
    // Reused across calls to `recv` and grown on demand. Starting it at
    // the host's configured max frame size means, per the ABI's own
    // design, it should never need to grow in practice.
    static RECV_BUF: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// Sends `data` to the destination numbered `dest`, as wired up for this
/// sandbox instance at launch time (the host's `-d` flag).
///
/// `send` never blocks and never reports failure: an unconfigured
/// destination, an unreachable peer, or a full mailbox on the receiving
/// side all drop the message silently. This is ExecSandbox's own design,
/// not a limitation of this wrapper (spec section 3.4).
///
/// ```no_run
/// execsandbox::send(1, b"hello");
/// ```
pub fn send(dest: u32, data: &[u8]) {
    send_with(&RealHost, dest, data)
}

fn send_with<H: Host>(host: &H, dest: u32, data: &[u8]) {
    host.send(dest as i32, data.as_ptr(), data.len() as i32);
}

/// Takes one message off this sandbox's mailbox.
///
/// `timeout` follows [`None`] for "wait indefinitely for a message",
/// [`Duration::ZERO`] for "return immediately", and any other
/// [`Duration`] for "wait up to that long" (spec section 5.3). An
/// `Option` is used, rather than a single signed value as Go's SDK does,
/// because Rust's [`Duration`] cannot be negative.
///
/// Returns [`None`] if no message arrived before the timeout elapsed.
///
/// ```no_run
/// use std::time::Duration;
///
/// if let Some(msg) = execsandbox::recv(Some(Duration::from_secs(5))) {
///     println!("{:?} {:?}", msg.kind, msg.data);
/// }
/// ```
pub fn recv(timeout: Option<Duration>) -> Option<Message> {
    let timeout_ms = duration_to_timeout_ms(timeout);
    RECV_BUF.with(|buf| recv_with(&RealHost, &mut buf.borrow_mut(), timeout_ms))
}

fn duration_to_timeout_ms(timeout: Option<Duration>) -> i32 {
    match timeout {
        None => -1,
        Some(d) => d.as_millis().min(i32::MAX as u128) as i32,
    }
}

fn recv_with<H: Host>(host: &H, buf: &mut Vec<u8>, timeout_ms: i32) -> Option<Message> {
    if buf.is_empty() {
        let size = host.max_frame();
        buf.resize(if size > 0 { size as usize } else { 4096 }, 0);
    }

    let mut meta = [0u8; 8];
    loop {
        let n = host.recv(
            meta.as_mut_ptr(),
            buf.as_mut_ptr(),
            buf.len() as i32,
            timeout_ms,
        );
        if n == -1 {
            return None;
        }
        if n < -1 {
            buf.resize((-(n + 1)) as usize, 0);
            continue;
        }

        let kind_raw = u32::from_le_bytes(meta[0..4].try_into().unwrap());
        if kind_raw > Kind::ConnClosed as u32 {
            // Forward compatibility: the host may add kinds this version
            // of the wrapper doesn't know about. Per spec section 5.3,
            // unknown kinds are skipped rather than surfaced. Note this
            // re-issues the same timeout rather than tracking a
            // remaining budget, so a stream of unknown kinds can make
            // recv wait longer than `timeout` in total.
            continue;
        }
        let conn_id = u32::from_le_bytes(meta[4..8].try_into().unwrap());
        let data = buf[..n as usize].to_vec();
        return Some(Message {
            kind: Kind::from_u32(kind_raw),
            conn_id,
            data,
        });
    }
}

/// Writes `data` to the external connection identified by `conn_id`.
/// Returns [`UnknownConnection`] if `conn_id` does not name a currently
/// open connection (spec section 5.4).
///
/// ```no_run
/// if let Some(msg) = execsandbox::recv(None) {
///     if msg.kind == execsandbox::Kind::ConnData {
///         let _ = execsandbox::conn_write(msg.conn_id, &msg.data);
///     }
/// }
/// ```
pub fn conn_write(conn_id: u32, data: &[u8]) -> Result<(), UnknownConnection> {
    conn_write_with(&RealHost, conn_id, data)
}

fn conn_write_with<H: Host>(host: &H, conn_id: u32, data: &[u8]) -> Result<(), UnknownConnection> {
    if host.conn_write(conn_id as i32, data.as_ptr(), data.len() as i32) == -1 {
        return Err(UnknownConnection(conn_id));
    }
    Ok(())
}

/// Reports the maximum frame size, in bytes, that this ExecSandbox
/// instance was configured with (the host's `--max-frame` flag).
///
/// ```no_run
/// let max_frame = execsandbox::max_frame();
/// ```
pub fn max_frame() -> usize {
    max_frame_with(&RealHost)
}

fn max_frame_with<H: Host>(host: &H) -> usize {
    host.max_frame() as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    type SendFn = Box<dyn FnMut(i32, *const u8, i32)>;
    type RecvFn = Box<dyn FnMut(*mut u8, *mut u8, i32, i32) -> i32>;
    type ConnWriteFn = Box<dyn FnMut(i32, *const u8, i32) -> i32>;
    type MaxFrameFn = Box<dyn FnMut() -> i32>;

    #[derive(Default)]
    struct MockHost {
        send: RefCell<Option<SendFn>>,
        recv: RefCell<Option<RecvFn>>,
        conn_write: RefCell<Option<ConnWriteFn>>,
        max_frame: RefCell<Option<MaxFrameFn>>,
    }

    impl Host for MockHost {
        fn send(&self, dest: i32, ptr: *const u8, len: i32) {
            (self
                .send
                .borrow_mut()
                .as_mut()
                .expect("send not configured"))(dest, ptr, len)
        }
        fn recv(&self, meta_ptr: *mut u8, buf_ptr: *mut u8, buf_cap: i32, timeout_ms: i32) -> i32 {
            (self
                .recv
                .borrow_mut()
                .as_mut()
                .expect("recv not configured"))(meta_ptr, buf_ptr, buf_cap, timeout_ms)
        }
        fn conn_write(&self, conn_id: i32, ptr: *const u8, len: i32) -> i32 {
            (self
                .conn_write
                .borrow_mut()
                .as_mut()
                .expect("conn_write not configured"))(conn_id, ptr, len)
        }
        fn max_frame(&self) -> i32 {
            (self
                .max_frame
                .borrow_mut()
                .as_mut()
                .expect("max_frame not configured"))()
        }
    }

    /// Builds a recv fake that returns the given (kind, conn_id, payload)
    /// on the first call and reports a timeout on every subsequent call,
    /// so a test cannot spin forever if the loop logic has a bug.
    fn fake_recv_once(
        kind: Kind,
        conn_id: u32,
        payload: &'static [u8],
    ) -> impl FnMut(*mut u8, *mut u8, i32, i32) -> i32 {
        let mut served = false;
        move |meta_ptr, buf_ptr, buf_cap, _timeout_ms| {
            if served {
                return -1;
            }
            served = true;
            assert!(
                payload.len() as i32 <= buf_cap,
                "fake_recv_once: payload ({} bytes) does not fit in the buffer given by recv ({} bytes)",
                payload.len(),
                buf_cap
            );
            unsafe {
                let meta = std::slice::from_raw_parts_mut(meta_ptr, 8);
                meta[0..4].copy_from_slice(&(kind as u32).to_le_bytes());
                meta[4..8].copy_from_slice(&conn_id.to_le_bytes());
                let buf = std::slice::from_raw_parts_mut(buf_ptr, buf_cap as usize);
                buf[..payload.len()].copy_from_slice(payload);
            }
            payload.len() as i32
        }
    }

    #[test]
    fn send_passes_dest_and_length() {
        let host = MockHost::default();
        let captured = Rc::new(RefCell::new((0i32, 0i32, false)));
        let captured2 = captured.clone();
        *host.send.borrow_mut() = Some(Box::new(move |dest, ptr, len| {
            *captured2.borrow_mut() = (dest, len, !ptr.is_null());
        }));

        send_with(&host, 3, b"hi");

        let (dest, len, ptr_nonnull) = *captured.borrow();
        assert_eq!(dest, 3);
        assert_eq!(len, 2);
        assert!(
            ptr_nonnull,
            "send should be called with a non-null pointer for non-empty data"
        );
    }

    #[test]
    fn recv_success_decodes_kind_conn_id_and_data() {
        let host = MockHost::default();
        *host.max_frame.borrow_mut() = Some(Box::new(|| 4096));
        *host.recv.borrow_mut() = Some(Box::new(fake_recv_once(Kind::ConnData, 7, b"payload")));

        let msg = recv_with(&host, &mut Vec::new(), 1000).expect("expected a message");
        assert_eq!(msg.kind, Kind::ConnData);
        assert_eq!(msg.conn_id, 7);
        assert_eq!(msg.data, b"payload");
    }

    #[test]
    fn recv_reports_timeout_as_none() {
        let host = MockHost::default();
        *host.max_frame.borrow_mut() = Some(Box::new(|| 4096));
        *host.recv.borrow_mut() = Some(Box::new(|_, _, _, _| -1));

        assert!(recv_with(&host, &mut Vec::new(), 100).is_none());
    }

    #[test]
    fn duration_to_timeout_ms_matches_abi_convention() {
        assert_eq!(
            duration_to_timeout_ms(None),
            -1,
            "no timeout should block forever (-1)"
        );
        assert_eq!(
            duration_to_timeout_ms(Some(Duration::ZERO)),
            0,
            "a zero duration should return immediately (0)"
        );
        assert_eq!(
            duration_to_timeout_ms(Some(Duration::from_millis(250))),
            250,
            "a positive duration should convert to milliseconds"
        );
    }

    #[test]
    fn recv_grows_buffer_on_undersized_reply() {
        let host = MockHost::default();
        *host.max_frame.borrow_mut() = Some(Box::new(|| 4)); // deliberately too small
        let payload: &'static [u8] = b"this does not fit in 4 bytes";
        let calls = Rc::new(RefCell::new(0));
        let calls2 = calls.clone();
        *host.recv.borrow_mut() = Some(Box::new(move |meta_ptr, buf_ptr, buf_cap, _timeout_ms| {
            *calls2.borrow_mut() += 1;
            if payload.len() as i32 > buf_cap {
                return -(payload.len() as i32 + 1);
            }
            unsafe {
                let meta = std::slice::from_raw_parts_mut(meta_ptr, 8);
                meta[0..4].copy_from_slice(&(Kind::Message as u32).to_le_bytes());
                meta[4..8].copy_from_slice(&0u32.to_le_bytes());
                let buf = std::slice::from_raw_parts_mut(buf_ptr, buf_cap as usize);
                buf[..payload.len()].copy_from_slice(payload);
            }
            payload.len() as i32
        }));

        let mut buf = Vec::new();
        let msg = recv_with(&host, &mut buf, -1).expect("expected a message after the buffer grew");
        assert_eq!(msg.data, payload);
        assert_eq!(
            *calls.borrow(),
            2,
            "recv should be called twice: undersized, then resized"
        );
        assert!(buf.len() >= payload.len());
    }

    #[test]
    fn recv_skips_unknown_kind() {
        let host = MockHost::default();
        *host.max_frame.borrow_mut() = Some(Box::new(|| 4096));
        let calls = Rc::new(RefCell::new(0));
        let calls2 = calls.clone();
        *host.recv.borrow_mut() = Some(Box::new(move |meta_ptr, buf_ptr, buf_cap, _timeout_ms| {
            *calls2.borrow_mut() += 1;
            let call = *calls2.borrow();
            unsafe {
                let meta = std::slice::from_raw_parts_mut(meta_ptr, 8);
                let buf = std::slice::from_raw_parts_mut(buf_ptr, buf_cap as usize);
                if call == 1 {
                    // A kind the host may add in the future, unknown to
                    // this wrapper: spec section 5.3 says to skip it.
                    meta[0..4].copy_from_slice(&99u32.to_le_bytes());
                    meta[4..8].copy_from_slice(&0u32.to_le_bytes());
                    buf[..b"ignored".len()].copy_from_slice(b"ignored");
                    return b"ignored".len() as i32;
                }
                meta[0..4].copy_from_slice(&(Kind::Message as u32).to_le_bytes());
                meta[4..8].copy_from_slice(&0u32.to_le_bytes());
                buf[..b"real".len()].copy_from_slice(b"real");
                b"real".len() as i32
            }
        }));

        let msg =
            recv_with(&host, &mut Vec::new(), -1).expect("expected the second, known-kind message");
        assert_eq!(*calls.borrow(), 2);
        assert_eq!(msg.kind, Kind::Message);
        assert_eq!(msg.data, b"real");
    }

    #[test]
    fn conn_write_success_returns_ok() {
        let host = MockHost::default();
        let captured = Rc::new(RefCell::new(0i32));
        let captured2 = captured.clone();
        *host.conn_write.borrow_mut() = Some(Box::new(move |conn_id, _, _| {
            *captured2.borrow_mut() = conn_id;
            0
        }));

        assert!(conn_write_with(&host, 42, b"data").is_ok());
        assert_eq!(*captured.borrow(), 42);
    }

    #[test]
    fn conn_write_unknown_conn_returns_error() {
        let host = MockHost::default();
        *host.conn_write.borrow_mut() = Some(Box::new(|_, _, _| -1));

        assert_eq!(
            conn_write_with(&host, 42, b"data"),
            Err(UnknownConnection(42))
        );
    }

    #[test]
    fn max_frame_passes_through_host_value() {
        let host = MockHost::default();
        *host.max_frame.borrow_mut() = Some(Box::new(|| 1_048_576));

        assert_eq!(max_frame_with(&host), 1_048_576);
    }
}
