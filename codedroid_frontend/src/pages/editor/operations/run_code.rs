use crate::api;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

pub fn make_run_code(
    ppath: String,
    plang: String,
    code: RwSignal<String>,
    active_tab: RwSignal<Option<String>>,
    is_running: RwSignal<bool>,
    current_pid: RwSignal<Option<u32>>,
    preview_url: RwSignal<Option<String>>,
    show_desktop_preview: RwSignal<bool>,
    output: RwSignal<String>,
    is_error: RwSignal<bool>,
    bottom_tab: RwSignal<usize>,
    bottom_open: RwSignal<bool>,
    terminal_auto_cmd: RwSignal<Option<String>>,
    save_current: Callback<bool>,
) -> Callback<()> {
    Callback::new(move |_: ()| {
        if is_running.get_untracked() {
            return;
        }

        save_current.run(true);
        bottom_open.set(true);
        bottom_tab.set(0);

        let current_code = code.get_untracked();
        let lang = plang.clone();
        let path = ppath.clone();
        let file_path = active_tab.get_untracked();

        spawn_local(async move {
            is_running.set(true);

            let res = api::run_code(
                &current_code,
                &lang,
                &path,
                file_path.as_deref(),
            )
            .await;

            match res {
                Ok(r) => {
                    if !r.error.is_empty() {
                        output.set(format!("❌ Run error:\n{}", r.error));
                        is_error.set(true);
                        bottom_tab.set(0);
                        is_running.set(false);
                        return;
                    }

                    if let Some(url) = r.url.clone() {
                        preview_url.set(Some(url));
                        show_desktop_preview.set(true);
                        if let Some(pid) = r.pid {
                            current_pid.set(Some(pid));
                        }
                        if !r.output.is_empty() {
                            let mut current = output.get_untracked();
                            current.push_str(&r.output);
                            output.set(current);
                        }
                        if r.pid.is_some() {
                            return;
                        }
                        is_running.set(false);
                        return;
                    }

                    if let Some(pid) = r.pid {
                        current_pid.set(Some(pid));
                        if !r.output.is_empty() {
                            let mut current = output.get_untracked();
                            current.push_str(&r.output);
                            output.set(current);
                        }
                        return;
                    }

                    if r.is_command != Some(true) {
                        if !r.output.is_empty() {
                            let mut current = output.get_untracked();
                            current.push_str(&r.output);
                            output.set(current);
                        }
                        is_running.set(false);
                        return;
                    }

                    let command_raw = r.output.trim();
                    let command_clean = command_raw
                        .replace('\r', "")
                        .chars()
                        .filter(|&c| !c.is_control() || c == '\n' || c == '\t')
                        .collect::<String>();
                    let command = command_clean.trim();

                    if command.is_empty() {
                        is_running.set(false);
                        return;
                    }

                    let mut current = output.get_untracked();
                    current.push_str(&format!("▶ {}\n", command));
                    output.set(current);

                    // Wrap so the interactive shell emits a completion marker when the
                    // foreground command finishes (the shell itself stays alive).
                    let wrapped = format!("{}; echo '[CODE_RUN_ENDED]'", command);
                    terminal_auto_cmd.set(Some(wrapped));
                }
                Err(e) => {
                    output.set(format!("❌ Run connection error:\n{}", e));
                    is_error.set(true);
                    bottom_tab.set(0);
                    is_running.set(false);
                }
            }
        });
    })
}
