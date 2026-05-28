use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::pages::editor::utils::*;
use crate::components::icon::LucideIcon;
use web_sys::{Event, KeyboardEvent, MouseEvent};

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
    sidebar_mode: RwSignal<usize>,
) -> impl IntoView {
    let (show_new_file, set_show_new_file) = signal(false);
    let (show_new_folder, set_show_new_folder) = signal(false);
    let (new_name, set_new_name) = signal(String::new());
    let (show_rename, set_show_rename) = signal(Option::<FileEntry>::None);
    let (rename_name, set_rename_name) = signal(String::new());

    let (press_id, set_press_id) = signal(0i32);
    let (collapsed_dirs, set_collapsed_dirs) = signal(std::collections::HashSet::<String>::new());
    let (selected_path, set_selected_path) = signal(Option::<String>::None);
    let get_target_dir = move || {
        selected_path.get().map(|path| {
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
        }).unwrap_or_default()
    };
    
    let start_long_press = Callback::new({
        let paste_entry = paste_entry.clone();
        move |target_dir: Option<String>| {
            let next_id = press_id.get_untracked() + 1;
            set_press_id.set(next_id);
            let paste_entry = paste_entry.clone();
            spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(500).await;
                if press_id.get_untracked() == next_id {
                    paste_entry.run(target_dir);
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
                    start_long_press.run(None);
                }
            }
            on:mouseup=move |_| cancel_long_press.run(())
            on:mouseleave=move |_| cancel_long_press.run(())
            on:touchstart=move |e| {
                if e.target() == e.current_target() {
                    set_selected_path.set(None);
                    start_long_press.run(None);
                }
            }
            on:touchend=move |_| cancel_long_press.run(())
            on:touchcancel=move |_| cancel_long_press.run(())
        >
            <div class="sidebar-tabs">
                <button 
                    class=move || if sidebar_mode.get() == 0 { "sidebar-tab active" } else { "sidebar-tab" }
                    on:click=move |_| sidebar_mode.set(0)
                >
                    <LucideIcon name="folder" size="14" />
                    <span>"Files"</span>
                </button>
                <button 
                    class=move || if sidebar_mode.get() == 1 { "sidebar-tab active" } else { "sidebar-tab" }
                    on:click=move |_| sidebar_mode.set(1)
                >
                    <LucideIcon name="search" size="14" />
                    <span>"Search"</span>
                </button>
            </div>

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
                            let fname_mousedown = f.name.clone();
                            let fname_touchstart = f.name.clone();
                            let fname_active = f.name.clone();
                            let fname_lang = f.name.clone();
                            
                            let f_click = f.clone();
                            let f_copy_btn = f.clone();
                            let f_delete_btn = f.clone();
                            let f_rename_btn = f.clone();
                            
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
                                    on:mousedown=move |_| start_long_press.run(if f_click.is_dir { Some(fname_mousedown.clone()) } else { None })
                                    on:mouseup=move |_| cancel_long_press.run(())
                                    on:mouseleave=move |_| cancel_long_press.run(())
                                    on:touchstart=move |_| start_long_press.run(if f_click.is_dir { Some(fname_touchstart.clone()) } else { None })
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
                                    
                                    <div class="file-item-actions" style="display:flex; gap:6px; flex-shrink:0; align-items:center">
                                        {
                                            let target_dir_outer = fname_click.clone();
                                            move || {
                                                let is_dir = f_click.is_dir;
                                                let target_dir = target_dir_outer.clone();
                                                let paste_entry = paste_entry.clone();
                                                (is_dir && copied_item.get().is_some()).then(|| view! {
                                                    <button 
                                                        class="btn-tree-action" 
                                                        style="background:transparent; border:none; color:var(--accent2); cursor:pointer; padding:4px; display:flex; align-items:center; justify-content:center;"
                                                        title="Paste here"
                                                        on:click=move |e| {
                                                            e.stop_propagation();
                                                            paste_entry.run(Some(target_dir.clone()));
                                                        }
                                                    >
                                                        <LucideIcon name="clipboard" size="13" />
                                                    </button>
                                                })
                                            }
                                        }
                                        <button 
                                            class="btn-tree-action" 
                                            style="background:transparent; border:none; color:var(--text2); cursor:pointer; padding:4px; display:flex; align-items:center; justify-content:center; opacity: 0.6;"
                                            title="Rename/Move"
                                            on:click=move |e| {
                                                e.stop_propagation();
                                                let rename_target = f_rename_btn.clone();
                                                let path = rename_target.name.clone();
                                                set_show_rename.set(Some(rename_target));
                                                set_rename_name.set(path);
                                                set_show_new_file.set(false);
                                                set_show_new_folder.set(false);
                                            }
                                        >
                                            <LucideIcon name="edit" size="13" />
                                        </button>
                                        <button 
                                            class="btn-tree-action" 
                                            style="background:transparent; border:none; color:var(--text2); cursor:pointer; padding:4px; display:flex; align-items:center; justify-content:center; opacity: 0.6;"
                                            title="Copy"
                                            on:click=move |e| {
                                                e.stop_propagation();
                                                copy_entry.run(f_copy_btn.clone());
                                            }
                                        >
                                            <LucideIcon name="copy" size="13" />
                                        </button>
                                        <button 
                                            class="btn-tree-action" 
                                            style="background:transparent; border:none; color:#ff453a; cursor:pointer; padding:4px; display:flex; align-items:center; justify-content:center; opacity: 0.6;"
                                            title="Delete"
                                            on:click=move |e| {
                                                e.stop_propagation();
                                                delete_entry.run(f_delete_btn.clone());
                                            }
                                        >
                                            <LucideIcon name="trash" size="13" />
                                        </button>
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

#[component]
pub fn BottomPanel(
    bottom_tab: RwSignal<usize>,
    output: RwSignal<String>,
    is_error: Signal<bool>,
    show_snack: Callback<String>,
    diagnostics_list: Signal<Vec<crate::api::Diagnostic>>,
    on_click_problem: Callback<(Option<String>, u32, u32)>,
    code: RwSignal<String>,
    language: Signal<String>,
    references_list: RwSignal<Vec<crate::api::Location>>,
    on_click_reference: Callback<crate::api::Location>,
) -> impl IntoView {
    let expanded_idx = RwSignal::new(Option::<usize>::None);
    let suggestions_state = RwSignal::new(Option::<Vec<crate::api::CodeSuggestion>>::None);
    let loading_suggestions = RwSignal::new(false);

    view! {
        <div class="bottom-panel">
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
                {move || (bottom_tab.get() == 0).then(|| view! {
                    <>
                    <button class="btn btn-icon" style="font-size:12px" title="Copy output"
                        on:click=move |_| {
                            let w = web_sys::window().unwrap();
                            let _ = w.navigator().clipboard().write_text(&output.get_untracked());
                            show_snack.run("Output copied!".to_string());
                        }
                    >
                        <LucideIcon name="copy" size="16" />
                    </button>
                    <button class="btn btn-icon" style="font-size:12px" title="Clear terminal"
                        on:click=move |_| {
                            output.set(String::new());
                        }
                    >
                        <LucideIcon name="trash" size="16" />
                    </button>
                    </>
                })}
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
                        return view! {
                            <div class="problems-container empty">
                                "No problems have been detected in the workspace so far."
                            </div>
                        }.into_any();
                    }
                    return view! {
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
                } else if bottom_tab.get() == 2 {
                    let refs = references_list.get();
                    if refs.is_empty() {
                        view! {
                            <div class="problems-container empty">
                                "No references or definition locations have been resolved yet."
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="problems-container">
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
                                    
                                    let click_cb = on_click_reference;
                                    view! {
                                        <div class="problem-item info" on:click=move |_| click_cb.run(loc_clone.clone()) style="cursor:pointer">
                                            <div style="display:flex; align-items:center; gap:8px;">
                                                <span class="problem-severity">"🔍"</span>
                                                <div style="flex:1">
                                                    <div style="font-weight:600; color:var(--text); font-size:13px;">
                                                        {display_name}
                                                        <span style="color:var(--text2); font-weight:normal; font-size:11px; margin-left:8px;">
                                                            {format!("(Line {}, Col {})", line + 1, col + 1)}
                                                        </span>
                                                    </div>
                                                    <div style="font-size:11px; color:var(--text2); overflow:hidden; text-overflow:ellipsis; white-space:nowrap; margin-top:2px;">
                                                        {display_path}
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                } else {
                    view! {
                        <div
                            class=move || if is_error.get() { "terminal error" } else { "terminal" }
                        >
                            {move || output.get()}
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
