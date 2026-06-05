use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::components::icon::LucideIcon;
use crate::pages::editor::utils::{file_to_lsp_lang, pos_to_index};

#[component]
pub fn OutlinePanel(
    project_path: String,
    active_tab: RwSignal<Option<String>>,
    code: RwSignal<String>,
    cursor_pos: RwSignal<u32>,
    cursor_coords: RwSignal<(f64, f64)>,
    check_error_at_cursor: Callback<(u32, u32)>,
    show_snack: Callback<String>,
    sidebar_open: Signal<bool>,
    close_sidebar: Callback<()>,
) -> impl IntoView {
    let symbols = RwSignal::new(Vec::<api::DocumentSymbolResponse>::new());
    let filter = RwSignal::new(String::new());
    let is_loading = RwSignal::new(false);

    // Fetch symbols function
    let fetch_symbols = {
        let project_path = project_path.clone();
        let active_tab = active_tab.clone();
        let code = code.clone();
        let show_snack = show_snack.clone();
        let is_loading = is_loading.clone();
        let symbols = symbols.clone();
        
        Callback::new(move |_: ()| {
            let active_file = match active_tab.get_untracked() {
                Some(f) => f,
                None => {
                    symbols.set(Vec::new());
                    return;
                }
            };
            let code_content = code.get_untracked();
            let lang = file_to_lsp_lang(&active_file);
            let proj_path = project_path.clone();
            let symbols = symbols.clone();
            let is_loading = is_loading.clone();
            let show_snack = show_snack.clone();

            is_loading.set(true);
            spawn_local(async move {
                match api::get_symbols_api(&code_content, &lang, &proj_path, Some(&active_file)).await {
                    Ok(resp) => {
                        symbols.set(resp.symbols);
                    }
                    Err(e) => {
                        show_snack.run(format!("LSP symbols failed: {}", e));
                    }
                }
                is_loading.set(false);
            });
        })
    };

    // Refetch whenever active_tab or code changes (with a small debounce or on active tab change)
    Effect::new({
        let fetch_symbols = fetch_symbols.clone();
        move |_| {
            let _ = active_tab.get();
            fetch_symbols.run(());
        }
    });

    // Jump to symbol function
    let jump_to_symbol = {
        let code = code.clone();
        let cursor_pos = cursor_pos.clone();
        let cursor_coords = cursor_coords.clone();
        let check_error_at_cursor = check_error_at_cursor.clone();
        let show_snack = show_snack.clone();
        
        Callback::new(move |(line, character, name): (u32, u32, String)| {
            let current_code = code.get_untracked();
            let index = pos_to_index(&current_code, line, character);
            cursor_pos.set(index);

            use wasm_bindgen::JsCast;
            if let Some(target) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.query_selector(".editor-pane.active .code-editor").ok().flatten().or_else(|| d.query_selector(".code-editor").ok().flatten()))
            {
                if let Ok(target) = target.dyn_into::<web_sys::HtmlTextAreaElement>() {
                    let _ = target.focus();
                    let _ = target.set_selection_range(index, index);
                    
                    let client_height = target.client_height();
                    let scroll_top = ((line as i32 * 25) - (client_height / 3)).max(0);
                    target.set_scroll_top(scroll_top);
                    
                    if let Some(mirror) = web_sys::window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .get_element_by_id("cursor-mirror")
                    {
                        let text_before = &current_code[..index as usize];
                        mirror.set_text_content(Some(text_before));
                        let span = web_sys::window()
                            .unwrap()
                            .document()
                            .unwrap()
                            .create_element("span")
                            .unwrap();
                        span.set_text_content(Some("|"));
                        let _ = mirror.append_child(&span);
                        let span_el = span.dyn_into::<web_sys::HtmlElement>().unwrap();
                        cursor_coords.set((
                            span_el.offset_left() as f64,
                            span_el.offset_top() as f64 + 20.0,
                        ));
                    }
                    check_error_at_cursor.run((line, character));
                    show_snack.run(format!("Jumped to {}", name));
                }
            }
        })
    };

    // Filter symbols based on search text
    let filtered_symbols = move || {
        let term = filter.get().to_lowercase();
        let syms = symbols.get();
        if term.is_empty() {
            syms
        } else {
            syms.into_iter()
                .filter(|s| s.name.to_lowercase().contains(&term))
                .collect()
        }
    };

    view! {
        {move || sidebar_open.get().then(|| view! {
            <div class="sidebar-overlay" on:click=move |_| close_sidebar.run(()) />
        })}

        <div class=move || if sidebar_open.get() { "file-tree-panel outline-panel open" } else { "file-tree-panel outline-panel" }>
            // Header
            <div class="git-panel-header">
                <div class="git-panel-header-left">
                    <div class="git-panel-title">"Outline"</div>
                    <div class="git-panel-subtitle">
                        {move || {
                            let count = filtered_symbols().len();
                            format!("{} symbols", count)
                        }}
                    </div>
                </div>
                <div class="git-panel-header-actions">
                    <button
                        class="git-action-btn"
                        title="Refresh Outline"
                        disabled=move || is_loading.get()
                        on:click=move |_| fetch_symbols.run(())
                    >
                        <LucideIcon name="rotate-cw" size="16" />
                    </button>
                    <button
                        class="git-action-btn"
                        title="Close Panel"
                        on:click=move |_| close_sidebar.run(())
                    >
                        <LucideIcon name="x" size="16" />
                    </button>
                </div>
            </div>

            // Search Filter input
            <div style="padding: 8px 12px; border-bottom: 1px solid var(--border, #2d2d2d);">
                <div style="position: relative; display: flex; align-items: center; background: var(--input-bg, #1e1e1e); border: 1px solid var(--input-border, #3c3c3c); border-radius: 4px; padding: 4px 8px;">
                    <span style="opacity: 0.5; margin-right: 6px; display: flex; align-items: center;">
                        <LucideIcon name="search" size="14" />
                    </span>
                    <input
                        type="text"
                        placeholder="Filter symbols..."
                        style="background: transparent; border: none; color: var(--fg, #d4d4d4); font-size: 13px; width: 100%; outline: none;"
                        prop:value=filter
                        on:input=move |ev| filter.set(event_target_value(&ev))
                    />
                    {move || {
                        if !filter.get().is_empty() {
                            view! {
                                <button
                                    on:click=move |_| filter.set(String::new())
                                    style="background: transparent; border: none; color: var(--fg, #d4d4d4); opacity: 0.5; cursor: pointer; display: flex; align-items: center; padding: 0;"
                                >
                                    <LucideIcon name="x" size="14" />
                                </button>
                            }.into_any()
                        } else {
                            view! {}.into_any()
                        }
                    }}
                </div>
            </div>

            // Symbols List
            <div style="flex: 1; overflow-y: auto; padding: 8px 0;">
                {move || {
                    let list = filtered_symbols();
                    if list.is_empty() {
                        view! {
                            <div style="padding: 24px; text-align: center; color: var(--fg, #d4d4d4); opacity: 0.5; font-size: 13px;">
                                {if is_loading.get() {
                                    "Loading symbols..."
                                } else {
                                    "No symbols found in this file"
                                }}
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="outline-list">
                                {list.into_iter().map(|sym| {
                                    let jump = jump_to_symbol.clone();
                                    let kind_name = match sym.kind {
                                        5 => "Class",
                                        6 => "Method",
                                        10 => "Enum",
                                        11 => "Interface/Trait",
                                        12 => "Function",
                                        23 => "Struct",
                                        _ => "Symbol",
                                    };
                                    let icon_name = match sym.kind {
                                        5 => "book",
                                        6 => "edit",
                                        10 => "package",
                                        11 => "globe",
                                        12 => "code",
                                        23 => "square",
                                        _ => "minus",
                                    };
                                    let color = match sym.kind {
                                        5 => "var(--class-color, #4ec9b0)",
                                        6 => "var(--method-color, #dcdcaa)",
                                        10 => "var(--enum-color, #c586c0)",
                                        11 => "var(--interface-color, #4ec9b0)",
                                        12 => "var(--function-color, #dcdcaa)",
                                        23 => "var(--struct-color, #86c5c0)",
                                        _ => "var(--fg, #d4d4d4)",
                                    };

                                    let sym_name = sym.name.clone();
                                    let sym_name_for_view = sym.name.clone();
                                    let container_for_view = sym.container_name.clone();

                                    view! {
                                        <button
                                            class="outline-item"
                                            on:click={
                                                let sym_name = sym_name.clone();
                                                move |_| {
                                                    jump.run((sym.line, sym.character, sym_name.clone()));
                                                }
                                            }
                                            style="display: flex; align-items: center; width: 100%; padding: 6px 12px; background: transparent; border: none; text-align: left; cursor: pointer; transition: background 0.15s ease;"
                                        >
                                            <span style=format!("display: flex; align-items: center; margin-right: 8px; color: {};", color)>
                                                <LucideIcon name=icon_name size="14" />
                                            </span>
                                            <div style="flex: 1; min-width: 0;">
                                                <div style="font-size: 13px; font-weight: 500; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--fg, #d4d4d4);" title=kind_name>
                                                    {sym_name_for_view}
                                                </div>
                                                {if let Some(container) = container_for_view {
                                                    view! {
                                                        <div style="font-size: 11px; opacity: 0.5; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--fg, #d4d4d4);">
                                                            {container}
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! {}.into_any()
                                                }}
                                            </div>
                                            <span style="font-size: 11px; opacity: 0.4; font-family: monospace; margin-left: 8px; color: var(--fg, #d4d4d4);">
                                                {format!("L{}", sym.line + 1)}
                                            </span>
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
