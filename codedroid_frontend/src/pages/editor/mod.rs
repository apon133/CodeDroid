use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use wasm_bindgen_futures::spawn_local;

pub mod utils;
pub mod components;
pub mod preview;
pub mod footer;
pub mod search_bar;
pub mod code_area;
pub mod error_popover;
pub mod hover;
pub mod suggestions;
pub mod operations;
pub mod context_menu;

use utils::*;
use components::*;
use code_area::EditorCodeArea;
use preview::*;
use footer::*;
use search_bar::*;
use operations::*;
use crate::models::{Project, Settings, lang_icon};
use crate::store;
use crate::api;
use crate::components::app_bar::AppBar;
use crate::components::snackbar::Snackbar;
use crate::components::icon::LucideIcon;

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
    let show_mobile_full_preview: RwSignal<bool> = RwSignal::new(false);
    let refresh_key: RwSignal<u32> = RwSignal::new(0);
    let show_search: RwSignal<bool> = RwSignal::new(false);
    let find_text: RwSignal<String> = RwSignal::new(String::new());
    let find_index: RwSignal<usize> = RwSignal::new(0);
    let project_search_text: RwSignal<String> = RwSignal::new(String::new());
    let project_replace_text: RwSignal<String> = RwSignal::new(String::new());
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
    let diagnostics_list = RwSignal::new(Vec::<api::Diagnostic>::new());
    let references_list = RwSignal::new(Vec::<crate::api::Location>::new());
    let last_diag_req = RwSignal::new(0u64);
    let active_error = RwSignal::new(Option::<(api::Diagnostic, Vec<api::CodeSuggestion>, bool)>::None);

    // Callbacks
    let show_snack = Callback::new({
        let snack = snack_msg;
        move |msg: String| {
            snack.set(Some(msg));
            let s2 = snack;
            gloo_timers::callback::Timeout::new(3000, move || s2.set(None)).forget();
        }
    });

    let trigger_diagnostics = make_trigger_diagnostics(
        project.path.clone(),
        project.language.clone(),
        diagnostics_list,
        last_diag_req,
        active_tab,
    );

    let check_error_at_cursor = make_check_error_at_cursor(
        code,
        diagnostics_list,
        project.language.clone(),
        active_error,
        active_tab,
    );

    let pid = project.id.clone();
    let ppath_val = project.path.clone();
    let open_file = make_open_file(
        pid.clone(),
        ppath_val.clone(),
        code,
        active_tab,
        open_tabs,
        dirty,
        trigger_diagnostics.clone(),
    );



    let on_click_problem = make_on_click_problem(
        open_file.clone(),
        active_tab,
        code,
        check_error_at_cursor.clone(),
        cursor_coords.clone(),
    );

    let ppath = project.path.clone();
    let save_current = make_save_current(
        pid.clone(),
        ppath.clone(),
        code,
        dirty,
        active_tab,
        trigger_diagnostics.clone(),
    );

    let close_tab = make_close_tab(
        pid.clone(),
        code,
        active_tab,
        open_tabs,
    );

    let trigger_definition = make_trigger_definition(
        pid.clone(),
        code,
        cursor_pos,
        project_path_str.get_value(),
        active_tab,
        open_file.clone(),
        references_list,
        bottom_tab,
        show_snack.clone(),
        cursor_coords.clone(),
        check_error_at_cursor.clone(),
    );

    let trigger_references = make_trigger_references(
        code,
        cursor_pos,
        project_path_str.get_value(),
        active_tab,
        references_list,
        bottom_tab,
        show_snack.clone(),
    );

    let on_click_reference = make_on_click_reference(
        pid.clone(),
        open_file.clone(),
        active_tab,
        code,
        check_error_at_cursor.clone(),
        cursor_coords.clone(),
        project_path_str.get_value(),
    );

    let run_code = make_run_code(
        pid.clone(),
        ppath.clone(),
        project.language.clone(),
        code,
        is_running,
        output,
        is_error,
        current_pid,
        preview_url,
        save_current.clone(),
    );

    let stop_code = make_stop_code(
        current_pid,
        output,
        preview_url,
        bottom_tab,
    );

    let format_code = make_format_code(
        ppath.clone(),
        project.language.clone(),
        code,
        dirty,
        is_running,
        active_tab,
        output,
        is_error,
        bottom_tab,
        trigger_diagnostics.clone(),
    );

    let add_dep = make_add_dep(
        pid.clone(),
        ppath.clone(),
        project.language.clone(),
        dep_input,
        dep_output,
        open_file.clone(),
        file_tree_data.clone(),
    );

    let copy_code = make_copy_code(code, show_snack.clone());

    let on_select = make_on_select(code, dirty, suggestions, cursor_pos);

    let copied_item: RwSignal<Option<FileEntry>> = RwSignal::new(None);
    let sidebar_open: RwSignal<bool> = RwSignal::new(false);
    let sidebar_mode: RwSignal<usize> = RwSignal::new(0); // 0=files, 1=search

    let create_file = make_create_file(
        pid.clone(),
        ppath.clone(),
        show_snack.clone(),
        open_file.clone(),
        file_tree_data.clone(),
    );

    let create_folder = make_create_folder(
        pid.clone(),
        ppath.clone(),
        show_snack.clone(),
        file_tree_data.clone(),
    );

    let delete_entry = make_delete_entry(
        pid.clone(),
        ppath.clone(),
        show_snack.clone(),
        close_tab.clone(),
        file_tree_data.clone(),
    );

    let copy_entry = Callback::new({
        let show_snack = show_snack.clone();
        move |entry: FileEntry| {
            copied_item.set(Some(entry.clone()));
            show_snack.run(format!("Copied {}! Long-press folder/explorer to paste.", entry.name));
        }
    });

    let paste_entry = make_paste_entry(
        pid.clone(),
        ppath.clone(),
        show_snack.clone(),
        open_file.clone(),
        file_tree_data.clone(),
        copied_item,
    );

    let move_entry = make_move_entry(
        pid.clone(),
        ppath.clone(),
        show_snack.clone(),
        file_tree_data.clone(),
        active_tab,
        open_tabs,
    );


    // Sync all files from localStorage to backend filesystem on mount
    sync_project(project.id.clone(), project.path.clone());

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
                <button class="btn btn-icon btn-menu" title="Toggle Files"
                    style=move || {
                        let is_active = sidebar_open.get() && sidebar_mode.get() == 0;
                        if is_active {
                            "margin-right: 6px; background: rgba(99, 102, 241, 0.2); color: var(--accent); border-color: var(--accent);"
                        } else {
                            "margin-right: 6px;"
                        }
                    }
                    on:click=move |_| {
                        if sidebar_open.get() && sidebar_mode.get() == 0 {
                            sidebar_open.set(false);
                        } else {
                            sidebar_mode.set(0);
                            sidebar_open.set(true);
                        }
                    }>
                    <LucideIcon name="folder" size="20" />
                </button>
                <button class="btn btn-icon" title="Find in Current File (Ctrl+F)"
                    on:click=move |_| show_search.update(|v| *v = !*v)>
                    <LucideIcon name="search" size="20" />
                </button>
                <button class="btn btn-icon" title="Search and Replace Project"
                    style=move || {
                        let is_active = sidebar_open.get() && sidebar_mode.get() == 1;
                        if is_active {
                            "background: rgba(99, 102, 241, 0.2); color: var(--accent); border-color: var(--accent);"
                        } else {
                            ""
                        }
                    }
                    on:click=move |_| {
                        if sidebar_open.get() && sidebar_mode.get() == 1 {
                            sidebar_open.set(false);
                        } else {
                            sidebar_mode.set(1);
                            sidebar_open.set(true);
                        }
                    }>
                    <LucideIcon name="replace" size="20" />
                </button>

                {move || current_pid.get().map(|_| view! {
                    <button class="btn btn-danger" style="display:inline-flex; align-items:center; gap:6px;" on:click=move |_| stop_code.run(())>
                        <LucideIcon name="square" size="14" /> <span class="btn-text">"Stop"</span>
                    </button>
                })}
                <button class="btn btn-success" style="display:inline-flex; align-items:center; gap:6px;" disabled=move || is_running.get()
                    on:click=move |_| run_code.run(())
                >
                    {move || if is_running.get() {
                        view! { <><span class="spinner"></span><span class="btn-text">" Running..."</span></> }.into_any()
                    } else {
                        view! { <><LucideIcon name="play" size="14" /> <span class="btn-text">"Run"</span></> }.into_any()
                    }}
                </button>
                {move || preview_url.get().is_some().then(|| view! {
                    <button class="btn btn-success mobile-preview-toggle-btn"
                        style="display:inline-flex; align-items:center; gap:6px; background:#4f46e5; border-color:#4f46e5;"
                        on:click=move |_| show_mobile_full_preview.set(true)>
                        <LucideIcon name="eye" size="16" />
                        <span class="btn-text">"Preview"</span>
                    </button>
                })}
            </AppBar>

            <div class="editor-layout">
                {move || {
                    if sidebar_mode.get() == 0 {
                        view! {
                            <FileTree 
                                file_tree=file_tree_data.into()
                                active_tab=active_tab.into()
                                open_file=open_file
                                lang_icon=lang_icon(&project_lang_str.get_value()).to_string()
                                project_name=project.name.clone()
                                create_file=create_file
                                create_folder=create_folder
                                delete_entry=delete_entry
                                copy_entry=copy_entry
                                copied_item=copied_item.into()
                                paste_entry=paste_entry
                                move_entry=move_entry
                                sidebar_open=sidebar_open.into()
                                toggle_sidebar=Callback::new(move |_: ()| sidebar_open.set(false))
                                sidebar_mode=sidebar_mode
                            />
                        }.into_any()
                    } else {
                        view! {
                            <ProjectSearchReplacePanel
                                project_id=pid.clone()
                                project_path=ppath.clone()
                                file_tree=file_tree_data.into()
                                file_tree_data=file_tree_data
                                active_tab=active_tab
                                code=code
                                dirty=dirty
                                project_query=project_search_text
                                replace_text=project_replace_text
                                open_file=open_file
                                trigger_diagnostics=trigger_diagnostics
                                show_snack=show_snack
                                sidebar_open=sidebar_open.into()
                                close_sidebar=Callback::new(move |_: ()| sidebar_open.set(false))
                                sidebar_mode=sidebar_mode
                            />
                        }.into_any()
                    }
                }}

                <div class="editor-main">
                    <TabStrip 
                        open_tabs=open_tabs.into()
                        active_tab=active_tab.into()
                        dirty=dirty.into()
                        open_file=open_file
                        close_tab=close_tab
                    />

                    <SearchBar 
                        show_search=show_search
                        find_text=find_text
                        find_index=find_index
                        code=code
                    />

                    <EditorCodeArea
                        settings=settings
                        code=code
                        dirty=dirty
                        active_tab=active_tab
                        diagnostics_list=diagnostics_list
                        active_error=active_error
                        cursor_pos=cursor_pos
                        cursor_coords=cursor_coords
                        suggestions=suggestions
                        selected_idx=selected_idx
                        project_lang_str=project_lang_str
                        project_path_str=project_path_str
                        last_request_id=last_request_id
                        trigger_diagnostics=trigger_diagnostics
                        save_current=save_current
                        format_code=format_code
                        show_search=show_search
                        check_error_at_cursor=check_error_at_cursor
                        on_select=on_select
                        show_snack=show_snack
                        trigger_definition=trigger_definition
                        trigger_references=trigger_references
                        show_deps=show_deps
                    />

                    <BottomPanel 
                        bottom_tab=bottom_tab
                        output=output.into()
                        is_error=is_error.into()
                        show_snack=show_snack
                        diagnostics_list=diagnostics_list.into()
                        on_click_problem=on_click_problem
                        code=code
                        language=Signal::derive(move || project_lang_str.get_value())
                        references_list=references_list
                        on_click_reference=on_click_reference
                        active_tab=active_tab.into()
                    />

                    <EditorFooter
                        code=code
                        dirty=dirty
                        settings=settings
                        copy_code=copy_code
                    />
                </div>

                <PreviewPanel
                    preview_url=preview_url.into()
                    refresh_key=refresh_key
                />
            </div>

            <DependencyModal 
                show_deps=show_deps
                dep_input=dep_input
                dep_output=dep_output.into()
                add_dep=add_dep
            />

            <MobilePreviewOverlay
                show_mobile_full_preview=show_mobile_full_preview
                preview_url=preview_url.into()
                refresh_key=refresh_key
            />

            <Snackbar message=snack_msg.read_only() />
        </div>
    }.into_any()
}

