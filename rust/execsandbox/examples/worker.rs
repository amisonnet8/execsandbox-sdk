//! A minimal ExecSandbox guest that demonstrates a service loop: `recv`
//! is called with a bounded, positive timeout so the loop can also do
//! periodic work (here, a heartbeat sent to destination 1) whenever no
//! message arrives before the timeout elapses. Every `Kind` is handled
//! explicitly, unlike the echo example, which only acts on
//! `Kind::ConnData`. Run it with a host launched as
//! `execsandbox -n <this-name> -s out -d 1=<peer-name> -l <address> ...`.

use execsandbox::Kind;
use std::time::Duration;

fn main() {
    loop {
        let Some(msg) = execsandbox::recv(Some(Duration::from_secs(1))) else {
            execsandbox::send(1, b"heartbeat");
            continue;
        };

        match msg.kind {
            Kind::Message => println!("message: {}", String::from_utf8_lossy(&msg.data)),
            Kind::ConnEstablished => println!("connection {} established", msg.conn_id),
            Kind::ConnData => {
                let _ = execsandbox::conn_write(msg.conn_id, &msg.data);
            }
            Kind::ConnClosed => println!("connection {} closed", msg.conn_id),
        }
    }
}
