use crate::api;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

pub fn make_stop_code(
    is_running: RwSignal<bool>,
    current_pid: RwSignal<Option<u32>>,
    preview_url: RwSignal<Option<String>>,
    terminal_interrupt: RwSignal<u32>,
    output: RwSignal<String>,
) -> Callback<()> {
    Callback::new(move |_: ()| {
        if !is_running.get_untracked() {
            return;
        }

        let pid = current_pid.get_untracked();
        let had_preview = preview_url.get_untracked().is_some();

        spawn_local(async move {
            if pid.is_some() || had_preview {
                let stop_live = had_preview && pid.is_none();
                let _ = api::stop_process(pid, stop_live).await;
            }

            current_pid.set(None);
            if had_preview {
                preview_url.set(None);
            }

            terminal_interrupt.update(|n| *n += 1);

            let mut current = output.get_untracked();
            current.push_str("\n■ Run stopped.\n");
            output.set(current);

            is_running.set(false);
        });
    })
}
