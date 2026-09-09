fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Native macOS adapter dependencies include Swift bridge code. Ensure every
    // golamd binary and test target can resolve the system Swift concurrency
    // runtime instead of relying on dependency-local linker arguments.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
    }
}
