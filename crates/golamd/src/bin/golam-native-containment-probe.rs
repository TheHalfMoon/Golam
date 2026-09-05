#![forbid(unsafe_code)]

#[allow(dead_code)]
#[path = "../native_containment.rs"]
mod native_containment;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn main() {
    match native_containment::compile_seccomp_deny_filter() {
        Ok(program) => {
            println!(
                "profile={} seccomp_bpf_instructions={} production_admitted=no",
                native_containment::PROFILE_TOKEN,
                program.len()
            );
        }
        Err(error) => {
            eprintln!("native containment probe failed: {error}");
            std::process::exit(2);
        }
    }
}

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
fn main() {
    println!(
        "profile={} platform_supported=no production_admitted=no",
        native_containment::PROFILE_TOKEN
    );
}
