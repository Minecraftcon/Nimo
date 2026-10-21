mod config;
mod editor;
mod syntax;
mod theme;

use std::env;
use std::io::{self, stdout};
use std::path::PathBuf;

use crossterm::{
    cursor::{self, SetCursorStyle},
    event,
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use config::Config;
use editor::Editor;

/// RAII Terminal Guard ensuring terminal reset under ANY circumstance (even panic or Ctrl+C)
struct CleanTerminal;

impl CleanTerminal {
    fn new(blinking: bool) -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut out = stdout();
        execute!(out, EnterAlternateScreen)?;
        if blinking {
            let _ = execute!(out, SetCursorStyle::BlinkingBlock);
        } else {
            let _ = execute!(out, SetCursorStyle::SteadyBlock);
        }
        Ok(Self)
    }
}

impl Drop for CleanTerminal {
    fn drop(&mut self) {
        let mut out = stdout();
        let _ = execute!(
            out,
            cursor::Show,
            SetCursorStyle::DefaultUserShape,
            LeaveAlternateScreen
        );
        let _ = terminal::disable_raw_mode();
    }
}

fn main() -> io::Result<()> {
    // 1. Load configuration from .nimorc
    let config = Config::load();

    // 2. Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "-h" | "--help" => {
                println!("Nimo 1.0.0 - Nano-Inspired Modern Terminal Editor in Rust");
                println!();
                println!("USAGE:");
                println!("  nimo [OPTIONS] [FILE]");
                println!();
                println!("OPTIONS:");
                println!("  -h, --help       Print help information");
                println!("  -v, --version    Print version information");
                println!();
                println!("SHORTCUTS:");
                println!("  Ctrl+O           WriteOut (Save active file)");
                println!("  Ctrl+X           Exit editor (prompts if unsaved)");
                println!("  Ctrl+K           Cut line");
                println!("  Ctrl+U           Paste line");
                println!("  Ctrl+N           Toggle line numbers");
                println!("  Ctrl+S           Toggle vertical separator");
                println!("  Ctrl+B           Toggle cursor blink");
                println!("  Ctrl+P           Cycle separator styles");
                println!("  Ctrl+C           Show cursor position");
                return Ok(());
            }
            "-v" | "--version" => {
                println!("nimo 1.0.0");
                return Ok(());
            }
            _ => {}
        }
    }

    let file_path = if args.len() > 1 {
        Some(PathBuf::from(&args[1]))
    } else {
        None
    };

    // 3. Setup crash-proof RAII terminal guard
    let _guard = CleanTerminal::new(config.blinking_cursor)?;

    // 4. Initialize Editor
    let mut editor = Editor::new(file_path, config);
    let mut stdout = stdout();

    // 5. Main event loop
    while !editor.should_quit {
        editor.render(&mut stdout)?;

        let event = event::read()?;
        editor.handle_event(event)?;
    }

    Ok(())
}
