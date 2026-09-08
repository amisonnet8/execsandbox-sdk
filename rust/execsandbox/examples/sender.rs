//! A minimal ExecSandbox guest: sends one message to destination 1 and
//! exits. Run it with a host launched as
//! `execsandbox -d 1=<peer-name> ...` so destination 1 resolves to a
//! peer sandbox.

fn main() {
    execsandbox::send(1, b"hello from execsandbox-sdk");
}
