use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::{
    cursor::{self, SetCursorStyle},
    event::{Event, KeyCode, KeyModifiers},
    execute, queue,
    style::{SetBackgroundColor, SetForegroundColor},
    terminal,
};

use crate::config::Config;
use crate::syntax::{highlight_line, Language};
use crate::theme::Palette;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptState {
    None,
    SaveConfirm { input: String, quit_after: bool },
    ExitConfirm,
}

pub struct Editor {
    pub lines: Vec<String>,
    pub saved_lines: Vec<String>,
    pub cursor_row: usize, // 0-indexed in lines
    pub cursor_col: usize, // 0-indexed in current line (char count)
    pub row_offset: usize, // Viewport scroll top line
    pub col_offset: usize, // Viewport scroll left col
    pub file_path: Option<PathBuf>,
    pub lang: Language,
    pub is_modified: bool,
    pub status_message: String,
    pub status_time: Option<Instant>,
    pub cut_buffer: Option<String>,
    pub should_quit: bool,
    pub config: Config,
    pub term_width: u16,
    pub term_height: u16,
    pub prompt_state: PromptState,
}

impl Editor {
    pub fn new(file_path: Option<PathBuf>, config: Config) -> Self {
        let mut lines = Vec::new();
        let is_modified = false;

        if let Some(ref path) = file_path {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(path) {
                    for line in content.lines() {
                        lines.push(line.to_string());
                    }
                }
            }
        }

        if lines.is_empty() {
            lines.push(String::new());
        }

        let (term_width, term_height) = terminal::size().unwrap_or((80, 24));
        let lang = Language::from_path(file_path.as_deref());
        let saved_lines = lines.clone();

