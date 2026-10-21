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

use crate::config::{CompactFooterMode, Config};
use crate::syntax::{highlight_line, Language};
use crate::theme::Palette;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptState {
    None,
    SaveConfirm { input: String, quit_after: bool },
    ExitConfirm,
    Search { query: String, origin: (usize, usize) },
    Help { scroll: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelpLine {
    Title(&'static str),
    Section(&'static str),
    Entry(&'static str, &'static str),
    Note(&'static str),
    Blank,
}

pub fn get_help_lines() -> Vec<HelpLine> {
    vec![
        HelpLine::Blank,
        HelpLine::Title("  NIMO EDITOR - COMMANDS & TOGGLES REFERENCE"),
        HelpLine::Note("  Fast, elegant terminal text editor • Catppuccin Mocha Truecolor"),
        HelpLine::Blank,

        HelpLine::Section("FILE & BUFFER OPERATIONS"),
        HelpLine::Entry("^O (or F2)", "Save / WriteOut buffer to file"),
        HelpLine::Entry("^X", "Exit Nimo (prompts to save modified buffer)"),
        HelpLine::Entry("^K", "Cut current line to clipboard"),
        HelpLine::Entry("^U", "Paste clipboard line below cursor"),
        HelpLine::Blank,

        HelpLine::Section("SEARCH & HIGHLIGHTS"),
        HelpLine::Entry("^W", "WhereIs / Search (with live match highlights)"),
        HelpLine::Entry("Enter / Down", "Cycle to next search match"),
        HelpLine::Entry("Up", "Cycle to previous search match"),
        HelpLine::Entry("M-W (or F3)", "Repeat search forward (Find Next)"),
        HelpLine::Entry("M-Q (or S-F3)", "Repeat search backward (Find Prev)"),
        HelpLine::Entry("Esc", "Done / Clear search match highlights"),
        HelpLine::Entry("^C", "Display current cursor line, column & char count"),
        HelpLine::Blank,

        HelpLine::Section("INTERFACE TOGGLES"),
        HelpLine::Entry("M-L (or M-Z)", "Toggle WarpLine / Soft Wrap (wrap long lines with |_>)"),
        HelpLine::Entry("^N", "Toggle Line Numbers in gutter"),
        HelpLine::Entry("^S", "Toggle Gutter Separator divider"),
        HelpLine::Entry("^P", "Cycle Separator Style (pipe, thin, double, dots, slash, none)"),
        HelpLine::Entry("^B", "Toggle Blinking Cursor (blinking vs steady block)"),
        HelpLine::Entry("C-S-A / M-A", "Toggle Auto-Completions & Auto-Pairing (code files)"),
        HelpLine::Entry("^H (or F1)", "Show this Help Screen (Esc to exit)"),
        HelpLine::Blank,

        HelpLine::Section("CURSOR & NAVIGATION"),
        HelpLine::Entry("Arrow Keys", "Move cursor (smooth across soft-wrapped line chunks)"),
        HelpLine::Entry("Home", "Jump to current chunk start, then line start"),
        HelpLine::Entry("End", "Jump to current chunk end, then line end"),
        HelpLine::Entry("PgUp / PgDn", "Scroll one screen page up or down"),
        HelpLine::Blank,

        HelpLine::Section("CONFIGURATION (.nimorc & .nanorc)"),
        HelpLine::Entry("WarpLine = true", "Enable soft line wrapping (nanorc: set softwrap)"),
        HelpLine::Entry("CompactFooter = auto", "Adaptive mobile 1-row / desktop 2-row footer (nanorc: set minibar)"),
        HelpLine::Entry("Highlight = true", "Faint editing glow on cursor line (nanorc: set spotlight)"),
        HelpLine::Entry("HighlightColor = mauve", "Glow color: mauve, peach, blue, hex (nanorc: set stripecolor)"),
        HelpLine::Entry("HighlightOpacity = 0.08", "Glow opacity: 0.08, 15% (nanorc: set opacity 15%)"),
        HelpLine::Entry("AutoCompletions = true", "Auto-pair (), [], {}, \"\", '', `` on code files"),
        HelpLine::Entry("CustomSeparator = '{line} :'", "Custom gutter format string"),
        HelpLine::Blank,
        HelpLine::Note("  [ Press Esc or q to exit help and return to editor ]"),
    ]
}

pub const WRAP_ARROW: &str = "|_> ";

#[derive(Debug, Clone)]
pub struct DisplayRow {
    pub line_idx: usize,
    pub chunk_idx: usize,
    pub char_start: usize,
    pub char_end: usize,
    pub is_continuation: bool,
}

pub struct Editor {
    pub lines: Vec<String>,
    pub saved_lines: Vec<String>,
    pub cursor_row: usize, // 0-indexed in lines
    pub cursor_col: usize, // 0-indexed in current line (char count)
    pub row_offset: usize, // Viewport scroll top line
    pub wrap_sub_row: usize, // Viewport scroll sub-row in wrapped line
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
    pub last_search: Option<String>,
}

/// Find all non-overlapping matches of `query` in `line`, case-insensitively.
/// Returns a Vec of `(char_start, char_end)` character indices.
pub fn find_all_matches_in_line(line: &str, query: &str) -> Vec<(usize, usize)> {
    if query.is_empty() {
        return Vec::new();
    }
    let query_chars: Vec<char> = query.to_lowercase().chars().collect();
    let line_chars: Vec<char> = line.chars().collect();
    let q_len = query_chars.len();
    let l_len = line_chars.len();
    if q_len == 0 || l_len < q_len {
        return Vec::new();
    }

    let mut matches = Vec::new();
    let mut i = 0;
    while i + q_len <= l_len {
        let mut matched = true;
        for k in 0..q_len {
            let line_c_lowers: Vec<char> = line_chars[i + k].to_lowercase().collect();
            if line_c_lowers.len() == 1 && line_c_lowers[0] == query_chars[k] {
                continue;
            }
            if line_chars[i + k] == query_chars[k] {
                continue;
            }
            matched = false;
            break;
        }
        if matched {
            matches.push((i, i + q_len));
            i += q_len;
        } else {
            i += 1;
        }
    }
    matches
}

/// Computes the number of wrapped chunks for a line given available width.
pub fn line_wrap_chunks_count(line: &str, available_w: usize) -> usize {
    let char_count = line.chars().count();
    let prefix_w = WRAP_ARROW.chars().count();
    if available_w <= prefix_w || char_count <= available_w {
        1
    } else {
        let cont_cap = available_w - prefix_w;
        1 + (char_count - available_w + cont_cap - 1) / cont_cap
    }
}

/// Maps a logical cursor column in a line to (chunk_index, column_within_that_chunk).
pub fn cursor_to_wrap_pos(cursor_col: usize, available_w: usize) -> (usize, usize) {
    let prefix_w = WRAP_ARROW.chars().count();
    if available_w <= prefix_w || cursor_col < available_w {
        (0, cursor_col)
    } else {
        let cont_cap = (available_w - prefix_w).max(1);
        let rem = cursor_col - available_w;
        let chunk_idx = 1 + rem / cont_cap;
        let col_in_chunk = rem % cont_cap;
        (chunk_idx, col_in_chunk)
    }
}

/// Splits a line into visual chunks (char_start, char_end, is_continuation).
pub fn line_wrap_chunks(
    line: &str,
    is_cursor_line: bool,
    cursor_col: usize,
    available_w: usize,
) -> Vec<(usize, usize, bool)> {
    let char_count = line.chars().count();
    let prefix_w = WRAP_ARROW.chars().count();
    if available_w <= prefix_w {
        return vec![(0, char_count, false)];
    }

    let cont_cap = available_w - prefix_w;
    let mut chunks = Vec::new();

    if char_count <= available_w {
        chunks.push((0, char_count, false));
    } else {
        chunks.push((0, available_w, false));
        let mut start = available_w;
        while start < char_count {
            let end = (start + cont_cap).min(char_count);
            chunks.push((start, end, true));
            start = end;
        }
    }

    if is_cursor_line {
        let (needed_chunk, _) = cursor_to_wrap_pos(cursor_col, available_w);
        while chunks.len() <= needed_chunk {
            chunks.push((char_count, char_count, true));
        }
    }

    chunks
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
            wrap_sub_row: 0,
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
            last_search: None,
        }
    }

    pub fn update_modified_status(&mut self) {
        self.is_modified = self.lines != self.saved_lines;
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = msg.into();
        self.status_time = Some(Instant::now());
    }

    /// Search for `query` in buffer starting from current cursor position.
    /// If `skip_current` is true, skips any match at current cursor position (for Find Next).
    /// Returns `(found, wrapped)` indicating if a match was found and if wrap-around occurred.
    pub fn find_next_match_wrapped(&mut self, query: &str, skip_current: bool) -> (bool, bool) {
        if query.is_empty() || self.lines.is_empty() {
            return (false, false);
        }

        let start_row = self.cursor_row.min(self.lines.len() - 1);
        let start_col = self.cursor_col;

        // 1. Current line after or at cursor
        let matches = find_all_matches_in_line(&self.lines[start_row], query);
        for &(m_start, _) in &matches {
            if (skip_current && m_start > start_col) || (!skip_current && m_start >= start_col) {
                self.cursor_row = start_row;
                self.cursor_col = m_start;
                return (true, false);
            }
        }

        // 2. Lines after current line
        for r in (start_row + 1)..self.lines.len() {
            let m = find_all_matches_in_line(&self.lines[r], query);
            if let Some(&(m_start, _)) = m.first() {
                self.cursor_row = r;
                self.cursor_col = m_start;
                return (true, false);
            }
        }

        // 3. Wrap around: Lines from 0 to start_row - 1
        for r in 0..start_row {
            let m = find_all_matches_in_line(&self.lines[r], query);
            if let Some(&(m_start, _)) = m.first() {
                self.cursor_row = r;
                self.cursor_col = m_start;
                return (true, true);
            }
        }

        // 4. Wrap around: Current line before or at cursor
        for &(m_start, _) in &matches {
            if m_start <= start_col {
                self.cursor_row = start_row;
                self.cursor_col = m_start;
                let wrapped = skip_current || m_start < start_col;
                return (true, wrapped);
            }
        }

        (false, false)
    }

    #[allow(dead_code)]
    pub fn find_next_match(&mut self, query: &str, skip_current: bool) -> bool {
        self.find_next_match_wrapped(query, skip_current).0
    }

    /// Search backwards for `query` in buffer from current cursor position.
    /// Returns `(found, wrapped)` indicating if a match was found and if wrap-around occurred.
    pub fn find_prev_match_wrapped(&mut self, query: &str) -> (bool, bool) {
        if query.is_empty() || self.lines.is_empty() {
            return (false, false);
        }

        let start_row = self.cursor_row.min(self.lines.len() - 1);
        let start_col = self.cursor_col;

        // 1. Current line before cursor
        let matches = find_all_matches_in_line(&self.lines[start_row], query);
        for &(m_start, _) in matches.iter().rev() {
            if m_start < start_col {
                self.cursor_row = start_row;
                self.cursor_col = m_start;
                return (true, false);
            }
        }

        // 2. Lines before current line (in reverse order)
        for r in (0..start_row).rev() {
            let m = find_all_matches_in_line(&self.lines[r], query);
            if let Some(&(m_start, _)) = m.last() {
                self.cursor_row = r;
                self.cursor_col = m_start;
                return (true, false);
            }
        }

        // 3. Wrap around: Lines after current line from end of file down to start_row + 1
        for r in ((start_row + 1)..self.lines.len()).rev() {
            let m = find_all_matches_in_line(&self.lines[r], query);
            if let Some(&(m_start, _)) = m.last() {
                self.cursor_row = r;
                self.cursor_col = m_start;
                return (true, true);
            }
        }

        // 4. Wrap around: Current line at or after cursor (the latest match on start_row)
        for &(m_start, _) in matches.iter().rev() {
            if m_start >= start_col {
                self.cursor_row = start_row;
                self.cursor_col = m_start;
                return (true, true);
            }
        }

        (false, false)
    }

    #[allow(dead_code)]
    pub fn find_prev_match(&mut self, query: &str) -> bool {
        self.find_prev_match_wrapped(query).0
    }

    /// Count total matches and current match index (1-based) in the buffer.
    pub fn count_all_matches(&self, query: &str) -> (usize, usize) {
        if query.is_empty() {
            return (0, 0);
        }
        let mut total = 0;
        let mut current_idx = 0;

        for (r, line) in self.lines.iter().enumerate() {
            let matches = find_all_matches_in_line(line, query);
            for &(m_start, _) in &matches {
                total += 1;
                if r < self.cursor_row || (r == self.cursor_row && m_start <= self.cursor_col) {
                    current_idx = total;
                }
            }
        }

        (current_idx, total)
    }

    /// Number of footer rows to render (0, 1, or 2), adapting to screen size
    pub fn footer_height(&self) -> usize {
        if !self.config.footer {
            return 0;
        }
        match self.config.compact_footer {
            CompactFooterMode::Always => 1,
            CompactFooterMode::Never => 2,
            CompactFooterMode::Auto => {
                // Adaptive: on short Android / mobile screens (height <= 18), use 1 row.
                // On normal desktop screens, use 2 rows (like classic nano).
                if self.term_height <= 18 {
                    1
                } else {
                    2
                }
            }
        }
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
        let f_h = self.footer_height();
        h = h.saturating_sub(f_h);
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

    /// Format custom separator continuation line (spaces in place of line number)
    pub fn format_custom_gutter_continuation(&self) -> String {
        if let Some(ref custom) = self.config.custom_separator {
            if let Some(pos) = custom.find("{line}") {
                let prefix = &custom[..pos];
                let suffix = &custom[pos + 6..];
                format!("{}   {}", prefix, suffix)
            } else {
                custom.clone()
            }
        } else {
            String::new()
        }
    }

    pub fn update_viewport(&mut self) {
        let text_h = self.text_area_height();

        if self.config.warp_line {
            self.col_offset = 0;
            let gutter_w = self.gutter_width();
            let available_w = (self.term_width as usize).saturating_sub(gutter_w).max(1);
            let (target_chunk, _) = cursor_to_wrap_pos(self.cursor_col, available_w);

            if self.cursor_row < self.row_offset
                || (self.cursor_row == self.row_offset && target_chunk < self.wrap_sub_row)
            {
                self.row_offset = self.cursor_row;
                self.wrap_sub_row = target_chunk;
            }

            // Count visual rows from (self.row_offset, self.wrap_sub_row) to (self.cursor_row, target_chunk)
            let mut visual_rows_above = 0;
            if self.cursor_row == self.row_offset {
                visual_rows_above = target_chunk.saturating_sub(self.wrap_sub_row);
            } else if self.cursor_row > self.row_offset {
                let first_line_chunks = line_wrap_chunks_count(&self.lines[self.row_offset], available_w);
                visual_rows_above += first_line_chunks.saturating_sub(self.wrap_sub_row);
                for r in (self.row_offset + 1)..self.cursor_row {
                    visual_rows_above += line_wrap_chunks_count(&self.lines[r], available_w);
                    if visual_rows_above >= text_h {
                        break;
                    }
                }
                visual_rows_above += target_chunk;
            }

            if visual_rows_above >= text_h {
                let mut steps = text_h.saturating_sub(1);
                let mut curr_line = self.cursor_row;
                let mut curr_chunk = target_chunk;

                while steps > 0 {
                    if curr_chunk > 0 {
                        curr_chunk -= 1;
                        steps -= 1;
                    } else if curr_line > 0 {
                        curr_line -= 1;
                        let chunks_count = line_wrap_chunks_count(&self.lines[curr_line], available_w);
                        curr_chunk = chunks_count.saturating_sub(1);
                        steps -= 1;
                    } else {
                        break;
                    }
                }
                self.row_offset = curr_line;
                self.wrap_sub_row = curr_chunk;
            }
        } else {
            self.wrap_sub_row = 0;
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
            let width = self.term_width as usize;

            if matches!(self.prompt_state, PromptState::Help { .. }) {
                queue!(
                    stdout,
                    cursor::MoveTo(0, current_terminal_row),
                    SetBackgroundColor(Palette::MAUVE),
                    SetForegroundColor(Palette::CRUST)
                )?;
                write!(stdout, "{}", " ".repeat(width))?;

                let title = "  NIMO HELP & SHORTCUTS";
                let esc_hint = "[Esc to Exit]";
                queue!(
                    stdout,
                    cursor::MoveTo(0, current_terminal_row),
                    SetBackgroundColor(Palette::MAUVE),
                    SetForegroundColor(Palette::CRUST)
                )?;
                write!(stdout, "{}", title)?;

                if width > title.chars().count() + esc_hint.chars().count() + 4 {
                    let esc_x = (width - esc_hint.chars().count() - 2) as u16;
                    queue!(
                        stdout,
                        cursor::MoveTo(esc_x, current_terminal_row),
                        SetBackgroundColor(Palette::PEACH),
                        SetForegroundColor(Palette::CRUST)
                    )?;
                    write!(stdout, "{}", esc_hint)?;
                }
            } else {
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
            }

            current_terminal_row += 1;
        }

        // 2. Editor Text Area
        let text_h = self.text_area_height();
        let gutter_w = self.gutter_width();
        let available_w = (self.term_width as usize).saturating_sub(gutter_w);

        let display_rows = if !matches!(self.prompt_state, PromptState::Help { .. }) && self.config.warp_line {
            let mut rows = Vec::with_capacity(text_h);
            let mut curr_line = self.row_offset;
            let mut curr_chunk = self.wrap_sub_row;

            while rows.len() < text_h && curr_line < self.lines.len() {
                let line = &self.lines[curr_line];
                let is_cursor_line = curr_line == self.cursor_row;
                let chunks = line_wrap_chunks(line, is_cursor_line, self.cursor_col, available_w);

                if curr_chunk < chunks.len() {
                    let (c_start, c_end, is_cont) = chunks[curr_chunk];
                    rows.push(Some(DisplayRow {
                        line_idx: curr_line,
                        chunk_idx: curr_chunk,
                        char_start: c_start,
                        char_end: c_end,
                        is_continuation: is_cont,
                    }));
                    curr_chunk += 1;
                } else {
                    curr_line += 1;
                    curr_chunk = 0;
                }
            }
            while rows.len() < text_h {
                rows.push(None);
            }
            rows
        } else {
            Vec::new()
        };

        if let PromptState::Help { scroll } = self.prompt_state {
            let help_items = get_help_lines();
            for i in 0..text_h {
                let screen_y = current_terminal_row + i as u16;
                queue!(
                    stdout,
                    cursor::MoveTo(0, screen_y),
                    SetBackgroundColor(Palette::BASE),
                    SetForegroundColor(Palette::TEXT)
                )?;

                let line_idx = scroll + i;
                if line_idx < help_items.len() {
                    match &help_items[line_idx] {
                        HelpLine::Blank => {
                            write!(stdout, "{}", " ".repeat(self.term_width as usize))?;
                        }
                        HelpLine::Title(title) => {
                            queue!(
                                stdout,
                                SetBackgroundColor(Palette::BASE),
                                SetForegroundColor(Palette::PEACH)
                            )?;
                            let pad = (self.term_width as usize).saturating_sub(title.chars().count());
                            write!(stdout, "{}{}", title, " ".repeat(pad))?;
                        }
                        HelpLine::Note(note) => {
                            queue!(
                                stdout,
                                SetBackgroundColor(Palette::BASE),
                                SetForegroundColor(Palette::SUBTEXT0)
                            )?;
                            let pad = (self.term_width as usize).saturating_sub(note.chars().count());
                            write!(stdout, "{}{}", note, " ".repeat(pad))?;
                        }
                        HelpLine::Section(sec) => {
                            queue!(
                                stdout,
                                SetBackgroundColor(Palette::BASE),
                                SetForegroundColor(Palette::MAUVE)
                            )?;
                            let sec_str = format!("  ── {} ──", sec);
                            let pad = (self.term_width as usize).saturating_sub(sec_str.chars().count());
                            write!(stdout, "{}{}", sec_str, " ".repeat(pad))?;
                        }
                        HelpLine::Entry(key, desc) => {
                            let key_w = 22.min((self.term_width as usize).saturating_sub(10));
                            let padded_key = format!("    {:<width$}", key, width = key_w.saturating_sub(4));
                            queue!(
                                stdout,
                                SetBackgroundColor(Palette::BASE),
                                SetForegroundColor(Palette::PEACH)
                            )?;
                            write!(stdout, "{}", padded_key)?;

                            queue!(
                                stdout,
                                SetBackgroundColor(Palette::BASE),
                                SetForegroundColor(Palette::TEXT)
                            )?;
                            let written = padded_key.chars().count();
                            let avail_desc = (self.term_width as usize).saturating_sub(written);
                            let desc_slice: String = desc.chars().take(avail_desc).collect();
                            let d_len = desc_slice.chars().count();
                            let pad = avail_desc.saturating_sub(d_len);
                            write!(stdout, "{}{}", desc_slice, " ".repeat(pad))?;
                        }
                    }
                } else {
                    write!(stdout, "{}", " ".repeat(self.term_width as usize))?;
                }
            }
        } else if self.config.warp_line {
            for i in 0..text_h {
                let screen_y = current_terminal_row + i as u16;
                queue!(stdout, cursor::MoveTo(0, screen_y))?;

                if let Some(ref d) = display_rows[i] {
                    let row_idx = d.line_idx;
                    let is_active_row = row_idx == self.cursor_row;
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

                    if gutter_w > 0 {
                        queue!(stdout, SetBackgroundColor(gutter_bg))?;
                        if d.is_continuation {
                            if self.config.custom_separator.is_some() {
                                let cont_str = self.format_custom_gutter_continuation();
                                queue!(stdout, SetForegroundColor(Palette::GUTTER_FG))?;
                                write!(stdout, "{}", cont_str)?;
                            } else {
                                if self.config.line_numbers {
                                    queue!(stdout, SetForegroundColor(num_fg))?;
                                    write!(stdout, "    ")?;
                                }
                                if self.config.separator {
                                    queue!(stdout, SetForegroundColor(Palette::GUTTER_FG))?;
                                    write!(stdout, "{}", self.config.separator_style.as_str())?;
                                }
                            }
                        } else {
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
                    }

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

                    let mut rendered_w: usize = 0;
                    if d.is_continuation {
                        let arrow_fg = if is_active_row {
                            Palette::PEACH
                        } else {
                            Palette::LAVENDER
                        };
                        queue!(stdout, SetForegroundColor(arrow_fg))?;
                        write!(stdout, "{}", WRAP_ARROW)?;
                        rendered_w += WRAP_ARROW.chars().count();
                    }

                    let line = &self.lines[row_idx];
                    let spans = highlight_line(line, self.lang);

                    let active_query: Option<&str> = match &self.prompt_state {
                        PromptState::Search { query, .. } => {
                            if !query.is_empty() {
                                Some(query.as_str())
                            } else {
                                self.last_search.as_deref().filter(|s| !s.is_empty())
                            }
                        }
                        _ => self.last_search.as_deref().filter(|s| !s.is_empty()),
                    };

                    let line_matches = if let Some(q) = active_query {
                        find_all_matches_in_line(line, q)
                    } else {
                        Vec::new()
                    };

                    let mut line_chars_styled: Vec<(char, crossterm::style::Color)> = Vec::with_capacity(line.chars().count());
                    for span in spans {
                        for c in span.text.chars() {
                            line_chars_styled.push((c, span.color));
                        }
                    }

                    let visible_start = d.char_start;
                    let visible_end = d.char_end.min(line_chars_styled.len());

                    let mut chunks: Vec<(String, crossterm::style::Color, crossterm::style::Color)> = Vec::new();
                    for c_idx in visible_start..visible_end {
                        let (c, syntax_fg) = line_chars_styled[c_idx];
                        let mut fg = syntax_fg;
                        let mut bg = line_bg;

                        for &(m_start, m_end) in &line_matches {
                            if c_idx >= m_start && c_idx < m_end {
                                let is_active = row_idx == self.cursor_row
                                    && (self.cursor_col == m_start || (self.cursor_col > m_start && self.cursor_col < m_end));
                                if is_active {
                                    bg = Palette::PEACH;
                                    fg = Palette::CRUST;
                                } else {
                                    bg = Palette::YELLOW;
                                    fg = Palette::CRUST;
                                }
                                break;
                            }
                        }

                        if let Some(last) = chunks.last_mut() {
                            if last.1 == fg && last.2 == bg {
                                last.0.push(c);
                                continue;
                            }
                        }
                        chunks.push((c.to_string(), fg, bg));
                    }

                    for (chunk_str, chunk_fg, chunk_bg) in chunks {
                        queue!(stdout, SetBackgroundColor(chunk_bg), SetForegroundColor(chunk_fg))?;
                        write!(stdout, "{}", chunk_str)?;
                        rendered_w += chunk_str.chars().count();
                    }

                    if rendered_w < available_w {
                        queue!(stdout, SetBackgroundColor(line_bg), SetForegroundColor(Palette::TEXT))?;
                        write!(stdout, "{}", " ".repeat(available_w - rendered_w))?;
                    }
                } else {
                    if gutter_w > 0 {
                        queue!(stdout, SetBackgroundColor(Palette::GUTTER_BG))?;
                        write!(stdout, "{}", " ".repeat(gutter_w))?;
                    }
                    queue!(stdout, SetBackgroundColor(Palette::BASE), SetForegroundColor(Palette::TEXT))?;
                    write!(stdout, "{}", " ".repeat(available_w))?;
                }
            }
        } else {
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

                if row_idx < self.lines.len() {
                    let line = &self.lines[row_idx];
                    let spans = highlight_line(line, self.lang);

                    let active_query: Option<&str> = match &self.prompt_state {
                        PromptState::Search { query, .. } => {
                            if !query.is_empty() {
                                Some(query.as_str())
                            } else {
                                self.last_search.as_deref().filter(|s| !s.is_empty())
                            }
                        }
                        _ => self.last_search.as_deref().filter(|s| !s.is_empty()),
                    };

                    let line_matches = if let Some(q) = active_query {
                        find_all_matches_in_line(line, q)
                    } else {
                        Vec::new()
                    };

                    if line_matches.is_empty() {
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
                            queue!(stdout, SetBackgroundColor(line_bg), SetForegroundColor(Palette::TEXT))?;
                            write!(stdout, "{}", " ".repeat(available_w - rendered_w))?;
                        }
                    } else {
                        // Line contains search matches: highlight them with Catppuccin Mocha colors
                        let mut line_chars_styled: Vec<(char, crossterm::style::Color)> = Vec::with_capacity(line.chars().count());
                        for span in spans {
                            for c in span.text.chars() {
                                line_chars_styled.push((c, span.color));
                            }
                        }

                        let visible_start = self.col_offset;
                        let visible_end = (self.col_offset + available_w).min(line_chars_styled.len());

                        let mut chunks: Vec<(String, crossterm::style::Color, crossterm::style::Color)> = Vec::new();

                        for c_idx in visible_start..visible_end {
                            let (c, syntax_fg) = line_chars_styled[c_idx];
                            let mut fg = syntax_fg;
                            let mut bg = line_bg;

                            for &(m_start, m_end) in &line_matches {
                                if c_idx >= m_start && c_idx < m_end {
                                    let is_active = row_idx == self.cursor_row
                                        && (self.cursor_col == m_start || (self.cursor_col > m_start && self.cursor_col < m_end));
                                    if is_active {
                                        bg = Palette::PEACH;
                                        fg = Palette::CRUST;
                                    } else {
                                        bg = Palette::YELLOW;
                                        fg = Palette::CRUST;
                                    }
                                    break;
                                }
                            }

                            if let Some(last) = chunks.last_mut() {
                                if last.1 == fg && last.2 == bg {
                                    last.0.push(c);
                                    continue;
                                }
                            }
                            chunks.push((c.to_string(), fg, bg));
                        }

                        let mut rendered_w: usize = 0;
                        for (chunk_str, chunk_fg, chunk_bg) in chunks {
                            queue!(
                                stdout,
                                SetBackgroundColor(chunk_bg),
                                SetForegroundColor(chunk_fg)
                            )?;
                            write!(stdout, "{}", chunk_str)?;
                            rendered_w += chunk_str.chars().count();
                        }

                        if rendered_w < available_w {
                            queue!(
                                stdout,
                                SetBackgroundColor(line_bg),
                                SetForegroundColor(Palette::TEXT)
                            )?;
                            write!(stdout, "{}", " ".repeat(available_w - rendered_w))?;
                        }
                    }
                } else {
                    write!(stdout, "{}", " ".repeat(available_w))?;
                }
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
                PromptState::Search { query, .. } => {
                    let active_q = if !query.is_empty() {
                        query.as_str()
                    } else if let Some(ref last) = self.last_search {
                        last.as_str()
                    } else {
                        ""
                    };

                    let (curr, total) = self.count_all_matches(active_q);
                    let count_str = if total > 0 {
                        format!(" [{}/{}]", curr, total)
                    } else if !active_q.is_empty() {
                        " [0/0]".to_string()
                    } else {
                        String::new()
                    };

                    if query.is_empty() {
                        if let Some(ref last) = self.last_search {
                            format!(" Search [{}]{}: ", last, count_str)
                        } else {
                            " Search: ".to_string()
                        }
                    } else {
                        format!(" Search: {}{}", query, count_str)
                    }
                }
                PromptState::Help { .. } => {
                    " [ Help & Shortcuts ]  Scroll: Up/Down/PgUp/PgDn  •  Press Esc to Exit".to_string()
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
        let f_h = self.footer_height();
        if f_h > 0 && current_terminal_row + f_h as u16 <= self.term_height {
            if f_h == 1 {
                let row = match &self.prompt_state {
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
                    PromptState::Search { .. } => [
                        ("Enter", "Next"),
                        ("Up", "Prev"),
                        ("Esc", "Done"),
                        ("^C", "Cancel"),
                        ("M-W", "Next"),
                        ("M-Q", "Prev"),
                    ],
                    PromptState::Help { .. } => [
                        ("Esc", "Exit"),
                        ("Up", "Prev"),
                        ("Down", "Next"),
                        ("PgUp", "PgUp"),
                        ("PgDn", "PgDn"),
                        ("^C", "Cancel"),
                    ],
                    PromptState::None => [
                        ("^O", "Save"),
                        ("^X", "Exit"),
                        ("^W", "WhereIs"),
                        ("^H", "Help"),
                        ("M-W", "Next"),
                        ("M-L", "Wrap"),
                    ],
                };
                self.render_shortcut_row(stdout, current_terminal_row, &row)?;
            } else {
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
                    PromptState::Search { .. } => [
                        ("Enter", "Next"),
                        ("Up", "Prev"),
                        ("Down", "Next"),
                        ("Esc", "Done"),
                        ("^C", "Cancel"),
                        ("", ""),
                    ],
                    PromptState::Help { .. } => [
                        ("Esc", "Exit"),
                        ("Up", "Prev"),
                        ("Down", "Next"),
                        ("PgUp", "PgUp"),
                        ("PgDn", "PgDn"),
                        ("^C", "Cancel"),
                    ],
                    PromptState::None => [
                        ("^O", "Save"),
                        ("^X", "Exit"),
                        ("^W", "WhereIs"),
                        ("^K", "Cut"),
                        ("^U", "Paste"),
                        ("M-W", "Next"),
                    ],
                };

                let row2 = match &self.prompt_state {
                    PromptState::Search { .. } => [
                        ("M-W", "Next"),
                        ("M-Q", "Prev"),
                        ("F3", "Next"),
                        ("S-F3", "Prev"),
                        ("", ""),
                        ("", ""),
                    ],
                    PromptState::Help { .. } => [
                        ("Home", "Top"),
                        ("End", "Bottom"),
                        ("q", "Exit"),
                        ("Enter", "Exit"),
                        ("", ""),
                        ("", ""),
                    ],
                    PromptState::None => [
                        ("M-Q", "Prev"),
                        ("^H", "Help"),
                        ("^N", "Num"),
                        ("^S", "Sep"),
                        ("M-L", "Wrap"),
                        ("^B", "Blink"),
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
        }

        // 5. Place cursor at exact position
        let (screen_cursor_x, screen_cursor_y) = match &self.prompt_state {
            PromptState::Help { .. } => (0, 0),
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
            PromptState::Search { query, .. } => {
                let prompt_prefix = if query.is_empty() {
                    if let Some(ref last) = self.last_search {
                        format!(" Search [{}]: ", last)
                    } else {
                        " Search: ".to_string()
                    }
                } else {
                    " Search: ".to_string()
                };
                let cursor_x = (prompt_prefix.chars().count() + query.chars().count())
                    .min(self.term_width as usize - 1) as u16;
                (cursor_x, status_row_y)
            }
            PromptState::None => {
                if self.config.warp_line {
                    let (target_chunk, col_in_chunk) = cursor_to_wrap_pos(self.cursor_col, available_w);
                    let row_pos = display_rows.iter().position(|r| {
                        r.as_ref().map_or(false, |d| d.line_idx == self.cursor_row && d.chunk_idx == target_chunk)
                    });

                    if let Some(screen_i) = row_pos {
                        let y = if self.config.top_bar {
                            1 + screen_i as u16
                        } else {
                            screen_i as u16
                        };
                        let prefix_w = if target_chunk > 0 { WRAP_ARROW.chars().count() } else { 0 };
                        let x = (gutter_w + prefix_w + col_in_chunk).min(self.term_width as usize - 1) as u16;
                        (x, y)
                    } else {
                        let y = if self.config.top_bar { 1 } else { 0 };
                        (gutter_w as u16, y)
                    }
                } else {
                    let y = if self.config.top_bar {
                        1 + (self.cursor_row - self.row_offset) as u16
                    } else {
                        (self.cursor_row - self.row_offset) as u16
                    };
                    let x = (gutter_w + (self.cursor_col - self.col_offset)) as u16;
                    (x, y)
                }
            }
        };

        if matches!(self.prompt_state, PromptState::Help { .. }) {
            queue!(stdout, cursor::Hide)?;
        } else {
            queue!(
                stdout,
                cursor::MoveTo(screen_cursor_x, screen_cursor_y),
                cursor::Show
            )?;
        }

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
        let active_items: Vec<(&str, &str)> = items
            .iter()
            .copied()
            .filter(|(k, _)| !k.trim().is_empty())
            .collect();

        let term_w = self.term_width as usize;
        queue!(
            stdout,
            cursor::MoveTo(0, y),
            SetBackgroundColor(Palette::MANTLE)
        )?;

        if active_items.is_empty() || term_w == 0 {
            write!(stdout, "{}", " ".repeat(term_w))?;
            return Ok(());
        }

        // Adaptive column count: Ensure items never overlap or wrap.
        // Minimum comfortable slot width is 7 characters (e.g. "^O Save")
        let max_fit = (term_w / 7).max(1);
        let count = active_items.len().min(max_fit);
        let items_to_show = &active_items[..count];

        let slot_w = term_w / count;
        let mut col_idx = 0;

        for (i, (key, desc)) in items_to_show.iter().enumerate() {
            let this_slot_w = if i + 1 == count {
                term_w.saturating_sub(col_idx)
            } else {
                slot_w
            };

            let k_trimmed = key.trim();
            let k_len = k_trimmed.chars().count();
            let d_len = desc.chars().count();

            // Total text length: k_len + 1 (space) + d_len
            let needed = k_len + 1 + d_len;
            let display_desc = if needed > this_slot_w {
                let available_d = this_slot_w.saturating_sub(k_len + 1);
                if available_d > 0 {
                    desc.chars().take(available_d).collect::<String>()
                } else {
                    String::new()
                }
            } else {
                desc.to_string()
            };

            // 1. Move to column and write key in Peach
            queue!(
                stdout,
                cursor::MoveTo(col_idx as u16, y),
                SetBackgroundColor(Palette::MANTLE),
                SetForegroundColor(Palette::PEACH)
            )?;
            write!(stdout, "{}", k_trimmed)?;

            // 2. Write space + description in Text color
            let mut written_w = k_len;
            if !display_desc.is_empty() {
                queue!(
                    stdout,
                    SetBackgroundColor(Palette::MANTLE),
                    SetForegroundColor(Palette::TEXT)
                )?;
                write!(stdout, " {}", display_desc)?;
                written_w += 1 + display_desc.chars().count();
            }

            // 3. Pad remaining space in this slot with Mantle spaces
            if this_slot_w > written_w {
                let pad = " ".repeat(this_slot_w - written_w);
                queue!(stdout, SetBackgroundColor(Palette::MANTLE))?;
                write!(stdout, "{}", pad)?;
            }

            col_idx += this_slot_w;
        }

        // Fill any trailing space on the line
        if col_idx < term_w {
            queue!(stdout, SetBackgroundColor(Palette::MANTLE))?;
            write!(stdout, "{}", " ".repeat(term_w - col_idx))?;
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
                PromptState::Search { query, origin } => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        if key.code == KeyCode::Char('c') || key.code == KeyCode::Char('C') {
                            let (r, c) = *origin;
                            self.cursor_row = r;
                            self.cursor_col = c;
                            self.prompt_state = PromptState::None;
                            self.set_status("[Search cancelled]");
                            return Ok(());
                        }
                        if key.code == KeyCode::Char('w') || key.code == KeyCode::Char('W') {
                            let q = if !query.is_empty() {
                                query.clone()
                            } else if let Some(ref last) = self.last_search {
                                last.clone()
                            } else {
                                return Ok(());
                            };
                            self.last_search = Some(q.clone());
                            let (found, wrapped) = self.find_next_match_wrapped(&q, true);
                            if found {
                                if wrapped {
                                    self.set_status(format!("Search: \"{}\" [Search wrapped]", q));
                                } else {
                                    self.set_status(format!("Search: \"{}\"", q));
                                }
                            } else {
                                self.set_status(format!("\"{}\" not found", q));
                            }
                            return Ok(());
                        }
                    }

                    if key.modifiers.contains(KeyModifiers::ALT) {
                        if key.code == KeyCode::Char('w') || key.code == KeyCode::Char('W') {
                            let q = if !query.is_empty() {
                                query.clone()
                            } else if let Some(ref last) = self.last_search {
                                last.clone()
                            } else {
                                return Ok(());
                            };
                            self.last_search = Some(q.clone());
                            let (found, wrapped) = self.find_next_match_wrapped(&q, true);
                            if found {
                                let (curr, total) = self.count_all_matches(&q);
                                if wrapped {
                                    self.set_status(format!("[Search wrapped] Search: \"{}\" [{}/{}]", q, curr, total));
                                } else {
                                    self.set_status("");
                                }
                            } else {
                                self.set_status(format!("\"{}\" not found", q));
                            }
                            return Ok(());
                        }
                        if key.code == KeyCode::Char('q') || key.code == KeyCode::Char('Q') {
                            let q = if !query.is_empty() {
                                query.clone()
                            } else if let Some(ref last) = self.last_search {
                                last.clone()
                            } else {
                                return Ok(());
                            };
                            self.last_search = Some(q.clone());
                            let (found, wrapped) = self.find_prev_match_wrapped(&q);
                            if found {
                                let (curr, total) = self.count_all_matches(&q);
                                if wrapped {
                                    self.set_status(format!("[Search wrapped] Search: \"{}\" [{}/{}]", q, curr, total));
                                } else {
                                    self.set_status("");
                                }
                            } else {
                                self.set_status(format!("\"{}\" not found", q));
                            }
                            return Ok(());
                        }
                    }

                    match key.code {
                        KeyCode::Enter => {
                            let q = if !query.is_empty() {
                                query.clone()
                            } else if let Some(ref last) = self.last_search {
                                last.clone()
                            } else {
                                self.prompt_state = PromptState::None;
                                self.set_status("[Search cancelled]");
                                return Ok(());
                            };

                            self.last_search = Some(q.clone());

                            // Jump to NEXT match without closing prompt!
                            let (found, wrapped) = self.find_next_match_wrapped(&q, true);
                            if found {
                                let (curr, total) = self.count_all_matches(&q);
                                if wrapped {
                                    self.set_status(format!("[Search wrapped] Search: \"{}\" [{}/{}]", q, curr, total));
                                } else {
                                    self.set_status("");
                                }
                            } else {
                                self.set_status(format!("\"{}\" not found", q));
                            }
                            return Ok(());
                        }
                        KeyCode::Down => {
                            let q = if !query.is_empty() {
                                query.clone()
                            } else if let Some(ref last) = self.last_search {
                                last.clone()
                            } else {
                                return Ok(());
                            };
                            self.last_search = Some(q.clone());
                            let (found, wrapped) = self.find_next_match_wrapped(&q, true);
                            if found {
                                let (curr, total) = self.count_all_matches(&q);
                                if wrapped {
                                    self.set_status(format!("[Search wrapped] Search: \"{}\" [{}/{}]", q, curr, total));
                                } else {
                                    self.set_status("");
                                }
                            } else {
                                self.set_status(format!("\"{}\" not found", q));
                            }
                            return Ok(());
                        }
                        KeyCode::Up => {
                            let q = if !query.is_empty() {
                                query.clone()
                            } else if let Some(ref last) = self.last_search {
                                last.clone()
                            } else {
                                return Ok(());
                            };
                            self.last_search = Some(q.clone());
                            let (found, wrapped) = self.find_prev_match_wrapped(&q);
                            if found {
                                let (curr, total) = self.count_all_matches(&q);
                                if wrapped {
                                    self.set_status(format!("[Search wrapped] Search: \"{}\" [{}/{}]", q, curr, total));
                                } else {
                                    self.set_status("");
                                }
                            } else {
                                self.set_status(format!("\"{}\" not found", q));
                            }
                            return Ok(());
                        }
                        KeyCode::Esc => {
                            // Finish search prompt and keep cursor at current match ready for editing!
                            if !query.is_empty() {
                                self.last_search = Some(query.clone());
                            }
                            self.prompt_state = PromptState::None;
                            let q = self.last_search.as_deref().unwrap_or("");
                            let (curr, total) = self.count_all_matches(q);
                            if total > 0 {
                                self.set_status(format!("Search: \"{}\" [{}/{}]", q, curr, total));
                            } else {
                                self.set_status("");
                            }
                            return Ok(());
                        }
                        KeyCode::Char(c)
                            if !key.modifiers.contains(KeyModifiers::CONTROL)
                                && !key.modifiers.contains(KeyModifiers::ALT) =>
                        {
                            query.push(c);
                            let q = query.clone();
                            let orig = *origin;
                            self.cursor_row = orig.0;
                            self.cursor_col = orig.1;
                            let (found, _) = self.find_next_match_wrapped(&q, false);
                            if !found {
                                self.set_status(format!("\"{}\" not found", q));
                            } else {
                                self.set_status("");
                            }
                            return Ok(());
                        }
                        KeyCode::Backspace => {
                            query.pop();
                            let q = query.clone();
                            let orig = *origin;
                            self.cursor_row = orig.0;
                            self.cursor_col = orig.1;
                            if !q.is_empty() {
                                let (found, _) = self.find_next_match_wrapped(&q, false);
                                if !found {
                                    self.set_status(format!("\"{}\" not found", q));
                                } else {
                                    self.set_status("");
                                }
                            } else {
                                self.set_status("");
                            }
                            return Ok(());
                        }
                        _ => return Ok(()),
                    }
                }
                PromptState::Help { scroll } => {
                    let help_lines = get_help_lines();
                    let total_help_lines = help_lines.len();
                    let page_size = self.term_height.saturating_sub(4) as usize;

                    if key.code == KeyCode::Esc
                        || key.code == KeyCode::Enter
                        || key.code == KeyCode::Char('q')
                        || key.code == KeyCode::Char('Q')
                        || (key.modifiers.contains(KeyModifiers::CONTROL)
                            && (key.code == KeyCode::Char('c')
                                || key.code == KeyCode::Char('C')
                                || key.code == KeyCode::Char('x')
                                || key.code == KeyCode::Char('X')
                                || key.code == KeyCode::Char('h')
                                || key.code == KeyCode::Char('H')))
                        || key.code == KeyCode::F(1)
                    {
                        self.prompt_state = PromptState::None;
                        self.set_status("");
                        return Ok(());
                    }

                    match key.code {
                        KeyCode::Up => {
                            *scroll = scroll.saturating_sub(1);
                            return Ok(());
                        }
                        KeyCode::Down => {
                            let max_scroll = total_help_lines.saturating_sub(page_size.min(total_help_lines));
                            if *scroll < max_scroll {
                                *scroll += 1;
                            }
                            return Ok(());
                        }
                        KeyCode::PageUp => {
                            *scroll = scroll.saturating_sub(page_size);
                            return Ok(());
                        }
                        KeyCode::PageDown => {
                            let max_scroll = total_help_lines.saturating_sub(page_size.min(total_help_lines));
                            *scroll = (*scroll + page_size).min(max_scroll);
                            return Ok(());
                        }
                        KeyCode::Home => {
                            *scroll = 0;
                            return Ok(());
                        }
                        KeyCode::End => {
                            let max_scroll = total_help_lines.saturating_sub(page_size.min(total_help_lines));
                            *scroll = max_scroll;
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
                    KeyCode::Char('w') | KeyCode::Char('W') => {
                        let origin = (self.cursor_row, self.cursor_col);
                        self.prompt_state = PromptState::Search {
                            query: String::new(),
                            origin,
                        };
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
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        if key.modifiers.contains(KeyModifiers::SHIFT) || key.code == KeyCode::Char('A') {
                            self.config.auto_completions = !self.config.auto_completions;
                            let state = if self.config.auto_completions { "ON" } else { "OFF" };
                            self.set_status(format!("Auto-completions: {}", state));
                            return Ok(());
                        }
                    }
                    KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Char('g') | KeyCode::Char('G') => {
                        self.prompt_state = PromptState::Help { scroll: 0 };
                        return Ok(());
                    }
                    KeyCode::Backspace => {
                        self.prompt_state = PromptState::Help { scroll: 0 };
                        return Ok(());
                    }
                    _ => {}
                }
            }

            // Alt hotkeys
            if key.modifiers.contains(KeyModifiers::ALT) && (key.code == KeyCode::Char('a') || key.code == KeyCode::Char('A')) {
                self.config.auto_completions = !self.config.auto_completions;
                let state = if self.config.auto_completions { "ON" } else { "OFF" };
                self.set_status(format!("Auto-completions: {}", state));
                return Ok(());
            }

            if key.modifiers.contains(KeyModifiers::ALT)
                && (key.code == KeyCode::Char('l')
                    || key.code == KeyCode::Char('L')
                    || key.code == KeyCode::Char('z')
                    || key.code == KeyCode::Char('Z'))
            {
                self.config.warp_line = !self.config.warp_line;
                let state = if self.config.warp_line { "ON" } else { "OFF" };
                self.set_status(format!("Wrap lines: {}", state));
                return Ok(());
            }

            let is_next = (key.modifiers.contains(KeyModifiers::ALT) && (key.code == KeyCode::Char('w') || key.code == KeyCode::Char('W')))
                || (key.code == KeyCode::F(3) && !key.modifiers.contains(KeyModifiers::SHIFT));

            if is_next {
                if let Some(ref q) = self.last_search.clone() {
                    let (found, wrapped) = self.find_next_match_wrapped(q, true);
                    if found {
                        let (curr, total) = self.count_all_matches(q);
                        if wrapped {
                            self.set_status(format!("[Search wrapped] Search: \"{}\" [{}/{}]", q, curr, total));
                        } else {
                            self.set_status(format!("Search: \"{}\" [{}/{}]", q, curr, total));
                        }
                    } else {
                        self.set_status(format!("\"{}\" not found", q));
                    }
                } else {
                    self.set_status("No previous search");
                }
                return Ok(());
            }

            let is_prev = (key.modifiers.contains(KeyModifiers::ALT) && (key.code == KeyCode::Char('q') || key.code == KeyCode::Char('Q')))
                || (key.code == KeyCode::F(3) && key.modifiers.contains(KeyModifiers::SHIFT))
                || key.code == KeyCode::F(4);

            if is_prev {
                if let Some(ref q) = self.last_search.clone() {
                    let (found, wrapped) = self.find_prev_match_wrapped(q);
                    if found {
                        let (curr, total) = self.count_all_matches(q);
                        if wrapped {
                            self.set_status(format!("[Search wrapped] Search: \"{}\" [{}/{}]", q, curr, total));
                        } else {
                            self.set_status(format!("Search: \"{}\" [{}/{}]", q, curr, total));
                        }
                    } else {
                        self.set_status(format!("\"{}\" not found", q));
                    }
                } else {
                    self.set_status("No previous search");
                }
                return Ok(());
            }

            // Normal typing and editing
            match key.code {
                KeyCode::F(1) => {
                    self.prompt_state = PromptState::Help { scroll: 0 };
                    return Ok(());
                }
                KeyCode::Esc => {
                    if self.last_search.is_some() {
                        self.last_search = None;
                        self.set_status("[Search highlights cleared]");
                    }
                }
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
                    if self.config.warp_line {
                        let gutter_w = self.gutter_width();
                        let available_w = (self.term_width as usize).saturating_sub(gutter_w).max(1);
                        let (chunk_idx, _) = cursor_to_wrap_pos(self.cursor_col, available_w);
                        let chunks = line_wrap_chunks(&self.lines[self.cursor_row], true, self.cursor_col, available_w);
                        if chunk_idx < chunks.len() {
                            let (start, _, _) = chunks[chunk_idx];
                            if self.cursor_col > start {
                                self.cursor_col = start;
                            } else {
                                self.cursor_col = 0;
                            }
                        } else {
                            self.cursor_col = 0;
                        }
                    } else {
                        self.cursor_col = 0;
                    }
                }
                KeyCode::End => {
                    let max_col = self.lines[self.cursor_row].chars().count();
                    if self.config.warp_line {
                        let gutter_w = self.gutter_width();
                        let available_w = (self.term_width as usize).saturating_sub(gutter_w).max(1);
                        let (chunk_idx, _) = cursor_to_wrap_pos(self.cursor_col, available_w);
                        let chunks = line_wrap_chunks(&self.lines[self.cursor_row], true, self.cursor_col, available_w);
                        if chunk_idx < chunks.len() {
                            let (_, end, _) = chunks[chunk_idx];
                            if self.cursor_col < end {
                                self.cursor_col = end;
                            } else {
                                self.cursor_col = max_col;
                            }
                        } else {
                            self.cursor_col = max_col;
                        }
                    } else {
                        self.cursor_col = max_col;
                    }
                }
                KeyCode::PageUp => {
                    let page_size = self.text_area_height();
                    if self.config.warp_line {
                        for _ in 0..page_size {
                            self.move_cursor_up();
                        }
                    } else {
                        self.cursor_row = self.cursor_row.saturating_sub(page_size);
                        self.clamp_cursor_col();
                    }
                }
                KeyCode::PageDown => {
                    let page_size = self.text_area_height();
                    if self.config.warp_line {
                        for _ in 0..page_size {
                            self.move_cursor_down();
                        }
                    } else {
                        self.cursor_row = (self.cursor_row + page_size).min(self.lines.len() - 1);
                        self.clamp_cursor_col();
                    }
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
        let auto_complete = self.config.auto_completions && self.lang.is_code();

        if auto_complete {
            let line = &self.lines[self.cursor_row];
            let chars: Vec<char> = line.chars().collect();
            let next_char = chars.get(self.cursor_col).copied();
            let prev_char = if self.cursor_col > 0 {
                chars.get(self.cursor_col - 1).copied()
            } else {
                None
            };

            // 1. Skip over closing bracket/quote if already at cursor
            if Some(c) == next_char
                && (c == ')' || c == ']' || c == '}' || c == '"' || c == '\'' || c == '`')
            {
                self.cursor_col += 1;
                return;
            }

            // 2. Auto-pair opening characters
            let pair = match c {
                '(' => Some(')'),
                '[' => Some(']'),
                '{' => Some('}'),
                '"' => Some('"'),
                '`' => Some('`'),
                '\'' => {
                    // Don't auto-pair if preceded by alphanumeric (like lifetime 'a or don't)
                    if let Some(prev) = prev_char {
                        if prev.is_alphanumeric() {
                            None
                        } else {
                            Some('\'')
                        }
                    } else {
                        Some('\'')
                    }
                }
                _ => None,
            };

            if let Some(closing) = pair {
                let line_mut = &mut self.lines[self.cursor_row];
                let mut line_chars: Vec<char> = line_mut.chars().collect();
                if self.cursor_col >= line_chars.len() {
                    line_chars.push(c);
                    line_chars.push(closing);
                } else {
                    line_chars.insert(self.cursor_col, c);
                    line_chars.insert(self.cursor_col + 1, closing);
                }
                *line_mut = line_chars.into_iter().collect();
                self.cursor_col += 1;
                self.update_modified_status();
                return;
            }
        }

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
        let auto_complete = self.config.auto_completions && self.lang.is_code();

        // Smart Enter between { and }
        if auto_complete
            && self.cursor_col > 0
            && chars.get(self.cursor_col - 1) == Some(&'{')
            && chars.get(self.cursor_col) == Some(&'}')
        {
            let leading_spaces: usize = current_line.chars().take_while(|c| *c == ' ').count();
            let base_indent = " ".repeat(leading_spaces);
            let inner_indent = " ".repeat(leading_spaces + 4);

            let (left, right) = chars.split_at(self.cursor_col);
            let new_left: String = left.iter().collect();
            let new_right: String = right.iter().collect();

            self.lines[self.cursor_row] = new_left;
            self.lines.insert(self.cursor_row + 1, inner_indent.clone());
            self.lines.insert(
                self.cursor_row + 2,
                format!("{}{}", base_indent, new_right.trim_start()),
            );

            self.cursor_row += 1;
            self.cursor_col = inner_indent.len();
            self.update_modified_status();
            return;
        }

        // Standard newline with auto-indentation on code files
        let leading_spaces: usize = if auto_complete {
            current_line.chars().take_while(|c| *c == ' ').count()
        } else {
            0
        };
        let indent = " ".repeat(leading_spaces);

        let (left, right) = if self.cursor_col >= chars.len() {
            (chars.as_slice(), &[][..])
        } else {
            chars.split_at(self.cursor_col)
        };

        let new_left: String = left.iter().collect();
        let new_right: String = format!("{}{}", indent, right.iter().collect::<String>().trim_start());

        self.lines[self.cursor_row] = new_left;
        self.lines.insert(self.cursor_row + 1, new_right);

        self.cursor_row += 1;
        self.cursor_col = leading_spaces;
        self.update_modified_status();
    }

    pub fn delete_backspace(&mut self) {
        if self.cursor_col > 0 {
            let auto_complete = self.config.auto_completions && self.lang.is_code();
            let line = &mut self.lines[self.cursor_row];
            let mut chars: Vec<char> = line.chars().collect();

            // Smart backspace: delete both opening and closing if empty pair
            if auto_complete {
                let prev = chars[self.cursor_col - 1];
                let next = chars.get(self.cursor_col).copied();
                let is_pair = matches!(
                    (prev, next),
                    ('(', Some(')'))
                        | ('[', Some(']'))
                        | ('{', Some('}'))
                        | ('"', Some('"'))
                        | ('\'', Some('\''))
                        | ('`', Some('`'))
                );
                if is_pair {
                    chars.remove(self.cursor_col);
                    chars.remove(self.cursor_col - 1);
                    *line = chars.into_iter().collect();
                    self.cursor_col -= 1;
                    self.update_modified_status();
                    return;
                }
            }

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
        if self.config.warp_line {
            let gutter_w = self.gutter_width();
            let available_w = (self.term_width as usize).saturating_sub(gutter_w).max(1);
            let (chunk_idx, col_in_chunk) = cursor_to_wrap_pos(self.cursor_col, available_w);

            if chunk_idx > 0 {
                let chunks = line_wrap_chunks(&self.lines[self.cursor_row], true, self.cursor_col, available_w);
                if chunk_idx - 1 < chunks.len() {
                    let (start, end, _) = chunks[chunk_idx - 1];
                    let chunk_len = end.saturating_sub(start);
                    self.cursor_col = start + col_in_chunk.min(chunk_len);
                    return;
                }
            } else if self.cursor_row > 0 {
                self.cursor_row -= 1;
                let prev_chunks = line_wrap_chunks(&self.lines[self.cursor_row], false, 0, available_w);
                if let Some(&(start, end, _)) = prev_chunks.last() {
                    let chunk_len = end.saturating_sub(start);
                    self.cursor_col = start + col_in_chunk.min(chunk_len);
                } else {
                    self.cursor_col = 0;
                }
                return;
            }
        }

        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.clamp_cursor_col();
        }
    }

    fn move_cursor_down(&mut self) {
        if self.config.warp_line {
            let gutter_w = self.gutter_width();
            let available_w = (self.term_width as usize).saturating_sub(gutter_w).max(1);
            let (chunk_idx, col_in_chunk) = cursor_to_wrap_pos(self.cursor_col, available_w);
            let chunks = line_wrap_chunks(&self.lines[self.cursor_row], false, 0, available_w);

            if chunk_idx + 1 < chunks.len() {
                let (start, end, _) = chunks[chunk_idx + 1];
                let chunk_len = end.saturating_sub(start);
                self.cursor_col = start + col_in_chunk.min(chunk_len);
                return;
            } else if self.cursor_row + 1 < self.lines.len() {
                self.cursor_row += 1;
                let next_chunks = line_wrap_chunks(&self.lines[self.cursor_row], false, 0, available_w);
                if let Some(&(start, end, _)) = next_chunks.first() {
                    let chunk_len = end.saturating_sub(start);
                    self.cursor_col = start + col_in_chunk.min(chunk_len);
                } else {
                    self.cursor_col = 0;
                }
                return;
            }
        }

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

    #[test]
    fn test_auto_pair_and_skip_over_code_file() {
        let mut editor = Editor::new(Some(PathBuf::from("test.c")), Config::default());
        assert_eq!(editor.lang, Language::C);
        assert!(editor.lang.is_code());

        // Type '(' -> should insert "()" with cursor in between (col 1)
        editor.insert_char('(');
        assert_eq!(editor.lines[0], "()");
        assert_eq!(editor.cursor_col, 1);

        // Type ')' -> should skip over, not duplicate
        editor.insert_char(')');
        assert_eq!(editor.lines[0], "()");
        assert_eq!(editor.cursor_col, 2);

        // Type '{' -> should append "{}" and place cursor inside
        editor.insert_char('{');
        assert_eq!(editor.lines[0], "(){}");
        assert_eq!(editor.cursor_col, 3);

        // Press Enter inside '{}' -> should expand to 3 lines with 4 spaces indent
        editor.insert_newline();
        assert_eq!(editor.lines.len(), 3);
        assert_eq!(editor.lines[0], "(){");
        assert_eq!(editor.lines[1], "    ");
        assert_eq!(editor.lines[2], "}");
        assert_eq!(editor.cursor_row, 1);
        assert_eq!(editor.cursor_col, 4);
    }

    #[test]
    fn test_auto_pair_smart_backspace() {
        let mut editor = Editor::new(Some(PathBuf::from("test.rs")), Config::default());
        editor.insert_char('[');
        assert_eq!(editor.lines[0], "[]");
        assert_eq!(editor.cursor_col, 1);

        editor.delete_backspace();
        assert_eq!(editor.lines[0], "");
        assert_eq!(editor.cursor_col, 0);
    }

    #[test]
    fn test_auto_pair_disabled_on_plain_text() {
        let mut editor = Editor::new(Some(PathBuf::from("notes.txt")), Config::default());
        assert_eq!(editor.lang, Language::Plain);
        assert!(!editor.lang.is_code());

        // Type '(' on txt file -> should insert single '(' without auto-closing
        editor.insert_char('(');
        assert_eq!(editor.lines[0], "(");
        assert_eq!(editor.cursor_col, 1);
    }

    #[test]
    fn test_find_all_matches_in_line() {
        let line = "Hello World hello HELLO";
        let matches = find_all_matches_in_line(line, "hello");
        assert_eq!(matches, vec![(0, 5), (12, 17), (18, 23)]);

        // Unicode line matching
        let uni_line = "🦀 Rust code rust 🦀";
        let uni_matches = find_all_matches_in_line(uni_line, "rust");
        assert_eq!(uni_matches, vec![(2, 6), (12, 16)]);

        // Empty cases
        assert!(find_all_matches_in_line("", "test").is_empty());
        assert!(find_all_matches_in_line("test", "").is_empty());
    }

    #[test]
    fn test_find_next_match_and_wrap_around() {
        let mut editor = Editor::new(None, Config::default());
        editor.lines = vec![
            "first line match".to_string(),
            "second line nothing".to_string(),
            "third line match here".to_string(),
        ];
        editor.cursor_row = 0;
        editor.cursor_col = 0;

        // 1. Search for "match" starting at (0, 0)
        let found = editor.find_next_match("match", false);
        assert!(found);
        assert_eq!(editor.cursor_row, 0);
        assert_eq!(editor.cursor_col, 11);

        // 2. Find next match (skip current)
        let found = editor.find_next_match("match", true);
        assert!(found);
        assert_eq!(editor.cursor_row, 2);
        assert_eq!(editor.cursor_col, 11);

        // 3. Find next match again (wrap around to line 0)
        let (found, wrapped) = editor.find_next_match_wrapped("match", true);
        assert!(found);
        assert!(wrapped);
        assert_eq!(editor.cursor_row, 0);
        assert_eq!(editor.cursor_col, 11);

        // 4. Search for nonexistent term
        let found = editor.find_next_match("nonexistent", false);
        assert!(!found);
    }

    #[test]
    fn test_count_all_matches_and_find_prev() {
        let mut editor = Editor::new(None, Config::default());
        editor.lines = vec![
            "pub fn one() {}".to_string(),
            "pub fn two() {}".to_string(),
            "pub fn three() {}".to_string(),
        ];
        editor.cursor_row = 0;
        editor.cursor_col = 0;

        let (curr, total) = editor.count_all_matches("pub");
        assert_eq!(total, 3);
        assert_eq!(curr, 1);

        // Move to line 1
        editor.cursor_row = 1;
        editor.cursor_col = 0;
        let (curr, total) = editor.count_all_matches("pub");
        assert_eq!(total, 3);
        assert_eq!(curr, 2);

        // Find prev match from line 1 -> should go to line 0
        let (found, wrapped) = editor.find_prev_match_wrapped("pub");
        assert!(found);
        assert!(!wrapped);
        assert_eq!(editor.cursor_row, 0);

        // Find prev match from line 0 -> should wrap around to line 2
        let (found, wrapped) = editor.find_prev_match_wrapped("pub");
        assert!(found);
        assert!(wrapped);
        assert_eq!(editor.cursor_row, 2);
    }

    #[test]
    fn test_search_prompt_realtime_and_cancel_with_ctrl_c() {
        let mut editor = Editor::new(None, Config::default());
        editor.lines = vec![
            "fn main() {".to_string(),
            "    let count = 42;".to_string(),
            "    println!(\"{}\", count);".to_string(),
            "}".to_string(),
        ];
        editor.cursor_row = 0;
        editor.cursor_col = 0;

        // Press Ctrl+W to start search
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Char('w'),
                KeyModifiers::CONTROL,
            )))
            .unwrap();

        assert_eq!(
            editor.prompt_state,
            PromptState::Search {
                query: String::new(),
                origin: (0, 0)
            }
        );

        // Type 'c', 'o', 'u', 'n', 't'
        for ch in "count".chars() {
            editor
                .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                    KeyCode::Char(ch),
                    KeyModifiers::NONE,
                )))
                .unwrap();
        }

        // Real-time incremental search jumped to line 1, col 8 ("count")
        assert_eq!(editor.cursor_row, 1);
        assert_eq!(editor.cursor_col, 8);

        // Cancel with Ctrl+C -> cursor restores to origin (0, 0)
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL,
            )))
            .unwrap();

        assert_eq!(editor.prompt_state, PromptState::None);
        assert_eq!(editor.cursor_row, 0);
        assert_eq!(editor.cursor_col, 0);
        assert_eq!(editor.status_message, "[Search cancelled]");
    }

    #[test]
    fn test_search_prompt_enter_cycle_and_esc_finish() {
        let mut editor = Editor::new(None, Config::default());
        editor.lines = vec![
            "let alpha = 1;".to_string(),
            "let beta = 2;".to_string(),
            "let alpha_again = 3;".to_string(),
        ];
        editor.cursor_row = 0;
        editor.cursor_col = 0;

        // Start search
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Char('w'),
                KeyModifiers::CONTROL,
            )))
            .unwrap();

        for ch in "alpha".chars() {
            editor
                .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                    KeyCode::Char(ch),
                    KeyModifiers::NONE,
                )))
                .unwrap();
        }

        // Initially at match 1 (line 0, col 4)
        assert_eq!(editor.cursor_row, 0);
        assert_eq!(editor.cursor_col, 4);

        // Press Enter to cycle to NEXT match (line 2, col 4) without exiting prompt!
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Enter,
                KeyModifiers::NONE,
            )))
            .unwrap();

        assert!(matches!(editor.prompt_state, PromptState::Search { .. }));
        assert_eq!(editor.cursor_row, 2);
        assert_eq!(editor.cursor_col, 4);

        // Press Up to navigate to PREVIOUS match (line 0, col 4)
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Up,
                KeyModifiers::NONE,
            )))
            .unwrap();

        assert_eq!(editor.cursor_row, 0);
        assert_eq!(editor.cursor_col, 4);

        // Press Esc to finish search and edit at current match
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Esc,
                KeyModifiers::NONE,
            )))
            .unwrap();

        assert_eq!(editor.prompt_state, PromptState::None);
        assert_eq!(editor.last_search, Some("alpha".to_string()));
        assert_eq!(editor.cursor_row, 0);
        assert_eq!(editor.cursor_col, 4);

        // Press Alt+W to repeat last search in normal mode (Find Next)
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Char('w'),
                KeyModifiers::ALT,
            )))
            .unwrap();

        assert_eq!(editor.cursor_row, 2);
        assert_eq!(editor.cursor_col, 4);

        // Press Alt+Q to repeat last search backwards (Find Prev)
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Char('q'),
                KeyModifiers::ALT,
            )))
            .unwrap();

        assert_eq!(editor.cursor_row, 0);
        assert_eq!(editor.cursor_col, 4);

        // Press Esc in normal mode -> clears search highlights
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Esc,
                KeyModifiers::NONE,
            )))
            .unwrap();

        assert_eq!(editor.last_search, None);
        assert_eq!(editor.status_message, "[Search highlights cleared]");
    }

    #[test]
    fn test_adaptive_footer_height() {
        let mut editor = Editor::new(None, Config::default());
        editor.config.footer = true;

        // Auto mode: <= 18 rows is 1 (compact on mobile/Android), > 18 is 2 (desktop)
        editor.config.compact_footer = CompactFooterMode::Auto;
        editor.term_height = 14;
        assert_eq!(editor.footer_height(), 1);

        editor.term_height = 18;
        assert_eq!(editor.footer_height(), 1);

        editor.term_height = 24;
        assert_eq!(editor.footer_height(), 2);

        // Always compact mode: always 1
        editor.config.compact_footer = CompactFooterMode::Always;
        editor.term_height = 50;
        assert_eq!(editor.footer_height(), 1);

        // Never compact mode: always 2
        editor.config.compact_footer = CompactFooterMode::Never;
        editor.term_height = 10;
        assert_eq!(editor.footer_height(), 2);

        // Footer disabled
        editor.config.footer = false;
        assert_eq!(editor.footer_height(), 0);
    }

    #[test]
    fn test_line_wrap_chunks_and_cursor_pos() {
        let available_w = 20; // 20 cols. prefix "|_> " is 4 cols. cont_cap = 16 cols.

        // Short line: 10 chars
        let chunks_short = line_wrap_chunks("hello rust", false, 0, available_w);
        assert_eq!(chunks_short.len(), 1);
        assert_eq!(chunks_short[0], (0, 10, false));

        // Long line: 40 chars
        let long_line = "0123456789012345678901234567890123456789"; // 40 chars
        let chunks_long = line_wrap_chunks(long_line, false, 0, available_w);
        // Chunk 0: 0..20 (20 chars)
        // Chunk 1: 20..36 (16 chars)
        // Chunk 2: 36..40 (4 chars)
        assert_eq!(chunks_long.len(), 3);
        assert_eq!(chunks_long[0], (0, 20, false));
        assert_eq!(chunks_long[1], (20, 36, true));
        assert_eq!(chunks_long[2], (36, 40, true));

        // Test cursor position mapping
        assert_eq!(cursor_to_wrap_pos(0, available_w), (0, 0));
        assert_eq!(cursor_to_wrap_pos(19, available_w), (0, 19));
        assert_eq!(cursor_to_wrap_pos(20, available_w), (1, 0));
        assert_eq!(cursor_to_wrap_pos(25, available_w), (1, 5));
        assert_eq!(cursor_to_wrap_pos(36, available_w), (2, 0));
        assert_eq!(cursor_to_wrap_pos(40, available_w), (2, 4));
    }

    #[test]
    fn test_line_wrap_cursor_up_down_navigation() {
        let mut editor = Editor::new(None, Config::default());
        editor.config.warp_line = true;
        editor.term_width = 24; // Gutter 4 cols + text 20 cols
        editor.config.line_numbers = true;
        editor.config.separator = false; // gutter_w = 4, available_w = 20

        editor.lines = vec![
            "Short line".to_string(), // 10 chars -> 1 chunk
            "012345678901234567890123456789012345".to_string(), // 36 chars -> 2 chunks: 0..20, 20..36
            "End line".to_string(),
        ];

        editor.cursor_row = 1;
        editor.cursor_col = 5; // Chunk 0, col 5

        // Move Down -> jumps to Chunk 1 (col 20 + 5 = 25)
        editor.move_cursor_down();
        assert_eq!(editor.cursor_row, 1);
        assert_eq!(editor.cursor_col, 25);

        // Move Down again -> jumps to line 2 (End line), col 5
        editor.move_cursor_down();
        assert_eq!(editor.cursor_row, 2);
        assert_eq!(editor.cursor_col, 5);

        // Move Up -> jumps back to Chunk 1 of line 1 (col 25)
        editor.move_cursor_up();
        assert_eq!(editor.cursor_row, 1);
        assert_eq!(editor.cursor_col, 25);

        // Move Up -> jumps to Chunk 0 of line 1 (col 5)
        editor.move_cursor_up();
        assert_eq!(editor.cursor_row, 1);
        assert_eq!(editor.cursor_col, 5);

        // Move Up -> jumps to line 0
        editor.move_cursor_up();
        assert_eq!(editor.cursor_row, 0);
        assert_eq!(editor.cursor_col, 5);
    }

    #[test]
    fn test_line_wrap_home_end_and_alt_l_toggle() {
        let mut editor = Editor::new(None, Config::default());
        editor.config.warp_line = true;
        editor.term_width = 24; // available_w = 20
        editor.config.line_numbers = true;
        editor.config.separator = false;

        editor.lines = vec![
            "012345678901234567890123456789012345".to_string(), // 36 chars: Chunk 0 [0..20], Chunk 1 [20..36]
        ];

        // Cursor at col 25 (inside Chunk 1 [20..36])
        editor.cursor_row = 0;
        editor.cursor_col = 25;

        // Press Home -> moves to start of chunk (20)
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Home,
                KeyModifiers::NONE,
            )))
            .unwrap();
        assert_eq!(editor.cursor_col, 20);

        // Press Home again -> moves to start of line (0)
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Home,
                KeyModifiers::NONE,
            )))
            .unwrap();
        assert_eq!(editor.cursor_col, 0);

        // Press End -> moves to end of Chunk 0 (20)
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::End,
                KeyModifiers::NONE,
            )))
            .unwrap();
        assert_eq!(editor.cursor_col, 20);

        // Press End again -> moves to end of entire line (36)
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::End,
                KeyModifiers::NONE,
            )))
            .unwrap();
        assert_eq!(editor.cursor_col, 36);

        // Press Alt+L -> toggle wrap lines off
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Char('l'),
                KeyModifiers::ALT,
            )))
            .unwrap();
        assert!(!editor.config.warp_line);
        assert_eq!(editor.status_message, "Wrap lines: OFF");

        // Press Alt+L again -> toggle wrap lines on
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Char('l'),
                KeyModifiers::ALT,
            )))
            .unwrap();
        assert!(editor.config.warp_line);
        assert_eq!(editor.status_message, "Wrap lines: ON");
    }

    #[test]
    fn test_help_screen_navigation_and_esc_exit() {
        let mut editor = Editor::new(None, Config::default());
        editor.lines = vec!["Hello world".to_string()];
        editor.cursor_row = 0;
        editor.cursor_col = 5;
        editor.term_width = 80;
        editor.term_height = 24;

        // 1. Trigger Help with Ctrl+H
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Char('h'),
                KeyModifiers::CONTROL,
            )))
            .unwrap();
        assert_eq!(editor.prompt_state, PromptState::Help { scroll: 0 });

        // 2. Scroll down in Help
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Down,
                KeyModifiers::NONE,
            )))
            .unwrap();
        assert_eq!(editor.prompt_state, PromptState::Help { scroll: 1 });

        // 3. Scroll up in Help
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Up,
                KeyModifiers::NONE,
            )))
            .unwrap();
        assert_eq!(editor.prompt_state, PromptState::Help { scroll: 0 });

        // 4. Press Esc to exit Help back to editor
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Esc,
                KeyModifiers::NONE,
            )))
            .unwrap();
        assert_eq!(editor.prompt_state, PromptState::None);
        assert_eq!(editor.cursor_row, 0);
        assert_eq!(editor.cursor_col, 5);

        // 5. Open with F1
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::F(1),
                KeyModifiers::NONE,
            )))
            .unwrap();
        assert_eq!(editor.prompt_state, PromptState::Help { scroll: 0 });

        // Exit with 'q'
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Char('q'),
                KeyModifiers::NONE,
            )))
            .unwrap();
        assert_eq!(editor.prompt_state, PromptState::None);

        // 6. Open with Ctrl+Backspace
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Backspace,
                KeyModifiers::CONTROL,
            )))
            .unwrap();
        assert_eq!(editor.prompt_state, PromptState::Help { scroll: 0 });

        // Exit with Ctrl+C
        editor
            .handle_event(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL,
            )))
            .unwrap();
        assert_eq!(editor.prompt_state, PromptState::None);
    }
}

