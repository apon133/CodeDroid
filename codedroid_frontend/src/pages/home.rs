use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use web_sys::MouseEvent;
use uuid::Uuid;

use crate::models::{Project, lang_icon, lang_color};
use crate::store;
use crate::components::app_bar::AppBar;
use crate::components::snackbar::Snackbar;
use crate::components::new_project_modal::{NewProjectModal, NewProjectResult};
use crate::components::icon::LucideIcon;

fn default_files(lang: &str, framework: &str, name: &str) -> Vec<(String, String)> {
    match lang {
        "rust" => vec![
            ("src/main.rs".into(), format!("fn main() {{\n    println!(\"Hello, Rust!\");\n}}")),
            ("Cargo.toml".into(), format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]")),
        ],
        "go" => vec![
            ("main.go".into(), format!("package main\n\nimport \"fmt\"\n\nfunc main() {{\n    fmt.Println(\"Hello, Go!\")\n}}")),
            ("go.mod".into(), format!("module {name}\n\ngo 1.21")),
        ],
        "python" => vec![
            ("main.py".into(), "print(\"Hello, Python!\")".into()),
            ("requirements.txt".into(), String::new()),
        ],
        "dart" => vec![
            ("main.dart".into(), "void main() {\n  print(\"Hello, Dart!\");\n}".into()),
        ],
        "java" => vec![
            ("Main.java".into(), "public class Main {\n    public static void main(String[] args) {\n        System.out.println(\"Hello, Java!\");\n    }\n}".into()),
        ],
        "c" => vec![
            ("main.c".into(), "#include <stdio.h>\n\nint main() {\n    printf(\"Hello, C!\\n\");\n    return 0;\n}".into()),
        ],
        "cpp" => vec![
            ("main.cpp".into(), "#include <iostream>\n\nint main() {\n    std::cout << \"Hello, C++!\" << std::endl;\n    return 0;\n}".into()),
        ],
        "csharp" => vec![
            ("Program.cs".into(), "Console.WriteLine(\"Hello, C#!\");".into()),
            (format!("{name}.csproj"), "<Project Sdk=\"Microsoft.NET.Sdk\">\n  <PropertyGroup>\n    <OutputType>Exe</OutputType>\n    <TargetFramework>net8.0</TargetFramework>\n  </PropertyGroup>\n</Project>".into()),
        ],
        "kotlin" => vec![
            ("main.kt".into(), "fun main() {\n    println(\"Hello, Kotlin!\")\n}".into()),
        ],
        "swift" => vec![
            ("main.swift".into(), "print(\"Hello, Swift!\")".into()),
        ],
        "ruby" => vec![
            ("main.rb".into(), "puts \"Hello, Ruby!\"".into()),
            ("Gemfile".into(), "source \"https://rubygems.org\"".into()),
        ],
        "javascript" | "typescript" => {
            let ext = if lang == "typescript" { "ts" } else { "js" };
            match framework {
                "none" | "" => vec![
                    (format!("main.{ext}"), format!("console.log(\"Hello, {}!\");", lang.to_uppercase())),
                ],
                "vanilla" => vec![
                    ("index.html".into(), format!("<!DOCTYPE html>\n<html>\n<head><title>{name}</title><link rel=\"stylesheet\" href=\"style.css\"></head>\n<body>\n  <div id=\"app\"></div>\n  <script type=\"module\" src=\"/main.{ext}\"></script>\n</body>\n</html>")),
                    (format!("main.{ext}"), format!("document.getElementById('app').innerHTML = '<h1>Hello {name}!</h1>';")),
                    ("style.css".into(), "body { font-family: sans-serif; display:flex; justify-content:center; align-items:center; height:100vh; margin:0; background:#f0f0f0; }".into()),
                    ("package.json".into(), format!("{{\n  \"name\": \"{name}\",\n  \"type\": \"module\",\n  \"scripts\": {{ \"dev\": \"vite --host 0.0.0.0 --port 0\" }},\n  \"devDependencies\": {{ \"vite\": \"latest\" }}\n}}")),
                ],
                "react" => vec![
                    ("index.html".into(), "<!DOCTYPE html><html><body><div id=\"root\"></div><script type=\"module\" src=\"/src/main.jsx\"></script></body></html>".into()),
                    ("src/main.jsx".into(), "import React from 'react';\nimport ReactDOM from 'react-dom/client';\nReactDOM.createRoot(document.getElementById('root')).render(<h1>Hello React!</h1>);".into()),
                    ("package.json".into(), format!("{{\n  \"name\": \"{name}\",\n  \"type\": \"module\",\n  \"scripts\": {{ \"dev\": \"vite --host 0.0.0.0\" }},\n  \"dependencies\": {{ \"react\": \"^18.0.0\", \"react-dom\": \"^18.0.0\" }},\n  \"devDependencies\": {{ \"vite\": \"^5.0.0\", \"@vitejs/plugin-react\": \"^4.0.0\" }}\n}}")),
                ],
                "vue" => vec![
                    ("src/App.vue".into(), "<template><h1>Hello Vue!</h1></template>".into()),
                    ("src/main.js".into(), "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');".into()),
                    ("package.json".into(), format!("{{\n  \"name\": \"{name}\",\n  \"type\": \"module\",\n  \"scripts\": {{ \"dev\": \"vite --host 0.0.0.0\" }},\n  \"dependencies\": {{ \"vue\": \"^3.0.0\" }},\n  \"devDependencies\": {{ \"vite\": \"^5.0.0\", \"@vitejs/plugin-vue\": \"^5.0.0\" }}\n}}")),
                ],
                "svelte" => vec![
                    ("index.html".into(), "<!DOCTYPE html>\n<html>\n<head>\n  <meta charset=\"UTF-8\" />\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />\n  <title>Svelte App</title>\n</head>\n<body>\n  <div id=\"app\"></div>\n  <script type=\"module\" src=\"/src/main.js\"></script>\n</body>\n</html>".into()),
                    ("src/main.js".into(), "import App from './App.svelte';\n\nconst app = new App({\n  target: document.getElementById('app'),\n});\n\nexport default app;".into()),
                    ("src/App.svelte".into(), "<script>\n  let name = 'Svelte';\n</script>\n\n<main>\n  <h1>Hello {name}!</h1>\n  <p>Welcome to your CodeDroid Svelte project.</p>\n</main>\n\n<style>\n  main {\n    text-align: center;\n    padding: 1em;\n    font-family: sans-serif;\n  }\n  h1 {\n    color: #ff3e00;\n    font-size: 2.5rem;\n  }\n</style>".into()),
                    ("vite.config.js".into(), "import { defineConfig } from 'vite';\nimport { svelte } from '@sveltejs/vite-plugin-svelte';\n\nexport default defineConfig({\n  plugins: [svelte()],\n});".into()),
                    ("package.json".into(), format!("{{\n  \"name\": \"{name}\",\n  \"type\": \"module\",\n  \"scripts\": {{ \"dev\": \"vite --host 0.0.0.0\" }},\n  \"dependencies\": {{ \"svelte\": \"^4.0.0\" }},\n  \"devDependencies\": {{ \"vite\": \"^5.0.0\", \"@vitejs/plugin-svelte\": \"^3.0.0\" }}\n}}")),
                ],
                "nextjs" => vec![
                    ("pages/index.js".into(), "export default function Home() { return <h1>Hello Next.js!</h1>; }".into()),
                    ("package.json".into(), format!("{{\n  \"name\": \"{name}\",\n  \"scripts\": {{ \"dev\": \"next dev -H 0.0.0.0\" }},\n  \"dependencies\": {{ \"next\": \"latest\", \"react\": \"latest\", \"react-dom\": \"latest\" }}\n}}")),
                ],
                _ => vec![(format!("main.{ext}"), format!("console.log('Hello {framework}!');"))],
            }
        }
        _ => vec![("main.txt".into(), "Hello, World!".into())],
    }
}

