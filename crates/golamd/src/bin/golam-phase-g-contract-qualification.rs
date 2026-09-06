#![forbid(unsafe_code)]

//! Compile-and-test carrier for Phase G contracts that remain deliberately unwired from the
//! production daemon until their ordered admission/launch gates complete.

#[allow(dead_code)]
#[path = "../native_process_supervisor.rs"]
mod native_process_supervisor;
#[allow(dead_code)]
#[path = "../process_secret_evidence.rs"]
mod process_secret_evidence;

fn main() {}
