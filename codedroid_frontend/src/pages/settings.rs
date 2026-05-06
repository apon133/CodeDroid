use leptos::prelude::*;
use web_sys::Event;
use crate::models::Settings;
use crate::store;
use crate::components::app_bar::AppBar;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let settings: RwSignal<Settings> = RwSignal::new(store::load_settings());

    // Save whenever settings change
    let save = move || {
        store::save_settings(&settings.get_untracked());
    };

    view! {
        <div>
            <AppBar title="Settings".to_string() back=true />

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
                    <div class="settings-section-title">"About"</div>
                    <div style="color:var(--text2);font-size:13px;line-height:1.8">
                        <p>"🦀 "<strong>"CodeDroid IDE"</strong>" — Built with Rust + Leptos"</p>
                        <p>"Backend: Axum API running on "<code style="color:var(--accent)">"http://localhost:3000"</code></p>
                        <p>"Frontend: Leptos (WASM) compiled with Trunk"</p>
                    </div>
                </div>
            </div>
        </div>
    }
}
