use leptos::prelude::*;
use crate::pages::editor::utils::*;
use web_sys::{Event, KeyboardEvent, MouseEvent};

#[component]
pub fn FileTree(
    file_tree: Signal<Vec<FileEntry>>,
    active_tab: Signal<Option<String>>,
    open_file: Callback<String>,
    lang_icon: String,
    project_name: String,
) -> impl IntoView {
    view! {
        <div class="file-tree-panel">
            <div class="file-tree-header">
                <span>{lang_icon}" "{project_name.to_uppercase()}</span>
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
