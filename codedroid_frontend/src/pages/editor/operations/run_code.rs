use crate::api;
use crate::store;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

pub fn make_run_code(
    pid: String,
    ppath: String,
    plang: String,
    code: RwSignal<String>,
    is_running: RwSignal<bool>,
    output: RwSignal<String>,
    is_error: RwSignal<bool>,
    current_pid: RwSignal<Option<u32>>,
    preview_url: RwSignal<Option<String>>,
    save_current: Callback<()>,
) -> Callback<()> {
    Callback::new(move |_: ()| {
        if is_running.get_untracked() {
            return;
        }
        save_current.run(());
        let current_code = code.get_untracked();
        let lang = plang.clone();
        let path = ppath.clone();
        let pid2 = pid.clone();

        is_running.set(true);
        output.set("Compiling and running...".to_string());
        is_error.set(false);

        let cargo_toml = if lang == "rust" {
            let k = store::file_key(&pid2, "Cargo.toml");
            let v = store::load_file(&k);
            if v.is_empty() {
                None
            } else {
                Some(v)
            }
        } else {
            None
        };

        spawn_local(async move {
            let res = api::run_code(&current_code, &lang, &path, cargo_toml.as_deref()).await;
            match res {
                Ok(r) => {
                    let mut out = r.output.clone();
                    if !r.error.is_empty() {
                        if !out.is_empty() {
                            out.push('\n');
                        }
                        out.push_str(&r.error);
                    }
                    if out.is_empty() {
                        out = "Code executed with no output.".to_string();
                    }
                    output.set(out);
                    is_error.set(!r.error.is_empty());
                    current_pid.set(r.pid);
                    if let Some(url) = r.url {
                        preview_url.set(Some(url));
                    }
                }
                Err(e) => {
                    output.set(format!("❌ Error: Could not connect to API.\n{e}"));
                    is_error.set(true);
                }
            }
            is_running.set(false);
        });
    })
}
