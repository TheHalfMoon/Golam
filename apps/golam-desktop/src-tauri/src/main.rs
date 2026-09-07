#![forbid(unsafe_code)]

mod host;

use serde_json::{Value, json};

#[tauri::command]
fn desktop_status() -> Value {
    let status = host::authenticate_status();
    json!({
        "connected": status.connected,
        "clientId": status.client_id.map(|value| value.to_string()),
        "reason": status.reason,
        "controlState": if status.connected { "idle" } else { "authentication-disconnected" }
    })
}

fn protected_control_unavailable(operation: &str) -> Result<Value, String> {
    Err(format!(
        "{operation} is unavailable until the native host is bound to the protected golamd desktop control-lease path"
    ))
}

#[tauri::command]
fn desktop_pause() -> Result<Value, String> {
    protected_control_unavailable("pause")
}

#[tauri::command]
fn desktop_stop() -> Result<Value, String> {
    protected_control_unavailable("stop")
}

#[tauri::command]
fn desktop_takeover() -> Result<Value, String> {
    protected_control_unavailable("takeover")
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            desktop_status,
            desktop_pause,
            desktop_stop,
            desktop_takeover
        ])
        .run(tauri::generate_context!())
        .expect("Golam desktop host failed to start");
}
