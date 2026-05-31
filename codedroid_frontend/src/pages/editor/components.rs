use crate::components::icon::LucideIcon;
use crate::pages::editor::utils::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use web_sys::{Event, KeyboardEvent, MouseEvent, TouchEvent};
use gloo_storage::Storage;

#[derive(Clone, PartialEq)]
pub struct ContextMenuState {
    pub x: f64,
    pub y: f64,
    pub entry: FileEntry,
}

#[component]
pub fn FileTree(
    file_tree: Signal<Vec<FileEntry>>,
    active_tab: Signal<Option<String>>,
    open_file: Callback<String>,
    lang_icon: String,
    project_name: String,
    create_file: Callback<String>,
    create_folder: Callback<String>,
    delete_entry: Callback<FileEntry>,
    copy_entry: Callback<FileEntry>,
    copied_item: Signal<Option<FileEntry>>,
    paste_entry: Callback<Option<String>>,
    move_entry: Callback<(FileEntry, String)>,
    sidebar_open: Signal<bool>,
    toggle_sidebar: Callback<()>,
    _sidebar_mode: RwSignal<usize>,
    project_path: String,
    terminal_trigger: RwSignal<Option<String>>,
) -> impl IntoView {
    let (show_new_file, set_show_new_file) = signal(false);
    let (show_new_folder, set_show_new_folder) = signal(false);
    let (new_name, set_new_name) = signal(String::new());
    let (show_rename, set_show_rename) = signal(Option::<FileEntry>::None);
    let (rename_name, set_rename_name) = signal(String::new());

    let (context_menu, set_context_menu) = signal(Option::<ContextMenuState>::None);
    let (press_id, set_press_id) = signal(0i32);
    let (collapsed_dirs, set_collapsed_dirs) = signal(std::collections::HashSet::<String>::new());
    let (selected_path, set_selected_path) = signal(Option::<String>::None);
    
    let get_target_dir = move || {
        selected_path
            .get()
            .map(|path| {
                let is_dir = file_tree.get().iter().any(|f| f.name == path && f.is_dir);
                if is_dir {
                    path
                } else {
                    if let Some(pos) = path.rfind('/') {
                        path[..pos].to_string()
                    } else {
                        String::new()
                    }
                }
            })
            .unwrap_or_default()
    };

    let start_long_press = Callback::new({
        let set_context_menu = set_context_menu.clone();
        move |(x, y, entry): (f64, f64, FileEntry)| {
            let next_id = press_id.get_untracked() + 1;
            set_press_id.set(next_id);
            let set_context_menu = set_context_menu.clone();
            spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(500).await;
                if press_id.get_untracked() == next_id {
                    set_context_menu.set(Some(ContextMenuState { x, y, entry }));
                }
            });
        }
    });

    let cancel_long_press = Callback::new(move |_: ()| {
        set_press_id.update(|id| *id += 1);
    });

    view! {
        {move || sidebar_open.get().then(|| view! {
            <div class="sidebar-overlay" on:click=move |_| toggle_sidebar.run(()) />
        })}

        <div
            class=move || if sidebar_open.get() { "file-tree-panel open" } else { "file-tree-panel" }
            on:mousedown=move |e| {
                if e.target() == e.current_target() {
                    set_selected_path.set(None);
                }
            }
            on:touchstart=move |e| {
                if e.target() == e.current_target() {
                    set_selected_path.set(None);
                }
            }
        >


            <div class="file-tree-header" style="display:flex; justify-content:space-between; align-items:center;">
                <span style="overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font-weight:700; display:flex; align-items:center; gap:6px;">
                    <img src=lang_icon class="lang-icon-header" alt="lang icon" style="width:16px; height:16px; object-fit:contain;" />
                    " "{project_name.to_uppercase()}
                </span>
                <div style="display:flex; gap:8px; flex-shrink:0; align-items:center;">
                    <button class="btn-tree-action-header" title="New File" style="background:none; border:none; cursor:pointer; display:flex; align-items:center; justify-content:center; padding:4px;"
                        on:click=move |_| {
                            set_show_new_file.set(true);
                            set_show_new_folder.set(false);
                            let target = get_target_dir();
                            if !target.is_empty() {
                                set_new_name.set(format!("{}/", target));
                            } else {
                                set_new_name.set(String::new());
                            }
                        }
                    >
                        <LucideIcon name="file-plus" size="18" />
                    </button>
                    <button class="btn-tree-action-header" title="New Folder" style="background:none; border:none; cursor:pointer; display:flex; align-items:center; justify-content:center; padding:4px;"
                        on:click=move |_| {
                            set_show_new_folder.set(true);
                            set_show_new_file.set(false);
                            let target = get_target_dir();
                            if !target.is_empty() {
                                set_new_name.set(format!("{}/", target));
                            } else {
                                set_new_name.set(String::new());
                            }
                        }
                    >
                        <LucideIcon name="folder-plus" size="18" />
                    </button>
                    <button
                        class="btn-tree-action-header"
                        title="Paste"
                        style=move || {
                            let has_copied = copied_item.get().is_some();
                            let opacity = if has_copied { "1.0" } else { "0.25" };
                            let pointer = if has_copied { "pointer" } else { "default" };
                            let events = if has_copied { "auto" } else { "none" };
                            format!("background:none; border:none; cursor:{}; display:flex; align-items:center; justify-content:center; padding:4px; opacity:{}; transition: opacity 0.2s ease; pointer-events:{};", pointer, opacity, events)
                        }
                        on:click=move |_| {
                            if copied_item.get().is_some() {
                                let target = get_target_dir();
                                paste_entry.run(if target.is_empty() { None } else { Some(target) });
                            }
                        }
                    >
                        <LucideIcon name="clipboard" size="18" />
                    </button>
                </div>
            </div>

            {move || (show_new_file.get() || show_new_folder.get()).then(|| {
                let is_folder = show_new_folder.get();
                view! {
                    <div style="padding: 10px 14px; display:flex; flex-direction:column; gap:8px; border-bottom: 1px solid var(--border); background: rgba(255,255,255,0.02)">
                        <input
                            class="input"
                            style="font-size:12px; padding:6px 10px"
                            type="text"
                            placeholder=move || if is_folder { "Folder path (e.g. src/utils)..." } else { "File name (e.g. test.rs)..." }
                            prop:value=move || new_name.get()
                            on:input=move |e| set_new_name.set(event_target_value(&e))
                            on:keydown=move |e: KeyboardEvent| {
                                if e.key() == "Enter" {
                                    let val = new_name.get();
                                    if !val.trim().is_empty() {
                                        if is_folder {
                                            create_folder.run(val);
                                            set_show_new_folder.set(false);
                                        } else {
                                            create_file.run(val);
                                            set_show_new_file.set(false);
                                        }
                                        set_new_name.set(String::new());
                                    }
                                } else if e.key() == "Escape" {
                                    set_show_new_file.set(false);
                                    set_show_new_folder.set(false);
                                    set_new_name.set(String::new());
                                }
                            }
                        />
                        <div style="display:flex; gap:6px; justify-content:flex-end">
                            <button class="btn" style="padding:4px 8px; font-size:11px; background:transparent; border:1px solid var(--border); color:var(--text2); box-shadow:none"
                                on:click=move |_| {
                                    set_show_new_file.set(false);
                                    set_show_new_folder.set(false);
                                    set_new_name.set(String::new());
                                }
                            >"Cancel"</button>
                            <button class="btn btn-primary" style="padding:4px 8px; font-size:11px"
                                on:click=move |_| {
                                    let val = new_name.get();
                                    if !val.trim().is_empty() {
                                        if is_folder {
                                            create_folder.run(val);
                                            set_show_new_folder.set(false);
                                        } else {
                                            create_file.run(val);
                                            set_show_new_file.set(false);
                                        }
                                        set_new_name.set(String::new());
                                    }
                                }
                            >"Create"</button>
                        </div>
                    </div>
                }
            })}

            {move || show_rename.get().map(|entry| {
                let entry_kd = entry.clone();
                let entry_click = entry.clone();
                view! {
                    <div style="padding: 10px 14px; display:flex; flex-direction:column; gap:8px; border-bottom: 1px solid var(--border); background: rgba(255,255,255,0.02)">
                        <div style="font-size:10px; color:var(--text2)">
                            "Rename / Move: " <strong style="color: var(--accent2);">{entry.name.clone()}</strong>
                        </div>
                        <input
                            class="input"
                            style="font-size:12px; padding:6px 10px"
                            type="text"
                            placeholder="New path (e.g. src/new_name.rs)..."
                            prop:value=move || rename_name.get()
                            on:input=move |e| set_rename_name.set(event_target_value(&e))
                            on:keydown=move |e: KeyboardEvent| {
                                if e.key() == "Enter" {
                                    let val = rename_name.get();
                                    if !val.trim().is_empty() {
                                        move_entry.run((entry_kd.clone(), val));
                                        set_show_rename.set(None);
                                        set_rename_name.set(String::new());
                                    }
                                } else if e.key() == "Escape" {
                                    set_show_rename.set(None);
                                    set_rename_name.set(String::new());
                                }
                            }
                        />
                        <div style="display:flex; gap:6px; justify-content:flex-end">
                            <button class="btn" style="padding:4px 8px; font-size:11px; background:transparent; border:1px solid var(--border); color:var(--text2); box-shadow:none"
                                on:click=move |_| {
                                    set_show_rename.set(None);
                                    set_rename_name.set(String::new());
                                }
                            >"Cancel"</button>
                            <button class="btn btn-primary" style="padding:4px 8px; font-size:11px"
                                on:click=move |_| {
                                    let val = rename_name.get();
                                    if !val.trim().is_empty() {
                                        move_entry.run((entry_click.clone(), val));
                                        set_show_rename.set(None);
                                        set_rename_name.set(String::new());
                                    }
                                }
                            >"Rename / Move"</button>
                        </div>
                    </div>
                }
            })}

            <div style="flex: 1; overflow-y: auto;">
                {move || {
                    let collapsed = collapsed_dirs.get();
                    file_tree.get().into_iter()
                        .filter(|f| {
                            let ancestors = get_ancestors(&f.name);
                            !ancestors.into_iter().any(|a| collapsed.contains(&a))
                        })
                        .map(|f| {
                            let fname_click = f.name.clone();
                            let fname_active = f.name.clone();
                            let fname_lang = f.name.clone();

                            let f_click = f.clone();
                            let f_click_context = f_click.clone();
                            let f_click_mouse = f_click.clone();
                            let f_click_touch = f_click.clone();

                            let depth = path_depth(&f.name);
                            let indent = depth * 16;
                            let display_name = path_basename(&f.name).to_string();

                            view! {
                                <div
                                    class=move || {
                                        let active = active_tab.get().as_deref() == Some(&fname_active);
                                        let selected = selected_path.get().as_deref() == Some(&fname_active);
                                        let base = if f_click.is_dir { "file-item dir-item" } else { "file-item" };
                                        let mut classes = base.to_string();
                                        if active {
                                            classes.push_str(" active");
                                        }
                                        if selected {
                                            classes.push_str(" selected");
                                        }
                                        classes
                                    }
                                    style=format!("display: flex; justify-content: space-between; align-items: center; padding-right: 12px; padding-left: {}px; cursor: pointer; user-select: none;", 12 + indent)
                                    on:click=move |e| {
                                        e.stop_propagation();
                                        set_selected_path.set(Some(fname_click.clone()));
                                        if !f_click.is_dir {
                                            open_file.run(fname_click.clone());
                                            toggle_sidebar.run(()); // Auto-close drawer on mobile when clicking file
                                        } else {
                                            set_collapsed_dirs.update(|set| {
                                                if set.contains(&fname_click) {
                                                    set.remove(&fname_click);
                                                } else {
                                                    set.insert(fname_click.clone());
                                                }
                                            });
                                        }
                                    }
                                    on:contextmenu=move |e: MouseEvent| {
                                        e.prevent_default();
                                        e.stop_propagation();
                                        set_context_menu.set(Some(ContextMenuState {
                                            x: e.client_x() as f64,
                                            y: e.client_y() as f64,
                                            entry: f_click_context.clone(),
                                        }));
                                    }
                                    on:mousedown=move |e: MouseEvent| {
                                        if e.button() == 0 { // left click only
                                            let x = e.client_x() as f64;
                                            let y = e.client_y() as f64;
                                            start_long_press.run((x, y, f_click_mouse.clone()));
                                        }
                                    }
                                    on:mouseup=move |_| cancel_long_press.run(())
                                    on:mouseleave=move |_| cancel_long_press.run(())
                                    on:touchstart=move |e: TouchEvent| {
                                        if let Some(touch) = e.touches().get(0) {
                                            let x = touch.client_x() as f64;
                                            let y = touch.client_y() as f64;
                                            start_long_press.run((x, y, f_click_touch.clone()));
                                        }
                                    }
                                    on:touchmove=move |_| cancel_long_press.run(())
                                    on:touchend=move |_| cancel_long_press.run(())
                                    on:touchcancel=move |_| cancel_long_press.run(())
                                >
                                    <div style="display:flex; align-items:center; gap:6px; min-width:0; flex:1">
                                        {if f.is_dir {
                                            let name_for_collapsed = f.name.clone();
                                            let is_collapsed = move || collapsed_dirs.get().contains(&name_for_collapsed);
                                            view! {
                                                <span style="display:inline-flex; align-items:center; opacity:0.6; cursor:pointer; width: 14px; justify-content: center;">
                                                    {move || if is_collapsed() {
                                                        view! { <LucideIcon name="chevron-right" size="12" /> }.into_any()
                                                    } else {
                                                        view! { <LucideIcon name="chevron-down" size="12" /> }.into_any()
                                                    }}
                                                </span>
                                                <span style="color: var(--accent2); display: inline-flex;"><LucideIcon name="folder" size="15" /></span>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <span style="width: 14px;"></span>
                                                <span style="display:inline-flex; align-items:center;">
                                                    <img src=file_icon(&f.name) class="file-icon-img" alt="" style="width:14px; height:14px; object-fit:contain;" />
                                                </span>
                                            }.into_any()
                                        }}
                                        <span style="overflow:hidden; text-overflow:ellipsis; white-space:nowrap; flex:1; margin-left: 4px;">
                                            {display_name}
                                            {move || (!f.is_dir).then(|| view! {
                                                <span style="font-size: 10px; opacity: 0.5; margin-left: 6px; font-weight: 500; font-family: var(--font-ui)">
                                                    {format!("({})", file_lang_name(&fname_lang))}
                                                </span>
                                            })}
                                        </span>
                                    </div>
                                </div>
                            }
                        }).collect_view()
                }}
            </div>

            {move || copied_item.get().map(|item| view! {
                <div style="font-size: 10px; color: var(--accent2); padding: 8px 14px; border-top: 1px solid var(--border); background: rgba(99, 102, 241, 0.05); display: flex; flex-direction: column; gap: 2px;">
                    <div><strong>"📋 Copied: "</strong> {item.name}</div>
                    <div style="opacity: 0.7">"Long-press folder/tree to paste"</div>
                </div>
            })}

            {move || context_menu.get().map(|menu| {
                let entry_rename = menu.entry.clone();
                let entry_copy = menu.entry.clone();
                let entry_delete = menu.entry.clone();
                let entry_paste = menu.entry.clone();
                let is_dir = menu.entry.is_dir;
                let has_copied = copied_item.get().is_some();
                let set_context_menu = set_context_menu.clone();
                let copy_entry = copy_entry.clone();
                let delete_entry = delete_entry.clone();
                let paste_entry = paste_entry.clone();
                let set_show_rename = set_show_rename.clone();
                let set_rename_name = set_rename_name.clone();
                let set_show_new_file = set_show_new_file.clone();
                let set_show_new_folder = set_show_new_folder.clone();
                
                view! {
                    <div 
                        class="vscode-context-menu-overlay"
                        on:click=move |_| set_context_menu.set(None)
                        on:contextmenu=move |e| {
                            e.prevent_default();
                            set_context_menu.set(None);
                        }
                    >
                        <div 
                            class="vscode-context-menu"
                            style=format!("top: {}px; left: {}px;", menu.y, menu.x)
                            on:click=move |e| e.stop_propagation()
                        >
                            <button class="context-menu-item" on:click=move |_| {
                                set_context_menu.set(None);
                                let path = entry_rename.name.clone();
                                set_show_rename.set(Some(entry_rename.clone()));
                                set_rename_name.set(path);
                                set_show_new_file.set(false);
                                set_show_new_folder.set(false);
                            }>
                                <LucideIcon name="edit" size="14" />
                                <span>"Rename / Move..."</span>
                            </button>
                            <button class="context-menu-item" on:click=move |_| {
                                set_context_menu.set(None);
                                copy_entry.run(entry_copy.clone());
                            }>
                                <LucideIcon name="copy" size="14" />
                                <span>"Copy"</span>
                            </button>
                            {if is_dir {
                                let entry_paste = entry_paste.clone();
                                let paste_entry = paste_entry.clone();
                                view! {
                                    <button 
                                        class=format!("context-menu-item {}", if has_copied { "" } else { "disabled" })
                                        disabled=!has_copied
                                        on:click=move |_| {
                                            set_context_menu.set(None);
                                            paste_entry.run(Some(entry_paste.name.clone()));
                                        }
                                    >
                                        <LucideIcon name="clipboard" size="14" />
                                        <span>"Paste Into Folder"</span>
                                    </button>
                                }.into_any()
                            } else {
                                view! { "" }.into_any()
                            }}
                            <div class="context-menu-divider"></div>
                            <button class="context-menu-item" on:click={
                                let entry = menu.entry.clone();
                                let project_path = project_path.clone();
                                let terminal_trigger = terminal_trigger.clone();
                                move |_| {
                                    set_context_menu.set(None);
                                    let relative_dir = if entry.is_dir {
                                        entry.name.clone()
                                    } else {
                                        if let Some(pos) = entry.name.rfind('/') {
                                            entry.name[..pos].to_string()
                                        } else {
                                            String::new()
                                        }
                                    };
                                    let target_dir = if relative_dir.is_empty() {
                                        project_path.clone()
                                    } else {
                                        format!("{}/{}", project_path, relative_dir)
                                    };
                                    terminal_trigger.set(Some(target_dir));
                                }
                            }>
                                <LucideIcon name="terminal" size="14" />
                                <span>"Open in Integrated Terminal"</span>
                            </button>
                            <div class="context-menu-divider"></div>
                            <button class="context-menu-item danger" on:click=move |_| {
                                set_context_menu.set(None);
                                delete_entry.run(entry_delete.clone());
                            }>
                                <LucideIcon name="trash" size="14" />
                                <span>"Delete"</span>
                            </button>
                        </div>
                    </div>
                }
            })}
        </div>
    }
}