fn format_project_path(path: &str) -> String {
    if path.starts_with("/Codedroid_Projects") {
        let is_mobile = web_sys::window()
            .and_then(|w| w.navigator().user_agent().ok())
            .map(|ua| {
                let ua_lower = ua.to_lowercase();
                ua_lower.contains("android") || ua_lower.contains("iphone") || ua_lower.contains("ipad")
            })
            .unwrap_or(false);

        if is_mobile {
            format!("phone/download/codedroid{}", &path["/Codedroid_Projects".len()..])
        } else {
            path.to_string()
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

    // Show snackbar for 3s then hide
    let show_snack = move |msg: &str| {
        let msg = msg.to_string();
        snack_msg.set(Some(msg));
        let snack = snack_msg;
        gloo_timers::callback::Timeout::new(3000, move || { snack.set(None); }).forget();
    };

    let on_create = {
        let navigate = navigate.clone();
        Callback::new(move |result: NewProjectResult| {
            let path = format!("/Codedroid_Projects/{}", result.name);
            let project = Project {
                id: Uuid::new_v4().to_string(),
                name: result.name.clone(),
                language: result.lang.clone(),
                path: path.clone(),
                created_at: js_sys::Date::now() as u64,
            };

            // Store default files in localStorage
            let files = default_files(&result.lang, &result.framework, &result.name);
            for (filename, content) in &files {
                let key = store::file_key(&project.id, filename);
                store::save_file(&key, content);
            }

            let pid = project.id.clone();
            store::add_project(&projects, project);
            show_modal.set(false);

            let nav = navigate.clone();
            nav(&format!("/editor/{}", pid), Default::default());
        })
    };

    let on_cancel = Callback::new(move |()| show_modal.set(false));

    view! {
        <div>
            <AppBar title="CodeDroid".to_string()>
                <button class="btn btn-icon" title="Copy API Code"
                    on:click=move |_| {
                        let window = web_sys::window().unwrap();
                        let _ = window.navigator().clipboard().write_text(
                            "curl https://github.com/your/codedroid_api"
                        );
                        show_snack("API code copied!");
                    }
                >
                    <LucideIcon name="copy" size="20" />
                </button>
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
                                    let pid2 = p.id.clone();
                                    let nav = navigate.clone();
                                    let (color, bg) = lang_color(&p.language);
                                    let display_path = format_project_path(&p.path);
                                    let lang_name = p.language.clone();
                                    let lang_name_alt = p.language.clone();
                                    view! {
                                        <div class="project-card"
                                            on:click=move |_| nav(&format!("/editor/{}", pid), Default::default())
                                        >
                                            <div class="project-icon">
                                                <img src=lang_icon(&lang_name) class="lang-icon-img" alt=lang_name_alt />
                                            </div>
                                            <div class="project-info">
                                                <div class="project-name">{p.name.clone()}</div>
                                                <div class="project-path">{display_path.clone()}</div>
                                            </div>
                                            <span class="lang-badge" style=format!("color:{color};border-color:{color};background:{bg}")>
                                                {p.language.to_uppercase()}
                                            </span>
                                            <button class="btn btn-icon"
                                                style="color:#ff453a;font-size:16px;display:flex;align-items:center;justify-content:center"
                                                title="Delete"
                                                on:click=move |e: MouseEvent| {
                                                    e.stop_propagation();
                                                    store::delete_project(&projects, &pid2);
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

            <button class="fab" title="New Project" on:click=move |_| show_modal.set(true)>
                <LucideIcon name="plus" size="24" />
            </button>

            {move || show_modal.get().then(|| view! {
                <NewProjectModal on_create=on_create on_cancel=on_cancel />
            })}

            <Snackbar message=snack_msg.read_only() />
        </div>
    }
}
