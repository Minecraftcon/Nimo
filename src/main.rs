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

    // 2. Parse command line arguments for filename
    let args: Vec<String> = env::args().collect();
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
