use std::fs;
use std::path::{Path, PathBuf};
use crossterm::style::Color;
use crate::theme::Palette;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactFooterMode {
    Auto,   // Adaptive: 1 row on small / mobile / short screens (term_height <= 18), 2 rows on desktop
    Always, // Always 1 compact row
    Never,  // Always 2 rows (classic nano)
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
    pub compact_footer: CompactFooterMode,
    pub highlight_enabled: bool,
    pub highlight_color: Color,
    pub highlight_opacity: f32,
    pub auto_completions: bool,
    pub warp_line: bool,
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
            compact_footer: CompactFooterMode::Auto,
            highlight_enabled: true,
            highlight_color: Palette::MAUVE,
            highlight_opacity: 0.08,
            auto_completions: true,
            warp_line: false,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let mut config = Config::default();

        // 1. Home directory configs (lower priority)
        if let Ok(home) = std::env::var("HOME") {
            let home_dir = PathBuf::from(home);

            let home_nanorc = home_dir.join(".nanorc");
            if home_nanorc.exists() {
                if let Ok(content) = fs::read_to_string(&home_nanorc) {
                    config.parse(&content);
                }
            }

            let home_nimorc = home_dir.join(".nimorc");
            if home_nimorc.exists() {
                if let Ok(content) = fs::read_to_string(&home_nimorc) {
                    config.parse(&content);
                }
            }
        }

        // 2. Current directory configs (higher priority)
        let local_nanorc = Path::new(".nanorc");
        if local_nanorc.exists() {
            if let Ok(content) = fs::read_to_string(local_nanorc) {
                config.parse(&content);
            }
        }

        let local_nimorc = Path::new(".nimorc");
        if local_nimorc.exists() {
            if let Ok(content) = fs::read_to_string(local_nimorc) {
                config.parse(&content);
            }
        }

        config
    }

    pub fn active_line_bg(&self, base: Color) -> Color {
        if !self.highlight_enabled || self.highlight_opacity <= 0.0 {
            return base;
        }

        let (base_r, base_g, base_b) = match base {
            Color::Rgb { r, g, b } => (r as f32, g as f32, b as f32),
            _ => (30.0, 30.0, 46.0),
        };

        let (hl_r, hl_g, hl_b) = match self.highlight_color {
            Color::Rgb { r, g, b } => (r as f32, g as f32, b as f32),
            _ => (203.0, 166.0, 247.0),
        };

        let alpha = self.highlight_opacity.clamp(0.0, 1.0);
        let r = (base_r * (1.0 - alpha) + hl_r * alpha).round().clamp(0.0, 255.0) as u8;
        let g = (base_g * (1.0 - alpha) + hl_g * alpha).round().clamp(0.0, 255.0) as u8;
        let b = (base_b * (1.0 - alpha) + hl_b * alpha).round().clamp(0.0, 255.0) as u8;

        Color::Rgb { r, g, b }
    }

    pub fn parse(&mut self, content: &str) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            // 1. Nanorc directive support: "set <option> [val]" and "unset <option>"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                if parts[0].eq_ignore_ascii_case("set") && parts.len() >= 2 {
                    let opt = parts[1].to_lowercase();
                    let val = parts[2..].join(" ");
                    self.apply_nano_set(&opt, &val);
                    continue;
                } else if parts[0].eq_ignore_ascii_case("unset") && parts.len() >= 2 {
                    let opt = parts[1].to_lowercase();
                    self.apply_nano_unset(&opt);
                    continue;
                }
            }

            // 2. Key = Value or Key: Value support
            if let Some((key, val)) = line.split_once('=').or_else(|| line.split_once(':')) {
                let key = key.trim().to_lowercase();
                let val = val.trim();
                self.apply_key_value(&key, val);
            }
        }
    }

    fn apply_nano_set(&mut self, opt: &str, val: &str) {
        match opt {
            "spotlight" | "linehighlight" | "highlight" => {
                if val.is_empty() {
                    self.highlight_enabled = true;
                } else {
                    self.highlight_enabled = parse_bool(val, true);
                }
            }
            "stripecolor" | "highlightcolor" | "highlight_color" => {
                if let Some(col) = parse_color(val) {
                    self.highlight_color = col;
                }
            }
            "highlightopacity" | "highlight_opacity" | "opacity" => {
                if let Some(op) = parse_opacity(val) {
                    self.highlight_opacity = op;
                }
            }
            "linenumbers" | "numbers" => {
                self.line_numbers = true;
            }
            "cursorblink" | "blinkingcursor" => {
                self.blinking_cursor = true;
            }
            "constantshow" => {
                self.status_bar = true;
            }
            "autocompletions" | "auto_completions" | "autopairs" | "auto_pairs" => {
                self.auto_completions = if val.is_empty() { true } else { parse_bool(val, true) };
            }
            "minibar" | "compactfooter" | "compact" => {
                self.compact_footer = CompactFooterMode::Always;
            }
            "softwrap" | "wrap" | "warpline" | "warp_line" | "wrapline" | "wrap_line" | "wraplines" => {
                self.warp_line = if val.is_empty() { true } else { parse_bool(val, true) };
            }
            _ => {}
        }
    }

    fn apply_nano_unset(&mut self, opt: &str) {
        match opt {
            "spotlight" | "linehighlight" | "highlight" => {
                self.highlight_enabled = false;
            }
            "linenumbers" | "numbers" => {
                self.line_numbers = false;
            }
            "cursorblink" | "blinkingcursor" => {
                self.blinking_cursor = false;
            }
            "constantshow" => {
                self.status_bar = false;
            }
            "autocompletions" | "auto_completions" | "autopairs" | "auto_pairs" => {
                self.auto_completions = false;
            }
            "minibar" | "compactfooter" | "compact" => {
                self.compact_footer = CompactFooterMode::Never;
            }
            "softwrap" | "wrap" | "nowrap" | "warpline" | "warp_line" | "wrapline" | "wrap_line" | "wraplines" => {
                self.warp_line = false;
            }
            _ => {}
        }
    }

    fn apply_key_value(&mut self, key: &str, val: &str) {
        match key {
            "compactfooter" | "compact_footer" | "minibar" => {
                match val.to_lowercase().as_str() {
                    "always" | "true" | "yes" | "1" | "on" => {
                        self.compact_footer = CompactFooterMode::Always;
                    }
                    "never" | "false" | "no" | "0" | "off" => {
                        self.compact_footer = CompactFooterMode::Never;
                    }
                    _ => {
                        self.compact_footer = CompactFooterMode::Auto;
                    }
                }
            }
            "autocompletions" | "auto_completions" | "autopairs" | "auto_pairs" => {
                self.auto_completions = parse_bool(val, self.auto_completions);
            }
            "warpline" | "warp_line" | "wrapline" | "wrap_line" | "wraplines" | "wrap_lines" | "softwrap" | "soft_wrap" => {
                self.warp_line = parse_bool(val, self.warp_line);
            }
            "highlight" | "linehighlight" | "spotlight" => {
                self.highlight_enabled = parse_bool(val, self.highlight_enabled);
            }
            "highlightcolor" | "highlight_color" | "stripecolor" | "glowcolor" => {
                if let Some(col) = parse_color(val) {
                    self.highlight_color = col;
                }
            }
            "highlightopacity" | "highlight_opacity" | "opacity" | "glowopacity" => {
                if let Some(op) = parse_opacity(val) {
                    self.highlight_opacity = op;
                }
            }
            "blinkingcursor" | "blinking_cursor" | "cursorblink" => {
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

pub fn parse_color(val: &str) -> Option<Color> {
    let s = val.trim().to_lowercase();
    let s = s.trim_matches(|c| c == '\'' || c == '"');

    // 1. Hex: #RRGGBB or #RGB or RRGGBB
    if s.starts_with('#') || (s.len() == 6 && s.chars().all(|c| c.is_ascii_hexdigit())) {
        let hex_str = s.trim_start_matches('#');
        if hex_str.len() == 6 {
            if let Ok(num) = u32::from_str_radix(hex_str, 16) {
                let r = ((num >> 16) & 0xFF) as u8;
                let g = ((num >> 8) & 0xFF) as u8;
                let b = (num & 0xFF) as u8;
                return Some(Color::Rgb { r, g, b });
            }
        } else if hex_str.len() == 3 {
            let mut expanded = String::new();
            for c in hex_str.chars() {
                expanded.push(c);
                expanded.push(c);
            }
            if let Ok(num) = u32::from_str_radix(&expanded, 16) {
                let r = ((num >> 16) & 0xFF) as u8;
                let g = ((num >> 8) & 0xFF) as u8;
                let b = (num & 0xFF) as u8;
                return Some(Color::Rgb { r, g, b });
            }
        }
    }

    // 2. rgb(r, g, b) or r, g, b
    let rgb_inner = if s.starts_with("rgb(") && s.ends_with(')') {
        &s[4..s.len() - 1]
    } else {
        &s
    };
    let parts: Vec<&str> = rgb_inner.split(',').collect();
    if parts.len() == 3 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            parts[0].trim().parse::<u8>(),
            parts[1].trim().parse::<u8>(),
            parts[2].trim().parse::<u8>(),
        ) {
            return Some(Color::Rgb { r, g, b });
        }
    }

    // 3. Named colors (Catppuccin & standard)
    match s {
        "mauve" | "purple" | "magenta" => Some(Palette::MAUVE),
        "blue" => Some(Palette::BLUE),
        "lavender" => Some(Palette::LAVENDER),
        "sapphire" => Some(Palette::SAPPHIRE),
        "teal" | "cyan" => Some(Palette::TEAL),
        "green" => Some(Palette::GREEN),
        "yellow" => Some(Palette::YELLOW),
        "peach" | "orange" => Some(Palette::PEACH),
        "red" => Some(Palette::RED),
        "maroon" => Some(Color::Rgb { r: 235, g: 160, b: 172 }),
        "pink" | "flamingo" => Some(Color::Rgb { r: 242, g: 205, b: 205 }),
        "rosewater" => Some(Color::Rgb { r: 245, g: 224, b: 220 }),
        "text" | "white" => Some(Palette::TEXT),
        "subtext" | "gray" | "grey" => Some(Palette::SUBTEXT0),
        "surface" | "surface0" => Some(Palette::SURFACE0),
        "crust" | "black" => Some(Palette::CRUST),
        "mantle" => Some(Palette::MANTLE),
        _ => None,
    }
}

pub fn parse_opacity(val: &str) -> Option<f32> {
    let s = val.trim().trim_matches(|c| c == '\'' || c == '"');
    let is_percent = s.ends_with('%');
    let num_str = s.trim_end_matches('%').trim();
    if let Ok(mut num) = num_str.parse::<f32>() {
        if is_percent || num > 1.0 {
            num /= 100.0;
        }
        return Some(num.clamp(0.0, 1.0));
    }
    None
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nanorc_spotlight_and_colors() {
        let nanorc = r#"
            set spotlight
            set stripecolor mauve
            set opacity 15%
        "#;
        let mut cfg = Config::default();
        cfg.parse(nanorc);
        assert!(cfg.highlight_enabled);
        assert_eq!(cfg.highlight_color, Palette::MAUVE);
        assert!((cfg.highlight_opacity - 0.15).abs() < 1e-4);
    }

    #[test]
    fn test_nimorc_key_value() {
        let nimorc = r#"
            Highlight = true
            HighlightColor = #89b4fa
            HighlightOpacity = 0.25
        "#;
        let mut cfg = Config::default();
        cfg.parse(nimorc);
        assert!(cfg.highlight_enabled);
        assert_eq!(cfg.highlight_color, Color::Rgb { r: 137, g: 180, b: 250 });
        assert!((cfg.highlight_opacity - 0.25).abs() < 1e-4);
    }

    #[test]
    fn test_active_line_bg_blending() {
        let mut cfg = Config::default();
        cfg.highlight_color = Color::Rgb { r: 200, g: 200, b: 200 };
        cfg.highlight_opacity = 0.5;
        let blended = cfg.active_line_bg(Color::Rgb { r: 100, g: 100, b: 100 });
        assert_eq!(blended, Color::Rgb { r: 150, g: 150, b: 150 });
    }

    #[test]
    fn test_compact_footer_config() {
        let mut cfg = Config::default();
        assert_eq!(cfg.compact_footer, CompactFooterMode::Auto);

        cfg.parse("CompactFooter = true");
        assert_eq!(cfg.compact_footer, CompactFooterMode::Always);

        cfg.parse("CompactFooter = false");
        assert_eq!(cfg.compact_footer, CompactFooterMode::Never);

        cfg.parse("CompactFooter = auto");
        assert_eq!(cfg.compact_footer, CompactFooterMode::Auto);

        cfg.parse("set minibar");
        assert_eq!(cfg.compact_footer, CompactFooterMode::Always);

        cfg.parse("unset minibar");
        assert_eq!(cfg.compact_footer, CompactFooterMode::Never);
    }

    #[test]
    fn test_warp_line_config() {
        let mut cfg = Config::default();
        assert!(!cfg.warp_line);

        cfg.parse("WarpLine = true");
        assert!(cfg.warp_line);

        cfg.parse("WarpLine = false");
        assert!(!cfg.warp_line);

        cfg.parse("WrapLines = on");
        assert!(cfg.warp_line);

        cfg.parse("set softwrap");
        assert!(cfg.warp_line);

        cfg.parse("unset softwrap");
        assert!(!cfg.warp_line);
    }
}

