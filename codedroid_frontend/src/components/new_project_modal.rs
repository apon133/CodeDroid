use crate::api;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, MouseEvent};

// ─── Language & Framework data ────────────────────────────────────────────
pub const LANGUAGES: &[(&str, &str)] = &[
    ("rust", "Rust"),
    ("go", "Go"),
    ("dart", "Dart"),
    ("c", "C"),
    ("cpp", "C++"),
    ("csharp", "C#"),
    ("java", "Java"),
    ("python", "Python"),
    ("kotlin", "Kotlin"),
    ("swift", "Swift"),
    ("ruby", "Ruby"),
    ("javascript", "JavaScript"),
    ("typescript", "TypeScript"),
];

pub const FRAMEWORKS: &[(&str, &str, &str)] = &[
    ("vanilla", "Vanilla JS", "Pure JavaScript"),
    ("react", "React", "Vite + React"),
    ("vue", "Vue", "Vite + Vue"),
    ("svelte", "Svelte", "Vite + Svelte"),
    ("angular", "Angular", "Angular CLI"),
    ("nextjs", "Next.js", "Full-stack React"),
    ("remix", "Remix", "Web Standards"),
];

// ─── Props ────────────────────────────────────────────────────────────────
#[derive(Clone)]
pub struct NewProjectResult {
    pub name: String,
    pub lang: String,
    pub framework: String,
}

