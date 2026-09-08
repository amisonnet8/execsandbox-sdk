//! A minimal ExecSandbox guest: blocks until one sandbox-to-sandbox
//! message arrives, prints it, and exits. Run it with a host launched as
//! `execsandbox -n <this-name> -s out ...` so it has an identity to
//! receive on and its stdout is visible.

fn main() {
    match execsandbox::recv(None) {
        Some(msg) => println!(
            "kind={} data={}",
            msg.kind as u32,
            String::from_utf8_lossy(&msg.data)
        ),
        None => println!("timed out waiting for a message"),
    }
}
