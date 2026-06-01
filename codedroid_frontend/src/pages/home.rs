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
            ("pubspec.yaml".into(), "name: project\ndescription: A new Dart project.\nversion: 1.0.0\nenvironment:\n  sdk: '>=2.17.0 <4.0.0'\ndependencies:\n".into()),
        ],
        "java" => vec![
            ("Main.java".into(), "public class Main {\n    public static void main(String[] args) {\n        System.out.println(\"Hello, Java!\");\n    }\n}".into()),
            ("pom.xml".into(), r#"<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.project</groupId>
    <artifactId>project</artifactId>
    <version>1.0-SNAPSHOT</version>
    <dependencies>
    </dependencies>
</project>"#.into()),
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
            ("pom.xml".into(), r#"<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.project</groupId>
    <artifactId>project</artifactId>
    <version>1.0-SNAPSHOT</version>
    <dependencies>
    </dependencies>
</project>"#.into()),
        ],
        "swift" => vec![
            ("main.swift".into(), "print(\"Hello, Swift!\")".into()),
            ("Package.swift".into(), "// swift-tools-version: 5.9\nimport PackageDescription\n\nlet package = Package(\n    name: \"Project\",\n    targets: [.executableTarget(name: \"Project\")]\n)".into()),
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
                    ("package.json".into(), format!("{{\n  \"name\": \"{name}\",\n  \"version\": \"1.0.0\",\n  \"main\": \"main.{ext}\",\n  \"dependencies\": {{}}\n}}")),
                ],
                "vanilla" => vec![
                    ("index.html".into(), format!("<!DOCTYPE html>\n<html>\n<head><title>{name}</title><link rel=\"stylesheet\" href=\"style.css\"></head>\n<body>\n  <div id=\"app\"></div>\n  <script type=\"module\" src=\"/main.{ext}\"></script>\n</body>\n</html>")),
                    (format!("main.{ext}"), format!("document.getElementById('app').innerHTML = '<h1>Hello {name}!</h1>';")),
                    ("style.css".into(), "body { font-family: sans-serif; display:flex; justify-content:center; align-items:center; height:100vh; margin:0; background:#f0f0f0; }".into()),
                    ("package.json".into(), format!("{{\n  \"name\": \"{name}\",\n  \"type\": \"module\",\n  \"scripts\": {{ \"dev\": \"vite --host 0.0.0.0 --port 0\" }},\n  \"devDependencies\": {{ \"vite\": \"latest\" }}\n}}")),
                ],
                "react" => vec![
                    ("index.html".into(), "<!DOCTYPE html>\n<html>\n<head>\n  <meta charset=\"UTF-8\" />\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />\n  <title>React App</title>\n</head>\n<body>\n  <div id=\"root\"></div>\n  <script type=\"module\" src=\"/src/main.jsx\"></script>\n</body>\n</html>".into()),
                    ("src/main.jsx".into(), "import React from 'react';\nimport ReactDOM from 'react-dom/client';\n\nconst App = () => (\n  <div style={{ textAlign: 'center', fontFamily: 'sans-serif', padding: '1em' }}>\n    <h1 style={{ color: '#61dafb' }}>Hello React!</h1>\n    <p>Welcome to your CodeDroid React project.</p>\n  </div>\n);\n\nReactDOM.createRoot(document.getElementById('root')).render(<App />);".into()),
                    ("vite.config.js".into(), "import { defineConfig } from 'vite';\nimport react from '@vitejs/plugin-react';\n\nexport default defineConfig({\n  plugins: [react()],\n});".into()),
                    ("package.json".into(), format!("{{\n  \"name\": \"{name}\",\n  \"type\": \"module\",\n  \"scripts\": {{ \"dev\": \"vite --host 0.0.0.0\" }},\n  \"dependencies\": {{ \"react\": \"^18.0.0\", \"react-dom\": \"^18.0.0\" }},\n  \"devDependencies\": {{ \"vite\": \"^5.0.0\", \"@vitejs/plugin-react\": \"^4.0.0\" }}\n}}")),
                ],
                "vue" => vec![
                    ("index.html".into(), "<!DOCTYPE html>\n<html>\n<head>\n  <meta charset=\"UTF-8\" />\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />\n  <title>Vue App</title>\n</head>\n<body>\n  <div id=\"app\"></div>\n  <script type=\"module\" src=\"/src/main.js\"></script>\n</body>\n</html>".into()),
                    ("src/App.vue".into(), "<template>\n  <main>\n    <h1>Hello Vue!</h1>\n    <p>Welcome to your CodeDroid Vue project.</p>\n  </main>\n</template>\n\n<style>\nmain {\n  text-align: center;\n  padding: 1em;\n  font-family: sans-serif;\n}\nh1 {\n  color: #42b983;\n}\n</style>".into()),
                    ("src/main.js".into(), "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');".into()),
                    ("vite.config.js".into(), "import { defineConfig } from 'vite';\nimport vue from '@vitejs/plugin-vue';\n\nexport default defineConfig({\n  plugins: [vue()],\n});".into()),
                    ("package.json".into(), format!("{{\n  \"name\": \"{name}\",\n  \"type\": \"module\",\n  \"scripts\": {{ \"dev\": \"vite --host 0.0.0.0\" }},\n  \"dependencies\": {{ \"vue\": \"^3.4.0\" }},\n  \"devDependencies\": {{ \"vite\": \"^5.0.0\", \"@vitejs/plugin-vue\": \"^5.0.0\" }}\n}}")),
                ],
                "svelte" => vec![
                    ("index.html".into(), "<!DOCTYPE html>\n<html>\n<head>\n  <meta charset=\"UTF-8\" />\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />\n  <title>Svelte App</title>\n</head>\n<body>\n  <div id=\"app\"></div>\n  <script type=\"module\" src=\"/src/main.js\"></script>\n</body>\n</html>".into()),
                    ("src/main.js".into(), "import App from './App.svelte';\n\nconst app = new App({\n  target: document.getElementById('app'),\n});\n\nexport default app;".into()),
                    ("src/App.svelte".into(), "<script>\n  let name = 'Svelte';\n</script>\n\n<main>\n  <h1>Hello {name}!</h1>\n  <p>Welcome to your CodeDroid Svelte project.</p>\n</main>\n\n<style>\n  main {\n    text-align: center;\n    padding: 1em;\n    font-family: sans-serif;\n  }\n  h1 {\n    color: #ff3e00;\n    font-size: 2.5rem;\n  }\n</style>".into()),
                    ("vite.config.js".into(), "import { defineConfig } from 'vite';\nimport { svelte } from '@sveltejs/vite-plugin-svelte';\n\nexport default defineConfig({\n  plugins: [svelte()],\n});".into()),
                    ("package.json".into(), format!("{{\n  \"name\": \"{name}\",\n  \"type\": \"module\",\n  \"scripts\": {{ \"dev\": \"vite --host 0.0.0.0\" }},\n  \"dependencies\": {{ \"svelte\": \"^4.0.0\" }},\n  \"devDependencies\": {{ \"vite\": \"^5.0.0\", \"@sveltejs/vite-plugin-svelte\": \"^3.0.0\" }}\n}}")),
                ],
                "angular" => vec![
                    ("src/index.html".into(), "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"utf-8\">\n  <title>Angular App</title>\n  <base href=\"/\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n</head>\n<body>\n  <app-root></app-root>\n</body>\n</html>".into()),
                    ("src/main.ts".into(), "import { bootstrapApplication } from '@angular/platform-browser';\nimport { Component } from '@angular/core';\n\n@Component({\n  selector: 'app-root',\n  standalone: true,\n  template: `\n    <div style=\"text-align: center; font-family: sans-serif; padding: 1em;\">\n      <h1 style=\"color: #dd0031;\">Hello Angular!</h1>\n      <p>Welcome to your CodeDroid Angular project.</p>\n    </div>\n  `,\n})\nexport class App {}\n\nbootstrapApplication(App).catch((err) => console.error(err));".into()),
                    ("angular.json".into(), format!("{{\n  \"$schema\": \"./node_modules/@angular/cli/lib/config/schema.json\",\n  \"version\": 1,\n  \"newProjectRoot\": \"projects\",\n  \"projects\": {{\n    \"{name}\": {{\n      \"projectType\": \"application\",\n      \"schematics\": {{}},\n      \"root\": \"\",\n      \"sourceRoot\": \"src\",\n      \"prefix\": \"app\",\n      \"architect\": {{\n        \"build\": {{\n          \"builder\": \"@angular-devkit/build-angular:browser\",\n          \"options\": {{\n            \"outputPath\": \"dist/{name}\",\n            \"index\": \"src/index.html\",\n            \"main\": \"src/main.ts\",\n            \"polyfills\": [\"zone.js\"],\n            \"tsConfig\": \"tsconfig.app.json\",\n            \"assets\": [],\n            \"styles\": [],\n            \"scripts\": []\n          }}\n        }},\n        \"serve\": {{\n          \"builder\": \"@angular-devkit/build-angular:dev-server\",\n          \"options\": {{\n            \"buildTarget\": \"{name}:build\"\n          }}\n        }}\n      }}\n    }}\n  }}\n}}")),
                    ("tsconfig.json".into(), "{\n  \"compileOnSave\": false,\n  \"compilerOptions\": {\n    \"outDir\": \"./dist/out-tsc\",\n    \"forceConsistentCasingInFileNames\": true,\n    \"strict\": true,\n    \"noImplicitOverride\": true,\n    \"noPropertyAccessFromIndexSignature\": true,\n    \"noImplicitReturns\": true,\n    \"noFallthroughCasesInSwitch\": true,\n    \"skipLibCheck\": true,\n    \"esModuleInterop\": true,\n    \"experimentalDecorators\": true,\n    \"moduleResolution\": \"node\",\n    \"importHelpers\": true,\n    \"target\": \"ES2022\",\n    \"module\": \"ES2022\",\n    \"useDefineForClassFields\": false,\n    \"lib\": [\"ES2022\", \"dom\"]\n  },\n  \"angularCompilerOptions\": {\n    \"enableI18nLegacyMessageIdFormat\": false,\n    \"strictInjectionParameters\": true,\n    \"strictInputAccessModifiers\": true,\n    \"strictTemplates\": true\n  }\n}".into()),
                    ("tsconfig.app.json".into(), "{\n  \"extends\": \"./tsconfig.json\",\n  \"compilerOptions\": {\n    \"outDir\": \"./out-tsc/app\",\n    \"types\": []\n  },\n  \"files\": [\n    \"src/main.ts\"\n  ],\n  \"include\": [\n    \"src/**/*.d.ts\"\n  ]\n}".into()),
                    ("package.json".into(), format!("{{\n  \"name\": \"{name}\",\n  \"version\": \"0.0.0\",\n  \"scripts\": {{\n    \"dev\": \"ng serve --host 0.0.0.0 --port 4200\"\n  }},\n  \"dependencies\": {{\n    \"@angular/animations\": \"^17.3.0\",\n    \"@angular/common\": \"^17.3.0\",\n    \"@angular/compiler\": \"^17.3.0\",\n    \"@angular/core\": \"^17.3.0\",\n    \"@angular/forms\": \"^17.3.0\",\n    \"@angular/platform-browser\": \"^17.3.0\",\n    \"@angular/platform-browser-dynamic\": \"^17.3.0\",\n    \"@angular/router\": \"^17.3.0\",\n    \"rxjs\": \"~7.8.0\",\n    \"tslib\": \"^2.3.0\",\n    \"zone.js\": \"~0.14.3\"\n  }},\n  \"devDependencies\": {{\n    \"@angular-devkit/build-angular\": \"^17.3.0\",\n    \"@angular/cli\": \"^17.3.0\",\n    \"@angular/compiler-cli\": \"^17.3.0\",\n    \"typescript\": \"~5.4.2\"\n  }}\n}}")),
                ],
                "nextjs" => vec![
                    ("app/layout.jsx".into(), "export default function RootLayout({ children }) {\n  return (\n    <html lang=\"en\">\n      <body style={{ margin: 0, fontFamily: 'sans-serif' }}>{children}</body>\n    </html>\n  );\n}".into()),
                    ("app/page.jsx".into(), "export default function Home() {\n  return (\n    <div style={{ textAlign: 'center', padding: '2em' }}>\n      <h1 style={{ color: '#0070f3' }}>Hello Next.js!</h1>\n      <p>Welcome to your CodeDroid Next.js project using App Router.</p>\n    </div>\n  );\n}".into()),
                    ("package.json".into(), format!("{{\n  \"name\": \"{name}\",\n  \"private\": true,\n  \"scripts\": {{ \"dev\": \"next dev -H 0.0.0.0 -p 3001\" }},\n  \"dependencies\": {{\n    \"next\": \"^14.2.0\",\n    \"react\": \"^18.3.0\",\n    \"react-dom\": \"^18.3.0\"\n  }}\n}}")),
                ],
                "remix" => vec![
                    ("app/root.jsx".into(), "import { Links, Meta, Outlet, Scripts, ScrollRestoration } from '@remix-run/react';\n\nexport default function App() {\n  return (\n    <html lang=\"en\">\n      <head>\n        <meta charSet=\"utf-8\" />\n        <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />\n        <Meta />\n        <Links />\n      </head>\n      <body>\n        <Outlet />\n        <ScrollRestoration />\n        <Scripts />\n      </body>\n    </html>\n  );\n}".into()),
                    ("app/routes/_index.jsx".into(), "export default function Index() {\n  return (\n    <div style={{ textAlign: 'center', fontFamily: 'sans-serif', padding: '1em' }}>\n      <h1 style={{ color: '#319795' }}>Hello Remix!</h1>\n      <p>Welcome to your CodeDroid Remix project.</p>\n    </div>\n  );\n}".into()),
                    ("vite.config.js".into(), "import { vitePlugin as remix } from '@remix-run/dev';\nimport { defineConfig } from 'vite';\n\nexport default defineConfig({\n  plugins: [remix()],\n});".into()),
                    ("package.json".into(), format!("{{\n  \"name\": \"{name}\",\n  \"private\": true,\n  \"type\": \"module\",\n  \"scripts\": {{ \"dev\": \"vite --host 0.0.0.0\" }},\n  \"dependencies\": {{\n    \"@remix-run/node\": \"^2.9.0\",\n    \"@remix-run/react\": \"^2.9.0\",\n    \"@remix-run/serve\": \"^2.9.0\",\n    \"isbot\": \"^4.1.0\",\n    \"react\": \"^18.2.0\",\n    \"react-dom\": \"^18.2.0\"\n  }},\n  \"devDependencies\": {{\n    \"@remix-run/dev\": \"^2.9.0\",\n    \"vite\": \"^5.1.0\"\n  }}\n}}")),
                ],
                _ => vec![(format!("main.{ext}"), format!("console.log('Hello {framework}!');"))],
            }
        }
        _ => vec![("main.txt".into(), "Hello, World!".into())],
    }
}

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

            // Store default files in localStorage (only for non-cloned template projects)
            if result.lang != "auto" {
                let files = default_files(&result.lang, &result.framework, &result.name);
                for (filename, content) in &files {
                    let key = store::file_key(&project.id, filename);
                    store::save_file(&key, content);
                }
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

            <button class="fab" title="New Project" on:click=move |_| show_modal.set(true)>
                <LucideIcon name="plus" size="24" />
            </button>

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
