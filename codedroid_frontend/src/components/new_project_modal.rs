use leptos::prelude::*;
use web_sys::{Event, MouseEvent};

// ─── Language & Framework data ────────────────────────────────────────────
pub const LANGUAGES: &[(&str, &str)] = &[
    ("rust", "Rust"), ("go", "Go"), ("dart", "Dart"), ("c", "C"),
    ("cpp", "C++"), ("csharp", "C#"), ("java", "Java"), ("python", "Python"),
    ("kotlin", "Kotlin"), ("swift", "Swift"), ("ruby", "Ruby"),
    ("javascript", "JavaScript"), ("typescript", "TypeScript"),
];

pub const FRAMEWORKS: &[(&str, &str, &str)] = &[
    ("vanilla", "Vanilla JS",  "Pure JavaScript"),
    ("react",   "React",       "Vite + React"),
    ("vue",     "Vue",         "Vite + Vue"),
    ("svelte",  "Svelte",      "Vite + Svelte"),
    ("angular", "Angular",     "Angular CLI"),
    ("nextjs",  "Next.js",     "Full-stack React"),
    ("remix",   "Remix",       "Web Standards"),
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
    let name       = RwSignal::new(String::new());
    let lang       = RwSignal::new("rust".to_string());
    let framework  = RwSignal::new("vanilla".to_string());
    let active_tab = RwSignal::new(0usize); // 0 = Language, 1 = Web Framework

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
        if n.trim().is_empty() { return; }
        let fw = if active_tab.get_untracked() == 1 {
            framework.get_untracked()
        } else {
            "none".to_string()
        };
        on_create.run(NewProjectResult { name: n.trim().to_string(), lang: lang.get_untracked(), framework: fw });
    };

    let cancel = move |_: MouseEvent| on_cancel.run(());

    let stop_prop = move |e: MouseEvent| { e.stop_propagation(); };

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
                            on:click=move |_| on_tab(0)
                        >"Language"</button>
                        <button
                            class=move || if active_tab.get() == 1 { "tab-btn active" } else { "tab-btn" }
                            on:click=move |_| on_tab(1)
                        >"Web Framework"</button>
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

                <div class="modal-footer">
                    <button class="btn" on:click=cancel
                        style="background:transparent;color:var(--text2);border:1px solid var(--border)">
                        "Cancel"
                    </button>
                    <button class="btn btn-primary" on:click=create>"Create Project"</button>
                </div>
            </div>
        </div>
    }
}
