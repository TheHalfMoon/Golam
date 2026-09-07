#![forbid(unsafe_code)]

mod host;

use golam_ipc::desktop_control::DesktopControlRequest;
use serde_json::{Value, json};

fn host_status_value(status: host::DesktopHostStatus) -> Value {
    json!({
        "connected": status.connected,
        "clientId": status.client_id.map(|value| value.to_string()),
        "reason": status.reason,
        "controlState": status.control_state,
        "visibleChannelQualified": status.visible_channel_qualified
    })
}

fn control_result_value(result: host::DesktopControlResult) -> Value {
    json!({
        "clientId": result.client_id.to_string(),
        "controlState": result.reply.state.as_str(),
        "visibleChannelQualified": result.reply.visible_channel_qualified
    })
}

#[tauri::command]
fn desktop_status() -> Value {
    host_status_value(host::authenticate_status())
}

fn protected_control(request: DesktopControlRequest) -> Result<Value, String> {
    host::execute_desktop_control(request).map(control_result_value)
}

#[tauri::command]
fn desktop_pause() -> Result<Value, String> {
    protected_control(DesktopControlRequest::Pause)
}

#[tauri::command]
fn desktop_stop() -> Result<Value, String> {
    protected_control(DesktopControlRequest::Stop)
}

#[tauri::command]
fn desktop_takeover() -> Result<Value, String> {
    protected_control(DesktopControlRequest::Takeover)
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
