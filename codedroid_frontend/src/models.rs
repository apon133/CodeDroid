use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub language: String,
    pub path: String,
    pub created_at: u64,
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
        "rust" => "🦀",
        "go" => "🐹",
        "python" => "🐍",
        "javascript" | "typescript" => "⚡",
        "java" => "☕",
        "dart" => "🎯",
        "c" | "cpp" => "⚙️",
        "csharp" => "🔷",
        "kotlin" => "🟣",
        "swift" => "🍎",
        "ruby" => "💎",
        _ => "📄",
    }
}

pub fn lang_color(lang: &str) -> (&'static str, &'static str) {
    // (color, background)
    match lang {
        "rust"       => ("#DEA584", "rgba(222,165,132,.15)"),
        "go"         => ("#00ADD8", "rgba(0,173,216,.15)"),
        "python"     => ("#3776AB", "rgba(55,118,171,.15)"),
        "java"       => ("#007396", "rgba(0,115,150,.15)"),
        "dart"       => ("#0175C2", "rgba(1,117,194,.15)"),
        "javascript" => ("#F7DF1E", "rgba(247,223,30,.15)"),
        "typescript" => ("#3178C6", "rgba(49,120,198,.15)"),
        "kotlin"     => ("#7F52FF", "rgba(127,82,255,.15)"),
        "swift"      => ("#F05138", "rgba(240,81,56,.15)"),
        "c" | "cpp"  => ("#A8B9CC", "rgba(168,185,204,.15)"),
        "csharp"     => ("#239120", "rgba(35,145,32,.15)"),
        "ruby"       => ("#CC342D", "rgba(204,52,45,.15)"),
        _            => ("#007ACC", "rgba(0,122,204,.15)"),
    }
}
