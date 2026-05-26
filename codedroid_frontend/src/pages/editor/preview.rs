use leptos::prelude::*;
use crate::components::icon::LucideIcon;

#[component]
pub fn PreviewPanel(
    preview_url: Signal<Option<String>>,
    refresh_key: RwSignal<u32>,
) -> impl IntoView {
    view! {
        {move || preview_url.get().map(|url| {
            let final_url = move || {
                let k = refresh_key.get();
                if url.contains('?') {
                    format!("{}&refresh={}", url, k)
                } else {
                    format!("{}?refresh={}", url, k)
                }
            };
            view! {
                <>
                <div class="preview-resize-gutter"></div>
                <div class="desktop-preview-panel">
                    <div class="preview-header">
                        <span style="display:inline-flex; align-items:center; gap:6px; color:#fff; font-family:var(--font-ui); font-size:12px; font-weight:600;">
                            <LucideIcon name="globe" size="14" />
                            "Live Web Preview"
                        </span>
                        <button class="btn btn-xs" style="padding:2px 8px; font-size:11px; height:24px; background:var(--bg3); border:1px solid var(--border); border-radius:var(--radius-sm); color:var(--text); cursor:pointer; display:inline-flex; align-items:center; gap:4px;"
                            on:click=move |_| refresh_key.update(|k| *k += 1)>
                            <LucideIcon name="refresh" size="12" /> "Refresh"
                        </button>
                    </div>
                    <iframe class="preview-frame" src=final_url style="flex:1; border:none; background:#fff; width:100%; height:100%;" />
                </div>
                </>
            }
        })}
    }
}

#[component]
pub fn MobilePreviewOverlay(
    show_mobile_full_preview: RwSignal<bool>,
    preview_url: Signal<Option<String>>,
    refresh_key: RwSignal<u32>,
) -> impl IntoView {
    view! {
        {move || show_mobile_full_preview.get().then(|| {
            let url_opt = preview_url.get();
            url_opt.map(|url| {
                let final_url = move || {
                    let k = refresh_key.get();
                    if url.contains('?') {
                        format!("{}&refresh={}", url, k)
                    } else {
                        format!("{}?refresh={}", url, k)
                    }
                };
                view! {
                    <div class="mobile-preview-overlay active">
                        <div class="preview-header">
                            <button class="btn btn-icon" on:click=move |_| show_mobile_full_preview.set(false) title="Back to Code" style="background:transparent; border:none; color:var(--text); cursor:pointer; display:inline-flex; align-items:center; justify-content:center;">
                                <LucideIcon name="arrow-left" size="20" />
                            </button>
                            <span style="font-weight: 600; color: #fff; font-family: var(--font-ui); font-size: 14px;">"Web Preview"</span>
                            <button class="btn btn-xs" style="padding:4px 10px; font-size:11px; height:26px; background:var(--bg3); border:1px solid var(--border); border-radius:var(--radius-sm); color:var(--text); cursor:pointer; display:inline-flex; align-items:center; gap:4px;"
                                on:click=move |_| refresh_key.update(|k| *k += 1)>
                                <LucideIcon name="refresh" size="12" /> "Refresh"
                            </button>
                        </div>
                        <iframe class="preview-frame" src=final_url style="flex:1; border:none; background:#fff; width:100%; height:100%;" />
                    </div>
                }
            })
        })}
    }
}
