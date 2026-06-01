use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use uuid::Uuid;
use web_sys::MouseEvent;

use crate::components::app_bar::AppBar;
use crate::components::icon::LucideIcon;
use crate::components::new_project_modal::{NewProjectModal, NewProjectResult};
use crate::components::snackbar::Snackbar;
use crate::models::Project;
use crate::store;



fn format_project_path(path: &str) -> String {
    let is_mobile = web_sys::window()
        .and_then(|w| w.navigator().user_agent().ok())
        .map(|ua| {
            let ua_lower = ua.to_lowercase();
            ua_lower.contains("android")
                || ua_lower.contains("iphone")
                || ua_lower.contains("ipad")
        })
        .unwrap_or(false);

    if path.starts_with("/Codedroid_Projects") {
        let relative_path = &path["/Codedroid_Projects".len()..];
        if is_mobile {
            format!("phone/download/codedroid{}", relative_path)
        } else {
            format!("~/Codedroid_Projects{}", relative_path)
        }
    } else if path.starts_with("/Codedroid_Desktop") {
        let relative_path = &path["/Codedroid_Desktop".len()..];
        if is_mobile {
            format!("phone/download/codedroid_desktop{}", relative_path)
        } else {
            format!("~/Desktop{}", relative_path)
        }
    } else if path.starts_with("/Codedroid_Documents") {
        let relative_path = &path["/Codedroid_Documents".len()..];
        if is_mobile {
            format!("phone/documents/codedroid{}", relative_path)
        } else {
            format!("~/Documents{}", relative_path)
        }
    } else {
        path.to_string()
    }
}