        Self {
            lines,
            saved_lines,
            cursor_row: 0,
            cursor_col: 0,
            row_offset: 0,
            col_offset: 0,
            file_path,
            lang,
            is_modified,
            status_message: String::new(),
            status_time: None,
            cut_buffer: None,
            should_quit: false,
            config,
            term_width,
            term_height,
            prompt_state: PromptState::None,
        }
    }

    pub fn update_modified_status(&mut self) {
        self.is_modified = self.lines != self.saved_lines;
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = msg.into();
        self.status_time = Some(Instant::now());
    }

    /// Height available for text rows
    pub fn text_area_height(&self) -> usize {
        let mut h = self.term_height as usize;
        if self.config.top_bar && h > 0 {
            h -= 1;
        }
        if self.config.status_bar && h > 0 {
            h -= 1;
        }
        if self.config.footer && h >= 2 {
            h -= 2;
        }
        h.max(1)
    }

    /// Gutter width in columns
    pub fn gutter_width(&self) -> usize {
        if let Some(ref custom) = self.config.custom_separator {
            if custom.contains("{line}") {
                // Width with a 3-digit line number
                return custom.replace("{line}", "999").chars().count();
            } else {
                return custom.chars().count();
            }
        }

        if !self.config.line_numbers && !self.config.separator {
            return 0;
        }

        let num_w = if self.config.line_numbers { 4 } else { 0 };
        let sep_w = if self.config.separator {
            self.config.separator_style.width()
        } else {
            0
        };

        num_w + sep_w
    }

    /// Format custom separator for a specific line number
    pub fn format_custom_gutter(&self, line_num: usize) -> (String, usize, usize) {
        if let Some(ref custom) = self.config.custom_separator {
            if let Some(pos) = custom.find("{line}") {
                let num_str = format!("{:>3}", line_num % 1000);
                let prefix = &custom[..pos];
                let suffix = &custom[pos + 6..];
                let formatted = format!("{}{}{}", prefix, num_str, suffix);
                let num_start = prefix.chars().count();
                let num_len = num_str.chars().count();
                return (formatted, num_start, num_len);
            } else {
                let formatted = custom.clone();
                return (formatted, 0, 0);
            }
        }
        (String::new(), 0, 0)
    }

    pub fn update_viewport(&mut self) {
        let text_h = self.text_area_height();

        if self.cursor_row < self.row_offset {
            self.row_offset = self.cursor_row;
        }
        if self.cursor_row >= self.row_offset + text_h {
            self.row_offset = self.cursor_row - text_h + 1;
        }

        let gutter_w = self.gutter_width();
        let text_w = (self.term_width as usize).saturating_sub(gutter_w).max(1);

        if self.cursor_col < self.col_offset {
            self.col_offset = self.cursor_col;
        }
        if self.cursor_col >= self.col_offset + text_w {
            self.col_offset = self.cursor_col - text_w + 1;
        }
    }

    pub fn render(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        let (w, h) = terminal::size().unwrap_or((80, 24));
        self.term_width = w;
        self.term_height = h;

        self.update_viewport();

        queue!(stdout, cursor::Hide)?;

        let mut current_terminal_row: u16 = 0;

        // 1. Top Bar
        if self.config.top_bar {
            queue!(
                stdout,
                cursor::MoveTo(0, current_terminal_row),
                SetBackgroundColor(Palette::SURFACE0),
                SetForegroundColor(Palette::TEXT)
            )?;

            let filename = self
                .file_path
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "[New File]".to_string());

            let file_str = format!("File: {}", filename);
            let mod_str = if self.is_modified { " [Modified]" } else { "" };
            let full_title = format!("{}{}", file_str, mod_str);
            let width = self.term_width as usize;

            // Render highlighted top bar background (Mauve accent)
            queue!(
                stdout,
                cursor::MoveTo(0, current_terminal_row),
                SetBackgroundColor(Palette::MAUVE),
                SetForegroundColor(Palette::CRUST)
            )?;
            write!(stdout, "{}", " ".repeat(width))?;

            // Render centered File: <name> [Modified]
            let center_len = full_title.chars().count();
            if center_len < width {
                let start_x = ((width.saturating_sub(center_len)) / 2) as u16;
                queue!(
                    stdout,
                    cursor::MoveTo(start_x, current_terminal_row),
                    SetBackgroundColor(Palette::MAUVE),
                    SetForegroundColor(Palette::CRUST)
                )?;
                write!(stdout, "{}", file_str)?;

                if self.is_modified {
                    queue!(
                        stdout,
                        SetBackgroundColor(Palette::RED),
                        SetForegroundColor(Palette::CRUST)
                    )?;
                    write!(stdout, " [Modified]")?;
                }
            }

            current_terminal_row += 1;
        }

        // 2. Editor Text Area
        let text_h = self.text_area_height();
        let gutter_w = self.gutter_width();

        for i in 0..text_h {
            let row_idx = self.row_offset + i;
            let screen_y = current_terminal_row + i as u16;
            let is_active_row = row_idx == self.cursor_row;

            queue!(stdout, cursor::MoveTo(0, screen_y))?;

            // Gutter rendering
            if gutter_w > 0 {
                let gutter_bg = if is_active_row {
                    self.config.active_line_bg(Palette::GUTTER_BG)
                } else {
                    Palette::GUTTER_BG
                };
                let num_fg = if is_active_row {
                    Palette::MAUVE
                } else {
                    Palette::GUTTER_FG
                };

                queue!(stdout, SetBackgroundColor(gutter_bg))?;

                if self.config.custom_separator.is_some() {
                    let show_num = if self.config.line_numbers_type_only {
                        if row_idx < self.lines.len() {
                            !self.lines[row_idx].is_empty() || row_idx == self.cursor_row
                        } else {
                            false
                        }
                    } else {
                        row_idx < self.lines.len()
                    };

                    if show_num {
                        let (formatted, num_start, num_len) = self.format_custom_gutter(row_idx + 1);
                        let chars: Vec<char> = formatted.chars().collect();
                        for (idx, ch) in chars.iter().enumerate() {
                            if idx >= num_start && idx < num_start + num_len {
                                queue!(stdout, SetForegroundColor(num_fg))?;
                            } else {
                                queue!(stdout, SetForegroundColor(Palette::GUTTER_FG))?;
                            }
                            write!(stdout, "{}", ch)?;
                        }
                    } else {
                        write!(stdout, "{}", " ".repeat(gutter_w))?;
                    }
                } else {
                    if self.config.line_numbers {
                        queue!(stdout, SetForegroundColor(num_fg))?;

                        let show_num = if self.config.line_numbers_type_only {
                            if row_idx < self.lines.len() {
                                // Show number only if the line has characters OR it's the active cursor row
                                !self.lines[row_idx].is_empty() || row_idx == self.cursor_row
                            } else {
                                false
                            }
                        } else {
                            row_idx < self.lines.len()
                        };

                        if show_num {
                            let num_str = format!("{:>3} ", (row_idx + 1) % 1000);
                            write!(stdout, "{}", num_str)?;
                        } else {
                            write!(stdout, "    ")?;
                        }
                    }

                    if self.config.separator {
                        queue!(stdout, SetForegroundColor(Palette::GUTTER_FG))?;
                        write!(stdout, "{}", self.config.separator_style.as_str())?;
                    }
                }
            }

            // Line Text rendering
            let line_bg = if is_active_row {
                self.config.active_line_bg(Palette::BASE)
            } else {
                Palette::BASE
            };

            queue!(
                stdout,
                SetBackgroundColor(line_bg),
                SetForegroundColor(Palette::TEXT)
            )?;

            let available_w = (self.term_width as usize).saturating_sub(gutter_w);

            if row_idx < self.lines.len() {
                let line = &self.lines[row_idx];
                let spans = highlight_line(line, self.lang);

                let mut current_col: usize = 0;
                let mut rendered_w: usize = 0;

                for span in spans {
                    let span_len = span.text.chars().count();
                    let span_end = current_col + span_len;

                    // If span overlaps with [col_offset, col_offset + available_w]
                    if span_end > self.col_offset && current_col < self.col_offset + available_w {
                        let slice_start = self.col_offset.saturating_sub(current_col);
                        let chars: Vec<char> = span.text.chars().collect();
                        let max_take = (self.col_offset + available_w).saturating_sub(current_col + slice_start);
                        let visible_slice: String = chars
                            .into_iter()
                            .skip(slice_start)
                            .take(max_take)
                            .collect();

                        let slice_w = visible_slice.chars().count();
                        if slice_w > 0 {
                            queue!(stdout, SetForegroundColor(span.color))?;
                            write!(stdout, "{}", visible_slice)?;
                            rendered_w += slice_w;
                        }
                    }

                    current_col = span_end;
                    if current_col >= self.col_offset + available_w {
                        break;
                    }
                }

                if rendered_w < available_w {
                    queue!(stdout, SetForegroundColor(Palette::TEXT))?;
                    write!(stdout, "{}", " ".repeat(available_w - rendered_w))?;
                }
            } else {
                write!(stdout, "{}", " ".repeat(available_w))?;
            }
        }

        current_terminal_row += text_h as u16;

        // 3. Status Bar
        let status_row_y = current_terminal_row;
        if self.config.status_bar {
            queue!(
                stdout,
                cursor::MoveTo(0, current_terminal_row),
                SetBackgroundColor(Palette::SURFACE0),
                SetForegroundColor(Palette::TEXT)
            )?;

            let status = match &self.prompt_state {
                PromptState::SaveConfirm { input, .. } => {
                    format!(" File Name to Write: {}", input)
                }
                PromptState::ExitConfirm => {
                    " Save modified buffer? [y/n] ".to_string()
                }
                PromptState::None => {
                    if let Some(time) = self.status_time {
                        if time.elapsed() < Duration::from_secs(4) {
                            format!(" {}", self.status_message)
                        } else {
                            self.default_status()
                        }
                    } else {
                        self.default_status()
                    }
                }
            };

            let pad_len = (self.term_width as usize).saturating_sub(status.chars().count());
            let padded = format!("{}{}", status, " ".repeat(pad_len));
            write!(stdout, "{}", &padded[..self.term_width as usize])?;

            current_terminal_row += 1;
        }

        // 4. Footer (Nano Shortcuts)
        if self.config.footer && current_terminal_row + 1 < self.term_height {
            let row1 = match &self.prompt_state {
                PromptState::SaveConfirm { .. } => [
                    ("Enter", "Save"),
                    ("^C", "Cancel"),
                    ("", ""),
                    ("", ""),
                    ("", ""),
                    ("", ""),
                ],
                PromptState::ExitConfirm => [
                    ("Y", "Yes"),
                    ("N", "No"),
                    ("^C", "Cancel"),
                    ("", ""),
                    ("", ""),
                    ("", ""),
                ],
                PromptState::None => [
                    ("^O", "WriteOut"),
                    ("^X", "Exit"),
                    ("^K", "CutLine"),
                    ("^U", "Paste"),
                    ("^N", "ToggleNum"),
                    ("^S", "ToggleSep"),
                ],
            };

            let row2 = match &self.prompt_state {
                PromptState::None => [
                    ("^B", "BlinkCur"),
                    ("^P", "SepStyle"),
                    ("^C", "CurPos"),
                    ("^W", "WhereIs"),
                    ("^J", "Justify"),
                    ("^G", "GetHelp"),
                ],
                _ => [
                    ("", ""),
                    ("", ""),
                    ("", ""),
                    ("", ""),
                    ("", ""),
                    ("", ""),
                ],
            };

            self.render_shortcut_row(stdout, current_terminal_row, &row1)?;
            self.render_shortcut_row(stdout, current_terminal_row + 1, &row2)?;
        }

        // 5. Place cursor at exact position
        let (screen_cursor_x, screen_cursor_y) = match &self.prompt_state {
            PromptState::SaveConfirm { input, .. } => {
                let prompt_prefix = " File Name to Write: ";
                let cursor_x = (prompt_prefix.chars().count() + input.chars().count())
                    .min(self.term_width as usize - 1) as u16;
                (cursor_x, status_row_y)
            }
            PromptState::ExitConfirm => {
                let prompt_str = " Save modified buffer? [y/n] ";
                let cursor_x = prompt_str.chars().count().min(self.term_width as usize - 1) as u16;
                (cursor_x, status_row_y)
            }
            PromptState::None => {
                let y = if self.config.top_bar {
                    1 + (self.cursor_row - self.row_offset) as u16
                } else {
                    (self.cursor_row - self.row_offset) as u16
                };
                let x = (gutter_w + (self.cursor_col - self.col_offset)) as u16;
                (x, y)
            }
        };

        queue!(
            stdout,
            cursor::MoveTo(screen_cursor_x, screen_cursor_y),
            cursor::Show
        )?;

        stdout.flush()?;
        Ok(())
    }

    fn default_status(&self) -> String {
        let total_chars: usize = self.lines.iter().map(|l| l.chars().count() + 1).sum();
        format!(
            " [ Line {}/{} , Col {} ]  {} chars",
            self.cursor_row + 1,
            self.lines.len(),
            self.cursor_col + 1,
            total_chars
        )
    }

    fn render_shortcut_row(
        &self,
        stdout: &mut io::Stdout,
        y: u16,
        items: &[(&str, &str)],
    ) -> io::Result<()> {
        queue!(
            stdout,
            cursor::MoveTo(0, y),
            SetBackgroundColor(Palette::MANTLE),
            SetForegroundColor(Palette::SUBTEXT0)
        )?;

        let mut row_str = String::new();
        let slot_w = (self.term_width as usize / items.len()).max(8);

        for (key, desc) in items {
            let item_str = format!("{:<2} {:<7}", key, desc);
            let padded = format!("{:<width$}", item_str, width = slot_w);
            row_str.push_str(&padded);
        }

        let remaining = (self.term_width as usize).saturating_sub(row_str.chars().count());
        row_str.push_str(&" ".repeat(remaining));

        // Highlight the shortcut keys in Peach
        let mut col_idx = 0;
        for (key, desc) in items {
            let k_len = key.chars().count();

            queue!(
                stdout,
                cursor::MoveTo(col_idx as u16, y),
                SetBackgroundColor(Palette::MANTLE),
                SetForegroundColor(Palette::PEACH)
            )?;
            write!(stdout, "{}", key)?;

            queue!(
                stdout,
                cursor::MoveTo((col_idx + k_len + 1) as u16, y),
                SetBackgroundColor(Palette::MANTLE),
                SetForegroundColor(Palette::TEXT)
            )?;
            write!(stdout, "{}", desc)?;

            col_idx += slot_w;
        }

        Ok(())
    }

    pub fn handle_event(&mut self, event: Event) -> io::Result<()> {
        if let Event::Key(key) = event {
            // Handle prompt states first if active
            match &mut self.prompt_state {
                PromptState::SaveConfirm { input, quit_after } => {
                    let should_quit_after = *quit_after;
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && (key.code == KeyCode::Char('c') || key.code == KeyCode::Char('C'))
                    {
                        self.prompt_state = PromptState::None;
                        self.set_status("[Save cancelled]");
                        return Ok(());
                    }

                    match key.code {
                        KeyCode::Enter => {
                            let filename = if input.trim().is_empty() {
                                self.file_path
                                    .clone()
                                    .unwrap_or_else(|| PathBuf::from("unnamed.txt"))
                            } else {
                                PathBuf::from(input.trim())
                            };
                            self.prompt_state = PromptState::None;
                            self.save_file_to(filename);
                            if should_quit_after {
                                self.should_quit = true;
                            }
                            return Ok(());
                        }
                        KeyCode::Char(c) => {
                            input.push(c);
                            return Ok(());
                        }
                        KeyCode::Backspace => {
                            input.pop();
                            return Ok(());
                        }
                        KeyCode::Esc => {
                            self.prompt_state = PromptState::None;
                            self.set_status("[Save cancelled]");
                            return Ok(());
                        }
                        _ => return Ok(()),
                    }
                }
                PromptState::ExitConfirm => {
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && (key.code == KeyCode::Char('c') || key.code == KeyCode::Char('C'))
                    {
                        self.prompt_state = PromptState::None;
                        self.set_status("[Exit cancelled]");
                        return Ok(());
                    }

                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            let current_name = self
                                .file_path
                                .as_ref()
                                .and_then(|p| p.file_name())
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| "unnamed.txt".to_string());
                            self.prompt_state = PromptState::SaveConfirm {
                                input: current_name,
                                quit_after: true,
                            };
                            return Ok(());
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') => {
                            self.prompt_state = PromptState::None;
                            self.should_quit = true;
                            return Ok(());
                        }
                        KeyCode::Esc => {
                            self.prompt_state = PromptState::None;
                            self.set_status("[Exit cancelled]");
                            return Ok(());
                        }
                        _ => return Ok(()),
                    }
                }
                PromptState::None => {}
            }

            // Hotkeys with Ctrl
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                match key.code {
                    KeyCode::Char('x') | KeyCode::Char('X') => {
                        if self.is_modified {
                            self.prompt_state = PromptState::ExitConfirm;
                        } else {
                            self.should_quit = true;
                        }
                        return Ok(());
                    }
                    KeyCode::Char('o') | KeyCode::Char('O') => {
                        let current_name = self
                            .file_path
                            .as_ref()
                            .and_then(|p| p.file_name())
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "unnamed.txt".to_string());
                        self.prompt_state = PromptState::SaveConfirm {
                            input: current_name,
                            quit_after: false,
                        };
                        return Ok(());
                    }
                    KeyCode::Char('k') | KeyCode::Char('K') => {
                        self.cut_line();
                        return Ok(());
                    }
                    KeyCode::Char('u') | KeyCode::Char('U') => {
                        self.paste_line();
                        return Ok(());
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        self.set_status(format!(
                            "Cursor: Line {}, Column {}",
                            self.cursor_row + 1,
                            self.cursor_col + 1
                        ));
                        return Ok(());
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        self.config.line_numbers = !self.config.line_numbers;
                        self.set_status(format!(
                            "Line numbers: {}",
                            if self.config.line_numbers { "ON" } else { "OFF" }
                        ));
                        return Ok(());
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        self.config.separator = !self.config.separator;
                        self.set_status(format!(
                            "Separator: {}",
                            if self.config.separator { "ON" } else { "OFF" }
                        ));
                        return Ok(());
                    }
                    KeyCode::Char('b') | KeyCode::Char('B') => {
                        self.config.blinking_cursor = !self.config.blinking_cursor;
                        self.set_status(format!(
                            "Blinking cursor: {}",
                            if self.config.blinking_cursor { "ON" } else { "OFF" }
                        ));
                        self.apply_cursor_style();
                        return Ok(());
                    }
                    KeyCode::Char('p') | KeyCode::Char('P') => {
                        self.config.separator_style = self.config.separator_style.next();
                        self.set_status(format!(
                            "Separator style: {}",
                            self.config.separator_style.name()
                        ));
                        return Ok(());
                    }
                    _ => {}
                }
            }

            // Normal typing and editing
            match key.code {
                KeyCode::Char(c) => {
                    self.insert_char(c);
                }
                KeyCode::Enter => {
                    self.insert_newline();
                }
                KeyCode::Backspace => {
                    self.delete_backspace();
                }
                KeyCode::Delete => {
                    self.delete_forward();
                }
                KeyCode::Left => {
                    self.move_cursor_left();
                }
                KeyCode::Right => {
                    self.move_cursor_right();
                }
                KeyCode::Up => {
                    self.move_cursor_up();
                }
                KeyCode::Down => {
                    self.move_cursor_down();
                }
                KeyCode::Home => {
                    self.cursor_col = 0;
                }
                KeyCode::End => {
                    self.cursor_col = self.lines[self.cursor_row].chars().count();
                }
                KeyCode::PageUp => {
                    let page_size = self.text_area_height();
                    self.cursor_row = self.cursor_row.saturating_sub(page_size);
                    self.clamp_cursor_col();
                }
                KeyCode::PageDown => {
                    let page_size = self.text_area_height();
                    self.cursor_row = (self.cursor_row + page_size).min(self.lines.len() - 1);
                    self.clamp_cursor_col();
                }
                KeyCode::Tab => {
                    for _ in 0..4 {
                        self.insert_char(' ');
                    }
                }
                _ => {}
            }
        } else if let Event::Resize(w, h) = event {
            self.term_width = w;
            self.term_height = h;
        }

        Ok(())
    }

    pub fn apply_cursor_style(&self) {
        let mut stdout = io::stdout();
        if self.config.blinking_cursor {
            let _ = execute!(stdout, SetCursorStyle::BlinkingBlock);
        } else {
            let _ = execute!(stdout, SetCursorStyle::SteadyBlock);
        }
    }

    pub fn insert_char(&mut self, c: char) {
        let line = &mut self.lines[self.cursor_row];
        let mut chars: Vec<char> = line.chars().collect();

        if self.cursor_col >= chars.len() {
            chars.push(c);
        } else {
            chars.insert(self.cursor_col, c);
        }

        *line = chars.into_iter().collect();
        self.cursor_col += 1;
        self.update_modified_status();
    }

    pub fn insert_newline(&mut self) {
        let current_line = &self.lines[self.cursor_row];
        let chars: Vec<char> = current_line.chars().collect();

        let (left, right) = if self.cursor_col >= chars.len() {
            (chars.as_slice(), &[][..])
        } else {
            chars.split_at(self.cursor_col)
        };

        let new_left: String = left.iter().collect();
        let new_right: String = right.iter().collect();

        self.lines[self.cursor_row] = new_left;
        self.lines.insert(self.cursor_row + 1, new_right);

        self.cursor_row += 1;
        self.cursor_col = 0;
        self.update_modified_status();
    }

    pub fn delete_backspace(&mut self) {
        if self.cursor_col > 0 {
            let line = &mut self.lines[self.cursor_row];
            let mut chars: Vec<char> = line.chars().collect();
            chars.remove(self.cursor_col - 1);
            *line = chars.into_iter().collect();
            self.cursor_col -= 1;
            self.update_modified_status();
        } else if self.cursor_row > 0 {
            // Join with previous line
            let current_line = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            let prev_line = &mut self.lines[self.cursor_row];
            self.cursor_col = prev_line.chars().count();
            prev_line.push_str(&current_line);
            self.update_modified_status();
        }
    }

    pub fn delete_forward(&mut self) {
        let line_len = self.lines[self.cursor_row].chars().count();
        if self.cursor_col < line_len {
            let line = &mut self.lines[self.cursor_row];
            let mut chars: Vec<char> = line.chars().collect();
            chars.remove(self.cursor_col);
            *line = chars.into_iter().collect();
            self.update_modified_status();
        } else if self.cursor_row + 1 < self.lines.len() {
            // Join with next line
            let next_line = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next_line);
            self.update_modified_status();
        }
    }

    pub fn cut_line(&mut self) {
        if self.lines.len() == 1 {
            self.cut_buffer = Some(self.lines[0].clone());
            self.lines[0].clear();
            self.cursor_col = 0;
        } else {
            let removed = self.lines.remove(self.cursor_row);
            self.cut_buffer = Some(removed);
            if self.cursor_row >= self.lines.len() {
                self.cursor_row = self.lines.len() - 1;
            }
            self.clamp_cursor_col();
        }
        self.update_modified_status();
        self.set_status("Line cut to buffer");
    }

    pub fn paste_line(&mut self) {
        if let Some(ref text) = self.cut_buffer {
            self.lines.insert(self.cursor_row, text.clone());
            self.update_modified_status();
            self.set_status("Pasted line from buffer");
        } else {
            self.set_status("Cut buffer is empty");
        }
    }

    #[allow(dead_code)]
    pub fn save_file(&mut self) {
        let path = match &self.file_path {
            Some(p) => p.clone(),
            None => PathBuf::from("unnamed.txt"),
        };
        self.save_file_to(path);
    }

    pub fn save_file_to(&mut self, path: PathBuf) {
        self.lang = Language::from_path(Some(&path));
        self.file_path = Some(path.clone());
        let mut content = self.lines.join("\n");
        content.push('\n');

        match fs::write(&path, content) {
            Ok(_) => {
                self.saved_lines = self.lines.clone();
                self.is_modified = false;
                self.set_status(format!(
                    "Wrote {} lines to {}",
                    self.lines.len(),
                    path.display()
                ));
            }
            Err(e) => {
                self.set_status(format!("Error saving file: {}", e));
            }
        }
    }

    fn clamp_cursor_col(&mut self) {
        let max_col = self.lines[self.cursor_row].chars().count();
        if self.cursor_col > max_col {
            self.cursor_col = max_col;
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].chars().count();
        }
    }

    fn move_cursor_right(&mut self) {
        let max_col = self.lines[self.cursor_row].chars().count();
        if self.cursor_col < max_col {
            self.cursor_col += 1;
        } else if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    fn move_cursor_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.clamp_cursor_col();
        }
    }

    fn move_cursor_down(&mut self) {
        if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.clamp_cursor_col();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modification_cleared_on_backspace() {
        let mut editor = Editor::new(None, Config::default());
        assert!(!editor.is_modified);

        // Type "hello"
        for c in "hello".chars() {
            editor.insert_char(c);
        }
        assert!(editor.is_modified);

        // Backspace 5 times to clear what was typed
        for _ in 0..5 {
            editor.delete_backspace();
        }
        // Buffer is now identical to original buffer -> is_modified MUST be false!
        assert!(!editor.is_modified);
    }

    #[test]
    fn test_modification_newline_and_backspace() {
        let mut editor = Editor::new(None, Config::default());
        editor.insert_newline();
        assert!(editor.is_modified);

        editor.delete_backspace();
        assert!(!editor.is_modified);
    }
}
