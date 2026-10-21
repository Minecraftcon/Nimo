# 🚀 Nimo

> **Nano-Inspired Modern High-Performance Terminal Editor written in Rust**  
> Blazing Fast • Zero Terminal Artifacts • Instant Startup • 24-bit Catppuccin Mocha Aesthetics

---

## ✨ Features

- **Extreme Performance & Safety**: Written in high-performance **Rust** with RAII terminal guards. Guarantees zero terminal corruption, cursor state loss, or broken terminal modes on exit.
- **Catppuccin Mocha Truecolor Palette**: Full 24-bit RGB ANSI terminal styling with Mauve header bars, Surface0 gutter/line-numbering, Peach shortcut pills, and smooth cursor indicators.
- **Dynamic Line Numbers (Type-As-You-Go)**:
  - Line numbers **appear only as you type**! Empty trailing rows on new buffers remain clean until text is written or the cursor moves there.
- **Complete `.nimorc` Configuration System**:
  - Configure cursor blinking, line numbers, dynamic type-only modes, gutter separators, styles, and interface bars via `.nimorc`.
- **Customizable Separator Styles**:
  - `pipe` (` │ `) — Modern clean box divider
  - `thin` (` | `) — Classic ASCII pipe
  - `double` (` ║ `) — Bold double-line accent
  - `dots` (` ┆ `) — Subtle dotted line
  - `slash` (` // `) — Code style dual slash
  - `none` (`   `) — Invisible spacing
- **Live Runtime Toggles**:
  - `Ctrl+N` / `Alt+N` — Toggle Line Numbers ON / OFF on the fly
  - `Ctrl+S` / `Alt+S` — Toggle Separator ON / OFF
  - `Ctrl+B` / `Alt+B` — Toggle Blinking vs Steady Cursor
  - `Ctrl+P` / `Alt+P` — Cycle Separator Styles instantly
- **Double-Buffered Atomic Rendering**: 256 KB memory render buffer flushed in a single `sys_write` syscall per frame — completely eliminating terminal flicker at high refresh rates.
- **Nano-Familiar Shortcuts**:
  - `^O` (Ctrl+O) — **WriteOut** (Save buffer directly to disk)
  - `^X` (Ctrl+X) — **Exit** cleanly and restore cooked terminal mode
  - `^K` (Ctrl+K) — **Cut Line** to memory cut buffer
  - `^U` (Ctrl+U) — **Paste Line** from cut buffer at cursor
  - `^C` (Ctrl+C) — **Cur Pos** (Live line & column counter)
  - `Arrow Keys`, `Home`, `End`, `PageUp`, `PageDown` — Fast viewport navigation

---

## ⚙️ Configuration (`.nimorc`)

Nimo automatically loads `.nimorc` on startup:

```ini
# Blinking cursor: true for blinking block, false for steady block
BlinkingCursor = true

# Master toggle for line numbers
LineNumbers = true

# Line numbers appear only as you type
LineNumbersTypeOnly = true

# Master toggle for vertical separator
Separator = true

# Separator style: pipe | thin | double | dots | slash | none
SeparatorStyle = pipe

# Interface component toggles
TopBar = true
StatusBar = true
Footer = true
```

---

## 🛠️ Build & Installation

### Requirements
- Rust & Cargo (1.70+)

### Compiling & Setup

Using the universal build script (dynamically detects Termux or Linux desktop):
```bash
# Build binary to ./nimo
./build.sh

# Or install system-wide ($PREFIX/bin on Termux, /usr/local/bin on Linux)
./build.sh install
```

Or manually with Cargo:
```bash
cargo build --release
cp target/release/nimo ./nimo
```

### Running Nimo
```bash
# Open an existing file or create a new file
./nimo myfile.txt

# Or open an empty buffer
./nimo

# View CLI help and toggles
./nimo --help
```

---

## ⌨️ Shortcut & Toggle Reference

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Ctrl+O` | WriteOut | Save active file buffer to disk |
| `Ctrl+X` | Exit | Clean exit & restore terminal |
| `Ctrl+K` | Cut Line | Cut entire current line to cut buffer |
| `Ctrl+U` | Paste Line | Paste cut buffer at cursor position |
| `Ctrl+C` | Cur Pos | Display current line & column |
| `Ctrl+N` / `Alt+N` | Toggle Line Numbers | Turn line numbers ON / OFF live |
| `Ctrl+S` / `Alt+S` | Toggle Separator | Turn gutter divider ON / OFF live |
| `Ctrl+B` / `Alt+B` | Toggle Cursor Blink | Toggle blinking vs steady block cursor |
| `Ctrl+P` / `Alt+P` | Cycle Separator Style | Cycle through `pipe` → `thin` → `double` → `dots` → `slash` → `none` |
