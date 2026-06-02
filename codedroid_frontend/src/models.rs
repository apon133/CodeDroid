use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub language: String,
    pub path: String,
    pub created_at: u64,
    #[serde(default)]
    pub framework: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)]
pub struct Snippet {
    pub id: String,
    pub title: String,
    pub language: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub font_size: f32,
    pub font_family: String,
    pub show_line_numbers: bool,
    pub word_wrap: bool,
    pub auto_save: bool,
    pub tab_size: usize,
    pub api_url: String,
    #[serde(default = "default_ai_provider")]
    pub ai_provider: String,
    #[serde(default = "default_ai_key")]
    pub ai_api_key: String,
    #[serde(default = "default_ai_model")]
    pub ai_model: String,
    #[serde(default = "default_ai_endpoint")]
    pub ai_endpoint: String,
}

fn default_ai_provider() -> String {
    "openrouter".to_string()
}
fn default_ai_key() -> String {
    "".to_string()
}
fn default_ai_model() -> String {
    "meta-llama/llama-3-8b-instruct:free".to_string()
}
fn default_ai_endpoint() -> String {
    "https://openrouter.ai/api/v1".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            font_family: "FiraCode".to_string(),
            show_line_numbers: true,
            word_wrap: false,
            auto_save: true,
            tab_size: 4,
            api_url: "http://localhost:3000".to_string(),
            ai_provider: "openrouter".to_string(),
            ai_api_key: "".to_string(),
            ai_model: "meta-llama/llama-3-8b-instruct:free".to_string(),
            ai_endpoint: "https://openrouter.ai/api/v1".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResponse {
    pub output: String,
    pub error: String,
    pub pid: Option<u32>,
    pub url: Option<String>,
}

pub fn lang_icon(lang: &str) -> &'static str {
    match lang {
        "rust" => "/assets/icons/rust.svg",
        "go" => "/assets/icons/go.svg",
        "python" => "/assets/icons/python.svg",
        "javascript" => "/assets/icons/javascript.svg",
        "typescript" => "/assets/icons/typescript.svg",
        "java" => "/assets/icons/java.svg",
        "dart" => "/assets/icons/dart.svg",
        "c" => "/assets/icons/c.svg",
        "cpp" => "/assets/icons/cpp.svg",
        "csharp" => "/assets/icons/csharp.svg",
        "kotlin" => "/assets/icons/kotlin.svg",
        "swift" => "/assets/icons/swift.svg",
        "ruby" => "/assets/icons/ruby.svg",
        _ => "/assets/icons/generic.svg",
    }
}

pub fn lang_color(lang: &str) -> (&'static str, &'static str) {
    // (color, background)
    match lang {
        "rust" => ("#DEA584", "rgba(222,165,132,.15)"),
        "go" => ("#00ADD8", "rgba(0,173,216,.15)"),
        "python" => ("#3776AB", "rgba(55,118,171,.15)"),
        "java" => ("#007396", "rgba(0,115,150,.15)"),
        "dart" => ("#0175C2", "rgba(1,117,194,.15)"),
        "javascript" => ("#F7DF1E", "rgba(247,223,30,.15)"),
        "typescript" => ("#3178C6", "rgba(49,120,198,.15)"),
        "kotlin" => ("#7F52FF", "rgba(127,82,255,.15)"),
        "swift" => ("#F05138", "rgba(240,81,56,.15)"),
        "c" | "cpp" => ("#A8B9CC", "rgba(168,185,204,.15)"),
        "csharp" => ("#239120", "rgba(35,145,32,.15)"),
        "ruby" => ("#CC342D", "rgba(204,52,45,.15)"),
        _ => ("#007ACC", "rgba(0,122,204,.15)"),
    }
}

pub fn project_icon(lang: &str, framework: &str) -> &'static str {
    if !framework.is_empty() && framework != "none" {
        match framework {
            "react" => "/assets/icons/react.svg",
            "vue" => "/assets/icons/vue.svg",
            "svelte" => "/assets/icons/svelte.svg",
            "angular" => "/assets/icons/angular.svg",
            "nextjs" => "/assets/icons/nextjs.svg",
            "remix" => "/assets/icons/react.svg",
            "vanilla" => "/assets/icons/javascript.svg",
            _ => "/assets/icons/generic.svg",
        }
    } else {
        lang_icon(lang)
    }
}

pub fn project_badge_info(lang: &str, framework: &str) -> (String, &'static str, &'static str) {
    if !framework.is_empty() && framework != "none" {
        match framework {
            "react" => ("React".to_string(), "#61DAFB", "rgba(97,218,251,.15)"),
            "vue" => ("Vue".to_string(), "#4FC08D", "rgba(79,192,141,.15)"),
            "svelte" => ("Svelte".to_string(), "#FF3E00", "rgba(255,62,0,.15)"),
            "angular" => ("Angular".to_string(), "#DD0031", "rgba(221,0,49,.15)"),
            "nextjs" => ("Next.js".to_string(), "#FFFFFF", "rgba(255,255,255,.15)"),
            "remix" => ("Remix".to_string(), "#E3F2FD", "rgba(227,242,253,.15)"),
            "vanilla" => ("Vanilla JS".to_string(), "#F7DF1E", "rgba(247,223,30,.15)"),
            _ => (framework.to_uppercase(), "#007ACC", "rgba(0,122,204,.15)"),
        }
    } else {
        let (color, bg) = lang_color(lang);
        let name = match lang {
            "rust" => "Rust",
            "go" => "Go",
            "python" => "Python",
            "java" => "Java",
            "dart" => "Dart",
            "javascript" => "JavaScript",
            "typescript" => "TypeScript",
            "kotlin" => "Kotlin",
            "swift" => "Swift",
            "cpp" => "C++",
            "c" => "C",
            "csharp" => "C#",
            "ruby" => "Ruby",
            _ => lang,
        };
        (name.to_string(), color, bg)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageResponse {
    pub output: String,
    pub error: String,
    pub dependency_file_name: Option<String>,
    pub dependency_file_content: Option<String>,
}


