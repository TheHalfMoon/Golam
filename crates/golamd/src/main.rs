#![forbid(unsafe_code)]

use golam_core::PROTOCOL_VERSION;

fn main() {
    println!(
        "golamd bootstrap: protocol v{PROTOCOL_VERSION}; authority service remains fail-closed until a protected RuntimeLayout is composed"
    );
}
