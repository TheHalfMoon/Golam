#![forbid(unsafe_code)]

use golam_core::{PROTOCOL_VERSION, ResourceLimits};
use golam_ipc::{FrameHeader, FrameKind};

fn main() {
    let hello = FrameHeader {
        protocol_version: PROTOCOL_VERSION,
        kind: FrameKind::Hello,
        request_id: None,
        payload_len: 0,
    };
    let status = hello.validate(ResourceLimits::default()).is_ok();
    println!("golam bootstrap: protocol v{PROTOCOL_VERSION}; hello_valid={status}");
}