#[component]
pub fn TabStrip(
    open_tabs: Signal<Vec<String>>,
    active_tab: Signal<Option<String>>,
    dirty: Signal<bool>,
    open_file: Callback<String>,
    close_tab: Callback<String>,
) -> impl IntoView {
    view! {
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
                        <span style="display:inline-flex; align-items:center; gap:6px;">
                            <img src=file_icon(&tab) class="tab-icon-img" alt="" style="width:14px; height:14px; object-fit:contain;" />
                            {tab.clone()}
                        </span>
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
    }
}

pub fn apply_replacement(code: &str, range: &crate::api::Range, replacement: &str) -> String {
    let lines: Vec<&str> = code.lines().collect();
    let mut new_lines = Vec::new();

    let start_line = range.start.line as usize;
    let start_col = range.start.character as usize;
    let end_line = range.end.line as usize;
    let end_col = range.end.character as usize;

    if start_line == end_line && start_line < lines.len() {
        for (i, line) in lines.iter().enumerate() {
            if i == start_line {
                let chars: Vec<char> = line.chars().collect();
                let s = std::cmp::min(start_col, chars.len());
                let e = std::cmp::min(end_col, chars.len());

                let mut new_line = String::new();
                new_line.push_str(&chars[..s].iter().collect::<String>());
                new_line.push_str(replacement);
                new_line.push_str(&chars[e..].iter().collect::<String>());
                new_lines.push(new_line);
            } else {
                new_lines.push(line.to_string());
            }
        }
    } else if start_line < end_line && end_line < lines.len() {
        if start_line == 0 && start_col == 0 && end_line == 0 && end_col == 0 {
            let mut content = replacement.to_string();
            content.push_str(code);
            return content;
        }

        for (i, line) in lines.iter().enumerate() {
            if i < start_line || i > end_line {
                new_lines.push(line.to_string());
            } else if i == start_line {
                let chars: Vec<char> = line.chars().collect();
                let s = std::cmp::min(start_col, chars.len());
                let mut new_line = chars[..s].iter().collect::<String>();
                new_line.push_str(replacement);
                new_lines.push(new_line);
            } else if i == end_line {
                let chars: Vec<char> = line.chars().collect();
                let e = std::cmp::min(end_col, chars.len());
                if let Some(last) = new_lines.last_mut() {
                    last.push_str(&chars[e..].iter().collect::<String>());
                }
            }
        }
    } else {
        if start_line == 0 && start_col == 0 && end_line == 0 && end_col == 0 {
            let mut content = replacement.to_string();
            content.push_str(code);
            return content;
        }
        return code.to_string();
    }

    let mut result = new_lines.join("\n");
    if code.ends_with('\n') && !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SessionState {
    pub id: String,
    pub name: String,
    pub output: String,
    #[serde(default)]
    pub path: Option<String>,
}

#[component]
pub fn BottomPanel(
    bottom_tab: RwSignal<usize>,
    output: RwSignal<String>,
    _is_error: Signal<bool>,
    show_snack: Callback<String>,
    diagnostics_list: Signal<Vec<crate::api::Diagnostic>>,
    on_click_problem: Callback<(Option<String>, u32, u32)>,
    code: RwSignal<String>,
    language: Signal<String>,
    references_list: RwSignal<Vec<crate::api::Location>>,
    on_click_reference: Callback<crate::api::Location>,
    active_tab: Signal<Option<String>>,
    project_path: Signal<String>,
    project_id: String,
    file_tree_data: RwSignal<Vec<crate::pages::editor::utils::FileEntry>>,
    terminal_session_id: RwSignal<Option<String>>,
    is_running: RwSignal<bool>,
    terminal_history: RwSignal<Vec<String>>,
    terminal_trigger: RwSignal<Option<String>>,
) -> impl IntoView {
    let expanded_idx = RwSignal::new(Option::<usize>::None);
    let suggestions_state = RwSignal::new(Option::<Vec<crate::api::CodeSuggestion>>::None);
    let loading_suggestions = RwSignal::new(false);
    let project_id_stored = StoredValue::new(project_id);
    let command_input = RwSignal::new(String::new());
    let history_index = RwSignal::new(Option::<usize>::None);

    let input_ref = NodeRef::<leptos::html::Input>::new();
    let output_area_ref = NodeRef::<leptos::html::Div>::new();

    // Multi-session state
    let project_id_for_load = project_id_stored.get_value();
    let loaded_sessions: Vec<SessionState> = gloo_storage::LocalStorage::get(&format!("codedroid_term_sessions_{}", project_id_for_load))
        .unwrap_or_default();
    
    let loaded_active_idx: usize = gloo_storage::LocalStorage::get(&format!("codedroid_term_active_idx_{}", project_id_for_load))
        .unwrap_or(0);

    let sessions = RwSignal::new(loaded_sessions);
    let active_idx = RwSignal::new(loaded_active_idx);
    let is_maximized = RwSignal::new(false);

    // Initialize output and terminal_session_id from loaded state if not empty
    let sessions_list = sessions.get_untracked();
    if !sessions_list.is_empty() {
        let active = active_idx.get_untracked();
        if active < sessions_list.len() {
            output.set(sessions_list[active].output.clone());
            terminal_session_id.set(Some(sessions_list[active].id.clone()));
        }
    }

    // Effect to automatically save sessions to local storage
    let project_id_for_save = project_id_stored.get_value();
    let output_clone_for_save = output;
    Effect::new(move |_| {
        let mut sess_list = sessions.get();
        let idx = active_idx.get();
        let current_out = output_clone_for_save.get();
        if idx < sess_list.len() {
            sess_list[idx].output = current_out;
        }
        let _ = gloo_storage::LocalStorage::set(&format!("codedroid_term_sessions_{}", project_id_for_save), &sess_list);
    });

    // Effect to automatically save active index to local storage
    let project_id_for_idx_save = project_id_stored.get_value();
    Effect::new(move |_| {
        let idx = active_idx.get();
        let _ = gloo_storage::LocalStorage::set(&format!("codedroid_term_active_idx_{}", project_id_for_idx_save), &idx);
    });

    let active_session_name = Signal::derive(move || {
        let list = sessions.get();
        let idx = active_idx.get();
        if idx < list.len() {
            list[idx].name.clone()
        } else {
            "Terminal".to_string()
        }
    });

    // Auto-scroll effect
    Effect::new(move |_| {
        let _ = output.get();
        if let Some(div) = output_area_ref.get() {
            div.set_scroll_top(div.scroll_height());
        }
    });

    // Auto-focus when tab is active
    Effect::new(move |_| {
        if bottom_tab.get() == 0 {
            if let Some(input) = input_ref.get() {
                let _ = input.focus();
            }
        }
    });

    // Polling logic closure
    let start_polling = move |session_id: String| {
        let session_id_clone = session_id.clone();
        let sessions_clone = sessions;
        let active_idx_clone = active_idx;
        let terminal_session_id_clone = terminal_session_id;
        let output_sig = output;
        let is_running_clone = is_running;
        
        spawn_local(async move {
            let mut alive = true;
            while alive {
                gloo_timers::future::TimeoutFuture::new(150).await;
                if let Ok((new_output, is_alive)) = crate::api::poll_terminal_output_api(&session_id_clone).await {
                    alive = is_alive;
                    if !new_output.is_empty() {
                        let mut clean_output = new_output.clone();
                        let mut ended = false;
                        if clean_output.contains("[CODE_RUN_ENDED]") {
                            clean_output = clean_output.replace("[CODE_RUN_ENDED]", "");
                            ended = true;
                        }
                        
                        sessions_clone.update(|s_list| {
                            if let Some(s) = s_list.iter_mut().find(|s| s.id == session_id_clone) {
                                s.output.push_str(&clean_output);
                            }
                        });
                        
                        let current_list = sessions_clone.get_untracked();
                        let active = active_idx_clone.get_untracked();
                        if active < current_list.len() && current_list[active].id == session_id_clone {
                            let mut current = output_sig.get_untracked();
                            current.push_str(&clean_output);
                            output_sig.set(current);
                        }
                        
                        if ended {
                            let current_list = sessions_clone.get_untracked();
                            let active = active_idx_clone.get_untracked();
                            if active < current_list.len() && current_list[active].id == session_id_clone {
                                is_running_clone.set(false);
                            }
                        }
                    }
                    
                    if !is_alive {
                        sessions_clone.update(|s_list| {
                            if let Some(s) = s_list.iter_mut().find(|s| s.id == session_id_clone) {
                                s.output.push_str("\n[Process completed]\n");
                            }
                        });
                        
                        let current_list = sessions_clone.get_untracked();
                        let active = active_idx_clone.get_untracked();
                        if active < current_list.len() && current_list[active].id == session_id_clone {
                            let mut current = output_sig.get_untracked();
                            current.push_str("\n[Process completed]\n");
                            output_sig.set(current);
                            terminal_session_id_clone.set(None);
                            is_running_clone.set(false);
                        }
                        break;
                    }
                } else {
                    break;
                }
            }
        });
    };

    // Effect to handle external open-in-terminal triggers
    let terminal_trigger_clone = terminal_trigger;
    Effect::new({
        let sessions = sessions.clone();
        let active_idx = active_idx.clone();
        let terminal_session_id = terminal_session_id.clone();
        let output = output.clone();
        let start_polling = start_polling.clone();
        move |_| {
            if let Some(dir_path) = terminal_trigger_clone.get() {
                // Reset trigger immediately so it can fire again
                terminal_trigger_clone.set(None);
                
                // Switch tab to Terminal (index 0)
                bottom_tab.set(0);
                
                let sessions_clone = sessions;
                let active_idx_clone = active_idx;
                let terminal_session_id_clone = terminal_session_id;
                let output_clone = output;
                let start_polling_clone = start_polling.clone();
                
                // Save current session output first
                let current_active_idx = active_idx_clone.get_untracked();
                let current_out = output_clone.get_untracked();
                sessions_clone.update(|s_list| {
                    if current_active_idx < s_list.len() {
                        s_list[current_active_idx].output = current_out;
                    }
                });
                
                let folder_name = dir_path.split('/').last().unwrap_or("sh").to_string();
                let name = if folder_name.is_empty() { "sh".to_string() } else { format!("sh: {}", folder_name) };
                
                spawn_local(async move {
                    match crate::api::start_terminal_api(&dir_path).await {
                        Ok(session_id) => {
                            let new_sess = SessionState {
                                id: session_id.clone(),
                                name,
                                output: "Welcome to CodeDroid Terminal\n\n".to_string(),
                                path: Some(dir_path.clone()),
                            };
                            
                            sessions_clone.update(|s| s.push(new_sess));
                            let new_idx = sessions_clone.get_untracked().len() - 1;
                            active_idx_clone.set(new_idx);
                            
                            output_clone.set("Welcome to CodeDroid Terminal\n\n".to_string());
                            terminal_session_id_clone.set(Some(session_id.clone()));
                            start_polling_clone(session_id);
                        }
                        Err(e) => {
                            let mut current = output_clone.get_untracked();
                            current.push_str(&format!("❌ Failed to initialize terminal session: {}\n", e));
                            output_clone.set(current);
                        }
                    }
                });
            }
        }
    });

    // Start polling existing sessions loaded from storage
    for s in sessions.get_untracked() {
        start_polling(s.id.clone());
    }

    // Effect to start initial terminal session when tab is selected
    Effect::new(move |_| {
        if bottom_tab.get() == 0 && terminal_session_id.get().is_none() && sessions.get().is_empty() {
            let path = project_path.get_untracked();
            let sessions_clone = sessions;
            let active_idx_clone = active_idx;
            let terminal_session_id_clone = terminal_session_id;
            let output_clone = output;
            let start_polling_clone = start_polling;
            
            spawn_local(async move {
                match crate::api::start_terminal_api(&path).await {
                    Ok(session_id) => {
                        let initial_out = "Welcome to CodeDroid Terminal\nType commands below (e.g. ls, cargo test, git status)\n\n".to_string();
                        let new_sess = SessionState {
                            id: session_id.clone(),
                            name: "sh (1)".to_string(),
                            output: initial_out.clone(),
                            path: Some(path.clone()),
                        };
                        sessions_clone.set(vec![new_sess]);
                        active_idx_clone.set(0);
                        terminal_session_id_clone.set(Some(session_id.clone()));
                        output_clone.set(initial_out);
                        
                        start_polling_clone(session_id);
                    }
                    Err(e) => {
                        let mut current = output_clone.get_untracked();
                        current.push_str(&format!("❌ Failed to initialize terminal session: {}\n", e));
                        output_clone.set(current);
                    }
                }
            });
        }
    });

    // Effect to detect external session additions (e.g. from Run Code)
    Effect::new(move |_| {
        if let Some(session_id) = terminal_session_id.get() {
            let list = sessions.get_untracked();
            if !list.iter().any(|s| s.id == session_id) {
                let next_num = list.len() + 1;
                let name = format!("sh ({})", next_num);
                let initial_out = output.get_untracked();
                let new_sess = SessionState {
                    id: session_id.clone(),
                    name,
                    output: initial_out,
                    path: Some(project_path.get_untracked()),
                };
                sessions.update(|s| s.push(new_sess));
                active_idx.set(sessions.get_untracked().len() - 1);
                
                start_polling(session_id);
            }
        }
    });

    on_cleanup(move || {
        let list = sessions.get_untracked();
        for s in list {
            spawn_local(async move {
                let _ = crate::api::stop_terminal_api(&s.id).await;
            });
        }
    });

    let switch_to_session = move |target_idx: usize| {
        let list = sessions.get_untracked();
        if target_idx < list.len() {
            let current_active_idx = active_idx.get_untracked();
            let current_out = output.get_untracked();
            sessions.update(|s_list| {
                if current_active_idx < s_list.len() {
                    s_list[current_active_idx].output = current_out;
                }
            });

            active_idx.set(target_idx);
            let target_s = &list[target_idx];
            output.set(target_s.output.clone());
            terminal_session_id.set(Some(target_s.id.clone()));
            
            if let Some(input) = input_ref.get() {
                let _ = input.focus();
            }
        }
    };

    let add_new_session = move |_| {
        let proj_path = project_path.get_untracked();
        let sessions_clone = sessions;
        let active_idx_clone = active_idx;
        let terminal_session_id_clone = terminal_session_id;
        let output_clone = output;
        let start_polling_clone = start_polling;
        
        let current_active_idx = active_idx_clone.get_untracked();
        let current_out = output_clone.get_untracked();
        sessions_clone.update(|s_list| {
            if current_active_idx < s_list.len() {
                s_list[current_active_idx].output = current_out;
            }
        });

        spawn_local(async move {
            match crate::api::start_terminal_api(&proj_path).await {
                Ok(session_id) => {
                    let next_num = sessions_clone.get_untracked().len() + 1;
                    let name = format!("sh ({})", next_num);
                    
                    let new_sess = SessionState {
                        id: session_id.clone(),
                        name,
                        output: "Welcome to CodeDroid Terminal\n\n".to_string(),
                        path: Some(proj_path.clone()),
                    };
                    
                    sessions_clone.update(|s| s.push(new_sess));
                    let new_idx = sessions_clone.get_untracked().len() - 1;
                    active_idx_clone.set(new_idx);
                    
                    output_clone.set("Welcome to CodeDroid Terminal\n\n".to_string());
                    terminal_session_id_clone.set(Some(session_id.clone()));
                    
                    start_polling_clone(session_id);
                }
                Err(e) => {
                    let mut current = output_clone.get_untracked();
                    current.push_str(&format!("❌ Failed to initialize terminal session: {}\n", e));
                    output_clone.set(current);
                }
            }
        });
    };

    let kill_session_by_index = move |target_idx: usize| {
        let list = sessions.get_untracked();
        if target_idx < list.len() {
            let session_id = list[target_idx].id.clone();
            let sessions_clone = sessions;
            let active_idx_clone = active_idx;
            let terminal_session_id_clone = terminal_session_id;
            let output_clone = output;
            
            spawn_local(async move {
                let _ = crate::api::stop_terminal_api(&session_id).await;
                
                sessions_clone.update(|s_list| {
                    if target_idx < s_list.len() {
                        s_list.remove(target_idx);
                    }
                });
                
                let updated_list = sessions_clone.get_untracked();
                if updated_list.is_empty() {
                    terminal_session_id_clone.set(None);
                    output_clone.set("[No active terminal sessions]\n".to_string());
                    active_idx_clone.set(0);
                } else {
                    let current_active = active_idx_clone.get_untracked();
                    let new_idx = if current_active >= updated_list.len() {
                        updated_list.len() - 1
                    } else if current_active == target_idx {
                        if target_idx > 0 { target_idx - 1 } else { 0 }
                    } else if current_active > target_idx {
                        current_active - 1
                    } else {
                        current_active
                    };
                    
                    active_idx_clone.set(new_idx);
                    let target_s = &updated_list[new_idx];
                    output_clone.set(target_s.output.clone());
                    terminal_session_id_clone.set(Some(target_s.id.clone()));
                }
            });
        }
    };

    let kill_active_session = move |_| {
        let current_idx = active_idx.get_untracked();
        kill_session_by_index(current_idx);
    };

    let stop_command = move || {
        let proj_id_clone = project_id_stored.get_value();
        let proj_path_clone = project_path.get_untracked();
        let file_tree_data_clone = file_tree_data.clone();
        let terminal_session_id_clone = terminal_session_id;
        let output_clone = output;
        let sessions_clone = sessions;
        let active_idx_clone = active_idx;

        let mut current = output_clone.get_untracked();
        current.push_str("\n[Initializing terminal session...]\n");
        output_clone.set(current);

        spawn_local(async move {
            let active = active_idx_clone.get_untracked();
            let list = sessions_clone.get_untracked();
            
            if active < list.len() {
                let old_id = list[active].id.clone();
                let _ = crate::api::stop_terminal_api(&old_id).await;
            }

            match crate::api::start_terminal_api(&proj_path_clone).await {
                Ok(new_id) => {
                    if active < list.len() {
                        sessions_clone.update(|s_list| {
                            s_list[active].id = new_id.clone();
                            s_list[active].output = "Welcome to CodeDroid Terminal (Restarted)\n\n".to_string();
                        });
                        output_clone.set("Welcome to CodeDroid Terminal (Restarted)\n\n".to_string());
                    } else {
                        let name = format!("sh ({})", list.len() + 1);
                        let new_sess = SessionState {
                            id: new_id.clone(),
                            name,
                            output: "Welcome to CodeDroid Terminal\n\n".to_string(),
                            path: Some(proj_path_clone.clone()),
                        };
                        sessions_clone.update(|s| s.push(new_sess));
                        active_idx_clone.set(sessions_clone.get_untracked().len() - 1);
                        output_clone.set("Welcome to CodeDroid Terminal\n\n".to_string());
                    }
                    terminal_session_id_clone.set(Some(new_id.clone()));
                    
                    start_polling(new_id);
                }
                Err(e) => {
                    let mut current = output_clone.get_untracked();
                    current.push_str(&format!("❌ Failed to initialize terminal session: {}\n", e));
                    output_clone.set(current);
                }
            }

            crate::pages::editor::operations::sync_from_disk(
                proj_id_clone,
                proj_path_clone,
                file_tree_data_clone,
            );

            if let Some(input) = input_ref.get() {
                let _ = input.focus();
            }
        });
    };

    let project_name = Signal::derive(move || {
        let path = project_path.get();
        if let Some(last_slash) = path.rfind('/') {
            path[last_slash + 1..].to_string()
        } else {
            path
        }
    });

    let submit_cmd_fn = move |cmd: String| {
        command_input.set(String::new());
        history_index.set(None);

        if cmd.is_empty() {
            let mut current = output.get_untracked();
            current.push_str("\n");
            output.set(current);

            if let Some(session_id) = terminal_session_id.get_untracked() {
                spawn_local(async move {
                    let _ = crate::api::send_terminal_input_api(&session_id, "\n").await;
                });
            }
            return;
        }

        let proj_path = project_path.get_untracked();
        let proj_name = project_name.get_untracked();
        let proj_id = project_id_stored.get_value();

        if cmd.trim() == "clear" || cmd.trim() == "cls" {
            output.set(String::new());
            sessions.update(|s_list| {
                let active = active_idx.get_untracked();
                if active < s_list.len() {
                    s_list[active].output = String::new();
                }
            });
            let mut hist = terminal_history.get_untracked();
            if hist.last() != Some(&cmd) {
                hist.push(cmd.clone());
                crate::store::save_terminal_history(&proj_id, &hist);
                terminal_history.set(hist);
            }
            return;
        }

        let mut current = output.get_untracked();
        current.push_str(&format!("{} $ {}\n", proj_name, cmd));
        output.set(current.clone());
        
        sessions.update(|s_list| {
            let active = active_idx.get_untracked();
            if active < s_list.len() {
                s_list[active].output = current;
            }
        });

        let mut hist = terminal_history.get_untracked();
        if hist.last() != Some(&cmd) {
            hist.push(cmd.clone());
            crate::store::save_terminal_history(&proj_id, &hist);
            terminal_history.set(hist);
        }

        if let Some(session_id) = terminal_session_id.get_untracked() {
            let proj_id_clone = project_id_stored.get_value();
            let file_tree_data_clone = file_tree_data.clone();
            let cmd_clone = cmd.clone();
            let proj_path_clone = proj_path.clone();
            spawn_local(async move {
                let _ = crate::api::send_terminal_input_api(&session_id, &format!("{}\n", cmd_clone)).await;

                gloo_timers::future::TimeoutFuture::new(500).await;
                crate::pages::editor::operations::sync_from_disk(
                    proj_id_clone,
                    proj_path_clone,
                    file_tree_data_clone,
                );
            });
        }
    };

    let on_keydown = move |e: web_sys::KeyboardEvent| {
        let key = e.key();
        if key == "c" && e.ctrl_key() {
            if terminal_session_id.get_untracked().is_some() {
                e.prevent_default();
                stop_command();
            }
        } else if key == "Enter" {
            let cmd = command_input.get_untracked();
            submit_cmd_fn(cmd);
        } else if key == "ArrowUp" {
            e.prevent_default();
            let hist = terminal_history.get_untracked();
            if !hist.is_empty() {
                let next_idx = match history_index.get_untracked() {
                    None => hist.len() - 1,
                    Some(idx) => {
                        if idx > 0 {
                            idx - 1
                        } else {
                            0
                        }
                    }
                };
                history_index.set(Some(next_idx));
                command_input.set(hist[next_idx].clone());
            }
        } else if key == "ArrowDown" {
            e.prevent_default();
            let hist = terminal_history.get_untracked();
            if !hist.is_empty() {
                if let Some(idx) = history_index.get_untracked() {
                    if idx + 1 < hist.len() {
                        let next_idx = idx + 1;
                        history_index.set(Some(next_idx));
                        command_input.set(hist[next_idx].clone());
                    } else {
                        history_index.set(None);
                        command_input.set(String::new());
                    }
                }
            }
        }
    };

    let send_tab = move |_| {
        let val = command_input.get_untracked();
        command_input.set(format!("{}\t", val));
        if let Some(input) = input_ref.get() {
            let _ = input.focus();
        }
    };

    let send_ctrl_c = move |_| {
        if let Some(session_id) = terminal_session_id.get_untracked() {
            spawn_local(async move {
                let _ = crate::api::send_terminal_input_api(&session_id, "\x03").await;
            });
        } else {
            output.set(String::new());
        }
        if let Some(input) = input_ref.get() {
            let _ = input.focus();
        }
    };

    let send_ctrl_d = move |_| {
        if let Some(session_id) = terminal_session_id.get_untracked() {
            spawn_local(async move {
                let _ = crate::api::send_terminal_input_api(&session_id, "\x04").await;
            });
        }
        if let Some(input) = input_ref.get() {
            let _ = input.focus();
        }
    };

    let send_arrow_up = move |_| {
        let hist = terminal_history.get_untracked();
        if !hist.is_empty() {
            let next_idx = match history_index.get_untracked() {
                None => hist.len() - 1,
                Some(idx) => if idx > 0 { idx - 1 } else { 0 }
            };
            history_index.set(Some(next_idx));
            command_input.set(hist[next_idx].clone());
        }
        if let Some(input) = input_ref.get() {
            let _ = input.focus();
        }
    };

    let send_arrow_down = move |_| {
        let hist = terminal_history.get_untracked();
        if !hist.is_empty() {
            if let Some(idx) = history_index.get_untracked() {
                if idx + 1 < hist.len() {
                    let next_idx = idx + 1;
                    history_index.set(Some(next_idx));
                    command_input.set(hist[next_idx].clone());
                } else {
                    history_index.set(None);
                    command_input.set(String::new());
                }
            }
        }
        if let Some(input) = input_ref.get() {
            let _ = input.focus();
        }
    };

    let run_quick_cmd = move |cmd: String| {
        submit_cmd_fn(cmd);
        if let Some(input) = input_ref.get() {
            let _ = input.focus();
        }
    };

    let on_session_change = move |e: web_sys::Event| {
        use wasm_bindgen::JsCast;
        if let Some(target) = e.target() {
            if let Ok(select) = target.dyn_into::<web_sys::HtmlSelectElement>() {
                if let Ok(idx) = select.value().parse::<usize>() {
                    switch_to_session(idx);
                }
            }
        }
    };

    view! {
        <div class=move || if is_maximized.get() { "bottom-panel maximized" } else { "bottom-panel" }>
            <div class="bottom-tabs">
                <button
                    class=move || if bottom_tab.get() == 0 { "bottom-tab active" } else { "bottom-tab" }
                    on:click=move |_| bottom_tab.set(0)
                >"TERMINAL"</button>
                <button
                    class=move || if bottom_tab.get() == 1 { "bottom-tab active" } else { "bottom-tab" }
                    on:click=move |_| bottom_tab.set(1)
                >
                    "PROBLEMS"
                    {move || {
                        let count = diagnostics_list.get().len();
                        if count > 0 {
                            view! { <span class="problem-badge">{count}</span> }.into_any()
                        } else {
                            view! { "" }.into_any()
                        }
                    }}
                </button>
                <button
                    class=move || if bottom_tab.get() == 2 { "bottom-tab active" } else { "bottom-tab" }
                    on:click=move |_| bottom_tab.set(2)
                >
                    "REFERENCES"
                    {move || {
                        let count = references_list.get().len();
                        if count > 0 {
                            view! { <span class="problem-badge" style="background:var(--primary)">{count}</span> }.into_any()
                        } else {
                            view! { "" }.into_any()
                        }
                    }}
                </button>
                <div style="flex:1"/>
                {move || (bottom_tab.get() == 2).then(|| view! {
                    <button class="btn btn-icon" style="font-size:12px" title="Clear references"
                        on:click=move |_| {
                            references_list.set(Vec::new());
                        }
                    >
                        <LucideIcon name="trash" size="16" />
                    </button>
                })}
            </div>
            {move || {
                if bottom_tab.get() == 1 {
                    let diags = diagnostics_list.get();
                    if diags.is_empty() {
                        view! {
                            <div class="problems-container empty">
                                "No problems have been detected in the workspace so far."
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="problems-container">
                                {diags.into_iter().enumerate().map(|(idx, diag)| {
                                    let severity_class = match diag.severity.unwrap_or(1) {
                                        1 => "problem-item error",
                                        2 => "problem-item warning",
                                        3 => "problem-item info",
                                        4 => "problem-item hint",
                                        _ => "problem-item error",
                                    };
                                    let severity_icon = match diag.severity.unwrap_or(1) {
                                        1 => "🔴",
                                        2 => "🟡",
                                        3 => "🔵",
                                        4 => "⚪",
                                        _ => "🔴",
                                    };
                                    let file_name = diag.file.clone();
                                    let line = diag.range.start.line;
                                    let col = diag.range.start.character;
                                    let msg = diag.message.clone();
                                    let source = diag.source.clone().unwrap_or_default();
                                    let code_val = diag.code.as_ref().map(|c| {
                                        match c {
                                            serde_json::Value::String(s) => format!(" [{}]", s),
                                            serde_json::Value::Number(n) => format!(" [{}]", n),
                                            _ => String::new(),
                                        }
                                    }).unwrap_or_default();

                                    let diag_clone = diag.clone();
                                    let on_click_problem_cb = on_click_problem;
                                    let show_snack_cb = show_snack;
                                    let file_name_clone = file_name.clone();
                                    view! {
                                        <div class="problem-wrapper">
                                            <div class=severity_class on:click=move |_| {
                                                on_click_problem_cb.run((file_name_clone.clone(), line, col));
                                                let current_idx = expanded_idx.get_untracked();
                                                if current_idx == Some(idx) {
                                                    expanded_idx.set(None);
                                                    suggestions_state.set(None);
                                                } else {
                                                    expanded_idx.set(Some(idx));
                                                    suggestions_state.set(None);
                                                    loading_suggestions.set(true);

                                                    let code_val = code.get_untracked();
                                                    let lang_val = language.get_untracked();
                                                    let diag_val = diag_clone.clone();

                                                    spawn_local(async move {
                                                        if let Ok(resp) = crate::api::get_error_suggestions_api(&code_val, &lang_val, &diag_val).await {
                                                            suggestions_state.set(Some(resp.suggestions));
                                                        }
                                                        loading_suggestions.set(false);
                                                    });
                                                }
                                            }>
                                                <span class="problem-icon">{severity_icon}</span>
                                                <span class="problem-message">{msg}{code_val}</span>
                                                {if !source.is_empty() { view! { <span class="problem-source">"["{source}"]"</span> }.into_any() } else { view! { "" }.into_any() }}
                                                <span class="problem-location">
                                                    {if let Some(ref f) = file_name {
                                                        format!("{}: Ln {}, Col {}", f, line + 1, col + 1)
                                                    } else {
                                                        format!("Ln {}, Col {}", line + 1, col + 1)
                                                    }}
                                                </span>
                                            </div>
                                            {move || {
                                                if expanded_idx.get() == Some(idx) {
                                                    view! {
                                                        <div class="problem-expansion">
                                                            {move || {
                                                                if loading_suggestions.get() {
                                                                    view! {
                                                                        <div class="suggestion-loading">
                                                                            <div class="spinner" style="width:14px;height:14px;border-width:1.5px;display:inline-block;vertical-align:middle;margin-right:8px" />
                                                                            "Analyzing error and finding suggestions..."
                                                                        </div>
                                                                    }.into_any()
                                                                } else if let Some(suggs) = suggestions_state.get() {
                                                                    view! {
                                                                        <div class="suggestions-list">
                                                                            {suggs.into_iter().map(|sugg| {
                                                                                let title = sugg.title.clone();
                                                                                let explanation = sugg.explanation.clone();
                                                                                let replacement = sugg.replacement.clone();
                                                                                let range = sugg.range.clone();

                                                                                let code_sig = code;
                                                                                let snack = show_snack_cb;
                                                                                let has_fix = replacement.is_some() && range.is_some();

                                                                                let on_apply_fix = move |_| {
                                                                                    if let (Some(repl), Some(r)) = (&replacement, &range) {
                                                                                        let orig = code_sig.get_untracked();
                                                                                        let updated = apply_replacement(&orig, r, repl);
                                                                                        code_sig.set(updated);
                                                                                        snack.run("Quick Fix applied successfully!".to_string());
                                                                                    }
                                                                                };

                                                                                view! {
                                                                                    <div class="suggestion-card">
                                                                                        <div class="suggestion-card-header">
                                                                                            <span class="suggestion-card-icon">"💡"</span>
                                                                                            <span class="suggestion-card-title">{title}</span>
                                                                                        </div>
                                                                                        <div class="suggestion-card-explanation">{explanation}</div>
                                                                                        {has_fix.then(|| view! {
                                                                                            <button class="btn btn-primary btn-sm" on:click=on_apply_fix style="margin-top:8px">
                                                                                                "Apply Quick Fix"
                                                                                            </button>
                                                                                        })}
                                                                                    </div>
                                                                                }
                                                                            }).collect_view()}
                                                                        </div>
                                                                    }.into_any()
                                                                } else {
                                                                    view! { "" }.into_any()
                                                                }
                                                            }}
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! { "" }.into_any()
                                                }
                                            }}
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                } else if bottom_tab.get() == 2 {
                    let refs = references_list.get();
                    if refs.is_empty() {
                        view! {
                            <div class="problems-container empty">
                                "No references or definition locations have been resolved yet."
                            </div>
                        }.into_any()
                    } else {
                        let active = active_tab.get();
                        let current_code = code.get();
                        let lines: Vec<String> = current_code.lines().map(|s| s.to_string()).collect();

                        view! {
                            <div class="references-list-container">
                                {refs.into_iter().map(|loc| {
                                    let loc_clone = loc.clone();
                                    let display_name = if let Some(last_slash) = loc.uri.rfind('/') {
                                        loc.uri[last_slash + 1..].to_string()
                                    } else {
                                        loc.uri.clone()
                                    };
                                    let display_path = if loc.uri.starts_with("file://") {
                                        loc.uri.strip_prefix("file://").unwrap_or(&loc.uri).to_string()
                                    } else {
                                        loc.uri.clone()
                                    };
                                    let line = loc.range.start.line;
                                    let col = loc.range.start.character;

                                    let is_active_file = active.as_ref().map(|act| {
                                        let suffix = format!("/{}", act);
                                        loc.uri.ends_with(&suffix)
                                    }).unwrap_or(false);

                                    let line_preview = if is_active_file && (line as usize) < lines.len() {
                                        let content = lines[line as usize].trim().to_string();
                                        if !content.is_empty() {
                                            Some(content)
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    };

                                    let click_cb = on_click_reference;
                                    view! {
                                        <div class="reference-item" on:click=move |_| click_cb.run(loc_clone.clone())>
                                            <div class="reference-icon-wrap">
                                                <LucideIcon name="locate-fixed" size="14" />
                                            </div>
                                            <div class="reference-details">
                                                <div class="reference-meta">
                                                    <span class="reference-filename">{display_name}</span>
                                                    <span class="reference-badge">{format!("Line {}, Col {}", line + 1, col + 1)}</span>
                                                </div>
                                                <div class="reference-path">{display_path}</div>
                                                {line_preview.map(|snippet| view! {
                                                    <div class="reference-code-snippet">
                                                        {snippet}
                                                    </div>
                                                })}
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                } else {
                    view! {
                        <div class="terminal-container" on:click=move |_| {
                            if let Some(input) = input_ref.get() {
                                let _ = input.focus();
                            }
                        }>
                            <div class="terminal-toolbar" on:click=move |e| e.stop_propagation()>
                                <div class="terminal-toolbar-left">
                                    <span class="terminal-title-icon">"🐚"</span>
                                    <span class="terminal-active-name">{move || active_session_name.get()}</span>
                                </div>
                                <div class="terminal-toolbar-right">
                                    // Desktop VS Code-like horizontal tabs
                                    <div class="terminal-tabs-desktop">
                                        {move || {
                                            sessions.get().iter().enumerate().map(|(idx, s)| {
                                                let is_selected = idx == active_idx.get();
                                                let switch_cb = move |_| switch_to_session(idx);
                                                let close_cb = move |e: MouseEvent| {
                                                    e.stop_propagation();
                                                    kill_session_by_index(idx);
                                                };
                                                view! {
                                                    <button
                                                        class=move || if is_selected { "terminal-tab-desktop active" } else { "terminal-tab-desktop" }
                                                        on:click=switch_cb
                                                    >
                                                        <span>{s.name.clone()}</span>
                                                        <span class="terminal-tab-close-btn" on:click=close_cb>
                                                            <LucideIcon name="x" size="10" />
                                                        </span>
                                                    </button>
                                                }
                                            }).collect_view()
                                        }}
                                    </div>

                                    // Mobile select dropdown wrapper
                                    <div class="terminal-session-select-wrapper">
                                        <select class="terminal-session-select" on:change=on_session_change>
                                            {move || {
                                                sessions.get().iter().enumerate().map(|(idx, s)| {
                                                    let is_selected = idx == active_idx.get();
                                                    view! {
                                                        <option value=idx.to_string() prop:selected=is_selected>
                                                            {format!("{}: {}", idx + 1, s.name)}
                                                        </option>
                                                    }
                                                }).collect_view()
                                            }}
                                        </select>
                                    </div>
                                    <button class="terminal-toolbar-btn" on:click=add_new_session title="New Terminal">
                                        <LucideIcon name="plus" size="14" />
                                    </button>
                                    <button class="terminal-toolbar-btn btn-kill" on:click=kill_active_session title="Kill Terminal">
                                        <LucideIcon name="trash" size="14" />
                                    </button>
                                    <button class="terminal-toolbar-btn" on:click=move |_| {
                                        let w = web_sys::window().unwrap();
                                        let _ = w.navigator().clipboard().write_text(&output.get_untracked());
                                        show_snack.run("Output copied!".to_string());
                                    } title="Copy Output">
                                        <LucideIcon name="copy" size="14" />
                                    </button>
                                    <button class="terminal-toolbar-btn" on:click=move |_| {
                                        output.set(String::new());
                                        sessions.update(|s_list| {
                                            let active = active_idx.get_untracked();
                                            if active < s_list.len() {
                                                s_list[active].output = String::new();
                                            }
                                        });
                                    } title="Clear Terminal">
                                        <LucideIcon name="trash" size="14" />
                                    </button>
                                    <button class="terminal-toolbar-btn" on:click=move |_| is_maximized.update(|v| *v = !*v) title=move || if is_maximized.get() { "Minimize" } else { "Maximize" }>
                                        {move || if is_maximized.get() {
                                            view! { <LucideIcon name="chevron-down" size="14" /> }.into_any()
                                        } else {
                                            view! { <LucideIcon name="chevron-up" size="14" /> }.into_any()
                                        }}
                                    </button>
                                </div>
                            </div>
                            
                            <div class="terminal-output-area" node_ref=output_area_ref>
                                {move || output.get()}
                            </div>
                            
                            <div class="terminal-mobile-keys" on:click=move |e| e.stop_propagation()>
                                <button class="terminal-key-btn" on:click=send_tab>"TAB"</button>
                                <button class="terminal-key-btn" on:click=send_ctrl_c>"CTRL+C"</button>
                                <button class="terminal-key-btn" on:click=send_ctrl_d>"CTRL+D"</button>
                                <button class="terminal-key-btn" on:click=send_arrow_up>"↑"</button>
                                <button class="terminal-key-btn" on:click=send_arrow_down>"↓"</button>
                                <button class="terminal-key-btn" on:click=move |_| {
                                    output.set(String::new());
                                    sessions.update(|s_list| {
                                        let active = active_idx.get_untracked();
                                        if active < s_list.len() {
                                            s_list[active].output = String::new();
                                        }
                                    });
                                }>"CLEAR"</button>
                                
                                <button class="terminal-key-btn command-chip" on:click=move |_| run_quick_cmd("ls".to_string())>"ls"</button>
                                <button class="terminal-key-btn command-chip" on:click=move |_| run_quick_cmd("cd ..".to_string())>"cd .."</button>
                                <button class="terminal-key-btn command-chip" on:click=move |_| run_quick_cmd("git status".to_string())>"git status"</button>
                                {move || {
                                    let lang = language.get().to_lowercase();
                                    match lang.as_str() {
                                        "rust" => view! {
                                            <button class="terminal-key-btn command-chip" on:click=move |_| run_quick_cmd("cargo check".to_string())>"cargo check"</button>
                                            <button class="terminal-key-btn command-chip" on:click=move |_| run_quick_cmd("cargo test".to_string())>"cargo test"</button>
                                        }.into_any(),
                                        "javascript" | "typescript" => view! {
                                            <button class="terminal-key-btn command-chip" on:click=move |_| run_quick_cmd("npm run dev".to_string())>"npm run dev"</button>
                                            <button class="terminal-key-btn command-chip" on:click=move |_| run_quick_cmd("npm install".to_string())>"npm install"</button>
                                        }.into_any(),
                                        "python" => view! {
                                            <button class="terminal-key-btn command-chip" on:click=move |_| run_quick_cmd("python3 main.py".to_string())>"python3"</button>
                                        }.into_any(),
                                        _ => view! { "" }.into_any()
                                    }
                                }}
                            </div>
                            
                            <div class="terminal-input-line">
                                <span class="terminal-prompt">
                                    {move || {
                                        let list = sessions.get();
                                        let idx = active_idx.get();
                                        if idx < list.len() {
                                            if let Some(ref path) = list[idx].path {
                                                let clean_path = if path.starts_with("/Codedroid_Projects/") {
                                                    &path["/Codedroid_Projects/".len()..]
                                                } else if path.starts_with("/Codedroid_Projects") {
                                                    &path["/Codedroid_Projects".len()..]
                                                } else {
                                                    path.as_str()
                                                };
                                                if clean_path.is_empty() {
                                                    format!("{} $", project_name.get())
                                                } else {
                                                    format!("{} $", clean_path)
                                                }
                                            } else {
                                                format!("{} $", project_name.get())
                                            }
                                        } else {
                                            format!("{} $", project_name.get())
                                        }
                                    }}
                                </span>
                                <input
                                    type="text"
                                    class="terminal-input"
                                    node_ref=input_ref
                                    prop:value=move || command_input.get()
                                    on:input=move |e| command_input.set(event_target_value(&e))
                                    on:keydown=on_keydown
                                    placeholder="Type command or input..."
                                />
                            </div>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

#[component]
pub fn DependencyModal(
    show_deps: RwSignal<bool>,
    dep_input: RwSignal<String>,
    dep_output: Signal<String>,
    add_dep: Callback<()>,
) -> impl IntoView {
    view! {
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
    }
}
