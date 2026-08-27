#![forbid(unsafe_code)]

mod client;

use std::process::ExitCode;

use golam::parse_args;
use golam_core::paths::RuntimeLayout;
use golam_core::runtime_home::default_runtime_root;
use golam_ipc::command::Command;
use golam_ipc::request::ReplyStatus;

fn main() -> ExitCode {
    let command = match parse_args(std::env::args().skip(1)) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("golam: {error}");
            return ExitCode::from(2);
        }
    };
    let root = match default_runtime_root() {
        Ok(root) => root,
        Err(error) => {
            eprintln!("golam: runtime root resolution failed: {error}");
            return ExitCode::from(1);
        }
    };
    let runtime = match RuntimeLayout::initialize(root) {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("golam: runtime initialization failed: {error}");
            return ExitCode::from(1);
        }
    };

    match command {
        Command::ClientEnroll { client_id } => match client::enroll(&runtime, client_id) {
            Ok(output) => {
                print!("{output}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("golam: {error}");
                ExitCode::from(1)
            }
        },
        command => match client::execute(&runtime, &command) {
            Ok(reply) => {
                let text = String::from_utf8_lossy(&reply.body);
                if reply.status == ReplyStatus::Ok {
                    print!("{text}");
                    ExitCode::SUCCESS
                } else {
                    eprint!("{text}");
                    ExitCode::from(1)
                }
            }
            Err(error) => {
                eprintln!("golam: {error}");
                ExitCode::from(1)
            }
        },
    }
}