#[component]
pub fn NewProjectModal(
    on_create: Callback<NewProjectResult>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let lang = RwSignal::new("rust".to_string());
    let framework = RwSignal::new("vanilla".to_string());
    let active_tab = RwSignal::new(0usize); // 0 = Language, 1 = Web Framework, 2 = Git Clone
    let clone_url = RwSignal::new(String::new());
    let is_cloning = RwSignal::new(false);
    let error_msg = RwSignal::new(Option::<String>::None);

    // When switching to Web Framework tab, force JS
    let on_tab = move |idx: usize| {
        active_tab.set(idx);
        if idx == 1 {
            let l = lang.get_untracked();
            if l != "javascript" && l != "typescript" {
                lang.set("javascript".to_string());
            }
        }
    };

    let create = move |_: MouseEvent| {
        let n = name.get_untracked();
        let proj_name = n.trim().to_string();
        if proj_name.is_empty() {
            return;
        }

        let tab = active_tab.get_untracked();
        if tab == 2 {
            // Git Clone logic
            let url = clone_url.get_untracked();
            if url.trim().is_empty() {
                error_msg.set(Some("Please enter a repository URL.".to_string()));
                return;
            }

            is_cloning.set(true);
            error_msg.set(None);

            let on_create_clone = on_create.clone();
            spawn_local(async move {
                let res = api::git_clone_api(&url, &proj_name).await;
                match res {
                    Ok(resp) => {
                        if resp.success {
                            on_create_clone.run(NewProjectResult {
                                name: proj_name,
                                lang: "auto".to_string(),
                                framework: "none".to_string(),
                            });
                        } else {
                            let err = resp.error.unwrap_or_else(|| "Unknown git error".to_string());
                            error_msg.set(Some(format!("Git Clone failed: {}", err)));
                            is_cloning.set(false);
                        }
                    }
                    Err(e) => {
                        error_msg.set(Some(format!("API Error: {}", e)));
                        is_cloning.set(false);
                    }
                }
            });
        } else {
            let fw = if tab == 1 {
                framework.get_untracked()
            } else {
                "none".to_string()
            };
            on_create.run(NewProjectResult {
                name: proj_name,
                lang: lang.get_untracked(),
                framework: fw,
            });
        }
    };

    let cancel = move |_: MouseEvent| on_cancel.run(());

    let stop_prop = move |e: MouseEvent| {
        e.stop_propagation();
    };

    view! {
        <div class="modal-overlay" on:click=cancel>
            <div class="modal" on:click=stop_prop>
                <div class="modal-header">"New Project"</div>
                <div class="modal-body">
                    // Project name input
                    <div class="input-group">
                        <label>"Project Name"</label>
                        <input
                            class="input"
                            type="text"
                            placeholder="my_project"
                            autofocus
                            disabled=move || is_cloning.get()
                            prop:value=move || name.get()
                            on:input=move |e: Event| {
                                let v = event_target_value(&e);
                                name.set(v);
                            }
                        />
                    </div>

                    // Tabs
                    <div class="tabs">
                        <button
                            class=move || if active_tab.get() == 0 { "tab-btn active" } else { "tab-btn" }
                            disabled=move || is_cloning.get()
                            on:click=move |_| on_tab(0)
                        >"Language"</button>
                        <button
                            class=move || if active_tab.get() == 1 { "tab-btn active" } else { "tab-btn" }
                            disabled=move || is_cloning.get()
                            on:click=move |_| on_tab(1)
                        >"Web Framework"</button>
                        <button
                            class=move || if active_tab.get() == 2 { "tab-btn active" } else { "tab-btn" }
                            disabled=move || is_cloning.get()
                            on:click=move |_| on_tab(2)
                        >"Git Clone"</button>
                    </div>

                    // Language grid
                    <div class=move || if active_tab.get() == 0 { "tab-panel active" } else { "tab-panel" }>
                        <div class="lang-grid">
                            {LANGUAGES.iter().map(|(id, label)| {
                                let id = *id;
                                let label = *label;
                                view! {
                                    <div
                                        class=move || if lang.get() == id { "lang-item selected" } else { "lang-item" }
                                        on:click=move |_| lang.set(id.to_string())
                                    >
                                        {label}
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </div>

                    // Framework list
                    <div class=move || if active_tab.get() == 1 { "tab-panel active" } else { "tab-panel" }>
                        <div class="fw-panel-list">
                            {FRAMEWORKS.iter().map(|(id, name_fw, desc)| {
                                let id = *id;
                                let name_fw = *name_fw;
                                let desc = *desc;
                                view! {
                                    <div
                                        class=move || if framework.get() == id { "fw-item selected" } else { "fw-item" }
                                        on:click=move |_| {
                                            framework.set(id.to_string());
                                            lang.set("javascript".to_string());
                                        }
                                    >
                                        <div>
                                            <div class="fw-item-name">{name_fw}</div>
                                            <div class="fw-item-desc">{desc}</div>
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </div>

                    // Git Clone input panel
                    <div class=move || if active_tab.get() == 2 { "tab-panel active" } else { "tab-panel" }>
                        <div class="input-group" style="margin-top:12px;">
                            <label>"Repository URL"</label>
                            <input
                                class="input"
                                type="text"
                                placeholder="https://github.com/user/repo.git"
                                disabled=move || is_cloning.get()
                                prop:value=move || clone_url.get()
                                on:input=move |e: Event| {
                                    let v = event_target_value(&e);
                                    clone_url.set(v.clone());
                                    // Auto-fill project name from git repo URL if empty
                                    let current_name = name.get_untracked();
                                    if current_name.trim().is_empty() {
                                        let cleaned = v.trim_end_matches(".git");
                                        if let Some(last_slash) = cleaned.rfind('/') {
                                            let parsed_name = &cleaned[last_slash + 1..];
                                            if !parsed_name.is_empty() {
                                                name.set(parsed_name.to_string());
                                            }
                                        }
                                    }
                                }
                            />
                        </div>
                    </div>
                </div>

                {move || error_msg.get().map(|err| view! {
                    <div class="error-banner" style="color:#ff453a; font-size:13px; margin: 12px 20px 0; padding:8px 12px; background:rgba(255,69,58,0.1); border-radius:6px; border:1px solid rgba(255,69,58,0.2)">
                        {err}
                    </div>
                })}

                {move || is_cloning.get().then(|| view! {
                    <div class="cloning-loader" style="display:flex; align-items:center; justify-content:center; gap:8px; margin: 12px 20px 0; color:var(--text2); font-size:14px;">
                        <style>
                            "@keyframes spin {
                                0% { transform: rotate(0deg); }
                                100% { transform: rotate(360deg); }
                            }"
                        </style>
                        <span class="spinner" style="border:2px solid var(--border); border-top:2px solid var(--accent); border-radius:50%; width:16px; height:16px; display:inline-block; animation:spin 1s linear infinite"></span>
                        "Cloning repository... Please wait."
                    </div>
                })}

                <div class="modal-footer">
                    <button class="btn" on:click=cancel
                        disabled=move || is_cloning.get()
                        style="background:transparent;color:var(--text2);border:1px solid var(--border)">
                        "Cancel"
                    </button>
                    <button class="btn btn-primary" on:click=create disabled=move || is_cloning.get() || name.get().trim().is_empty()>
                        {move || if is_cloning.get() { "Cloning..." } else if active_tab.get() == 2 { "Clone & Create" } else { "Create Project" }}
                    </button>
                </div>
            </div>
        </div>
    }
}
