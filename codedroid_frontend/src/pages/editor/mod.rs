use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use wasm_bindgen_futures::spawn_local;

pub mod utils;
pub mod components;

use utils::*;
use components::*;
use crate::models::{Project, Settings, lang_icon};
use crate::store;
use crate::api;
use crate::components::app_bar::AppBar;
use crate::components::snackbar::Snackbar;

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
    let project_lang_str = StoredValue::new(project.language.clone());
    let project_path_str = StoredValue::new(project.path.clone());

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
    let file_tree_data: RwSignal<Vec<FileEntry>> = RwSignal::new(build_file_tree(&project.id));
    let show_deps: RwSignal<bool> = RwSignal::new(false);
    let dep_input: RwSignal<String> = RwSignal::new(String::new());
    let dep_output: RwSignal<String> = RwSignal::new(String::new());
    let suggestions = RwSignal::new(Vec::<api::CompletionItem>::new());
    let selected_idx = RwSignal::new(0);
    let cursor_pos = RwSignal::new(0);
    let cursor_coords = RwSignal::new((0.0, 0.0));
    let last_request_id = RwSignal::new(0u64);

    // Callbacks
    let show_snack = Callback::new({
        let snack = snack_msg;
        move |msg: String| {
            snack.set(Some(msg));
            let s2 = snack;
            gloo_timers::callback::Timeout::new(3000, move || s2.set(None)).forget();
        }
    });

    let pid = project.id.clone();
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

    let ppath = project.path.clone();
    let save_current = Callback::new({
        let pid = pid.clone();
        let ppath = ppath.clone();
        move |_: ()| {
            if let Some(tab) = active_tab.get_untracked() {
                let key = store::file_key(&pid, &tab);
                let content = code.get_untracked();
                store::save_file(&key, &content);
                dirty.set(false);

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

    let run_code = Callback::new({
        let pid = pid.clone();
        let ppath = ppath.clone();
        let plang = project.language.clone();
        move |_: ()| {
            if is_running.get_untracked() { return; }
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
                if v.is_empty() { None } else { Some(v) }
            } else { None };

            spawn_local(async move {
                let res = api::run_code(&current_code, &lang, &path, cargo_toml.as_deref()).await;
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

    let add_dep = Callback::new({
        let ppath = ppath.clone();
        let plang = project.language.clone();
        move |_: ()| {
            let pkg = dep_input.get_untracked();
            if pkg.trim().is_empty() { return; }
            let path = ppath.clone();
            let lang = plang.clone();
            dep_output.set(format!("Installing {}...", pkg));
            spawn_local(async move {
                match api::add_package(&pkg, &lang, &path).await {
                    Ok(r) => dep_output.set(if r.error.is_empty() { r.output } else { r.error }),
                    Err(e) => dep_output.set(format!("Error: {e}")),
                }
            });
        }
    });

    let copy_code = Callback::new({
        let show_snack = show_snack.clone();
        move |_: ()| {
            let c = code.get_untracked();
            if let Some(window) = web_sys::window() {
                let _ = window.navigator().clipboard().write_text(&c);
                show_snack.run("Code copied!".to_string());
            }
        }
    });

    let on_select = move |ins: String| {
        let cpos = cursor_pos.get_untracked();
        use wasm_bindgen::JsCast;
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            if let Ok(Some(target)) = doc.query_selector(".code-editor") {
                if let Ok(target) = target.dyn_into::<web_sys::HtmlTextAreaElement>() {
                    let start = target.selection_start().unwrap().unwrap_or(cpos);
                    let end = target.selection_end().unwrap().unwrap_or(cpos);
                    let val = js_sys::JsString::from(target.value());
                    let rust_val = String::from(val.clone());
                    let mut word_start = start as usize;
                    let chars_vec: Vec<char> = rust_val.chars().take(start as usize).collect();
                    for (i, c) in chars_vec.into_iter().enumerate().rev() {
                        if !c.is_alphanumeric() && c != '_' {
                            word_start = i + 1;
                            break;
                        }
                        if i == 0 { word_start = 0; }
                    }
                    let before = val.substring(0, word_start as u32);
                    let after = val.substring(end, val.length());
                    let new_val = format!("{}{}{}", String::from(before), ins, String::from(after));
                    code.set(new_val);
                    dirty.set(true);
                    suggestions.set(Vec::new());
                    let new_pos = word_start as u32 + ins.encode_utf16().count() as u32;
                    spawn_local(async move {
                        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                            if let Ok(Some(target)) = doc.query_selector(".code-editor") {
                                if let Ok(target) = target.dyn_into::<web_sys::HtmlTextAreaElement>() {
                                    let _ = target.focus();
                                    target.set_selection_start(Some(new_pos)).unwrap();
                                    target.set_selection_end(Some(new_pos)).unwrap();
                                }
                            }
                        }
                    });
                }
            }
        }
    };

    // Open default file on mount
    Effect::new(move |_| {
        let tree = file_tree_data.get();
        if !tree.is_empty() && active_tab.get_untracked().is_none() {
            let language = project_lang_str.get_value().to_lowercase();
            let mut best_match = None;
            
            // Priority 1: Match standard entry point for the project language
            for e in tree.iter() {
                let n = e.name.to_lowercase();
                match language.as_str() {
                    "rust" if n == "src/main.rs" || n == "main.rs" => best_match = Some(e.name.clone()),
                    "go" if n == "main.go" => best_match = Some(e.name.clone()),
                    "dart" if n == "main.dart" => best_match = Some(e.name.clone()),
                    "python" if n == "main.py" => best_match = Some(e.name.clone()),
                    "java" if n == "main.java" || n == "src/main.java" => best_match = Some(e.name.clone()),
                    "c" if n == "main.c" => best_match = Some(e.name.clone()),
                    "cpp" if n == "main.cpp" => best_match = Some(e.name.clone()),
                    "javascript" | "typescript" if n == "main.js" || n == "main.ts" || n == "index.js" || n == "index.ts" => best_match = Some(e.name.clone()),
                    _ => {}
                }
                if best_match.is_some() { break; }
            }

            // Priority 2: Match any entry point from the general list
            if best_match.is_none() {
                let main_files = [
                    "src/main.rs", "main.rs", "main.dart", "main.go", "main.py",
                    "main.js", "main.ts", "src/main.js", "src/main.ts",
                    "src/main.jsx", "src/main.tsx", "index.js", "index.ts",
                    "index.html", "Main.java", "main.c", "main.cpp",
                    "Program.cs", "main.kt", "main.swift", "main.rb",
                ];
                for e in tree.iter() {
                    let n = e.name.to_lowercase();
                    if main_files.iter().any(|&m| {
                        let m_low = m.to_lowercase();
                        n == m_low || n.ends_with(&format!("/{}", m_low))
                    }) {
                        best_match = Some(e.name.clone());
                        break;
                    }
                }
            }

            let default_file = best_match.unwrap_or_else(|| tree[0].name.clone());

            spawn_local(async move {
                // Small delay to ensure the editor and store are fully ready
                gloo_timers::future::TimeoutFuture::new(100).await;
                if active_tab.get_untracked().is_none() {
                    open_file.run(default_file);
                }
            });
        }
    });

    view! {
        <div class="editor-page-root">
            <AppBar title=project.name.clone() back=true>
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
                    <button class="btn btn-danger" on:click=move |_| stop_code.run(())>
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
                <FileTree 
                    file_tree=file_tree_data.into()
                    active_tab=active_tab.into()
                    open_file=open_file
                    lang_icon=lang_icon(&project_lang_str.get_value()).to_string()
                    project_name=project.name.clone()
                />

                <div class="editor-main">
                    <TabStrip 
                        open_tabs=open_tabs.into()
                        active_tab=active_tab.into()
                        dirty=dirty.into()
                        open_file=open_file
                        close_tab=close_tab
                    />

                    {move || show_search.get().then(|| view! {
                        <div class="search-bar">
                            <input class="input" type="text" placeholder="Find..."
                                prop:value=move || find_text.get()
                                on:input=move |e| find_text.set(event_target_value(&e))
                            />
                            <button class="btn btn-primary" style="padding:6px 12px;font-size:12px">"Find Next"</button>
                            <button class="btn btn-icon" on:click=move |_| show_search.set(false)>"×"</button>
                        </div>
                    })}

                    <div class="code-area" style="flex:2">
                        {move || {
                            let s = settings.get();
                            let content = code.get();
                            let line_count = content.lines().count().max(1);

                            view! {
                                <>
                                {move || s.show_line_numbers.then(|| view! {
                                    <div class="line-numbers" style=format!("font-size:{}px", s.font_size)>
                                        {(1..=line_count).map(|n| view! { <div>{n}</div> }).collect_view()}
                                    </div>
                                })}
                                <div class="code-container" style=move || format!(
                                        "font-size:{}px;white-space:{};tab-size:{}",
                                        settings.get().font_size,
                                        if settings.get().word_wrap { "pre-wrap" } else { "pre" },
                                        settings.get().tab_size,
                                    )>
                                    <div class="code-layer code-highlight" inner_html=move || {
                                        let c = code.get();
                                        let ext = active_tab.get().map(|n| file_extension(&n).to_string()).unwrap_or_default();
                                        highlight_code(&c, &ext)
                                    } />
                                    <textarea
                                        class="code-layer code-editor"
                                        spellcheck="false"
                                        prop:value=move || code.get()
                                        on:input=move |e: web_sys::Event| {
                                            use wasm_bindgen::JsCast;
                                            let target = e.target().unwrap().unchecked_into::<web_sys::HtmlTextAreaElement>();
                                            let val = target.value();
                                            code.set(val.clone());
                                            dirty.set(true);
                                            if settings.get_untracked().auto_save { save_current.run(()); }

                                            let start = target.selection_start().unwrap().unwrap_or(0);
                                            cursor_pos.set(start);
                                            
                                            if let Some(mirror) = web_sys::window().unwrap().document().unwrap().get_element_by_id("cursor-mirror") {
                                                let text_before = &val[..start as usize];
                                                mirror.set_text_content(Some(text_before));
                                                let span = web_sys::window().unwrap().document().unwrap().create_element("span").unwrap();
                                                span.set_text_content(Some("|"));
                                                let _ = mirror.append_child(&span);
                                                let span_el = span.dyn_into::<web_sys::HtmlElement>().unwrap();
                                                cursor_coords.set((span_el.offset_left() as f64, span_el.offset_top() as f64 + 20.0));
                                            }

                                            let (line, character) = {
                                                let text_before = &val[..start as usize];
                                                let lines: Vec<&str> = text_before.split('\n').collect();
                                                (lines.len().saturating_sub(1) as u32, lines.last().map(|l| l.chars().count()).unwrap_or(0) as u32)
                                            };
                                            selected_idx.set(0);

                                            let chars: Vec<char> = val.chars().collect();
                                            if start > 0 && start as usize <= chars.len() {
                                                let last_char = chars[(start - 1) as usize];
                                                if last_char.is_alphanumeric() || last_char == '.' {
                                                    let lang = project_lang_str.get_value();
                                                    let path = project_path_str.get_value();
                                                    let req_id = last_request_id.get_untracked() + 1;
                                                    last_request_id.set(req_id);
                                                    spawn_local(async move {
                                                        gloo_timers::future::TimeoutFuture::new(150).await;
                                                        if last_request_id.get_untracked() == req_id {
                                                            if let Ok(resp) = api::get_completions_api(&val, &lang, &path, line, character).await {
                                                                if last_request_id.get_untracked() == req_id { suggestions.set(resp.suggestions); }
                                                            }
                                                        }
                                                    });
                                                } else { suggestions.set(Vec::new()); }
                                            } else { suggestions.set(Vec::new()); }
                                        }
                                        on:keydown=move |e: web_sys::KeyboardEvent| {
                                            if (e.ctrl_key() || e.meta_key()) && e.key() == "s" { e.prevent_default(); save_current.run(()); }
                                            if (e.ctrl_key() || e.meta_key()) && e.key() == "f" { e.prevent_default(); show_search.update(|v| *v = !*v); }
                                            if !suggestions.get().is_empty() {
                                                let current = selected_idx.get();
                                                let total = suggestions.get().len();
                                                match e.key().as_str() {
                                                    "ArrowDown" => { e.prevent_default(); selected_idx.set((current + 1) % total); }
                                                    "ArrowUp" => { e.prevent_default(); selected_idx.set((current + total - 1) % total); }
                                                    "Enter" | "Tab" => { e.prevent_default(); if let Some(s) = suggestions.get().get(current) { on_select(s.label.clone()); } }
                                                    "Escape" => { suggestions.set(Vec::new()); }
                                                    _ => {}
                                                }
                                                return;
                                            }
                                            if e.ctrl_key() && e.key() == " " {
                                                e.prevent_default();
                                                use wasm_bindgen::JsCast;
                                                let target = e.target().unwrap().unchecked_into::<web_sys::HtmlTextAreaElement>();
                                                let val = target.value();
                                                let start = target.selection_start().unwrap().unwrap_or(0);
                                                let lang = project_lang_str.get_value();
                                                let path = project_path_str.get_value();
                                                let before_cursor = val.chars().take(start as usize).collect::<String>();
                                                let line = before_cursor.lines().count().saturating_sub(1) as u32;
                                                let character = before_cursor.lines().last().unwrap_or("").chars().count() as u32;
                                                spawn_local(async move {
                                                    if let Ok(resp) = api::get_completions_api(&val, &lang, &path, line, character).await {
                                                        suggestions.set(resp.suggestions);
                                                    }
                                                });
                                            }
                                            if e.key() == "Tab" {
                                                e.prevent_default();
                                                let spaces = " ".repeat(settings.get_untracked().tab_size);
                                                use wasm_bindgen::JsCast;
                                                let target = e.target().unwrap().unchecked_into::<web_sys::HtmlTextAreaElement>();
                                                let start = target.selection_start().unwrap().unwrap_or(0);
                                                let end = target.selection_end().unwrap().unwrap_or(0);
                                                let val = js_sys::JsString::from(target.value());
                                                let new_val = format!("{}{}{}", String::from(val.substring(0, start)), spaces, String::from(val.substring(end, val.length())));
                                                code.set(new_val);
                                                dirty.set(true);
                                                let new_pos = start + spaces.len() as u32;
                                                spawn_local(async move {
                                                    let _ = gloo_timers::future::sleep(std::time::Duration::from_millis(10)).await;
                                                    let _ = target.set_selection_range(new_pos, new_pos);
                                                });
                                            }
                                        }
                                    />
                                    {move || (!suggestions.get().is_empty()).then(|| {
                                        let coords = cursor_coords.get();
                                        let items = suggestions.get();
                                        let selected = selected_idx.get();
                                        let current_item = items.get(selected).cloned();
                                        view! {
                                            <div class="suggestions-floating" style=format!("left:{}px; top:{}px", coords.0, coords.1)>
                                                {move || suggestions.get().into_iter().enumerate().map(|(i, s)| {
                                                    let s2 = s.clone();
                                                    view! {
                                                        <button 
                                                            class=move || if selected_idx.get() == i { "suggestion-item selected" } else { "suggestion-item" }
                                                            on:click=move |_| on_select(s2.label.clone())
                                                            on:mouseenter=move |_| selected_idx.set(i)
                                                        >
                                                            <span class="suggestion-kind">{kind_icon(s.kind)}</span>
                                                            <span class="suggestion-label">{s.label.clone()}</span>
                                                            {s.detail.map(|d| view! { <span class="suggestion-detail">{d}</span> })}
                                                        </button>
                                                    }
                                                }).collect_view()}
                                                {move || current_item.as_ref().and_then(|item| item.documentation.as_ref()).map(|docs| view! {
                                                    <div class="suggestion-docs">{docs.clone()}</div>
                                                })}
                                            </div>
                                        }
                                    })}
                                    <div id="cursor-mirror" style=move || format!(
                                        "width:100%;font-size:{}px;line-height:1.6;tab-size:{}",
                                        settings.get().font_size,
                                        settings.get().tab_size
                                    ) />
                                </div>
                                </>
                            }
                        }}
                    </div>

                    <BottomPanel 
                        bottom_tab=bottom_tab
                        preview_url=preview_url.into()
                        output=output.into()
                        is_error=is_error.into()
                        show_snack=show_snack
                    />

                    <div class="editor-footer">
                        {["TAB","{}","[]","()","\"\"","''","->","=>","::","/ /","/* */"].iter().map(|s| {
                            let s_val = s.replace(" ", "");
                            let s_val_2 = s_val.clone();
                            view! {
                                <button class="btn btn-footer" on:click=move |_| {
                                    let ins = if s_val_2 == "TAB" { " ".repeat(settings.get_untracked().tab_size) } else { s_val_2.clone() };
                                    use wasm_bindgen::JsCast;
                                    if let Some(target) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.query_selector(".code-editor").ok().flatten()) {
                                        if let Ok(target) = target.dyn_into::<web_sys::HtmlTextAreaElement>() {
                                            let start = target.selection_start().unwrap().unwrap_or(0);
                                            let end = target.selection_end().unwrap().unwrap_or(0);
                                            let val = js_sys::JsString::from(target.value());
                                            code.set(format!("{}{}{}", String::from(val.substring(0, start)), ins, String::from(val.substring(end, val.length()))));
                                            dirty.set(true);
                                            let new_pos = start + ins.encode_utf16().count() as u32;
                                            spawn_local(async move {
                                                let _ = gloo_timers::future::sleep(std::time::Duration::from_millis(10)).await;
                                                let _ = target.focus();
                                                let _ = target.set_selection_range(new_pos, new_pos);
                                            });
                                        }
                                    }
                                }>{s_val}</button>
                            }
                        }).collect_view()}
                        <div style="flex:1" />
                        <button class="btn btn-footer" on:click=move |_| copy_code.run(())>"📋 Copy"</button>
                        <button class="btn btn-footer" on:click=move |_| { code.set(String::new()); dirty.set(true); }>"🗑 Clear"</button>
                    </div>
                </div>
            </div>

            <DependencyModal 
                show_deps=show_deps
                dep_input=dep_input
                dep_output=dep_output.into()
                add_dep=add_dep
            />

            <Snackbar message=snack_msg.read_only() />
        </div>
    }.into_any()
}
