use leptos::prelude::*;
use web_sys::Event;
use crate::models::Settings;
use crate::store;
use crate::components::app_bar::AppBar;
use crate::components::icon::LucideIcon;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let settings: RwSignal<Settings> = RwSignal::new(store::load_settings());

    // Save whenever settings change
    let save = move || {
        store::save_settings(&settings.get_untracked());
    };

    view! {
        <div>
            <AppBar title="Settings".to_string() back=true>
                <a href="/docs" style="text-decoration:none">
                    <button class="btn btn-icon" title="Documentation">
                        <LucideIcon name="book" size="20" />
                    </button>
                </a>
            </AppBar>

            <div class="settings">
                <div class="settings-section">
                    <div class="settings-section-title">"Editor Appearance"</div>

                    <div class="setting-row">
                        <div>
                            <div class="setting-label">"Font Size"</div>
                            <div class="setting-desc">{move || format!("{} px", settings.get().font_size as u32)}</div>
                        </div>
                        <div class="slider-control">
                            <span style="font-size:12px;color:var(--text2)">"8"</span>
                            <input type="range" class="range-input" min="8" max="32" step="1"
                                prop:value=move || settings.get().font_size.to_string()
                                on:input=move |e: Event| {
                                    if let Ok(v) = event_target_value(&e).parse::<f32>() {
                                        settings.update(|s| s.font_size = v);
                                        save();
                                    }
                                }
                            />
                            <span style="font-size:12px;color:var(--text2)">"32"</span>
                        </div>
                    </div>

                    <div class="setting-row">
                        <div>
                            <div class="setting-label">"Font Family"</div>
                            <div class="setting-desc">{move || settings.get().font_family.clone()}</div>
                        </div>
                        <select class="input"
                            prop:value=move || settings.get().font_family.clone()
                            on:change=move |e: Event| {
                                settings.update(|s| s.font_family = event_target_value(&e));
                                save();
                            }
                        >
                            <option value="FiraCode">"Fira Code"</option>
                            <option value="RobotoMono">"Roboto Mono"</option>
                            <option value="SourceCodePro">"Source Code Pro"</option>
                            <option value="Monospace">"Monospace"</option>
                        </select>
                    </div>

                    <div class="setting-row">
                        <div>
                            <div class="setting-label">"Show Line Numbers"</div>
                        </div>
                        <label class="toggle">
                            <input type="checkbox"
                                prop:checked=move || settings.get().show_line_numbers
                                on:change=move |e: Event| {
                                    let checked = event_target_checked(&e);
                                    settings.update(|s| s.show_line_numbers = checked);
                                    save();
                                }
                            />
                            <span class="toggle-slider"></span>
                        </label>
                    </div>

                    <div class="setting-row">
                        <div>
                            <div class="setting-label">"Word Wrap"</div>
                        </div>
                        <label class="toggle">
                            <input type="checkbox"
                                prop:checked=move || settings.get().word_wrap
                                on:change=move |e: Event| {
                                    let checked = event_target_checked(&e);
                                    settings.update(|s| s.word_wrap = checked);
                                    save();
                                }
                            />
                            <span class="toggle-slider"></span>
                        </label>
                    </div>
                </div>

                <div class="settings-section">
                    <div class="settings-section-title">"Behavior"</div>

                    <div class="setting-row">
                        <div>
                            <div class="setting-label">"Auto-save"</div>
                            <div class="setting-desc">"Save files automatically when content changes"</div>
                        </div>
                        <label class="toggle">
                            <input type="checkbox"
                                prop:checked=move || settings.get().auto_save
                                on:change=move |e: Event| {
                                    let checked = event_target_checked(&e);
                                    settings.update(|s| s.auto_save = checked);
                                    save();
                                }
                            />
                            <span class="toggle-slider"></span>
                        </label>
                    </div>

                    <div class="setting-row">
                        <div>
                            <div class="setting-label">"Tab Size"</div>
                            <div class="setting-desc">{move || format!("{} spaces", settings.get().tab_size)}</div>
                        </div>
                        <select class="input"
                            prop:value=move || settings.get().tab_size.to_string()
                            on:change=move |e: Event| {
                                if let Ok(v) = event_target_value(&e).parse::<usize>() {
                                    settings.update(|s| s.tab_size = v);
                                    save();
                                }
                            }
                        >
                            <option value="2">"2 spaces"</option>
                            <option value="4">"4 spaces"</option>
                            <option value="8">"8 spaces"</option>
                        </select>
                    </div>
                </div>

                <div class="settings-section">
                    <div class="settings-section-title">"🌐 Backend Server"</div>

                    <div class="setting-row" style="flex-direction:column;align-items:flex-start;gap:10px">
                        <div>
                            <div class="setting-label">"API Server URL"</div>
                            <div class="setting-desc">"Set a custom backend URL to use a remote server (e.g. Android on local WiFi). Leave empty to use default."</div>
                        </div>
                        <div style="display:flex;gap:8px;width:100%;align-items:center">
                            <input
                                type="url"
                                class="input"
                                style="flex:1;font-size:13px"
                                placeholder="http://localhost:3000"
                                prop:value=move || settings.get().api_url.clone()
                                on:input=move |e: Event| {
                                    settings.update(|s| s.api_url = event_target_value(&e));
                                    save();
                                }
                            />
                            <button
                                class="btn btn-secondary"
                                style="font-size:12px;padding:6px 12px;white-space:nowrap;display:inline-flex;align-items:center;gap:6px"
                                on:click=move |_| {
                                    let url = settings.get_untracked().api_url.clone();
                                    let url = if url.trim().is_empty() {
                                        crate::api::DEFAULT_API_URL.to_string()
                                    } else {
                                        url
                                    };
                                    leptos::task::spawn_local(async move {
                                        let test_url = format!("{}/ping", url);
                                        let win = web_sys::window().unwrap();
                                        match gloo_net::http::Request::get(&test_url).send().await {
                                            Ok(r) if r.ok() => {
                                                let _ = win.alert_with_message("✅ Connected! Backend is reachable.");
                                            }
                                            _ => {
                                                let _ = win.alert_with_message("❌ Cannot reach backend. Check URL and make sure the server is running.");
                                            }
                                        }
                                    });
                                }
                            >
                                <LucideIcon name="globe" size="14" />
                                "Test Connection"
                            </button>
                        </div>
                        <div style="font-size:11px;color:var(--text2)">
                            "Example: " <code style="color:var(--accent)">"http://192.168.1.100:3000"</code>
                            " (Android phone IP on same WiFi)"
                        </div>
                    </div>
                </div>

                <div class="settings-section">
                    <div class="settings-section-title">"Documentation"</div>
                    <a href="/docs" style="text-decoration:none;display:block;color:inherit">
                        <div class="setting-row" style="cursor:pointer">
                            <div>
                                <div class="setting-label">"View Workspace Documentation"</div>
                                <div class="setting-desc">"Preview README.md, setup guides, and project references."</div>
                            </div>
                            <div style="color:var(--accent);display:flex;align-items:center">
                                <LucideIcon name="chevron-right" size="20" />
                            </div>
                        </div>
                    </a>
                </div>

                <div class="settings-section">
                    <div class="settings-section-title">"About"</div>
                    <div style="color:var(--text2);font-size:13px;line-height:1.8">
                        <p>"🦀 "<strong>"CodeDroid IDE"</strong>" — Built with Rust + Leptos"</p>
                        <p>"Active backend: "<code style="color:var(--accent)">{move || {
                            let url = settings.get().api_url;
                            if url.trim().is_empty() { crate::api::DEFAULT_API_URL.to_string() } else { url }
                        }}</code></p>
                        <p>"Frontend: Leptos (WASM) compiled with Trunk"</p>
                    </div>
                </div>
            </div>
        </div>
    }
}
