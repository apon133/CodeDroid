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
    sidebar_open: Signal<bool>,
    toggle_sidebar: Callback<()>,
) -> impl IntoView {
    let (show_new_file, set_show_new_file) = create_signal(false);
    let (show_new_folder, set_show_new_folder) = create_signal(false);
    let (new_name, set_new_name) = create_signal(String::new());

    let (press_id, set_press_id) = create_signal(0i32);
    let (collapsed_dirs, set_collapsed_dirs) = create_signal(std::collections::HashSet::<String>::new());
    let (selected_path, set_selected_path) = create_signal(Option::<String>::None);
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
            <div class="file-tree-header" style="display:flex; justify-content:space-between; align-items:center;">
                <span style="overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font-weight:700;">
                    {lang_icon}" "{project_name.to_uppercase()}
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
                                                <span style="font-size:14px; display:flex; align-items:center;">
                                                    {file_icon(&f.name)}
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
    }
}

#[component]
pub fn BottomPanel(
    bottom_tab: RwSignal<usize>,
    preview_url: Signal<Option<String>>,
    output: Signal<String>,
    is_error: Signal<bool>,
    show_snack: Callback<String>,
) -> impl IntoView {
    view! {
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
                        on:click=move |_| { /* This logic should probably be passed in if output is ReadOnly */ }
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
