//! A minimal ExecSandbox guest: bridges an external connection back to
//! itself, writing back whatever data it receives on one. Run it with a
//! host launched as `execsandbox -l <address> ...`.
//!
//! Sandbox-to-sandbox messages and connection lifecycle events
//! (ConnEstablished, ConnClosed) are received but ignored, per spec
//! section 5.3's guidance to skip message kinds a guest doesn't act on
//! rather than treating them as errors.

use execsandbox::Kind;

fn main() {
    loop {
        let Some(msg) = execsandbox::recv(None) else {
            continue;
        };
        if msg.kind == Kind::ConnData {
            let _ = execsandbox::conn_write(msg.conn_id, &msg.data);
        }
    }
}