#[component]
pub fn HomePage() -> impl IntoView {
    let navigate = use_navigate();

    let projects: RwSignal<Vec<Project>> = RwSignal::new(store::load_projects());
    let show_modal = RwSignal::new(false);
    let snack_msg: RwSignal<Option<String>> = RwSignal::new(None);
    let confirm_delete: RwSignal<Option<Project>> = RwSignal::new(None);

    // Show snackbar for 3s then hide
    let _show_snack = move |msg: &str| {
        let msg = msg.to_string();
        snack_msg.set(Some(msg));
        let snack = snack_msg;
        gloo_timers::callback::Timeout::new(3000, move || {
            snack.set(None);
        })
        .forget();
    };

    let on_create = {
        let navigate = navigate.clone();
        Callback::new(move |result: NewProjectResult| {
            let path = result.path.clone();
            let project = Project {
                id: Uuid::new_v4().to_string(),
                name: result.name.clone(),
                language: result.lang.clone(),
                path: path.clone(),
                created_at: js_sys::Date::now() as u64,
                framework: result.framework.clone(),
            };

            let pid = project.id.clone();
            store::add_project(&projects, project);
            show_modal.set(false);

            let nav = navigate.clone();
            nav(&format!("/editor/{}", pid), Default::default());
        })
    };
    let on_cancel = Callback::new(move |()| show_modal.set(false));

    let on_open_project = {
        let navigate = navigate.clone();
        let projects = projects.clone();
        move |_| {
            let navigate = navigate.clone();
            let projects = projects.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(resp) = crate::api::pick_directory_api().await {
                    if resp.success {
                        if let Some(path) = resp.path {
                            let name = std::path::Path::new(&path)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("project")
                                .to_string();

                            let project = Project {
                                id: Uuid::new_v4().to_string(),
                                name,
                                language: "auto".to_string(),
                                path: path.clone(),
                                created_at: js_sys::Date::now() as u64,
                                framework: "none".to_string(),
                            };

                            let pid = project.id.clone();
                            store::add_project(&projects, project);

                            let nav = navigate.clone();
                            nav(&format!("/editor/{}", pid), Default::default());
                        }
                    }
                }
            });
        }
    };

    view! {
        <div>
            <AppBar title="CodeDroid".to_string()>
                <a href="https://github.com/apon133/CodeDroid" target="_blank" rel="noopener noreferrer" style="text-decoration:none;margin-right:8px">
                    <button class="btn btn-icon" title="GitHub Repository">
                        <LucideIcon name="github" size="20" />
                    </button>
                </a>
                <a href="/settings" style="text-decoration:none">
                    <button class="btn btn-icon" title="Settings">
                        <LucideIcon name="settings" size="20" />
                    </button>
                </a>
            </AppBar>

            <div class="home">
                {move || {
                    let projs = projects.get();
                    if projs.is_empty() {
                        view! {
                            <div class="home-empty">
                                <div class="icon" style="color:var(--text2); opacity: 0.5;">
                                    <LucideIcon name="folder" size="48" />
                                </div>
                                <p>"No projects yet — create one!"</p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="projects-list">
                                {projs.into_iter().map(|p| {
                                    let pid = p.id.clone();
                                    let nav = navigate.clone();
                                    let icon_url = crate::models::project_icon(&p.language, &p.framework);
                                    let (badge_text, color, bg) = crate::models::project_badge_info(&p.language, &p.framework);
                                    let display_path = format_project_path(&p.path);
                                    let alt_text = badge_text.clone();
                                    let p_clone = p.clone();
                                    view! {
                                        <div class="project-card"
                                            on:click=move |_| nav(&format!("/editor/{}", pid), Default::default())
                                        >
                                            <div class="project-icon">
                                                <img src=icon_url class="lang-icon-img" alt=alt_text />
                                            </div>
                                            <div class="project-info">
                                                <div class="project-name">{p.name.clone()}</div>
                                                <div class="project-path">{display_path.clone()}</div>
                                            </div>
                                            <span class="lang-badge" style=format!("color:{color};border-color:{color};background:{bg}")>
                                                {badge_text}
                                            </span>
                                            <button class="btn btn-icon"
                                                style="color:#ff453a;font-size:16px;display:flex;align-items:center;justify-content:center"
                                                title="Delete"
                                                on:click=move |e: MouseEvent| {
                                                    e.stop_propagation();
                                                    confirm_delete.set(Some(p_clone.clone()));
                                                }
                                            >
                                                <LucideIcon name="trash" size="18" />
                                            </button>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>

            <div class="fab-container" style="position:fixed; bottom:32px; right:32px; display:flex; flex-direction:column; gap:16px; z-index:100;">
                <button class="fab" title="Open Project" style="position:static; background:#1e1e24; border:1px solid var(--border); box-shadow: 0 8px 30px rgba(0,0,0,0.3);" on:click=on_open_project>
                    <LucideIcon name="folder" size="24" />
                </button>
                <button class="fab" title="New Project" style="position:static;" on:click=move |_| show_modal.set(true)>
                    <LucideIcon name="plus" size="24" />
                </button>
            </div>

            {move || show_modal.get().then(|| view! {
                <NewProjectModal on_create=on_create on_cancel=on_cancel />
            })}

            {move || confirm_delete.get().map(|proj| {
                let proj2 = proj.clone();
                let projects_clone = projects.clone();
                let close = move |_: MouseEvent| confirm_delete.set(None);
                let delete = move |_: MouseEvent| {
                    let path = proj2.path.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let _ = crate::api::delete_file_api(&path, true).await;
                    });
                    store::delete_project(&projects_clone, &proj2.id);
                    confirm_delete.set(None);
                };
                view! {
                    <div class="modal-overlay" on:click=close>
                        <div class="modal modal-destructive" on:click=move |e: MouseEvent| e.stop_propagation()
                            style="max-width:400px;text-align:center;padding:32px 24px;"
                        >
                            <div class="destructive-icon-container">
                                <LucideIcon name="alert-triangle" size="32" />
                            </div>
                            <div class="modal-title-destructive">
                                "Delete Project"
                            </div>
                            <div class="modal-desc-destructive">
                                "Are you sure you want to delete project "
                                <strong>{proj.name.clone()}</strong>
                                "? This action cannot be undone."
                            </div>
                            <div style="display:flex;justify-content:center;gap:12px;width:100%">
                                <button class="btn btn-cancel-destructive" on:click=close>
                                    "Cancel"
                                </button>
                                <button class="btn btn-delete-destructive" on:click=delete>
                                    "Delete"
                                </button>
                            </div>
                        </div>
                    </div>
                }
            })}

            <Snackbar message=snack_msg.read_only() />
        </div>
    }
}
