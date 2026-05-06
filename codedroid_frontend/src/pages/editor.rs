use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use web_sys::{Event, KeyboardEvent, MouseEvent};
use wasm_bindgen_futures::spawn_local;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;

thread_local! {
    pub static SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    pub static THEME_SET: ThemeSet = ThemeSet::load_defaults();
}

fn highlight_code(code: &str, ext: &str) -> String {
    let mapped_ext = match ext {
        "dart" | "kt" => "java",
        "ts" | "tsx" | "jsx" => "js",
        "swift" => "cs",
        _ => ext,
    };

    SYNTAX_SET.with(|ss| {
        THEME_SET.with(|ts| {
            let syntax = ss.find_syntax_by_extension(mapped_ext)
                .unwrap_or_else(|| ss.find_syntax_plain_text());
            let theme = &ts.themes["base16-ocean.dark"];
            highlighted_html_for_string(code, ss, syntax, theme).unwrap_or_else(|_| code.to_string())
        })
    })
}

use crate::models::{Project, Settings, lang_icon};
use crate::store;
use crate::api;
use crate::components::app_bar::AppBar;
use crate::components::snackbar::Snackbar;

// ─── File Tree ────────────────────────────────────────────────────────────
#[derive(Clone, PartialEq)]
struct FileEntry {
    name: String,
    key: String,
    is_dir: bool,
}

