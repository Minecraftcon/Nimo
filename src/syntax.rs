use std::path::Path;
use crossterm::style::Color;
use crate::theme::Palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    C,
    Cpp,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    Html,
    Css,
    Json,
    Toml,
    Yaml,
    Sh,
    Assembly,
    Markdown,
    Plain,
}

impl Language {
    pub fn is_code(&self) -> bool {
        !matches!(self, Language::Plain)
    }

    pub fn from_path(path: Option<&Path>) -> Self {
        let path = match path {
            Some(p) => p,
            None => return Language::Plain,
        };

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        if file_name == "dockerfile" {
            return Language::Sh;
        }
        if file_name == "makefile" || file_name == "gnuitemakefile" {
            return Language::Sh;
        }
        if file_name == "cargo.lock" {
            return Language::Toml;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "rs" => Language::Rust,
            "c" | "h" => Language::C,
            "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => Language::Cpp,
            "py" | "pyw" => Language::Python,
            "js" | "mjs" | "cjs" | "jsx" => Language::JavaScript,
            "ts" | "mts" | "cts" | "tsx" => Language::TypeScript,
            "go" => Language::Go,
            "java" | "kt" | "kts" | "scala" => Language::Java,
            "html" | "htm" | "xml" | "svg" => Language::Html,
            "css" | "scss" | "sass" | "less" => Language::Css,
            "json" => Language::Json,
            "toml" => Language::Toml,
            "yaml" | "yml" => Language::Yaml,
            "sh" | "bash" | "zsh" | "fish" => Language::Sh,
            "s" | "asm" | "inc" => Language::Assembly,
            "md" | "markdown" => Language::Markdown,
            _ => Language::Plain,
        }
    }

    pub fn line_comment_prefix(&self) -> Option<&'static str> {
        match self {
            Language::Rust
            | Language::C
            | Language::Cpp
            | Language::JavaScript
            | Language::TypeScript
            | Language::Go
            | Language::Java => Some("//"),
            Language::Python
            | Language::Sh
            | Language::Yaml
            | Language::Toml => Some("#"),
            Language::Assembly => Some("#"), // also support ; in parser
            _ => None,
        }
    }

    pub fn keywords(&self) -> &'static [&'static str] {
        match self {
            Language::Rust => &[
                "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else",
                "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop",
                "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static",
                "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while",
            ],
            Language::C | Language::Cpp => &[
                "auto", "break", "case", "char", "const", "continue", "default", "do", "double",
                "else", "enum", "extern", "float", "for", "goto", "if", "inline", "int", "long",
                "register", "restrict", "return", "short", "signed", "sizeof", "static", "struct",
                "switch", "typedef", "union", "unsigned", "void", "volatile", "while", "class",
                "public", "private", "protected", "template", "typename", "new", "delete",
                "namespace", "using", "true", "false", "nullptr", "virtual", "override",
            ],
            Language::Python => &[
                "and", "as", "assert", "async", "await", "break", "class", "continue", "def",
                "del", "elif", "else", "except", "False", "finally", "for", "from", "global",
                "if", "import", "in", "is", "lambda", "None", "nonlocal", "not", "or", "pass",
                "raise", "return", "True", "try", "while", "with", "yield", "self",
            ],
            Language::JavaScript | Language::TypeScript => &[
                "break", "case", "catch", "class", "const", "continue", "debugger", "default",
                "delete", "do", "else", "export", "extends", "false", "finally", "for",
                "function", "if", "import", "in", "instanceof", "new", "null", "return", "super",
                "switch", "this", "throw", "true", "try", "typeof", "var", "void", "while",
                "with", "yield", "let", "static", "async", "await", "interface", "type",
                "namespace", "from", "of",
            ],
            Language::Go => &[
                "break", "default", "func", "interface", "select", "case", "defer", "go", "map",
                "struct", "chan", "else", "goto", "package", "switch", "const", "fallthrough",
                "if", "range", "type", "continue", "for", "import", "return", "var", "true",
                "false", "nil",
            ],
            Language::Java => &[
                "abstract", "assert", "boolean", "break", "byte", "case", "catch", "char", "class",
                "const", "continue", "default", "do", "double", "else", "enum", "extends", "final",
                "finally", "float", "for", "goto", "if", "implements", "import", "instanceof", "int",
                "interface", "long", "native", "new", "package", "private", "protected", "public",
                "return", "short", "static", "strictfp", "super", "switch", "synchronized", "this",
                "throw", "throws", "transient", "try", "void", "volatile", "while", "true", "false",
                "null",
            ],
            Language::Assembly => &[
                "mov", "add", "sub", "mul", "imul", "div", "idiv", "inc", "dec", "xor", "and",
                "or", "not", "jmp", "je", "jne", "jz", "jnz", "jg", "jge", "jl", "jle", "ja",
                "jae", "jb", "jbe", "call", "ret", "push", "pop", "lea", "cmp", "test", "syscall",
                "nop", "int", "global", "extern", "section", "byte", "word", "dword", "qword", "ptr",
            ],
            Language::Sh => &[
                "if", "then", "else", "elif", "fi", "case", "esac", "for", "select", "while",
                "until", "do", "done", "in", "function", "time", "echo", "export", "return",
                "exit", "local",
            ],
            _ => &[],
        }
    }

    pub fn types(&self) -> &'static [&'static str] {
        match self {
            Language::Rust => &[
                "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128",
                "isize", "f32", "f64", "bool", "char", "str", "String", "Vec", "Option", "Some",
                "None", "Result", "Ok", "Err", "Box", "Rc", "Arc", "Path", "PathBuf",
            ],
            Language::C | Language::Cpp => &[
                "int", "char", "float", "double", "short", "long", "void", "size_t", "ssize_t",
                "uint8_t", "uint16_t", "uint32_t", "uint64_t", "int8_t", "int16_t", "int32_t",
                "int64_t", "bool", "string", "vector", "map", "set",
            ],
            Language::Go => &[
                "bool", "byte", "complex64", "complex128", "error", "float32", "float64", "int",
                "int8", "int16", "int32", "int64", "rune", "string", "uint", "uint8", "uint16",
                "uint32", "uint64", "uintptr",
            ],
            Language::TypeScript => &[
                "any", "boolean", "number", "string", "symbol", "void", "unknown", "never",
                "object", "Promise", "Array", "Record",
            ],
            Language::Assembly => &[
                "rax", "rbx", "rcx", "rdx", "rsi", "rdi", "rbp", "rsp", "r8", "r9", "r10",
                "r11", "r12", "r13", "r14", "r15", "eax", "ebx", "ecx", "edx", "esi", "edi",
                "esp", "ebp", "al", "bl", "cl", "dl", "sil", "dil",
            ],
            _ => &[],
        }
    }
}

