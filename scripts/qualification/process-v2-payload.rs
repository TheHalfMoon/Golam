#![forbid(unsafe_code)]

use std::io::{self, Write};
use std::net::TcpStream;
use std::process::Command;

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    match mode.as_str() {
        "success" => print!("SUCCESS\n"),
        "isolation" => {
            if std::env::vars_os().next().is_none() {
                print!("ENV_EMPTY\n");
            } else {
                print!("ENV_PRESENT\n");
            }
            if TcpStream::connect("127.0.0.1:9").is_err() {
                print!("NETWORK_DENIED\n");
            } else {
                print!("NETWORK_OPEN\n");
            }
            if Command::new("/bin/true").status().is_err() {
                print!("SPAWN_DENIED\n");
            } else {
                print!("SPAWN_OPEN\n");
            }
        }
        "output" => {
            let chunk = [b'x'; 4096];
            let mut stdout = io::stdout().lock();
            loop {
                if stdout.write_all(&chunk).is_err() || stdout.flush().is_err() {
                    break;
                }
            }
        }
        "spin" => loop {
            std::hint::spin_loop();
        },
        _ => std::process::exit(64),
    }
}
