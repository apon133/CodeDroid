use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::components::icon::LucideIcon;
use crate::pages::editor::utils::file_icon;

fn split_path(path: &str) -> (String, String) {
    let p = std::path::Path::new(path);
    let filename = p.file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string());
    let parent = p.parent()
        .map(|dir| dir.to_string_lossy().into_owned())
        .unwrap_or_default();
    (filename, parent)
}

#[component]
pub fn GitPanel(
    project_path: String,
    git_status: RwSignal<Option<api::GitStatusResponse>>,
    trigger_git_status: Callback<()>,
    open_file: Callback<String>,
    show_snack: Callback<String>,
    sidebar_open: Signal<bool>,
    close_sidebar: Callback<()>,
) -> impl IntoView {
    let commit_message = RwSignal::new(String::new());
    let is_loading = RwSignal::new(false);

    // Collapsible sections
    let input_expanded = RwSignal::new(true);
    let staged_expanded = RwSignal::new(true);
    let unstaged_expanded = RwSignal::new(true);
    let history_expanded = RwSignal::new(true);
    let gitlens_expanded = RwSignal::new(false);

    // Git Log/History
    let git_log = RwSignal::new(Option::<api::GitLogResponse>::None);

    let refresh_git_log = Callback::new({
        let ppath = project_path.clone();
        let log_signal = git_log.clone();
        move |_: ()| {
            let ppath = ppath.clone();
            let log_signal = log_signal.clone();
            spawn_local(async move {
                if let Ok(res) = api::git_log_api(&ppath).await {
                    log_signal.set(Some(res));
                }
            });
        }
    });

    // Reactive effect: Refresh commit log whenever status updates
    Effect::new(move |_| {
        let _ = git_status.get();
        refresh_git_log.run(());
    });

    // Group files into staged and unstaged/changes
    let staged_files = move || {
        let mut list = Vec::new();
        if let Some(ref status) = git_status.get() {
            for f in &status.files {
                let s = &f.status;
                if s != "??" && s.chars().next().map(|c| c != ' ' && c != '?').unwrap_or(false) {
                    list.push(f.clone());
                }
            }
        }
        list
    };

    let unstaged_files = move || {
        let mut list = Vec::new();
        if let Some(ref status) = git_status.get() {
            for f in &status.files {
                let s = &f.status;
                if s == "??" || s.chars().nth(1).map(|c| c != ' ').unwrap_or(false) {
                    list.push(f.clone());
                }
            }
        }
        list
    };

    // Actions
    let stage_file = Callback::new({
        let ppath = project_path.clone();
        let trigger_status = trigger_git_status.clone();
        let snack = show_snack.clone();
        move |file_path: String| {
            let ppath = ppath.clone();
            let trigger = trigger_status.clone();
            let s = snack.clone();
            spawn_local(async move {
                match api::git_stage_api(&ppath, &file_path).await {
                    Ok(res) => {
                        if res.success {
                            s.run(format!("Staged {}", file_path));
                            trigger.run(());
                        } else {
                            s.run(res.error.unwrap_or_else(|| "Failed to stage file".to_string()));
                        }
                    }
                    Err(e) => s.run(format!("Error: {}", e)),
                }
            });
        }
    });

    let unstage_file = Callback::new({
        let ppath = project_path.clone();
        let trigger_status = trigger_git_status.clone();
        let snack = show_snack.clone();
        move |file_path: String| {
            let ppath = ppath.clone();
            let trigger = trigger_status.clone();
            let s = snack.clone();
            spawn_local(async move {
                match api::git_unstage_api(&ppath, &file_path).await {
                    Ok(res) => {
                        if res.success {
                            s.run(format!("Unstaged {}", file_path));
                            trigger.run(());
                        } else {
                            s.run(res.error.unwrap_or_else(|| "Failed to unstage file".to_string()));
                        }
                    }
                    Err(e) => s.run(format!("Error: {}", e)),
                }
            });
        }
    });

    let discard_file = Callback::new({
        let ppath = project_path.clone();
        let trigger_status = trigger_git_status.clone();
        let snack = show_snack.clone();
        move |file_path: String| {
            let ppath = ppath.clone();
            let trigger = trigger_status.clone();
            let s = snack.clone();
            if let Some(window) = web_sys::window() {
                if let Ok(d) = window.confirm_with_message(&format!("Are you sure you want to discard changes in {}? This cannot be undone.", file_path)) {
                    if !d {
                        return;
                    }
                }
            }
            spawn_local(async move {
                match api::git_discard_api(&ppath, &file_path).await {
                    Ok(res) => {
                        if res.success {
                            s.run(format!("Discarded changes in {}", file_path));
                            trigger.run(());
                        } else {
                            s.run(res.error.unwrap_or_else(|| "Failed to discard changes".to_string()));
                        }
                    }
                    Err(e) => s.run(format!("Error: {}", e)),
                }
            });
        }
    });

    let commit_changes = Callback::new({
        let ppath = project_path.clone();
        let trigger_status = trigger_git_status.clone();
        let snack = show_snack.clone();
        let staged_fn = staged_files.clone();
        let unstaged_fn = unstaged_files.clone();
        move |_: ()| {
            let msg = commit_message.get_untracked();
            if msg.trim().is_empty() {
                snack.run("Please enter a commit message.".to_string());
                return;
            }

            is_loading.set(true);
            let ppath = ppath.clone();
            let trigger = trigger_status.clone();
            let s = snack.clone();
            let sf = staged_fn.clone();
            let uf = unstaged_fn.clone();
            spawn_local(async move {
                let staged = sf();
                if staged.is_empty() {
                    let unstaged = uf();
                    for file in unstaged {
                        let _ = api::git_stage_api(&ppath, &file.path).await;
                    }
                }

                match api::git_commit_api(&ppath, &msg).await {
                    Ok(res) => {
                        if res.success {
                            commit_message.set(String::new());
                            s.run("Changes committed successfully!".to_string());
                            trigger.run(());
                        } else {
                            s.run(res.error.unwrap_or_else(|| "Failed to commit".to_string()));
                        }
                    }
                    Err(e) => s.run(format!("Commit failed: {}", e)),
                }
                is_loading.set(false);
            });
        }
    });

    let _pull_changes = Callback::new({
        let ppath = project_path.clone();
        let trigger_status = trigger_git_status.clone();
        let snack = show_snack.clone();
        move |_: ()| {
            is_loading.set(true);
            let ppath = ppath.clone();
            let trigger = trigger_status.clone();
            let s = snack.clone();
            spawn_local(async move {
                match api::git_pull_api(&ppath).await {
                    Ok(res) => {
                        if res.success {
                            s.run("Pulled changes successfully!".to_string());
                            trigger.run(());
                        } else {
                            s.run(res.error.unwrap_or_else(|| "Failed to pull changes".to_string()));
                        }
                    }
                    Err(e) => s.run(format!("Pull failed: {}", e)),
                }
                is_loading.set(false);
            });
        }
    });

    let _push_changes = Callback::new({
        let ppath = project_path.clone();
        let trigger_status = trigger_git_status.clone();
        let snack = show_snack.clone();
        move |_: ()| {
            is_loading.set(true);
            let ppath = ppath.clone();
            let trigger = trigger_status.clone();
            let s = snack.clone();
            spawn_local(async move {
                match api::git_push_api(&ppath).await {
                    Ok(res) => {
                        if res.success {
                            s.run("Pushed changes successfully!".to_string());
                            trigger.run(());
                        } else {
                            s.run(res.error.unwrap_or_else(|| "Failed to push changes".to_string()));
                        }
                    }
                    Err(e) => s.run(format!("Push failed: {}", e)),
                }
                is_loading.set(false);
            });
        }
    });

    view! {
        {move || sidebar_open.get().then(|| view! {
            <div class="sidebar-overlay" on:click=move |_| close_sidebar.run(()) />
        })}

        <div class=move || if sidebar_open.get() { "file-tree-panel git-panel open" } else { "file-tree-panel git-panel" }>
            
            // Header
            <div class="git-panel-header">
                <div class="git-panel-title" style="font-size: 13px; font-weight: 500; text-transform: none; letter-spacing: normal; color: var(--text);">"Source Control"</div>
                <button class="git-action-btn" title="More Actions">
                    <LucideIcon name="more-horizontal" size="16" />
                </button>
            </div>

            // Repository List / Changed Files
            <div style="flex:1; overflow-y:auto; display:flex; flex-direction:column;">
                
                // 1. --- CHANGES SECTION (Commit input area) ---
                <button class="git-section-header-btn" on:click=move |_| input_expanded.set(!input_expanded.get())>
                    <div class="git-section-header-left">
                        <span class=move || if input_expanded.get() { "git-section-chevron" } else { "git-section-chevron collapsed" }>
                            <LucideIcon name="chevron-down" size="12" />
                        </span>
                        <span>"Changes"</span>
                    </div>
                </button>

                {move || input_expanded.get().then(|| {
                    view! {
                        <div class="git-commit-box">
                            <div class="git-commit-input-wrapper">
                                <textarea 
                                    class="git-commit-input" 
                                    placeholder="Commit message (Press Ctrl+Enter to commit)..."
                                    prop:value=move || commit_message.get()
                                    on:input=move |e| commit_message.set(event_target_value(&e))
                                    on:keydown=move |e: web_sys::KeyboardEvent| {
                                        if e.key() == "Enter" && e.ctrl_key() {
                                            e.prevent_default();
                                            commit_changes.run(());
                                        }
                                    }
                                    disabled=move || is_loading.get()
                                />
                            </div>
                            
                            // Split Commit Button Group
                            <div class="git-commit-btn-group">
                                <button class="git-commit-main-btn" on:click=move |_| commit_changes.run(()) disabled=move || is_loading.get()>
                                    {move || if is_loading.get() {
                                        view! { <span class="spinner" style="width:12px; height:12px; border-width:2px;"></span> }.into_any()
                                    } else {
                                        view! { <LucideIcon name="check" size="14" /> }.into_any()
                                    }}
                                    "Commit"
                                </button>
                                <button class="git-commit-arrow-btn" title="Commit Options">
                                    <LucideIcon name="chevron-down" size="14" />
                                </button>
                            </div>
                        </div>
                    }
                })}

                // 2. --- STAGED CHANGES SECTION (Staged files list) ---
                <button class="git-section-header-btn" style="margin-top: 4px;" on:click=move |_| staged_expanded.set(!staged_expanded.get())>
                    <div class="git-section-header-left">
                        <span class=move || if staged_expanded.get() { "git-section-chevron" } else { "git-section-chevron collapsed" }>
                            <LucideIcon name="chevron-down" size="12" />
                        </span>
                        <span>"Staged Changes"</span>
                    </div>
                    {move || {
                        let count = staged_files().len();
                        (count > 0).then(|| view! {
                            <span class="git-section-badge">{count}</span>
                        })
                    }}
                </button>

                {move || staged_expanded.get().then(|| {
                    let staged = staged_files();
                    view! {
                        <div class="git-file-list">
                            {if staged.is_empty() {
                                view! {
                                    <div style="padding: 12px 16px; font-size: 11px; color: var(--text3); opacity: 0.7;">
                                        "No staged changes."
                                    </div>
                                }.into_any()
                            } else {
                                staged.into_iter().map(|f| {
                                    let f_click = f.clone();
                                    let f_stage = f.clone();
                                    let f_discard = f.clone();
                                    let open_f = open_file;
                                    let (fname, fdir) = split_path(&f.path);

                                    let letter = f.status.chars().next().unwrap_or('M').to_string();
                                    let badge_class = match letter.as_str() {
                                        "A" => "git-status-badge-indicator added",
                                        "M" => "git-status-badge-indicator modified",
                                        "D" => "git-status-badge-indicator deleted",
                                        _ => "git-status-badge-indicator modified",
                                    };

                                    view! {
                                        <div class="git-file-item" on:click=move |_| {
                                            open_f.run(format!("git-diff://{}", f_click.path));
                                        }>
                                            <div class="git-file-info">
                                                <img src=file_icon(&f.path).to_string() style="width:14px; height:14px; object-fit:contain;" alt="" />
                                                <div class="git-file-path-container">
                                                    <span class="git-file-name">{fname}</span>
                                                    {(!fdir.is_empty()).then(|| view! {
                                                        <span class="git-file-dir">{fdir.clone()}</span>
                                                    })}
                                                </div>
                                            </div>
                                            <div style="display:flex; align-items:center; gap:8px;">
                                                <div class="git-file-actions">
                                                    <button class="git-action-btn unstage" title="Unstage Changes" on:click=move |e| {
                                                        e.stop_propagation();
                                                        unstage_file.run(f_stage.path.clone());
                                                    }>
                                                        <LucideIcon name="minus" size="14" />
                                                    </button>
                                                    <button class="git-action-btn discard" title="Discard Changes" on:click=move |e| {
                                                        e.stop_propagation();
                                                        discard_file.run(f_discard.path.clone());
                                                    }>
                                                        <LucideIcon name="trash" size="14" />
                                                    </button>
                                                </div>
                                                <span class=badge_class>{letter}</span>
                                            </div>
                                        </div>
                                    }
                                }).collect_view().into_any()
                            }}
                        </div>
                    }
                })}

                // 3. --- CHANGES SECTION (Unstaged files list) ---
                <button class="git-section-header-btn" style="margin-top: 4px;" on:click=move |_| unstaged_expanded.set(!unstaged_expanded.get())>
                    <div class="git-section-header-left">
                        <span class=move || if unstaged_expanded.get() { "git-section-chevron" } else { "git-section-chevron collapsed" }>
                            <LucideIcon name="chevron-down" size="12" />
                        </span>
                        <span>"Changes"</span>
                    </div>
                    {move || {
                        let count = unstaged_files().len();
                        (count > 0).then(|| view! {
                            <span class="git-section-badge">{count}</span>
                        })
                    }}
                </button>

                {move || unstaged_expanded.get().then(|| {
                    let unstaged = unstaged_files();
                    view! {
                        <div class="git-file-list">
                            {if unstaged.is_empty() {
                                view! {
                                    <div style="padding: 12px 16px; font-size: 11px; color: var(--text3); opacity: 0.7;">
                                        "No unstaged changes."
                                    </div>
                                }.into_any()
                            } else {
                                unstaged.into_iter().map(|f| {
                                    let f_click = f.clone();
                                    let f_stage = f.clone();
                                    let f_discard = f.clone();
                                    let open_f = open_file;
                                    let (fname, fdir) = split_path(&f.path);

                                    let letter = if f.status == "??" {
                                        "U".to_string()
                                    } else {
                                        f.status.chars().nth(1).unwrap_or('M').to_string()
                                    };
                                    let badge_class = match letter.as_str() {
                                        "U" => "git-status-badge-indicator untracked",
                                        "A" => "git-status-badge-indicator added",
                                        "M" => "git-status-badge-indicator modified",
                                        "D" => "git-status-badge-indicator deleted",
                                        _ => "git-status-badge-indicator modified",
                                    };

                                    view! {
                                        <div class="git-file-item" on:click=move |_| {
                                            open_f.run(format!("git-diff://{}", f_click.path));
                                        }>
                                            <div class="git-file-info">
                                                <img src=file_icon(&f.path).to_string() style="width:14px; height:14px; object-fit:contain;" alt="" />
                                                <div class="git-file-path-container">
                                                    <span class="git-file-name">{fname}</span>
                                                    {(!fdir.is_empty()).then(|| view! {
                                                        <span class="git-file-dir">{fdir.clone()}</span>
                                                    })}
                                                </div>
                                            </div>
                                            <div style="display:flex; align-items:center; gap:8px;">
                                                <div class="git-file-actions">
                                                    <button class="git-action-btn stage" title="Stage Changes" on:click=move |e| {
                                                        e.stop_propagation();
                                                        stage_file.run(f_stage.path.clone());
                                                    }>
                                                        <LucideIcon name="plus" size="14" />
                                                    </button>
                                                    <button class="git-action-btn discard" title="Discard Changes" on:click=move |e| {
                                                        e.stop_propagation();
                                                        discard_file.run(f_discard.path.clone());
                                                    }>
                                                        <LucideIcon name="trash" size="14" />
                                                    </button>
                                                </div>
                                                <span class=badge_class>{letter}</span>
                                            </div>
                                        </div>
                                    }
                                }).collect_view().into_any()
                            }}
                        </div>
                    }
                })}

                // 4. --- GIT GRAPH / HISTORY SECTION ---
                <button class="git-section-header-btn" style="margin-top: 4px;" on:click=move |_| history_expanded.set(!history_expanded.get())>
                    <div class="git-section-header-left">
                        <span class=move || if history_expanded.get() { "git-section-chevron" } else { "git-section-chevron collapsed" }>
                            <LucideIcon name="chevron-down" size="12" />
                        </span>
                        <span>"Gr..."</span>
                    </div>
                    <div class="git-section-header-actions" on:click=move |e| e.stop_propagation()>
                        <span style="font-size: 10px; color: var(--text3); cursor: pointer; margin-right: 4px;">"Auto"</span>
                        <button class="git-action-btn" title="Focus HEAD"><LucideIcon name="crosshair" size="12" /></button>
                        <button class="git-action-btn" title="View Graph"><LucideIcon name="git-branch" size="12" /></button>
                        <button class="git-action-btn" title="Sync Branch"><LucideIcon name="cloud" size="12" /></button>
                        <button class="git-action-btn" title="Refresh Log" on:click=move |_| refresh_git_log.run(())><LucideIcon name="rotate-cw" size="12" /></button>
                        <button class="git-action-btn" title="More Actions"><LucideIcon name="more-horizontal" size="12" /></button>
                    </div>
                </button>

                {move || history_expanded.get().then(|| {
                    let log_opt = git_log.get();
                    view! {
                        <div class="git-history-list">
                            {match log_opt {
                                None => view! {
                                    <div style="padding: 12px 16px; font-size: 11px; color: var(--text3); opacity: 0.7;">
                                        "Loading commit log..."
                                    </div>
                                }.into_any(),
                                Some(log) => {
                                    if log.commits.is_empty() {
                                        view! {
                                            <div style="padding: 12px 16px; font-size: 11px; color: var(--text3); opacity: 0.7;">
                                                "No commits in repository."
                                            </div>
                                        }.into_any()
                                    } else {
                                        log.commits.into_iter().map(|commit| {
                                            let refs_str = commit.refs.clone();
                                            let branch = if !refs_str.is_empty() {
                                                if let Some(start) = refs_str.find("-> ") {
                                                    let rest = &refs_str[start + 3..];
                                                    let end = rest.find(',').or_else(|| rest.find(')')).unwrap_or(rest.len());
                                                    Some(rest[..end].trim().to_string())
                                                } else if let Some(start) = refs_str.find("(") {
                                                    let rest = &refs_str[start + 1..];
                                                    let end = rest.find(',').or_else(|| rest.find(')')).unwrap_or(rest.len());
                                                    Some(rest[..end].trim().to_string())
                                                } else {
                                                    None
                                                }
                                            } else {
                                                None
                                            };

                                            view! {
                                                <div class="git-history-item" title=format!("{} - {}", commit.author_name, commit.relative_date)>
                                                    <div class="git-history-graph-col">
                                                        <div class="git-history-line"></div>
                                                        <div class="git-history-node"></div>
                                                    </div>
                                                    <div class="git-history-content">
                                                        <span class="git-history-msg">{commit.subject.clone()}</span>
                                                        <div class="git-history-meta">
                                                            <span class="git-history-hash">{commit.hash.clone()}</span>
                                                            <span>"•"</span>
                                                            <span style="opacity: 0.8;">{commit.relative_date.clone()}</span>
                                                            {branch.map(|br| view! {
                                                                <span class="git-history-branch">
                                                                    <LucideIcon name="git-branch" size="9" />
                                                                    {br}
                                                                </span>
                                                            })}
                                                        </div>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view().into_any()
                                    }
                                }
                            }}
                        </div>
                    }
                })}

                // 5. --- GIT LENS (Mock Collapsed Section) ---
                <button class="git-section-header-btn" style="margin-top: 4px; border-bottom: none;" on:click=move |_| gitlens_expanded.set(!gitlens_expanded.get())>
                    <div class="git-section-header-left">
                        <span class=move || if gitlens_expanded.get() { "git-section-chevron" } else { "git-section-chevron collapsed" }>
                            <LucideIcon name="chevron-down" size="12" />
                        </span>
                        <span>"GitLens"</span>
                    </div>
                </button>

            </div>
        </div>
    }
}
