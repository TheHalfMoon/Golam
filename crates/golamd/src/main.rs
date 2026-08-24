#![forbid(unsafe_code)]

use golam_core::PROTOCOL_VERSION;
use golam_kernel::{DenyByDefault, KernelApi};

fn main() {
    let _kernel = KernelApi::new(DenyByDefault);
    println!("golamd bootstrap: protocol v{PROTOCOL_VERSION}; authority policy=deny-by-default");
}
