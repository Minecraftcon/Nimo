use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeparatorStyle {
    Pipe,   // " │ "
    Thin,   // " | "
    Double, // " ║ "
    Dots,   // " ┆ "
    Slash,  // " // "
    None,   // "   "
}

impl SeparatorStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            SeparatorStyle::Pipe => " │ ",
            SeparatorStyle::Thin => " | ",
            SeparatorStyle::Double => " ║ ",
            SeparatorStyle::Dots => " ┆ ",
            SeparatorStyle::Slash => " // ",
            SeparatorStyle::None => "   ",
        }
    }

    pub fn width(&self) -> usize {
        match self {
            SeparatorStyle::Pipe => 3,
            SeparatorStyle::Thin => 3,
            SeparatorStyle::Double => 3,
            SeparatorStyle::Dots => 3,
            SeparatorStyle::Slash => 4,
            SeparatorStyle::None => 3,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            SeparatorStyle::Pipe => SeparatorStyle::Thin,
            SeparatorStyle::Thin => SeparatorStyle::Double,
            SeparatorStyle::Double => SeparatorStyle::Dots,
            SeparatorStyle::Dots => SeparatorStyle::Slash,
            SeparatorStyle::Slash => SeparatorStyle::None,
            SeparatorStyle::None => SeparatorStyle::Pipe,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            SeparatorStyle::Pipe => "pipe",
            SeparatorStyle::Thin => "thin",
            SeparatorStyle::Double => "double",
            SeparatorStyle::Dots => "dots",
            SeparatorStyle::Slash => "slash",
            SeparatorStyle::None => "none",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub blinking_cursor: bool,
    pub line_numbers: bool,
    pub line_numbers_type_only: bool,
    pub separator: bool,
    pub separator_style: SeparatorStyle,
    pub custom_separator: Option<String>,
    pub top_bar: bool,
    pub status_bar: bool,
    pub footer: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            blinking_cursor: true,
            line_numbers: true,
            line_numbers_type_only: true,
            separator: true,
            separator_style: SeparatorStyle::Pipe,
            custom_separator: None,
            top_bar: true,
            status_bar: true,
            footer: true,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let mut config = Config::default();

        // 1. Check current directory .nimorc
        let local_path = Path::new(".nimorc");
        if local_path.exists() {
            if let Ok(content) = fs::read_to_string(local_path) {
                config.parse(&content);
                return config;
            }
        }

        // 2. Check ~/.nimorc
        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(home).join(".nimorc");
            if home_path.exists() {
                if let Ok(content) = fs::read_to_string(home_path) {
                    config.parse(&content);
                    return config;
                }
            }
        }

        config
    }

    pub fn parse(&mut self, content: &str) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim().to_lowercase();
                let val = val.trim();

                match key.as_str() {
                    "blinkingcursor" | "blinking_cursor" => {
                        self.blinking_cursor = parse_bool(val, self.blinking_cursor);
                    }
                    "linenumbers" | "line_numbers" => {
                        self.line_numbers = parse_bool(val, self.line_numbers);
                    }
                    "linenumberstypeonly" | "line_numbers_type_only" => {
                        self.line_numbers_type_only = parse_bool(val, self.line_numbers_type_only);
                    }
                    "separator" => {
                        self.separator = parse_bool(val, self.separator);
                    }
                    "separatorstyle" | "separator_style" => {
                        match val.to_lowercase().as_str() {
                            "pipe" => self.separator_style = SeparatorStyle::Pipe,
                            "thin" => self.separator_style = SeparatorStyle::Thin,
                            "double" => self.separator_style = SeparatorStyle::Double,
                            "dots" => self.separator_style = SeparatorStyle::Dots,
                            "slash" => self.separator_style = SeparatorStyle::Slash,
                            "none" => self.separator_style = SeparatorStyle::None,
                            _ => {}
                        }
                    }
                    "customseparator" | "custom_separator" | "customseperator" | "custom_seperator" => {
                        let unquoted = unquote_str(val);
                        if unquoted.is_empty() {
                            self.custom_separator = None;
                        } else {
                            self.custom_separator = Some(unquoted);
                        }
                    }
                    "topbar" | "top_bar" => {
                        self.top_bar = parse_bool(val, self.top_bar);
                    }
                    "statusbar" | "status_bar" => {
                        self.status_bar = parse_bool(val, self.status_bar);
                    }
                    "footer" => {
                        self.footer = parse_bool(val, self.footer);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn parse_bool(s: &str, default: bool) -> bool {
    match s.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" | "t" => true,
        "false" | "0" | "no" | "off" | "f" => false,
        _ => default,
    }
}

fn unquote_str(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
        || (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}
