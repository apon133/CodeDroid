use crate::api;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

pub fn make_stop_code(
    current_pid: RwSignal<Option<u32>>,
    output: RwSignal<String>,
    preview_url: RwSignal<Option<String>>,
    bottom_tab: RwSignal<usize>,
    terminal_session_id: RwSignal<Option<String>>,
    is_running: RwSignal<bool>,
) -> Callback<()> {
    Callback::new(move |_: ()| {
        is_running.set(false);
        if let Some(pid_val) = current_pid.get_untracked() {
            let preview_url_clone = preview_url.clone();
            let current_pid_clone = current_pid.clone();
            let output_clone = output;
            spawn_local(async move {
                let _ = api::stop_process(pid_val).await;
                output_clone.update(|o| o.push_str("\n\n[Stopped by User]"));
                current_pid_clone.set(None);
                preview_url_clone.set(None);
            });
        } else if let Some(session_id) = terminal_session_id.get_untracked() {
            if session_id == "initializing" {
                terminal_session_id.set(None);
                is_running.set(false);
                return;
            }
            let terminal_session_id_clone = terminal_session_id.clone();
            let output_clone = output;
            spawn_local(async move {
                let _ = api::stop_terminal_api(&session_id).await;
                output_clone.update(|o| o.push_str("\n\n[Stopped by User]\n"));
                terminal_session_id_clone.set(None);
            });
        }
        bottom_tab.set(0);
    })
}
