use leptos::prelude::*;

#[component]
pub fn ContextMenu(
    show_menu: RwSignal<bool>,
    menu_coords: RwSignal<(f64, f64)>,
    trigger_definition: Callback<()>,
    trigger_references: Callback<()>,
    format_code: Callback<()>,
    save_document: Callback<()>,
    show_deps: RwSignal<bool>,
) -> impl IntoView {
    view! {
        {move || show_menu.get().then(|| {
            // Get screen dimensions and adjust coordinates so the menu stays inside the viewport
            let coords = menu_coords.get();
            let (adjusted_x, adjusted_y) = {
                if let Some(window) = web_sys::window() {
                    let win_width = window.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1000.0);
                    let win_height = window.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(800.0);

                    // Approximate size of our context menu
                    let menu_width = 220.0;
                    let menu_height = 200.0;

                    let mut x = coords.0;
                    let mut y = coords.1;

                    if x + menu_width > win_width {
                        x = (win_width - menu_width - 10.0).max(10.0);
                    }
                    if y + menu_height > win_height {
                        y = (win_height - menu_height - 10.0).max(10.0);
                    }
                    (x, y)
                } else {
                    coords
                }
            };

            view! {
                <div class="context-menu-backdrop"
                    on:mousedown=move |e| {
                        e.stop_propagation();
                        show_menu.set(false);
                    }
                    on:touchstart=move |e| {
                        e.stop_propagation();
                        show_menu.set(false);
                    }
                    on:contextmenu=move |e| {
                        e.prevent_default();
                        show_menu.set(false);
                    }
                />
                <div
                    class="context-menu-floating"
                    style=format!("left: {}px; top: {}px;", adjusted_x, adjusted_y)
                    on:mousedown=move |e| {
                        // Prevent textarea focus loss when context menu is interacted with
                        e.prevent_default();
                    }
                >
                    <button class="context-menu-item"
                        on:click=move |e: web_sys::MouseEvent| {
                            e.stop_propagation();
                            show_menu.set(false);
                            trigger_definition.run(());
                        }
                    >
                        <span class="context-menu-label">"Go to Definition"</span>
                        <span class="context-menu-shortcut">"F12"</span>
                    </button>
                    <button class="context-menu-item"
                        on:click=move |e: web_sys::MouseEvent| {
                            e.stop_propagation();
                            show_menu.set(false);
                            trigger_references.run(());
                        }
                    >
                        <span class="context-menu-label">"Find References"</span>
                        <span class="context-menu-shortcut">"Shift+F12"</span>
                    </button>
                    <div class="context-menu-divider" />
                    <button class="context-menu-item"
                        on:click=move |e: web_sys::MouseEvent| {
                            e.stop_propagation();
                            show_menu.set(false);
                            format_code.run(());
                        }
                    >
                        <span class="context-menu-label">"Format Document"</span>
                        <span class="context-menu-shortcut">"⌥⇧F"</span>
                    </button>
                    <button class="context-menu-item"
                        on:click=move |e: web_sys::MouseEvent| {
                            e.stop_propagation();
                            show_menu.set(false);
                            save_document.run(());
                        }
                    >
                        <span class="context-menu-label">"Save Document"</span>
                        <span class="context-menu-shortcut">"⌘S"</span>
                    </button>
                    <div class="context-menu-divider" />
                    <button class="context-menu-item"
                        on:click=move |e: web_sys::MouseEvent| {
                            e.stop_propagation();
                            show_menu.set(false);
                            show_deps.set(true);
                        }
                    >
                        <span class="context-menu-label">"Add Dependency"</span>
                        <span class="context-menu-shortcut">""</span>
                    </button>
                </div>
            }
        })}
    }
}
