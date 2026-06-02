use crate::components::app_bar::AppBar;
use crate::components::icon::LucideIcon;
use crate::models::Settings;
use crate::store;
use leptos::prelude::*;
use web_sys::Event;

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
                    <div class="settings-section-title">"🤖 AI Assistant Settings"</div>

                    <div class="setting-row">
                        <div>
                            <div class="setting-label">"Provider"</div>
                            <div class="setting-desc">"Select OpenRouter or local LM Studio"</div>
                        </div>
                        <select class="input"
                            prop:value=move || settings.get().ai_provider.clone()
                            on:change=move |e: Event| {
                                let val = event_target_value(&e);
                                settings.update(|s| {
                                    s.ai_provider = val.clone();
                                    if val == "openrouter" {
                                        s.ai_endpoint = "https://openrouter.ai/api/v1".to_string();
                                        s.ai_model = "meta-llama/llama-3-8b-instruct:free".to_string();
                                    } else if val == "lm-studio" {
                                        s.ai_endpoint = "http://localhost:1234/v1".to_string();
                                        s.ai_model = "meta-llama-3-8b-instruct".to_string();
                                    }
                                });
                                save();
                            }
                        >
                            <option value="openrouter">"OpenRouter"</option>
                            <option value="lm-studio">"LM Studio (Local)"</option>
                        </select>
                    </div>

                    {move || (settings.get().ai_provider == "openrouter").then(|| view! {
                        <div class="setting-row" style="flex-direction:column;align-items:flex-start;gap:10px">
                            <div style="width: 100%">
                                <div class="setting-label">"API Key"</div>
                                <div class="setting-desc">"Enter your OpenRouter API key"</div>
                            </div>
                            <input
                                type="password"
                                class="input"
                                style="width:100%;font-size:13px"
                                placeholder="sk-or-v1-..."
                                prop:value=move || settings.get().ai_api_key.clone()
                                on:input=move |e: Event| {
                                    settings.update(|s| s.ai_api_key = event_target_value(&e));
                                    save();
                                }
                            />
                        </div>
                    })}

                    <div class="setting-row" style="flex-direction:column;align-items:flex-start;gap:10px">
                        <div style="width: 100%">
                            <div class="setting-label">"Model Name"</div>
                            <div class="setting-desc">"The name of the LLM model to run"</div>
                        </div>
                        <input
                            type="text"
                            class="input"
                            style="width:100%;font-size:13px"
                            placeholder=move || {
                                if settings.get().ai_provider == "openrouter" {
                                    "meta-llama/llama-3-8b-instruct:free"
                                } else {
                                    "meta-llama-3-8b-instruct"
                                }
                            }
                            prop:value=move || settings.get().ai_model.clone()
                            on:input=move |e: Event| {
                                settings.update(|s| s.ai_model = event_target_value(&e));
                                save();
                            }
                        />
                    </div>

                    <div class="setting-row" style="flex-direction:column;align-items:flex-start;gap:10px">
                        <div style="width: 100%">
                            <div class="setting-label">"Endpoint / Base URL"</div>
                            <div class="setting-desc">"API Endpoint URL for LLM requests"</div>
                        </div>
                        <input
                            type="url"
                            class="input"
                            style="width:100%;font-size:13px"
                            prop:value=move || settings.get().ai_endpoint.clone()
                            on:input=move |e: Event| {
                                settings.update(|s| s.ai_endpoint = event_target_value(&e));
                                save();
                            }
                        />
                        {move || (settings.get().ai_provider == "lm-studio" && 
                                  (settings.get().ai_endpoint.contains("localhost") || settings.get().ai_endpoint.contains("127.0.0.1")))
                            .then(|| {
                                let api_url = settings.get().api_url;
                                let mut ip_suggestion = "http://<YOUR_PC_IP>:1234/v1".to_string();
                                if !api_url.is_empty() {
                                    if let Some(host) = api_url.strip_prefix("http://") {
                                        let clean_host = if let Some(slash_idx) = host.find('/') {
                                            &host[..slash_idx]
                                        } else {
                                            host
                                        };
                                        if let Some(colon_idx) = clean_host.find(':') {
                                            let ip = &clean_host[..colon_idx];
                                            if ip != "localhost" && ip != "127.0.0.1" {
                                                ip_suggestion = format!("http://{}:1234/v1", ip);
                                            }
                                        } else if !clean_host.is_empty() && clean_host != "localhost" && clean_host != "127.0.0.1" {
                                            ip_suggestion = format!("http://{}:1234/v1", clean_host);
                                        }
                                    }
                                }
                                
                                view! {
                                    <div style="margin-top:4px; padding:10px 12px; background:rgba(234,179,8,0.1); border:1px solid rgba(234,179,8,0.25); border-radius:6px; font-size:12px; color:#facc15; display:flex; flex-direction:column; gap:6px; line-height:1.4; width:100%">
                                        <div style="display:flex; align-items:center; gap:6px; font-weight:600">
                                            <span>"📱 Mobile Phone Access Tip"</span>
                                        </div>
                                        <div>
                                            "If you are accessing CodeDroid from your mobile phone, "
                                            <strong style="color:#fff">"localhost"</strong>
                                            " refers to the phone itself. Change the endpoint above to: "
                                        </div>
                                        <div>
                                            <code style="background:rgba(0,0,0,0.2); padding:2px 6px; border-radius:4px; font-family:monospace; color:#fff; word-break:break-all">{ip_suggestion}</code>
                                        </div>
                                        <div style="font-size:11px; opacity:0.8">
                                            "Make sure your PC and mobile phone are on the same Wi-Fi network, and that LM Studio's Network Connection is enabled (binding to 0.0.0.0)."
                                        </div>
                                    </div>
                                }
                            })
                        }
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
                    <div class="settings-section-title">"Community"</div>

                    <a href="https://discord.gg/5srCEqsht" target="_blank" rel="noopener noreferrer" style="text-decoration:none;display:block;color:inherit">
                        <div class="setting-row" style="cursor:pointer">
                            <div style="display:flex;align-items:center;gap:12px">
                                <div style="color:#5865F2;display:flex;align-items:center">
                                    <LucideIcon name="discord" size="20" />
                                </div>
                                <div>
                                    <div class="setting-label">"Discord Server"</div>
                                    <div class="setting-desc">"Join our Discord to chat, get support, and share feedback."</div>
                                </div>
                            </div>
                            <div style="color:var(--accent);display:flex;align-items:center">
                                <LucideIcon name="chevron-right" size="20" />
                            </div>
                        </div>
                    </a>

                    <a href="https://t.me/codedroid133" target="_blank" rel="noopener noreferrer" style="text-decoration:none;display:block;color:inherit">
                        <div class="setting-row" style="cursor:pointer">
                            <div style="display:flex;align-items:center;gap:12px">
                                <div style="color:#24A1DE;display:flex;align-items:center">
                                    <LucideIcon name="telegram" size="20" />
                                </div>
                                <div>
                                    <div class="setting-label">"Telegram Channel"</div>
                                    <div class="setting-desc">"Subscribe to our Telegram channel for the latest news and updates."</div>
                                </div>
                            </div>
                            <div style="color:var(--accent);display:flex;align-items:center">
                                <LucideIcon name="chevron-right" size="20" />
                            </div>
                        </div>
                    </a>

                    <a href="https://www.youtube.com/@CodeDroidYT" target="_blank" rel="noopener noreferrer" style="text-decoration:none;display:block;color:inherit">
                        <div class="setting-row" style="cursor:pointer">
                            <div style="display:flex;align-items:center;gap:12px">
                                <div style="color:#FF0000;display:flex;align-items:center">
                                    <LucideIcon name="youtube" size="20" />
                                </div>
                                <div>
                                    <div class="setting-label">"YouTube Channel"</div>
                                    <div class="setting-desc">"Watch video tutorials, setup guides, and features overview."</div>
                                </div>
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