/// A styled segment in a highlighted line
pub struct StyledSpan {
    pub text: String,
    pub color: Color,
}

pub fn highlight_line(line: &str, lang: Language) -> Vec<StyledSpan> {
    if lang == Language::Plain {
        return vec![StyledSpan {
            text: line.to_string(),
            color: Palette::TEXT,
        }];
    }

    let mut spans: Vec<StyledSpan> = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;

    let keywords = lang.keywords();
    let types = lang.types();
    let comment_prefix = lang.line_comment_prefix();

    while i < len {
        // 1. Check for single-line comment
        let is_comment = if let Some(prefix) = comment_prefix {
            let p_len = prefix.len();
            if i + p_len <= len {
                let slice: String = chars[i..i + p_len].iter().collect();
                slice == prefix
            } else {
                false
            }
        } else if lang == Language::Assembly {
            chars[i] == ';' || chars[i] == '#'
        } else {
            false
        };

        if is_comment {
            let comment_text: String = chars[i..].iter().collect();
            spans.push(StyledSpan {
                text: comment_text,
                color: Palette::OVERLAY0,
            });
            break;
        }

        // 2. Check for strings (double quote, single quote, or backtick)
        if chars[i] == '"' || chars[i] == '\'' || (chars[i] == '`' && (lang == Language::JavaScript || lang == Language::TypeScript || lang == Language::Go)) {
            let quote = chars[i];
            let mut s = String::new();
            s.push(quote);
            i += 1;
            while i < len {
                let c = chars[i];
                s.push(c);
                if c == '\\' && i + 1 < len {
                    i += 1;
                    s.push(chars[i]);
                } else if c == quote {
                    i += 1;
                    break;
                }
                i += 1;
            }
            spans.push(StyledSpan {
                text: s,
                color: Palette::GREEN,
            });
            continue;
        }

        // 3. Check for numbers
        if chars[i].is_ascii_digit() && (i == 0 || !chars[i - 1].is_alphanumeric() && chars[i - 1] != '_') {
            let mut num_str = String::new();
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '.' || chars[i] == '_') {
                num_str.push(chars[i]);
                i += 1;
            }
            spans.push(StyledSpan {
                text: num_str,
                color: Palette::PEACH,
            });
            continue;
        }

        // 4. Identifiers, Keywords & Types
        if chars[i].is_alphabetic() || chars[i] == '_' {
            let mut ident = String::new();
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                ident.push(chars[i]);
                i += 1;
            }

            let color = if keywords.contains(&ident.as_str()) {
                Palette::MAUVE
            } else if types.contains(&ident.as_str()) || (ident.chars().next().map_or(false, |c| c.is_uppercase()) && !ident.chars().all(|c| c.is_uppercase())) {
                Palette::YELLOW
            } else if i < len && chars[i] == '(' {
                Palette::BLUE
            } else {
                Palette::TEXT
            };

            spans.push(StyledSpan {
                text: ident,
                color,
            });
            continue;
        }

        // 5. Operators, punctuation, or spaces
        let mut symbols = String::new();
        while i < len && !chars[i].is_alphanumeric() && chars[i] != '_' && chars[i] != '"' && chars[i] != '\'' && chars[i] != '`' {
            if let Some(prefix) = comment_prefix {
                if chars[i..].iter().collect::<String>().starts_with(prefix) {
                    break;
                }
            }
            if lang == Language::Assembly && (chars[i] == ';' || chars[i] == '#') {
                break;
            }
            symbols.push(chars[i]);
            i += 1;
        }

        if !symbols.is_empty() {
            spans.push(StyledSpan {
                text: symbols,
                color: Palette::TEXT,
            });
        }
    }

    spans
}