fn uri_to_relative(uri: &str, project_path: &str) -> String {
    let uri_clean = uri.replace('\\', "/");
    let proj_clean = project_path.replace('\\', "/");
    
    let prefix = format!("file://{}/", proj_clean);
    let prefix_triple = format!("file:///{}", proj_clean);
    let prefix_triple_alt = format!("file://{}", proj_clean);
    
    if uri_clean.starts_with(&prefix) {
        let mut rel = uri_clean.strip_prefix(&prefix).unwrap_or(&uri_clean).to_string();
        if rel.starts_with('/') {
            rel = rel.trim_start_matches('/').to_string();
        }
        rel
    } else if uri_clean.starts_with(&prefix_triple) {
        let mut rel = uri_clean.strip_prefix(&prefix_triple).unwrap_or(&uri_clean).to_string();
        if rel.starts_with('/') {
            rel = rel.trim_start_matches('/').to_string();
        }
        rel
    } else if uri_clean.starts_with(&prefix_triple_alt) {
        let mut rel = uri_clean.strip_prefix(&prefix_triple_alt).unwrap_or(&uri_clean).to_string();
        if rel.starts_with('/') {
            rel = rel.trim_start_matches('/').to_string();
        }
        rel
    } else {
        let needle = format!("/{}", proj_clean.trim_start_matches('/'));
        if let Some(pos) = uri_clean.find(&needle) {
            let mut rel = uri_clean[pos + needle.len() + 1..].to_string();
            if rel.starts_with('/') {
                rel = rel.trim_start_matches('/').to_string();
            }
            rel
        } else {
            // Absolute path outside the project. Ensure it starts with '/'
            let path_without_scheme = uri_clean.strip_prefix("file://").unwrap_or(&uri_clean).to_string();
            if !path_without_scheme.starts_with('/') {
                format!("/{}", path_without_scheme)
            } else {
                path_without_scheme
            }
        }
    }
}
