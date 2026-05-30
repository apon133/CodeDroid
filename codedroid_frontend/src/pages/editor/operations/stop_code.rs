use crate::api;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

pub fn make_stop_code(
    current_pid: RwSignal<Option<u32>>,
    output: RwSignal<String>,
    preview_url: RwSignal<Option<String>>,
    bottom_tab: RwSignal<usize>,
) -> Callback<()> {
    Callback::new(move |_: ()| {
        if let Some(pid_val) = current_pid.get_untracked() {
            spawn_local(async move {
                let _ = api::stop_process(pid_val).await;
                output.update(|o| o.push_str("\n\n[Stopped by User]"));
                current_pid.set(None);
                preview_url.set(None);
                bottom_tab.set(0);
            });
        }
    })
}
