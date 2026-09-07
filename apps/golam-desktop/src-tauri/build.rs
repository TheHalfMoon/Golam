fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new().app_manifest(
            tauri_build::AppManifest::new().commands(&[
                "desktop_status",
                "desktop_pause",
                "desktop_stop",
                "desktop_takeover",
            ]),
        ),
    )
    .expect("Tauri desktop build configuration must be valid");
}
