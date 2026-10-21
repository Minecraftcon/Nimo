use crossterm::style::Color;

/// Catppuccin Mocha Truecolor Palette
#[allow(dead_code)]
pub struct Palette;

#[allow(dead_code)]
impl Palette {
    // Backgrounds
    pub const BASE: Color = Color::Rgb { r: 30, g: 30, b: 46 };       // #1e1e2e - Editor background
    pub const MANTLE: Color = Color::Rgb { r: 24, g: 24, b: 37 };     // #181825 - Footer / Title background
    pub const CRUST: Color = Color::Rgb { r: 17, g: 17, b: 27 };      // #11111b - Darker accents & key text
    pub const SURFACE0: Color = Color::Rgb { r: 49, g: 50, b: 68 };   // #313244 - Status bar & active gutter
    pub const GUTTER_BG: Color = Color::Rgb { r: 36, g: 37, b: 53 };  // #242535 - Normal gutter background
    pub const ACTIVE_LINE_BG: Color = Color::Rgb { r: 42, g: 44, b: 62 }; // #2a2c3e - Faint editing glow for cursor line

    // Foregrounds & Accents
    pub const TEXT: Color = Color::Rgb { r: 205, g: 214, b: 244 };    // #cdd6f4 - Main text
    pub const SUBTEXT0: Color = Color::Rgb { r: 166, g: 173, b: 200 };// #a6adc8 - Muted text
    pub const GUTTER_FG: Color = Color::Rgb { r: 108, g: 112, b: 134 };// #6c7086 - Line number text
    pub const MAUVE: Color = Color::Rgb { r: 203, g: 166, b: 247 };   // #cba6f7 - Accent / Header / Active line
    pub const PEACH: Color = Color::Rgb { r: 250, g: 179, b: 135 };   // #fab387 - Shortcut keys
    pub const RED: Color = Color::Rgb { r: 243, g: 139, b: 168 };     // #f38ba8 - Modified flag & regex
    pub const GREEN: Color = Color::Rgb { r: 166, g: 227, b: 161 };   // #a6e3a1 - Strings
    pub const YELLOW: Color = Color::Rgb { r: 249, g: 226, b: 175 };  // #f9e2af - Types & classes
    pub const BLUE: Color = Color::Rgb { r: 137, g: 180, b: 250 };    // #89b4fa - Functions & methods
    pub const TEAL: Color = Color::Rgb { r: 148, g: 226, b: 213 };    // #94e2d5 - Constants & numbers
    pub const SAPPHIRE: Color = Color::Rgb { r: 116, g: 199, b: 236 };// #74c7ec - Special identifiers
    pub const LAVENDER: Color = Color::Rgb { r: 180, g: 190, b: 254 };// #b4befe - Builtins
    pub const OVERLAY0: Color = Color::Rgb { r: 108, g: 112, b: 134 };// #6c7086 - Comments
}