fn build_file_tree(project_id: &str) -> Vec<FileEntry> {
    // Scan localStorage for files belonging to this project
    let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    let len = storage.length().unwrap_or(0);
    let prefix = format!("codedroid_file_{}_", project_id);
    let mut entries: Vec<FileEntry> = Vec::new();

    for i in 0..len {
        if let Ok(Some(k)) = storage.key(i) {
            if let Some(rel) = k.strip_prefix(&prefix) {
                entries.push(FileEntry {
                    name: rel.to_string(),
                    key: k.clone(),
                    is_dir: false,
                });
            }
        }
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

fn file_extension(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or("")
}

fn file_icon(name: &str) -> &'static str {
    match file_extension(name) {
        "rs"   => "🦀", "go"   => "🐹", "py"   => "🐍",
        "js" | "ts" | "jsx" | "tsx" => "⚡",
        "java" => "☕", "dart" => "🎯", "c" | "cpp" | "h" | "hpp" => "⚙️",
        "cs"   => "🔷", "kt"   => "🟣", "swift" => "🍎", "rb"   => "💎",
        "html" => "🌐", "css"  => "🎨", "toml" | "yaml" | "json" => "📋",
        _      => "📄",
    }
}

// ─── Editor Page ─────────────────────────────────────────────────────────
#[component]
pub fn EditorPage() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();

    // Resolve project
    let projects = store::load_projects();
    let project_id = move || params.get().get("id").unwrap_or_default().clone();
    let project: Option<Project> = {
        let id = project_id();
        projects.into_iter().find(|p| p.id == id)
    };

    if project.is_none() {
        let nav = navigate.clone();
        nav("/", Default::default());
        return view! { <div>"Redirecting..."</div> }.into_any();
    }
    let project = project.unwrap();

    // State
    let settings: RwSignal<Settings> = RwSignal::new(store::load_settings());
    let open_tabs: RwSignal<Vec<String>> = RwSignal::new(Vec::new());
    let active_tab: RwSignal<Option<String>> = RwSignal::new(None);
    let code: RwSignal<String> = RwSignal::new(String::new());
    let dirty: RwSignal<bool> = RwSignal::new(false);
    let output: RwSignal<String> = RwSignal::new("// Output will appear here...".to_string());
    let is_error: RwSignal<bool> = RwSignal::new(false);
    let is_running: RwSignal<bool> = RwSignal::new(false);
    let current_pid: RwSignal<Option<u32>> = RwSignal::new(None);
    let preview_url: RwSignal<Option<String>> = RwSignal::new(None);
    let bottom_tab: RwSignal<usize> = RwSignal::new(0); // 0=terminal 1=preview
    let show_search: RwSignal<bool> = RwSignal::new(false);
    let find_text: RwSignal<String> = RwSignal::new(String::new());
    let snack_msg: RwSignal<Option<String>> = RwSignal::new(None);
    let file_tree: RwSignal<Vec<FileEntry>> = RwSignal::new(build_file_tree(&project.id));
    let show_deps: RwSignal<bool> = RwSignal::new(false);
    let dep_input: RwSignal<String> = RwSignal::new(String::new());
    let dep_output: RwSignal<String> = RwSignal::new(String::new());

    let pid = project.id.clone();
    let ppath = project.path.clone();
    let plang = project.language.clone();

    // Open default file on mount
    {
        let tree = file_tree.get_untracked();
        if !tree.is_empty() {
            let first = &tree[0];
            let content = store::load_file(&first.key);
            open_tabs.update(|t| { if !t.contains(&first.name) { t.push(first.name.clone()); }});
            active_tab.set(Some(first.name.clone()));
            code.set(content);
        }
    }

    // ── Helpers ──────────────────────────────────────────────────────────
    let show_snack = Callback::new({
        let snack = snack_msg;
        move |msg: String| {
            snack.set(Some(msg));
            let s2 = snack;
            gloo_timers::callback::Timeout::new(3000, move || s2.set(None)).forget();
        }
    });

    let open_file = Callback::new({
        let pid = pid.clone();
        move |name: String| {
            let key = store::file_key(&pid, &name);
            let content = store::load_file(&key);
            open_tabs.update(|t| { if !t.contains(&name) { t.push(name.clone()); }});
            active_tab.set(Some(name));
            code.set(content);
            dirty.set(false);
        }
    });

    let ppath_save = ppath.clone();
    let save_current = Callback::new({
        let pid = pid.clone();
        let ppath = ppath_save;
        move |_: ()| {
            if let Some(tab) = active_tab.get_untracked() {
                let key = store::file_key(&pid, &tab);
                let content = code.get_untracked();
                store::save_file(&key, &content);
                dirty.set(false);

                // Sync with backend
                let base_path = ppath.clone();
                let tab_name = tab.clone();
                spawn_local(async move {
                    let full_path = format!("{}/{}", base_path, tab_name);
                    let _ = api::save_file_api(&full_path, &content).await;
                });
            }
        }
    });

    let close_tab = Callback::new({
        let pid = pid.clone();
        move |name: String| {
            open_tabs.update(|t| t.retain(|n| *n != name));
            let tabs = open_tabs.get_untracked();
            if active_tab.get_untracked().as_deref() == Some(&name) {
                if let Some(first) = tabs.first() {
                    let key = store::file_key(&pid, first);
                    code.set(store::load_file(&key));
                    active_tab.set(Some(first.clone()));
                } else {
                    active_tab.set(None);
                    code.set(String::new());
                }
            }
        }
    });

    // ── Run Code ─────────────────────────────────────────────────────────
    let ppath_run = ppath.clone();
    let run_code = Callback::new({
        let pid = pid.clone();
        let ppath = ppath_run;
        let plang = plang.clone();
        move |_: ()| {
            if is_running.get_untracked() { return; }

            // Save first
            save_current.run(());

            let current_code = code.get_untracked();
            let lang = plang.clone();
            let path = ppath.clone();
            let pid2 = pid.clone();

            is_running.set(true);
            output.set("Compiling and running...".to_string());
            is_error.set(false);

            // Load Cargo.toml for Rust projects
            let cargo_toml = if lang == "rust" {
                let k = store::file_key(&pid2, "Cargo.toml");
                let v = store::load_file(&k);
                if v.is_empty() { None } else { Some(v) }
            } else { None };

            spawn_local(async move {
                let res = api::run_code(
                    &current_code,
                    &lang,
                    &path,
                    cargo_toml.as_deref(),
                ).await;

                match res {
                    Ok(r) => {
                        let mut out = r.output.clone();
                        if !r.error.is_empty() {
                            if !out.is_empty() { out.push('\n'); }
                            out.push_str(&r.error);
                        }
                        if out.is_empty() { out = "Code executed with no output.".to_string(); }
                        output.set(out);
                        is_error.set(!r.error.is_empty());
                        current_pid.set(r.pid);
                        if let Some(url) = r.url {
                            preview_url.set(Some(url));
                            bottom_tab.set(1);
                        }
                    }
                    Err(e) => {
                        output.set(format!("❌ Error: Could not connect to API.\n{e}"));
                        is_error.set(true);
                    }
                }
                is_running.set(false);
            });
        }
    });

    // ── Stop ─────────────────────────────────────────────────────────────
    let stop_code = Callback::new(move |_: ()| {
        if let Some(pid_val) = current_pid.get_untracked() {
            spawn_local(async move {
                let _ = api::stop_process(pid_val).await;
                output.update(|o| o.push_str("\n\n[Stopped by User]"));
                current_pid.set(None);
                preview_url.set(None);
                bottom_tab.set(0);
            });
        }
    });

    // ── Copy code ────────────────────────────────────────────────────────
    let copy_code = Callback::new(move |_: ()| {
        let c = code.get_untracked();
        let window = web_sys::window().unwrap();
        let _ = window.navigator().clipboard().write_text(&c);
        show_snack.run("Code copied!".to_string());
    });

    // ─── Add dependency ──────────────────────────────────────────────────
    let add_dep = Callback::new({
        let ppath2 = ppath.clone();
        let plang2 = plang.clone();
        move |_: ()| {
            let pkg = dep_input.get_untracked();
            if pkg.trim().is_empty() { return; }
            let path = ppath2.clone();
            let lang = plang2.clone();
            dep_output.set(format!("Installing {}...", pkg));
            spawn_local(async move {
                match api::add_package(&pkg, &lang, &path).await {
                    Ok(r) => dep_output.set(if r.error.is_empty() { r.output } else { r.error }),
                    Err(e) => dep_output.set(format!("Error: {e}")),
                }
            });
        }
    });

    let proj_for_view = project.clone();

    view! {
        <div>
            <AppBar title=proj_for_view.name.clone() back=true>
                <button class="btn btn-icon" title="Search (Ctrl+F)"
                    on:click=move |_| show_search.update(|v| *v = !*v)>"🔍"</button>
                <button class="btn btn-icon" title="Dependencies"
                    on:click=move |_| show_deps.update(|v| *v = !*v)>"📦"</button>
                {move || dirty.get().then(|| view! {
                    <button class="btn btn-icon" title="Save (Ctrl+S)"
                        on:click=move |_| save_current.run(())
                    >"💾"</button>
                })}
                {move || current_pid.get().map(|_| view! {
                    <button class="btn btn-danger"
                        on:click=move |_| stop_code.run(())
                    >
                        <span>"⏹"</span>" Stop"
                    </button>
                })}
                <button class="btn btn-success" disabled=move || is_running.get()
                    on:click=move |_| run_code.run(())
                >
                    {move || if is_running.get() {
                        view! { <><span class="spinner"></span>" Running..."</> }.into_any()
                    } else {
                        view! { <>"▶ Run"</> }.into_any()
                    }}
                </button>
            </AppBar>

            <div class="editor-layout">

                <div class="file-tree-panel">
                    <div class="file-tree-header">
                        <span>{lang_icon(&proj_for_view.language)}" "{proj_for_view.name.to_uppercase()}</span>
                    </div>
                    {move || file_tree.get().into_iter().map(|f| {
                        let fname = f.name.clone();
                        let fname2 = f.name.clone();
                        view! {
                            <div
                                class=move || {
                                    let active = active_tab.get().as_deref() == Some(&fname2);
                                    if active { "file-item active" } else { "file-item" }
                                }
                                on:click=move |_| open_file.run(fname.clone())
                            >
                                <span>{file_icon(&f.name)}</span>
                                <span>{f.name.clone()}</span>
                            </div>
                        }
                    }).collect_view()}
                </div>

                <div class="editor-main">

                    <div class="editor-tabs">
                        {move || open_tabs.get().into_iter().map(|tab| {
                            let tab2 = tab.clone();
                            let tab3 = tab.clone();
                            view! {
                                <div
                                    class=move || {
                                        let mut c = "editor-tab".to_string();
                                        if active_tab.get().as_deref() == Some(&tab3) { c.push_str(" active"); }
                                        if dirty.get() && active_tab.get().as_deref() == Some(&tab3) { c.push_str(" dirty"); }
                                        c
                                    }
                                    on:click=move |_| open_file.run(tab2.clone())
                                >
                                    <span>{file_icon(&tab)}" "{tab.clone()}</span>
                                    <span class="tab-close"
                                        on:click=move |e: MouseEvent| {
                                            e.stop_propagation();
                                            close_tab.run(tab.clone());
                                        }
                                    >"×"</span>
                                </div>
                            }
                        }).collect_view()}
                    </div>

                    {move || show_search.get().then(|| view! {
                        <div class="search-bar">
                            <input class="input" type="text" placeholder="Find..."
                                prop:value=move || find_text.get()
                                on:input=move |e: Event| find_text.set(event_target_value(&e))
                            />
                            <button class="btn btn-primary" style="padding:6px 12px;font-size:12px"
                                on:click=move |_| { }
                            >"Find Next"</button>
                            <button class="btn btn-icon" on:click=move |_| show_search.set(false)>"×"</button>
                        </div>
                    })}

                    <div class="code-area" style="flex:2">
                        {move || {
                            let s = settings.get();
                            let font_size = s.font_size;
                            let show_ln = s.show_line_numbers;
                            let word_wrap = s.word_wrap;
                            let tab_size = s.tab_size;

                            let content = code.get();
                            let line_count = content.lines().count().max(1);

                            view! {
                                <>
                                {move || show_ln.then(|| view! {
                                    <div class="line-numbers" style=format!("font-size:{font_size}px")>
                                        {(1..=line_count).map(|n| view! {
                                            <div>{n}</div>
                                        }).collect_view()}
                                    </div>
                                })}
                                <div class="code-container" style=move || format!(
                                        "font-size:{}px;white-space:{};tab-size:{}",
                                        settings.get().font_size,
                                        if word_wrap { "pre-wrap" } else { "pre" },
                                        tab_size,
                                    )>
                                    <div class="code-layer code-highlight" inner_html=move || {
                                        let c = code.get();
                                        let ext = active_tab.get().map(|n| crate::pages::editor::file_extension(&n).to_string()).unwrap_or_default();
                                        highlight_code(&c, &ext)
                                    } />
                                    <textarea
                                        class="code-layer code-editor"
                                        spellcheck="false"
                                        prop:value=move || code.get()
                                        on:input=move |e: Event| {
                                        code.set(event_target_value(&e));
                                        dirty.set(true);
                                        // Auto-save
                                        if settings.get_untracked().auto_save {
                                            save_current.run(());
                                        }
                                    }
                                    on:keydown=move |e: KeyboardEvent| {
                                        // Ctrl+S / Cmd+S to save
                                        if (e.ctrl_key() || e.meta_key()) && e.key() == "s" {
                                            e.prevent_default();
                                            save_current.run(());
                                        }
                                        // Ctrl+F to toggle search
                                        if (e.ctrl_key() || e.meta_key()) && e.key() == "f" {
                                            e.prevent_default();
                                            show_search.update(|v| *v = !*v);
                                        }
                                        // Tab key → insert spaces
                                        if e.key() == "Tab" {
                                            e.prevent_default();
                                            let spaces = " ".repeat(settings.get_untracked().tab_size);
                                            use wasm_bindgen::JsCast;
                                            let target = e.target().unwrap().unchecked_into::<web_sys::HtmlTextAreaElement>();
                                            let start = target.selection_start().unwrap().unwrap_or(0);
                                            let end = target.selection_end().unwrap().unwrap_or(0);
                                            
                                            let val = js_sys::JsString::from(target.value());
                                            let before = val.substring(0, start);
                                            let after = val.substring(end, val.length());
                                            
                                            let new_val = format!("{}{}{}", String::from(before), spaces, String::from(after));
                                            
                                            code.set(new_val);
                                            dirty.set(true);
                                            
                                            let new_pos = start + spaces.len() as u32;
                                            wasm_bindgen_futures::spawn_local(async move {
                                                let _ = gloo_timers::future::sleep(std::time::Duration::from_millis(10)).await;
                                                let _ = target.set_selection_range(new_pos, new_pos);
                                            });
                                        }
                                    }
                                    />
                                </div>
                                </>
                            }
                        }}
                    </div>

                    <div class="bottom-panel">
                        <div class="bottom-tabs">
                            <button
                                class=move || if bottom_tab.get() == 0 { "bottom-tab active" } else { "bottom-tab" }
                                on:click=move |_| bottom_tab.set(0)
                            >"TERMINAL"</button>
                            {move || preview_url.get().map(|_| view! {
                                <button
                                    class=move || if bottom_tab.get() == 1 { "bottom-tab active" } else { "bottom-tab" }
                                    on:click=move |_| bottom_tab.set(1)
                                >"PREVIEW"</button>
                            })}
                            <div style="flex:1"/>
                            {move || (bottom_tab.get() == 0).then(|| view! {
                                <>
                                <button class="btn btn-icon" style="font-size:12px" title="Copy output"
                                    on:click=move |_| {
                                        let w = web_sys::window().unwrap();
                                        let _ = w.navigator().clipboard().write_text(&output.get_untracked());
                                        show_snack.run("Output copied!".to_string());
                                    }
                                >"📋"</button>
                                <button class="btn btn-icon" style="font-size:12px" title="Clear"
                                    on:click=move |_| output.set("// Output cleared...".to_string())
                                >"🗑"</button>
                                </>
                            })}
                        </div>
                        {move || {
                            if bottom_tab.get() == 1 {
                                if let Some(url) = preview_url.get() {
                                    return view! {
                                        <iframe class="preview-frame" src=url />
                                    }.into_any();
                                }
                            }
                            view! {
                                <div
                                    class=move || if is_error.get() { "terminal error" } else { "terminal" }
                                >
                                    {move || output.get()}
                                </div>
                            }.into_any()
                        }}
                    </div>

                    <div style="display:flex;gap:6px;padding:6px;background:var(--bg2);border-top:1px solid var(--border);overflow-x:auto">
                        {["TAB","{}","[]","()","\"\"","''","->","=>","::","/ /","/* */"].iter().map(|s| {
                            let s = s.replace(" ", "");
                            view! {
                                <button class="btn" style="padding:4px 10px;font-family:var(--font-mono);font-size:12px;background:var(--bg3);color:var(--text2);flex-shrink:0"
                                    on:click=move |_| {
                                        let ins = if s == "TAB" {
                                            " ".repeat(settings.get_untracked().tab_size)
                                        } else {
                                            s.to_string()
                                        };
                                        use wasm_bindgen::JsCast;
                                        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                            if let Ok(Some(target)) = doc.query_selector(".code-editor") {
                                                if let Ok(target) = target.dyn_into::<web_sys::HtmlTextAreaElement>() {
                                                    let start = target.selection_start().unwrap().unwrap_or(0);
                                                    let end = target.selection_end().unwrap().unwrap_or(0);
                                                    let val = js_sys::JsString::from(target.value());
                                                    let before = val.substring(0, start);
                                                    let after = val.substring(end, val.length());
                                                    let new_val = format!("{}{}{}", String::from(before), ins, String::from(after));
                                                    
                                                    code.set(new_val);
                                                    dirty.set(true);
                                                    
                                                    let new_pos = start + ins.encode_utf16().count() as u32;
                                                    wasm_bindgen_futures::spawn_local(async move {
                                                        let _ = gloo_timers::future::sleep(std::time::Duration::from_millis(10)).await;
                                                        let _ = target.focus();
                                                        let _ = target.set_selection_range(new_pos, new_pos);
                                                    });
                                                    return;
                                                }
                                            }
                                        }
                                        code.update(|c| c.push_str(&ins));
                                        dirty.set(true);
                                    }
                                >{s.clone()}</button>
                            }
                        }).collect_view()}
                        <div style="margin-left:auto;display:flex;gap:6px">
                            <button class="btn" style="padding:4px 10px;font-size:12px;background:var(--bg3);color:var(--text2)"
                                on:click=move |_| copy_code.run(())
                            >"📋 Copy"</button>
                            <button class="btn" style="padding:4px 10px;font-size:12px;background:var(--bg3);color:var(--text2)"
                                on:click=move |_| { code.set(String::new()); dirty.set(true); }
                            >"🗑 Clear"</button>
                        </div>
                    </div>
                </div>
            </div>

            {move || show_deps.get().then(|| view! {
                <div class="modal-overlay" on:click=move |_| show_deps.set(false)>
                    <div class="modal" on:click=move |e: MouseEvent| e.stop_propagation()>
                        <div class="modal-header">"📦 Add Dependency"</div>
                        <div class="modal-body">
                            <div class="input-group">
                                <label>"Package Name"</label>
                                <input class="input" type="text" placeholder="e.g. serde, tokio, numpy"
                                    prop:value=move || dep_input.get()
                                    on:input=move |e: Event| dep_input.set(event_target_value(&e))
                                    on:keydown=move |e: KeyboardEvent| {
                                        if e.key() == "Enter" { add_dep.run(()); }
                                    }
                                />
                            </div>
                            {move || {
                                let out = dep_output.get();
                                if !out.is_empty() {
                                    Some(view! {
                                        <div class="terminal" style="border-radius:6px;margin-top:8px;max-height:150px">{out}</div>
                                    })
                                } else { None }
                            }}
                        </div>
                        <div class="modal-footer">
                            <button class="btn" style="background:transparent;color:var(--text2);border:1px solid var(--border)"
                                on:click=move |_| show_deps.set(false)
                            >"Close"</button>
                            <button class="btn btn-primary" on:click=move |_| add_dep.run(())>"Install"</button>
                        </div>
                    </div>
                </div>
            })}

            <Snackbar message=snack_msg.read_only() />
        </div>
    }.into_any()
}
