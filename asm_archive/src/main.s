.intel_syntax noprefix
.include "include/nimo_defs.inc"
.include "include/theme_catppuccin.inc"

.global _start

.bss
/* Termios structures */
orig_termios:
    .space 64
raw_termios:
    .space 64

/* Window size struct: ws_row (uint16), ws_col (uint16), ws_xpixel (uint16), ws_ypixel (uint16) */
winsize:
    .space 8

/* File and buffer states */
filename_ptr:
    .quad 0
filename_len:
    .quad 0

/* Text Buffer (Gap Buffer / Dynamic stream: capacity 1MB) */
text_buffer:
    .space BUFFER_MAX
text_len:
    .quad 0
cursor_idx:      /* Offset in text_buffer (0 .. text_len) */
    .quad 0
cursor_row:      /* 0-indexed line number in entire file */
    .quad 0
cursor_col:      /* 0-indexed column within line */
    .quad 0

/* Viewport scrolling */
row_offset:      /* Top visible file line */
    .quad 0
col_offset:      /* Left visible column */
    .quad 0

/* Terminal dimensions */
term_rows:
    .space 8
term_cols:
    .space 8
text_rows:
    .space 8

/* Editor flags */
is_modified:
    .space 1
should_quit:
    .space 1

/* Status message string */
status_msg:
    .space 128
status_msg_len:
    .space 8

/* Configuration Toggles */
cfg_blinking_cursor:
    .space 1
cfg_line_numbers:
    .space 1
cfg_line_numbers_typeonly:
    .space 1
cfg_separator:
    .space 1
cfg_sep_style:
    .space 8
cfg_gutter_width:
    .space 8
cfg_top_bar:
    .space 1
cfg_status_bar:
    .space 1
cfg_footer:
    .space 1

/* RC file read buffer */
rc_buffer:
    .space 4096


/* Output Render Buffer (Double buffering for flicker-free 60fps+ rendering) */
render_buf:
    .space RENDER_BUF_MAX
render_len:
    .quad 0

/* Input read buffer */
input_buf:
    .space 32

/* General scratchpad for integer conversions */
num_buf:
    .space 32

.data
default_filename:
    .asciz "[New Buffer]"
default_filename_len = . - default_filename - 1

msg_welcome:
    .ascii "Welcome to Nimo - Pure Assembly High-Performance Editor | ^O WriteOut  ^X Exit"
msg_welcome_len = . - msg_welcome

str_help_text:
    .ascii "  NIMO - Nano-Inspired Modern Terminal Editor (Pure x86_64 Assembly)\n"
    .ascii "  Usage: ./nimo [options] [file]\n\n"
    .ascii "  Keybindings:\n"
    .ascii "    ^O (Ctrl+O)    WriteOut / Save buffer to disk\n"
    .ascii "    ^X (Ctrl+X)    Exit Nimo\n"
    .ascii "    ^K (Ctrl+K)    Cut current line\n"
    .ascii "    ^U (Ctrl+U)    Paste cut line\n"
    .ascii "    ^C (Ctrl+C)    Display current cursor line & col\n"
    .ascii "    Arrows         Smooth navigation\n"
    .ascii "    Home / End     Jump to line start / line end\n"
    .ascii "    PageUp/Down    Scroll viewport by page\n\n"
    .ascii "  Live Toggles (.nimorc & Hotkeys):\n"
    .ascii "    ^N (Ctrl+N)    Toggle Line Numbers ON / OFF\n"
    .ascii "    ^S (Ctrl+S)    Toggle Separator ON / OFF\n"
    .ascii "    ^B (Ctrl+B)    Toggle Blinking Cursor ON / OFF\n"
    .ascii "    ^P (Ctrl+P)    Cycle Separator Style (pipe, thin, double, dots, slash, none)\n\n"
    .ascii "  Configuration File (.nimorc):\n"
    .ascii "    BlinkingCursor = true / false\n"
    .ascii "    LineNumbers = true / false\n"
    .ascii "    LineNumbersTypeOnly = true / false\n"
    .ascii "    Separator = true / false\n"
    .ascii "    SeparatorStyle = pipe | thin | double | dots | slash | none\n"
    .ascii "    TopBar = true / false\n"
    .ascii "    StatusBar = true / false\n"
    .ascii "    Footer = true / false\n"
str_help_len = . - str_help_text


msg_saved:
    .ascii "[ File written successfully ]"
msg_saved_len = . - msg_saved

msg_save_error:
    .ascii "[ Error saving file! Check permissions ]"
msg_save_error_len = . - msg_save_error

msg_quit_confirm:
    .ascii "Save modified buffer before closing? (Y/N/C): "
msg_quit_confirm_len = . - msg_quit_confirm

/* Fixed UI string templates */
str_title_prefix:
    .ascii "  NIMO v1.0  "
str_title_prefix_len = . - str_title_prefix

str_gutter_pad:
    .ascii " "
str_gutter_sep:
    .ascii " │ "
str_gutter_sep_len = 5

/* Separator styles */
sep_str_pipe:
    .ascii " │ "
sep_str_pipe_len = . - sep_str_pipe

sep_str_thin:
    .ascii " | "
sep_str_thin_len = . - sep_str_thin

sep_str_double:
    .ascii " ║ "
sep_str_double_len = . - sep_str_double

sep_str_dots:
    .ascii " ┆ "
sep_str_dots_len = . - sep_str_dots

sep_str_slash:
    .ascii " // "
sep_str_slash_len = . - sep_str_slash

sep_str_blank:
    .ascii "   "
sep_str_blank_len = . - sep_str_blank

/* Active separator pointer and length */
sep_active_ptr:
    .quad sep_str_pipe
sep_active_len:
    .quad sep_str_pipe_len

/* Cursor escape sequences */
ansi_cursor_blinking_block:
    .ascii "\033[1 q"
ansi_cursor_blinking_block_len = . - ansi_cursor_blinking_block

ansi_cursor_steady_block:
    .ascii "\033[2 q"
ansi_cursor_steady_block_len = . - ansi_cursor_steady_block

ansi_cursor_blinking_bar:
    .ascii "\033[5 q"
ansi_cursor_blinking_bar_len = . - ansi_cursor_blinking_bar

ansi_cursor_steady_bar:
    .ascii "\033[6 q"
ansi_cursor_steady_bar_len = . - ansi_cursor_steady_bar

/* Config RC file paths */
rc_local_filename:
    .asciz ".nimorc"

str_key_blink:     .ascii "blinkingcursor"
str_key_typeonly:  .ascii "linenumberstypeonly"
str_key_lnum:      .ascii "linenumbers"
str_key_sep:       .ascii "separator"
str_key_sep2:      .ascii "seperator"
str_key_sepstyle:  .ascii "separatorstyle"
str_key_sepstyle2: .ascii "seperatorstyle"
str_key_topbar:    .ascii "topbar"
str_key_status:    .ascii "statusbar"
str_key_footer:    .ascii "footer"

str_val_pipe:      .ascii "pipe"
str_val_thin:      .ascii "thin"
str_val_double:    .ascii "double"
str_val_dots:      .ascii "dots"
str_val_slash:     .ascii "slash"
str_val_none:      .ascii "none"


/* Messages for toggles */
msg_toggle_lnum_on:
    .ascii "[ Line Numbers: ON ]"
msg_toggle_lnum_on_len = . - msg_toggle_lnum_on

msg_toggle_lnum_off:
    .ascii "[ Line Numbers: OFF ]"
msg_toggle_lnum_off_len = . - msg_toggle_lnum_off

msg_toggle_sep_on:
    .ascii "[ Separator: ON ]"
msg_toggle_sep_on_len = . - msg_toggle_sep_on

msg_toggle_sep_off:
    .ascii "[ Separator: OFF ]"
msg_toggle_sep_off_len = . - msg_toggle_sep_off

msg_toggle_blink_on:
    .ascii "[ Blinking Cursor: ON ]"
msg_toggle_blink_on_len = . - msg_toggle_blink_on

msg_toggle_blink_off:
    .ascii "[ Blinking Cursor: OFF ]"
msg_toggle_blink_off_len = . - msg_toggle_blink_off


/* Nano Footer rows */
/* Row 1 shortcuts */
k1_key: .ascii " ^G "
k1_lbl: .ascii " Get Help   "
k2_key: .ascii " ^O "
k2_lbl: .ascii " WriteOut   "
k3_key: .ascii " ^W "
k3_lbl: .ascii " Where Is   "
k4_key: .ascii " ^K "
k4_lbl: .ascii " Cut Line   "
k5_key: .ascii " ^J "
k5_lbl: .ascii " Justify    "
k6_key: .ascii " ^C "
k6_lbl: .ascii " Cur Pos    "

/* Row 2 shortcuts */
k7_key: .ascii " ^X "
k7_lbl: .ascii " Exit       "
k8_key: .ascii " ^R "
k8_lbl: .ascii " Read File  "
k9_key: .ascii " ^\\ "
k9_lbl: .ascii " Replace    "
k10_key: .ascii " ^U "
k10_lbl: .ascii " Paste      "
k11_key: .ascii " ^T "
k11_lbl: .ascii " To Spell   "
k12_key: .ascii " ^_ "
k12_lbl: .ascii " Go To Line "

/* Cut buffer for ^K and ^U */
cut_buf:
    .space 8192
cut_len:
    .quad 0

.text

/* -------------------------------------------------------------
 * _start: Entry Point (No LibC)
 * Stack layout on Linux entry:
 *   [rsp] = argc
 *   [rsp+8] = argv[0]
 *   [rsp+16] = argv[1] ...
 * ------------------------------------------------------------- */
_start:
    mov rbp, rsp
    mov r12, [rbp]       /* argc */

    /* Load configuration (.nimorc) */
    call init_default_config

    /* Check if filename passed in argv[1] */
    cmp r12, 1
    jle no_arg_file

    /* argv[1] is at [rbp + 16] */
    mov rsi, [rbp + 16]

    /* Check if arg is --help or -h */
    cmp byte ptr [rsi], '-'
    jne not_help_flag
    cmp byte ptr [rsi + 1], 'h'
    je print_help_and_exit
    cmp byte ptr [rsi + 1], '-'
    jne not_help_flag
    cmp byte ptr [rsi + 2], 'h'
    je print_help_and_exit
    cmp byte ptr [rsi + 2], 'v'
    je print_help_and_exit

not_help_flag:
    mov [filename_ptr], rsi

    /* Compute filename length */
    xor rcx, rcx
len_calc_loop:
    cmp byte ptr [rsi + rcx], 0
    je len_calc_done
    inc rcx
    jmp len_calc_loop
len_calc_done:
    mov [filename_len], rcx

    /* Open and read file into text_buffer */
    call load_file
    jmp file_setup_done

no_arg_file:
    lea rax, default_filename
    mov [filename_ptr], rax
    mov qword ptr [filename_len], default_filename_len
    mov byte ptr [is_modified], 0

    /* Set default welcome message */
    lea rsi, msg_welcome
    mov rcx, msg_welcome_len
    call set_status_msg

file_setup_done:
    /* Initialize Raw Terminal Mode */
    call enable_raw_mode
    call get_window_size
    call recompute_cursor_coords

main_loop:
    call render_screen
    call read_input_key
    cmp byte ptr [should_quit], 1
    jne main_loop

    jmp exit_editor

print_help_and_exit:
    mov rdi, STDOUT_FILENO
    lea rsi, str_help_text
    mov rdx, str_help_len
    syscall

    mov rax, SYS_exit
    xor rdi, rdi
    syscall


/* -------------------------------------------------------------
 * Configuration Initialization and .nimorc Parser
 * ------------------------------------------------------------- */
init_default_config:
    push rbp
    mov rbp, rsp

    /* Defaults: */
    /* BlinkingCursor = 1 (true) */
    mov byte ptr [cfg_blinking_cursor], 1
    /* LineNumbers = 1 (true) */
    mov byte ptr [cfg_line_numbers], 1
    /* LineNumbersTypeOnly = 1 (appear only as you type) */
    mov byte ptr [cfg_line_numbers_typeonly], 1
    /* Separator = 1 (true) */
    mov byte ptr [cfg_separator], 1
    /* SepStyle = 0 (pipe) */
    mov qword ptr [cfg_sep_style], 0
    lea rax, sep_str_pipe
    mov [sep_active_ptr], rax
    mov qword ptr [sep_active_len], sep_str_pipe_len
    /* Gutter width default = 7 */
    mov qword ptr [cfg_gutter_width], 7
    /* Bars: TopBar = 1, StatusBar = 1, Footer = 1 */
    mov byte ptr [cfg_top_bar], 1
    mov byte ptr [cfg_status_bar], 1
    mov byte ptr [cfg_footer], 1

    /* Attempt to load ./.nimorc */
    lea rdi, rc_local_filename
    call load_rc_file

    /* Apply cursor style based on cfg_blinking_cursor */
    call apply_cursor_style

    pop rbp
    ret

apply_cursor_style:
    push rbp
    mov rbp, rsp

    cmp byte ptr [cfg_blinking_cursor], 0
    je set_cursor_steady

    /* Blinking block: \033[1 q */
    mov rax, SYS_write
    mov rdi, STDOUT_FILENO
    lea rsi, ansi_cursor_blinking_block
    mov rdx, ansi_cursor_blinking_block_len
    syscall
    jmp cursor_style_done

set_cursor_steady:
    /* Steady block: \033[2 q */
    mov rax, SYS_write
    mov rdi, STDOUT_FILENO
    lea rsi, ansi_cursor_steady_block
    mov rdx, ansi_cursor_steady_block_len
    syscall

cursor_style_done:
    pop rbp
    ret

load_rc_file:
    push rbp
    mov rbp, rsp
    push r12
    push r13
    push r14
    push r15

    /* sys_open(rdi, O_RDONLY, 0) */
    mov rax, SYS_open
    /* rdi already has filename */
    xor rsi, rsi
    xor rdx, rdx
    syscall

    cmp rax, 0
    jl load_rc_done

    mov r12, rax        /* FD */

    /* sys_read(fd, rc_buffer, 4095) */
    mov rax, SYS_read
    mov rdi, r12
    lea rsi, rc_buffer
    mov rdx, 4095
    syscall

    push rax            /* Save bytes read */

    /* sys_close(fd) */
    mov rax, SYS_close
    mov rdi, r12
    syscall

    pop rax
    test rax, rax
    jle load_rc_done

    mov r13, rax        /* File size */
    lea r14, rc_buffer
    mov byte ptr [r14 + r13], 0   /* Null terminate */

    /* Parse line by line */
    xor r15, r15        /* Current offset in rc_buffer */

parse_rc_loop:
    cmp r15, r13
    jge load_rc_done

    /* Skip leading whitespace on line */
skip_ws:
    cmp r15, r13
    jge load_rc_done
    movzx eax, byte ptr [r14 + r15]
    cmp al, ' '
    je next_ws
    cmp al, '\t'
    je next_ws
    cmp al, '\r'
    je next_ws
    cmp al, '\n'
    je next_line_adv
    jmp check_comment
next_ws:
    inc r15
    jmp skip_ws

check_comment:
    /* If line starts with '#' or ';', skip till newline */
    cmp al, '#'
    je skip_to_nl
    cmp al, ';'
    je skip_to_nl

    /* Check key match */
    lea rdi, [r14 + r15]

    /* Check "BlinkingCursor" */
    lea rsi, str_key_blink
    mov rcx, 14
    call str_case_prefix_match
    test rax, rax
    jnz parse_blink

    /* Check "LineNumbersTypeOnly" */
    lea rsi, str_key_typeonly
    mov rcx, 19
    call str_case_prefix_match
    test rax, rax
    jnz parse_typeonly

    /* Check "LineNumbers" */
    lea rsi, str_key_lnum
    mov rcx, 11
    call str_case_prefix_match
    test rax, rax
    jnz parse_lnum

    /* Check "SeparatorStyle" or "SeperatorStyle" */
    lea rsi, str_key_sepstyle
    mov rcx, 14
    call str_case_prefix_match
    test rax, rax
    jnz parse_sep_style

    lea rsi, str_key_sepstyle2
    mov rcx, 14
    call str_case_prefix_match
    test rax, rax
    jnz parse_sep_style

    /* Check "Separator" or "Seperator" */
    lea rsi, str_key_sep
    mov rcx, 9
    call str_case_prefix_match
    test rax, rax
    jnz parse_sep

    lea rsi, str_key_sep2
    mov rcx, 9
    call str_case_prefix_match
    test rax, rax
    jnz parse_sep

    /* Check "TopBar" */
    lea rsi, str_key_topbar
    mov rcx, 6
    call str_case_prefix_match
    test rax, rax
    jnz parse_topbar

    /* Check "StatusBar" */
    lea rsi, str_key_status
    mov rcx, 9
    call str_case_prefix_match
    test rax, rax
    jnz parse_statusbar

    /* Check "Footer" or "ShortcutBar" */
    lea rsi, str_key_footer
    mov rcx, 6
    call str_case_prefix_match
    test rax, rax
    jnz parse_footer

    jmp skip_to_nl

parse_blink:
    add r15, 14
    call parse_bool_val
    mov [cfg_blinking_cursor], al
    jmp skip_to_nl

parse_typeonly:
    add r15, 19
    call parse_bool_val
    mov [cfg_line_numbers_typeonly], al
    jmp skip_to_nl

parse_lnum:
    add r15, 11
    call parse_bool_val
    mov [cfg_line_numbers], al
    jmp skip_to_nl

parse_sep:
    add r15, 9
    call parse_bool_val
    mov [cfg_separator], al
    jmp skip_to_nl

parse_topbar:
    add r15, 6
    call parse_bool_val
    mov [cfg_top_bar], al
    jmp skip_to_nl

parse_statusbar:
    add r15, 9
    call parse_bool_val
    mov [cfg_status_bar], al
    jmp skip_to_nl

parse_footer:
    add r15, 6
    call parse_bool_val
    mov [cfg_footer], al
    jmp skip_to_nl

parse_sep_style:
    add r15, 14
    /* Find '=' */
find_eq_sep:
    cmp r15, r13
    jge skip_to_nl
    mov al, [r14 + r15]
    cmp al, '='
    je found_eq_sep
    cmp al, '\n'
    je skip_to_nl
    inc r15
    jmp find_eq_sep

found_eq_sep:
    inc r15
    /* Skip spaces after '=' */
skip_eq_sp:
    cmp r15, r13
    jge skip_to_nl
    mov al, [r14 + r15]
    cmp al, ' '
    je next_eq_sp
    cmp al, '\t'
    je next_eq_sp
    jmp match_sep_style_name
next_eq_sp:
    inc r15
    jmp skip_eq_sp

match_sep_style_name:
    lea rdi, [r14 + r15]

    /* "pipe" or "single" -> sep_str_pipe */
    lea rsi, str_val_pipe
    mov rcx, 4
    call str_case_prefix_match
    test rax, rax
    jnz set_style_pipe

    /* "thin" or "ascii" -> sep_str_thin */
    lea rsi, str_val_thin
    mov rcx, 4
    call str_case_prefix_match
    test rax, rax
    jnz set_style_thin

    /* "double" -> sep_str_double */
    lea rsi, str_val_double
    mov rcx, 6
    call str_case_prefix_match
    test rax, rax
    jnz set_style_double

    /* "dots" or "dotted" -> sep_str_dots */
    lea rsi, str_val_dots
    mov rcx, 4
    call str_case_prefix_match
    test rax, rax
    jnz set_style_dots

    /* "slash" -> sep_str_slash */
    lea rsi, str_val_slash
    mov rcx, 5
    call str_case_prefix_match
    test rax, rax
    jnz set_style_slash

    /* "none" or "blank" -> sep_str_blank */
    lea rsi, str_val_none
    mov rcx, 4
    call str_case_prefix_match
    test rax, rax
    jnz set_style_blank

    jmp skip_to_nl

set_style_pipe:
    mov qword ptr [cfg_sep_style], 0
    lea rax, sep_str_pipe
    mov [sep_active_ptr], rax
    mov qword ptr [sep_active_len], sep_str_pipe_len
    jmp skip_to_nl

set_style_thin:
    mov qword ptr [cfg_sep_style], 1
    lea rax, sep_str_thin
    mov [sep_active_ptr], rax
    mov qword ptr [sep_active_len], sep_str_thin_len
    jmp skip_to_nl

set_style_double:
    mov qword ptr [cfg_sep_style], 2
    lea rax, sep_str_double
    mov [sep_active_ptr], rax
    mov qword ptr [sep_active_len], sep_str_double_len
    jmp skip_to_nl

set_style_dots:
    mov qword ptr [cfg_sep_style], 3
    lea rax, sep_str_dots
    mov [sep_active_ptr], rax
    mov qword ptr [sep_active_len], sep_str_dots_len
    jmp skip_to_nl

set_style_slash:
    mov qword ptr [cfg_sep_style], 4
    lea rax, sep_str_slash
    mov [sep_active_ptr], rax
    mov qword ptr [sep_active_len], sep_str_slash_len
    jmp skip_to_nl

set_style_blank:
    mov qword ptr [cfg_sep_style], 5
    lea rax, sep_str_blank
    mov [sep_active_ptr], rax
    mov qword ptr [sep_active_len], sep_str_blank_len
    jmp skip_to_nl

skip_to_nl:
    cmp r15, r13
    jge load_rc_done
    mov al, [r14 + r15]
    cmp al, '\n'
    je next_line_adv
    inc r15
    jmp skip_to_nl

next_line_adv:
    inc r15
    jmp parse_rc_loop

load_rc_done:
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbp
    ret

/* Parse boolean value after key: skips spaces, '=', spaces, then checks 't','1','y','f','0','n' */
/* Returns al = 1 or 0 */
parse_bool_val:
    push rbx
find_eq_loop:
    cmp r15, r13
    jge parse_bool_default
    mov bl, [r14 + r15]
    cmp bl, '='
    je found_eq_bool
    cmp bl, '\n'
    je parse_bool_default
    inc r15
    jmp find_eq_loop

found_eq_bool:
    inc r15
skip_ws_after_eq:
    cmp r15, r13
    jge parse_bool_default
    mov bl, [r14 + r15]
    cmp bl, ' '
    je adv_ws_eq
    cmp bl, '\t'
    je adv_ws_eq
    jmp evaluate_bool_char
adv_ws_eq:
    inc r15
    jmp skip_ws_after_eq

evaluate_bool_char:
    /* Convert bl to lower case */
    or bl, 0x20
    cmp bl, 't'         /* true */
    je is_val_true
    cmp bl, '1'         /* 1 */
    je is_val_true
    cmp bl, 'y'         /* yes */
    je is_val_true
    cmp bl, 'o'         /* on */
    je is_val_true
    /* Else false */
    xor al, al
    pop rbx
    ret

is_val_true:
    mov al, 1
    pop rbx
    ret

parse_bool_default:
    mov al, 1
    pop rbx
    ret

/* Case-insensitive prefix match: rdi = str1, rsi = str2, rcx = length */
/* Returns rax = 1 (match) or 0 (no match) */
str_case_prefix_match:
    push rbx
    push rdx
    push rcx
    push rsi
    push rdi

cmp_prefix_loop:
    test rcx, rcx
    jz prefix_matched
    mov al, [rdi]
    mov bl, [rsi]
    /* Convert both to uppercase */
    cmp al, 'a'
    jl not_lower1
    cmp al, 'z'
    jg not_lower1
    sub al, 0x20
not_lower1:
    cmp bl, 'a'
    jl not_lower2
    cmp bl, 'z'
    jg not_lower2
    sub bl, 0x20
not_lower2:
    cmp al, bl
    jne prefix_diff
    inc rdi
    inc rsi
    dec rcx
    jmp cmp_prefix_loop

prefix_matched:
    mov rax, 1
    jmp prefix_done

prefix_diff:
    xor rax, rax

prefix_done:
    pop rdi
    pop rsi
    pop rcx
    pop rdx
    pop rbx
    ret

exit_editor:
    /* Clean up terminal and restore cooked mode */
    call disable_raw_mode

    /* Exit system call */
    mov rax, SYS_exit
    xor rdi, rdi
    syscall

/* -------------------------------------------------------------
 * Raw Terminal Mode Management (tcgetattr / tcsetattr via ioctl)
 * ------------------------------------------------------------- */
enable_raw_mode:
    push rbp
    mov rbp, rsp

    /* ioctl(STDIN_FILENO, TCGETS, &orig_termios) */
    mov rax, SYS_ioctl
    mov rdi, STDIN_FILENO
    mov rsi, TCGETS
    lea rdx, orig_termios
    syscall

    /* Copy orig_termios to raw_termios */
    lea rsi, orig_termios
    lea rdi, raw_termios
    mov rcx, 15          /* 15 * 4 = 60 bytes */
copy_termios:
    mov eax, [rsi]
    mov [rdi], eax
    add rsi, 4
    add rdi, 4
    dec rcx
    jnz copy_termios

    /* Modify flags for raw mode: */
    /* iflag &= ~IFLAG_MASK */
    lea rbx, raw_termios
    mov eax, [rbx + TERMIOS_IFLAG]
    and eax, ~IFLAG_MASK
    mov [rbx + TERMIOS_IFLAG], eax

    /* oflag &= ~OFLAG_MASK */
    mov eax, [rbx + TERMIOS_OFLAG]
    and eax, ~OFLAG_MASK
    mov [rbx + TERMIOS_OFLAG], eax

    /* cflag |= CS8 */
    mov eax, [rbx + TERMIOS_CFLAG]
    or eax, CS8
    mov [rbx + TERMIOS_CFLAG], eax

    /* lflag &= ~LFLAG_MASK */
    mov eax, [rbx + TERMIOS_LFLAG]
    and eax, ~LFLAG_MASK
    mov [rbx + TERMIOS_LFLAG], eax

    /* c_cc[VMIN] = 1, c_cc[VTIME] = 0 */
    mov byte ptr [rbx + TERMIOS_CC + VMIN_IDX], 1
    mov byte ptr [rbx + TERMIOS_CC + VTIME_IDX], 0

    /* ioctl(STDIN_FILENO, TCSETSF, &raw_termios) */
    mov rax, SYS_ioctl
    mov rdi, STDIN_FILENO
    mov rsi, TCSETSF
    lea rdx, raw_termios
    syscall

    /* Switch to Alternate Screen Buffer: \033[?1049h */
    mov rax, SYS_write
    mov rdi, STDOUT_FILENO
    lea rsi, ansi_enter_alt_screen
    mov rdx, ansi_enter_alt_screen_len
    syscall

    pop rbp
    ret

disable_raw_mode:
    push rbp
    mov rbp, rsp

    /* Restore original termios cooked mode */
    mov rax, SYS_ioctl
    mov rdi, STDIN_FILENO
    mov rsi, TCSETSF
    lea rdx, orig_termios
    syscall

    /* Reset cursor shape/blink to system default: \033[0 q */
    mov rax, SYS_write
    mov rdi, STDOUT_FILENO
    lea rsi, ansi_reset_cursor
    mov rdx, ansi_reset_cursor_len
    syscall

    /* Show cursor: \033[?25h */
    mov rax, SYS_write
    mov rdi, STDOUT_FILENO
    lea rsi, ansi_show_cursor
    mov rdx, ansi_show_cursor_len
    syscall

    /* Reset text attributes / color: \033[0m */
    mov rax, SYS_write
    mov rdi, STDOUT_FILENO
    lea rsi, ansi_reset
    mov rdx, ansi_reset_len
    syscall

    /* Leave alternate screen buffer: \033[?1049l */
    mov rax, SYS_write
    mov rdi, STDOUT_FILENO
    lea rsi, ansi_leave_alt_screen
    mov rdx, ansi_leave_alt_screen_len
    syscall

    pop rbp
    ret

/* -------------------------------------------------------------
 * Window Size Query
 * ------------------------------------------------------------- */
get_window_size:
    push rbp
    mov rbp, rsp

    mov rax, SYS_ioctl
    mov rdi, STDOUT_FILENO
    mov rsi, TIOCGWINSZ
    lea rdx, winsize
    syscall

    /* winsize: struct { unsigned short ws_row, ws_col; } */
    movzx rax, word ptr [winsize]       /* ws_row */
    test rax, rax
    jz default_size
    mov [term_rows], rax

    movzx rax, word ptr [winsize + 2]   /* ws_col */
    test rax, rax
    jz default_size
    mov [term_cols], rax
    jmp finish_winsz

default_size:
    mov qword ptr [term_rows], 24
    mov qword ptr [term_cols], 80

finish_winsz:
    call compute_text_rows
    pop rbp
    ret

compute_text_rows:
    push rbp
    mov rbp, rsp
    mov rax, [term_rows]
    /* Subtract 1 if cfg_top_bar is set */
    cmp byte ptr [cfg_top_bar], 0
    je chk_sub_status
    dec rax
chk_sub_status:
    /* Subtract 1 if cfg_status_bar is set */
    cmp byte ptr [cfg_status_bar], 0
    je chk_sub_footer
    dec rax
chk_sub_footer:
    /* Subtract 2 if cfg_footer is set */
    cmp byte ptr [cfg_footer], 0
    je chk_rows_min
    sub rax, 2
chk_rows_min:
    cmp rax, 1
    jge store_text_rows
    mov rax, 1
store_text_rows:
    mov [text_rows], rax

    /* Update cfg_gutter_width */
    cmp byte ptr [cfg_line_numbers], 0
    jne set_full_gutter
    /* Line numbers off: if separator on, gutter is separator length (3 or 4), else 0 */
    cmp byte ptr [cfg_separator], 0
    je set_zero_gutter
    mov rax, [sep_active_len]
    mov [cfg_gutter_width], rax
    jmp compute_done
set_zero_gutter:
    mov qword ptr [cfg_gutter_width], 0
    jmp compute_done
set_full_gutter:
    /* 4 digits + separator len (or 0) */
    mov rax, 4
    cmp byte ptr [cfg_separator], 0
    je store_full_gw
    add rax, [sep_active_len]
store_full_gw:
    mov [cfg_gutter_width], rax

compute_done:
    pop rbp
    ret

/* -------------------------------------------------------------
 * File Loading and Saving
 * ------------------------------------------------------------- */
load_file:
    push rbp
    mov rbp, rsp

    /* sys_open([filename_ptr], O_RDONLY (0), 0) */
    mov rax, SYS_open
    mov rdi, [filename_ptr]
    xor rsi, rsi
    xor rdx, rdx
    syscall

    cmp rax, 0
    jl load_failed

    mov r13, rax        /* Save FD */

    /* sys_read(fd, text_buffer, BUFFER_MAX - 1) */
    mov rax, SYS_read
    mov rdi, r13
    lea rsi, text_buffer
    mov rdx, BUFFER_MAX - 1
    syscall

    test rax, rax
    js load_read_err
    mov [text_len], rax
    jmp close_load_fd

load_read_err:
    mov qword ptr [text_len], 0

close_load_fd:
    mov rax, SYS_close
    mov rdi, r13
    syscall

    mov byte ptr [is_modified], 0
    pop rbp
    ret

load_failed:
    /* If file doesn't exist, start new empty buffer */
    mov qword ptr [text_len], 0
    mov byte ptr [is_modified], 0
    pop rbp
    ret

save_file:
    push rbp
    mov rbp, rsp

    /* Don't save if filename is [New Buffer] */
    mov rax, [filename_ptr]
    lea rbx, default_filename
    cmp rax, rbx
    je save_error

    /* sys_open([filename_ptr], O_WRONLY | O_CREAT | O_TRUNC (01101), 0644 (420)) */
    mov rax, SYS_open
    mov rdi, [filename_ptr]
    mov rsi, 01101      /* O_WRONLY|O_CREAT|O_TRUNC */
    mov rdx, 420        /* 0644 */
    syscall

    cmp rax, 0
    jl save_error

    mov r13, rax        /* FD */

    /* sys_write(fd, text_buffer, text_len) */
    mov rax, SYS_write
    mov rdi, r13
    lea rsi, text_buffer
    mov rdx, [text_len]
    syscall

    /* Close FD */
    mov rax, SYS_close
    mov rdi, r13
    syscall

    mov byte ptr [is_modified], 0
    lea rsi, msg_saved
    mov rcx, msg_saved_len
    call set_status_msg
    pop rbp
    ret

save_error:
    lea rsi, msg_save_error
    mov rcx, msg_save_error_len
    call set_status_msg
    pop rbp
    ret

/* -------------------------------------------------------------
 * Status Message Helper
 * rsi = msg pointer, rcx = length
 * ------------------------------------------------------------- */
set_status_msg:
    push rdi
    push rsi
    push rcx
    lea rdi, status_msg
    cmp rcx, 120
    jle set_msg_copy
    mov rcx, 120
set_msg_copy:
    mov [status_msg_len], rcx
    rep movsb
    pop rcx
    pop rsi
    pop rdi
    ret

/* -------------------------------------------------------------
 * Coordinate Calculation:
 * Calculates (cursor_row, cursor_col) from cursor_idx in text_buffer
 * ------------------------------------------------------------- */
recompute_cursor_coords:
    push rbx
    push rcx
    push rdx
    push rsi

    xor rax, rax        /* row = 0 */
    xor rbx, rbx        /* col = 0 */
    xor rcx, rcx        /* offset = 0 */
    mov rdx, [cursor_idx]

coord_scan_loop:
    cmp rcx, rdx
    jge coord_scan_done
    lea rsi, text_buffer
    movzx r8d, byte ptr [rsi + rcx]
    cmp r8b, 10         /* '\n' */
    jne not_newline
    inc rax             /* row++ */
    xor rbx, rbx        /* col = 0 */
    jmp coord_next
not_newline:
    cmp r8b, 9          /* '\t' tab expands to 4 spaces */
    jne not_tab
    add rbx, 4
    and rbx, ~3
    jmp coord_next
not_tab:
    inc rbx             /* col++ */
coord_next:
    inc rcx
    jmp coord_scan_loop

coord_scan_done:
    mov [cursor_row], rax
    mov [cursor_col], rbx

    /* Adjust scrolling viewport */
    /* Vertical scroll */
    mov rax, [cursor_row]
    cmp rax, [row_offset]
    jge check_row_bottom
    mov [row_offset], rax
    jmp check_col_scroll

check_row_bottom:
    mov rdx, [row_offset]
    add rdx, [text_rows]
    cmp rax, rdx
    jl check_col_scroll
    /* row_offset = cursor_row - text_rows + 1 */
    mov rdx, rax
    sub rdx, [text_rows]
    inc rdx
    mov [row_offset], rdx

check_col_scroll:
    /* Horizontal scroll */
    mov rax, [cursor_col]
    cmp rax, [col_offset]
    jge check_col_right
    mov [col_offset], rax
    jmp coords_finished

check_col_right:
    mov rdx, [col_offset]
    /* Usable columns = term_cols - (cfg_gutter_width + 1) */
    mov r8, [term_cols]
    mov r9, [cfg_gutter_width]
    inc r9
    sub r8, r9
    cmp r8, 10
    jge col_ok
    mov r8, 10
col_ok:
    add rdx, r8
    cmp rax, rdx
    jl coords_finished
    mov rdx, rax
    sub rdx, r8
    inc rdx
    mov [col_offset], rdx

coords_finished:
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    ret

/* -------------------------------------------------------------
 * Append String to Render Buffer
 * rsi = source, rcx = length
 * ------------------------------------------------------------- */
buf_append:
    push rdi
    push rsi
    push rcx
    mov rdi, [render_len]
    lea rdi, [render_buf + rdi]
    rep movsb
    mov rax, [render_len]
    pop rcx
    add rax, rcx
    mov [render_len], rax
    pop rsi
    pop rdi
    ret

buf_append_byte:
    push rdi
    mov rdi, [render_len]
    mov [render_buf + rdi], al
    inc qword ptr [render_len]
    pop rdi
    ret

/* -------------------------------------------------------------
 * Buffer integer to string formatted append (Decimal)
 * rax = unsigned number, rbx = min width (pad spaces on left)
 * ------------------------------------------------------------- */
buf_append_uint:
    push rbp
    mov rbp, rsp
    push r12
    push r13
    push r14
    push r15

    mov r12, rax        /* number */
    mov r13, rbx        /* min width */
    lea r14, num_buf
    add r14, 30
    mov byte ptr [r14], 0
    mov r15, 0          /* length counter */

    mov rax, r12
    mov rcx, 10
div_loop:
    xor rdx, rdx
    div rcx
    add dl, '0'
    dec r14
    mov [r14], dl
    inc r15
    test rax, rax
    jnz div_loop

    /* Pad with spaces if r15 < r13 */
pad_loop:
    cmp r15, r13
    jge do_append_num
    dec r14
    mov byte ptr [r14], ' '
    inc r15
    jmp pad_loop

do_append_num:
    mov rsi, r14
    mov rcx, r15
    call buf_append

    pop r15
    pop r14
    pop r13
    pop r12
    pop rbp
    ret

/* -------------------------------------------------------------
 * Set ANSI Cursor Position: \033[<row>;<col>H (1-indexed)
 * rdi = row (1..), rsi = col (1..)
 * ------------------------------------------------------------- */
append_goto_xy:
    push rbx
    push r12
    push r13

    mov r12, rdi
    mov r13, rsi

    /* Append "\033[" */
    mov al, 0x1b
    call buf_append_byte
    mov al, '['
    call buf_append_byte

    /* Append row */
    mov rax, r12
    xor rbx, rbx
    call buf_append_uint

    /* Append ';' */
    mov al, ';'
    call buf_append_byte

    /* Append col */
    mov rax, r13
    xor rbx, rbx
    call buf_append_uint

    /* Append 'H' */
    mov al, 'H'
    call buf_append_byte

    pop r13
    pop r12
    pop rbx
    ret

/* -------------------------------------------------------------
 * Screen Rendering Engine
 * ------------------------------------------------------------- */
render_screen:
    push rbp
    mov rbp, rsp

    /* Reset buffer pointer */
    mov qword ptr [render_len], 0

    /* 1. Hide cursor during draw & home position */
    lea rsi, ansi_hide_cursor
    mov rcx, ansi_hide_cursor_len
    call buf_append

    /* Home cursor \033[H */
    mov al, 0x1b
    call buf_append_byte
    mov al, '['
    call buf_append_byte
    mov al, 'H'
    call buf_append_byte

    /* ---------------------------------------------------------
     * 2. Top Header Bar (Nano Title with Catppuccin Mauve)
     * --------------------------------------------------------- */
    cmp byte ptr [cfg_top_bar], 0
    je skip_top_bar

    lea rsi, style_top_bar
    mov rcx, style_top_bar_len
    call buf_append

    lea rsi, str_title_prefix
    mov rcx, str_title_prefix_len
    call buf_append

    /* Filename */
    mov rsi, [filename_ptr]
    mov rcx, [filename_len]
    call buf_append

    /* Modified flag pill */
    cmp byte ptr [is_modified], 0
    je top_bar_fill
    lea rsi, style_status_mod
    mov rcx, style_status_mod_len
    call buf_append

    mov al, ' '
    call buf_append_byte
    mov al, '*'
    call buf_append_byte
    mov al, 'M'
    call buf_append_byte
    mov al, 'o'
    call buf_append_byte
    mov al, 'd'
    call buf_append_byte
    mov al, 'i'
    call buf_append_byte
    mov al, 'f'
    call buf_append_byte
    mov al, 'i'
    call buf_append_byte
    mov al, 'e'
    call buf_append_byte
    mov al, 'd'
    call buf_append_byte
    mov al, ' '
    call buf_append_byte

    /* Back to top bar style */
    lea rsi, style_top_bar
    mov rcx, style_top_bar_len
    call buf_append

top_bar_fill:
    /* Clear to end of line: \033[K */
    mov al, 0x1b
    call buf_append_byte
    mov al, '['
    call buf_append_byte
    mov al, 'K'
    call buf_append_byte

    /* Newline */
    mov al, 13
    call buf_append_byte
    mov al, 10
    call buf_append_byte

skip_top_bar:

    /* ---------------------------------------------------------
     * 3. Text Area with Line Number Gutter
     * --------------------------------------------------------- */
    /* Find file offset for row_offset */
    xor r12, r12        /* current file line counter */
    xor r13, r13        /* text_buffer offset */
    mov r14, [text_len]

find_row_offset_loop:
    cmp r12, [row_offset]
    jge row_offset_found
    cmp r13, r14
    jge row_offset_found
    lea rbx, text_buffer
    cmp byte ptr [rbx + r13], 10
    jne not_row_nl
    inc r12
not_row_nl:
    inc r13
    jmp find_row_offset_loop

row_offset_found:
    /* Render text_rows lines */
    xor r15, r15        /* screen row index: 0 .. text_rows-1 */

render_line_loop:
    cmp r15, [text_rows]
    jge text_area_done

    /* Determine if line numbers and separator are drawn */
    mov rax, [row_offset]
    add rax, r15        /* Current line index (0-indexed) */

    /* Check if LineNumbers is enabled */
    cmp byte ptr [cfg_line_numbers], 0
    je skip_line_numbers_draw

    /* LineNumbers is ON. Now check if TypeOnly mode is enabled */
    cmp byte ptr [cfg_line_numbers_typeonly], 0
    je do_always_draw_lnum

    /* TypeOnly mode: Only show line number if line actually exists in file or is cursor_row */
    /* Check if this screen line has text (r13 < text_len or rax <= cursor_row) */
    cmp rax, [cursor_row]
    jle do_always_draw_lnum
    cmp r13, r14
    jl do_always_draw_lnum

    /* Otherwise, draw 4 spaces padding for gutter */
    lea rsi, style_gutter
    mov rcx, style_gutter_len
    call buf_append
    mov al, ' '
    call buf_append_byte
    call buf_append_byte
    call buf_append_byte
    call buf_append_byte
    jmp draw_sep_if_enabled

do_always_draw_lnum:
    cmp rax, [cursor_row]
    jne use_normal_gutter
    lea rsi, style_gutter_active
    mov rcx, style_gutter_active_len
    call buf_append
    jmp draw_gutter_num

use_normal_gutter:
    lea rsi, style_gutter
    mov rcx, style_gutter_len
    call buf_append

draw_gutter_num:
    /* Print line number padded to 4 chars (1-indexed) */
    mov rax, [row_offset]
    add rax, r15
    inc rax
    mov rbx, 4
    call buf_append_uint

draw_sep_if_enabled:
    /* Check if Separator is enabled */
    cmp byte ptr [cfg_separator], 0
    je finish_gutter_area

    lea rsi, style_gutter
    mov rcx, style_gutter_len
    call buf_append

    mov rsi, [sep_active_ptr]
    mov rcx, [sep_active_len]
    call buf_append
    jmp finish_gutter_area

skip_line_numbers_draw:
    /* Line numbers OFF: check if separator alone is drawn */
    cmp byte ptr [cfg_separator], 0
    je finish_gutter_area

    lea rsi, style_gutter
    mov rcx, style_gutter_len
    call buf_append

    mov rsi, [sep_active_ptr]
    mov rcx, [sep_active_len]
    call buf_append

finish_gutter_area:
    /* Switch to editor body style */
    lea rsi, style_editor_area
    mov rcx, style_editor_area_len
    call buf_append

    /* Draw characters for this line */
    xor r8, r8          /* column within line */
    mov r9, [col_offset]
    mov r10, [term_cols]
    mov rdx, [cfg_gutter_width]
    inc rdx
    sub r10, rdx        /* max text columns */
    cmp r10, 5
    jge chars_loop_start
    mov r10, 5

chars_loop_start:

draw_chars_loop:
    cmp r13, r14
    jge line_end_reached
    lea rbx, text_buffer
    movzx eax, byte ptr [rbx + r13]
    cmp al, 10          /* newline */
    je line_nl_reached

    /* Advance file pointer */
    inc r13

    /* Handle tab or normal char */
    cmp al, 9
    jne normal_char
    /* Tab expansion */
    mov ecx, 4
tab_spaces:
    cmp r8, r9
    jl tab_skip_render
    push rax
    mov rax, r8
    sub rax, r9
    cmp rax, r10
    jge tab_no_draw
    mov al, ' '
    call buf_append_byte
tab_no_draw:
    pop rax
tab_skip_render:
    inc r8
    dec ecx
    jnz tab_spaces
    jmp draw_chars_loop

normal_char:
    cmp r8, r9
    jl normal_skip_render
    push rax
    mov rax, r8
    sub rax, r9
    cmp rax, r10
    jge normal_no_draw
    pop rax
    call buf_append_byte
    jmp normal_char_done
normal_no_draw:
    pop rax
normal_skip_render:
normal_char_done:
    inc r8
    jmp draw_chars_loop

line_nl_reached:
    inc r13             /* Skip '\n' */
line_end_reached:
    /* Clear remainder of line to background: \033[K */
    mov al, 0x1b
    call buf_append_byte
    mov al, '['
    call buf_append_byte
    mov al, 'K'
    call buf_append_byte

    /* CR LF */
    mov al, 13
    call buf_append_byte
    mov al, 10
    call buf_append_byte

    inc r15
    jmp render_line_loop

text_area_done:
    /* ---------------------------------------------------------
     * 4. Status Bar
     * --------------------------------------------------------- */
    cmp byte ptr [cfg_status_bar], 0
    je skip_status_bar

    lea rsi, style_status_bar
    mov rcx, style_status_bar_len
    call buf_append

    /* If status_msg is set, show it; else show stats */
    cmp qword ptr [status_msg_len], 0
    je draw_default_status

    mov al, ' '
    call buf_append_byte
    mov rsi, offset status_msg
    mov rcx, [status_msg_len]
    call buf_append
    jmp finish_status_bar

draw_default_status:
    mov al, ' '
    call buf_append_byte
    mov al, 'L'
    call buf_append_byte
    mov al, 'n'
    call buf_append_byte
    mov al, ' '
    call buf_append_byte

    mov rax, [cursor_row]
    inc rax
    xor rbx, rbx
    call buf_append_uint

    mov al, ','
    call buf_append_byte
    mov al, ' '
    call buf_append_byte
    mov al, 'C'
    call buf_append_byte
    mov al, 'o'
    call buf_append_byte
    mov al, 'l'
    call buf_append_byte
    mov al, ' '
    call buf_append_byte

    mov rax, [cursor_col]
    inc rax
    xor rbx, rbx
    call buf_append_uint

    mov al, ' '
    call buf_append_byte
    mov al, '|'
    call buf_append_byte
    mov al, ' '
    call buf_append_byte
    mov al, 'B'
    call buf_append_byte
    mov al, 'y'
    call buf_append_byte
    mov al, 't'
    call buf_append_byte
    mov al, 'e'
    call buf_append_byte
    mov al, 's'
    call buf_append_byte
    mov al, ':'
    call buf_append_byte
    mov al, ' '
    call buf_append_byte

    mov rax, [text_len]
    xor rbx, rbx
    call buf_append_uint

finish_status_bar:
    /* Clear to end of status line: \033[K */
    mov al, 0x1b
    call buf_append_byte
    mov al, '['
    call buf_append_byte
    mov al, 'K'
    call buf_append_byte

    mov al, 13
    call buf_append_byte
    mov al, 10
    call buf_append_byte

skip_status_bar:
    /* ---------------------------------------------------------
     * 5. Nano Shortcut Rows 1 & 2
     * --------------------------------------------------------- */
    cmp byte ptr [cfg_footer], 0
    je skip_nano_footer

    /* Row 1 */
    call draw_nano_shortcut_row1

    mov al, 13
    call buf_append_byte
    mov al, 10
    call buf_append_byte

    /* Row 2 */
    call draw_nano_shortcut_row2

skip_nano_footer:
    /* ---------------------------------------------------------
     * 6. Place Hardware Terminal Cursor at Active Editing Pos
     * Screen Row = (cursor_row - row_offset) + 1 + (cfg_top_bar ? 1 : 0)
     * Screen Col = (cursor_col - col_offset) + 1 + cfg_gutter_width
     * --------------------------------------------------------- */
    mov rdi, [cursor_row]
    sub rdi, [row_offset]
    inc rdi             /* 1-indexed */
    cmp byte ptr [cfg_top_bar], 0
    je compute_cursor_x
    inc rdi             /* +1 for top bar */

compute_cursor_x:
    mov rsi, [cursor_col]
    sub rsi, [col_offset]
    inc rsi             /* 1-indexed */
    add rsi, [cfg_gutter_width]

    call append_goto_xy

    /* Show cursor */
    lea rsi, ansi_show_cursor
    mov rcx, ansi_show_cursor_len
    call buf_append

    /* 7. Flush entire frame in a single atomic sys_write */
    mov rax, SYS_write
    mov rdi, STDOUT_FILENO
    lea rsi, render_buf
    mov rdx, [render_len]
    syscall

    pop rbp
    ret

/* -------------------------------------------------------------
 * Nano Footer Shortcut Row Drawing
 * ------------------------------------------------------------- */
draw_nano_shortcut_row1:
    push rbp
    mov rbp, rsp

    /* Item 1: ^G Get Help */
    lea rsi, k1_key
    mov rcx, 4
    lea r8, k1_lbl
    mov r9, 12
    call draw_nano_pill

    /* Item 2: ^O WriteOut */
    lea rsi, k2_key
    mov rcx, 4
    lea r8, k2_lbl
    mov r9, 12
    call draw_nano_pill

    /* Item 3: ^W Where Is */
    lea rsi, k3_key
    mov rcx, 4
    lea r8, k3_lbl
    mov r9, 12
    call draw_nano_pill

    /* Item 4: ^K Cut Line */
    lea rsi, k4_key
    mov rcx, 4
    lea r8, k4_lbl
    mov r9, 12
    call draw_nano_pill

    /* Item 5: ^J Justify */
    lea rsi, k5_key
    mov rcx, 4
    lea r8, k5_lbl
    mov r9, 12
    call draw_nano_pill

    /* Item 6: ^C Cur Pos */
    lea rsi, k6_key
    mov rcx, 4
    lea r8, k6_lbl
    mov r9, 12
    call draw_nano_pill

    /* Clear rest of row */
    lea rsi, style_footer_bg
    mov rcx, style_footer_bg_len
    call buf_append
    mov al, 0x1b
    call buf_append_byte
    mov al, '['
    call buf_append_byte
    mov al, 'K'
    call buf_append_byte

    pop rbp
    ret

draw_nano_shortcut_row2:
    push rbp
    mov rbp, rsp

    /* Item 7: ^X Exit */
    lea rsi, k7_key
    mov rcx, 4
    lea r8, k7_lbl
    mov r9, 12
    call draw_nano_pill

    /* Item 8: ^R Read File */
    lea rsi, k8_key
    mov rcx, 4
    lea r8, k8_lbl
    mov r9, 12
    call draw_nano_pill

    /* Item 9: ^\ Replace */
    lea rsi, k9_key
    mov rcx, 4
    lea r8, k9_lbl
    mov r9, 12
    call draw_nano_pill

    /* Item 10: ^U Paste */
    lea rsi, k10_key
    mov rcx, 4
    lea r8, k10_lbl
    mov r9, 12
    call draw_nano_pill

    /* Item 11: ^T To Spell */
    lea rsi, k11_key
    mov rcx, 4
    lea r8, k11_lbl
    mov r9, 12
    call draw_nano_pill

    /* Item 12: ^_ Go To Line */
    lea rsi, k12_key
    mov rcx, 4
    lea r8, k12_lbl
    mov r9, 12
    call draw_nano_pill

    /* Clear rest of row */
    lea rsi, style_footer_bg
    mov rcx, style_footer_bg_len
    call buf_append
    mov al, 0x1b
    call buf_append_byte
    mov al, '['
    call buf_append_byte
    mov al, 'K'
    call buf_append_byte

    pop rbp
    ret

/* Helper: draw 1 shortcut pill: rsi=key ptr, rcx=key len, r8=desc ptr, r9=desc len */
draw_nano_pill:
    push rbp
    mov rbp, rsp
    push r8
    push r9

    /* Key pill style */
    push rsi
    push rcx
    lea rsi, style_shortcut_key
    mov rcx, style_shortcut_key_len
    call buf_append
    pop rcx
    pop rsi

    /* Key text */
    call buf_append

    /* Desc style */
    lea rsi, style_shortcut_desc
    mov rcx, style_shortcut_desc_len
    call buf_append

    /* Desc text */
    pop rcx
    pop rsi
    call buf_append

    pop rbp
    ret

/* -------------------------------------------------------------
 * Input Handling Engine
 * Reads byte sequences from STDIN, parses ANSI arrows, shortcuts, keys
 * ------------------------------------------------------------- */
read_input_key:
    push rbp
    mov rbp, rsp
    push r12
    push r13
    push r14

    /* sys_read(STDIN_FILENO, input_buf, 32) */
    mov rax, SYS_read
    mov rdi, STDIN_FILENO
    lea rsi, input_buf
    mov rdx, 32
    syscall

    test rax, rax
    jle read_input_done

    mov r12, rax        /* Total bytes read */
    xor r13, r13        /* Current offset in input_buf (0 .. r12-1) */

process_next_byte:
    cmp r13, r12
    jge read_input_done

    lea rsi, input_buf
    add rsi, r13
    movzx eax, byte ptr [rsi]

    /* Check for Escape sequence:  */
    cmp al, 0x1b
    jne check_ctrl_keys

    /* If escape sequence at end of buffer, or standalone ESC */
    mov rdx, r12
    sub rdx, r13
    cmp rdx, 1
    je esc_single

    /* Check for Alt+key sequences:  followed by a letter */
    movzx ebx, byte ptr [rsi + 1]
    cmp bl, 'n'
    je do_alt_n
    cmp bl, 's'
    je do_alt_s
    cmp bl, 'b'
    je do_alt_b
    cmp bl, 'p'
    je do_alt_p

    /* Check if '[' follows */
    cmp bl, '['
    jne check_ctrl_keys

    /* If only [ without 3rd char */
    cmp rdx, 2
    jle esc_consume_all

    movzx ebx, byte ptr [rsi + 2]
    cmp bl, 'A'
    je do_arrow_up
    cmp bl, 'B'
    je do_arrow_down
    cmp bl, 'C'
    je do_arrow_right
    cmp bl, 'D'
    je do_arrow_left
    cmp bl, 'H'
    je do_key_home
    cmp bl, 'F'
    je do_key_end
    cmp bl, '3'
    je do_key_delete
    cmp bl, '5'
    je do_page_up
    cmp bl, '6'
    je do_page_down

    /* Unrecognized ANSI seq, advance 3 */
    add r13, 3
    jmp process_next_byte

do_arrow_up:
    call move_cursor_up
    add r13, 3
    jmp process_next_byte

do_arrow_down:
    call move_cursor_down
    add r13, 3
    jmp process_next_byte

do_arrow_right:
    call do_move_right
    add r13, 3
    jmp process_next_byte

do_arrow_left:
    call do_move_left
    add r13, 3
    jmp process_next_byte

do_key_home:
    call move_cursor_home
    add r13, 3
    jmp process_next_byte

do_key_end:
    call move_cursor_end
    add r13, 3
    jmp process_next_byte

do_key_delete:
    call delete_char_at_cursor
    /* [3~ is 4 bytes */
    mov rdx, r12
    sub rdx, r13
    cmp rdx, 4
    jl del_adv3
    cmp byte ptr [rsi + 3], '~'
    jne del_adv3
    add r13, 4
    jmp process_next_byte
del_adv3:
    add r13, 3
    jmp process_next_byte

do_page_up:
    call page_up_action
    add r13, 4
    jmp process_next_byte

do_page_down:
    call page_down_action
    add r13, 4
    jmp process_next_byte

esc_single:
    mov qword ptr [status_msg_len], 0
    inc r13
    jmp process_next_byte

esc_consume_all:
    mov r13, r12
    jmp read_input_done

check_ctrl_keys:
    /* Runtime Toggles: */
    /* Ctrl+N = 14 (0x0E): Toggle Line Numbers ON/OFF */
    cmp al, 14
    je do_ctrl_n

    /* Ctrl+S = 19 (0x13): Toggle Separator ON/OFF */
    cmp al, 19
    je do_ctrl_s

    /* Ctrl+B = 2 (0x02): Toggle Blinking Cursor ON/OFF */
    cmp al, 2
    je do_ctrl_b

    /* Ctrl+P = 16 (0x10): Cycle Separator Styles (Pipe -> Thin -> Double -> Dots -> Slash -> None) */
    cmp al, 16
    je do_ctrl_p

    /* Ctrl+X = 24 (0x18): Exit */
    cmp al, 24
    je do_ctrl_x

    /* Ctrl+O = 15 (0x0F): WriteOut / Save */
    cmp al, 15
    je do_ctrl_o

    /* Ctrl+K = 11 (0x0B): Cut Line */
    cmp al, 11
    je do_ctrl_k

    /* Ctrl+U = 21 (0x15): Paste Line */
    cmp al, 21
    je do_ctrl_u

    /* Ctrl+C = 3 (0x03): Show Cursor Position */
    cmp al, 3
    je do_ctrl_c

    /* Enter (CR = 13 or LF = 10) */
    cmp al, 13
    je do_enter
    cmp al, 10
    je do_enter

    /* Backspace = 127 or 8 */
    cmp al, 127
    je do_backspace
    cmp al, 8
    je do_backspace

    /* Tab = 9 */
    cmp al, 9
    je do_tab

    /* Printable ASCII or UTF-8 */
    cmp al, 32
    jl byte_skip

    mov dil, al
    call insert_char
    inc r13
    jmp process_next_byte

do_enter:
    mov dil, 10
    call insert_char
    inc r13
    jmp process_next_byte

do_tab:
    mov dil, 9
    call insert_char
    inc r13
    jmp process_next_byte

do_backspace:
    call delete_char_before_cursor
    inc r13
    jmp process_next_byte




do_ctrl_n:
    inc r13
    jmp do_toggle_linenumbers

do_ctrl_s:
    inc r13
    jmp do_toggle_separator

do_ctrl_b:
    inc r13
    jmp do_toggle_blink

do_ctrl_p:
    inc r13
    jmp do_cycle_separator_style

do_alt_n:
    add r13, 2
    jmp do_toggle_linenumbers

do_alt_s:
    add r13, 2
    jmp do_toggle_separator

do_alt_b:
    add r13, 2
    jmp do_toggle_blink

do_alt_p:
    add r13, 2
    jmp do_cycle_separator_style

do_toggle_linenumbers:
    xor byte ptr [cfg_line_numbers], 1
    call compute_text_rows
    cmp byte ptr [cfg_line_numbers], 0
    je msg_lnum_off
    lea rsi, msg_toggle_lnum_on
    mov rcx, msg_toggle_lnum_on_len
    call set_status_msg
    jmp toggle_lnum_done
msg_lnum_off:
    lea rsi, msg_toggle_lnum_off
    mov rcx, msg_toggle_lnum_off_len
    call set_status_msg
toggle_lnum_done:
    jmp process_next_byte

do_toggle_separator:
    xor byte ptr [cfg_separator], 1
    call compute_text_rows
    cmp byte ptr [cfg_separator], 0
    je msg_sep_off
    lea rsi, msg_toggle_sep_on
    mov rcx, msg_toggle_sep_on_len
    call set_status_msg
    jmp toggle_sep_done
msg_sep_off:
    lea rsi, msg_toggle_sep_off
    mov rcx, msg_toggle_sep_off_len
    call set_status_msg
toggle_sep_done:
    jmp process_next_byte

do_toggle_blink:
    xor byte ptr [cfg_blinking_cursor], 1
    call apply_cursor_style
    cmp byte ptr [cfg_blinking_cursor], 0
    je msg_blink_off
    lea rsi, msg_toggle_blink_on
    mov rcx, msg_toggle_blink_on_len
    call set_status_msg
    jmp toggle_blink_done
msg_blink_off:
    lea rsi, msg_toggle_blink_off
    mov rcx, msg_toggle_blink_off_len
    call set_status_msg
toggle_blink_done:
    jmp process_next_byte

do_cycle_separator_style:
    mov rax, [cfg_sep_style]
    inc rax
    cmp rax, 6
    jl store_cycled_style
    xor rax, rax
store_cycled_style:
    mov [cfg_sep_style], rax
    cmp rax, 0
    je cycle_pipe
    cmp rax, 1
    je cycle_thin
    cmp rax, 2
    je cycle_double
    cmp rax, 3
    je cycle_dots
    cmp rax, 4
    je cycle_slash
    jmp cycle_blank

cycle_pipe:
    lea rbx, sep_str_pipe
    mov [sep_active_ptr], rbx
    mov qword ptr [sep_active_len], sep_str_pipe_len
    jmp cycle_done
cycle_thin:
    lea rbx, sep_str_thin
    mov [sep_active_ptr], rbx
    mov qword ptr [sep_active_len], sep_str_thin_len
    jmp cycle_done
cycle_double:
    lea rbx, sep_str_double
    mov [sep_active_ptr], rbx
    mov qword ptr [sep_active_len], sep_str_double_len
    jmp cycle_done
cycle_dots:
    lea rbx, sep_str_dots
    mov [sep_active_ptr], rbx
    mov qword ptr [sep_active_len], sep_str_dots_len
    jmp cycle_done
cycle_slash:
    lea rbx, sep_str_slash
    mov [sep_active_ptr], rbx
    mov qword ptr [sep_active_len], sep_str_slash_len
    jmp cycle_done
cycle_blank:
    lea rbx, sep_str_blank
    mov [sep_active_ptr], rbx
    mov qword ptr [sep_active_len], sep_str_blank_len

cycle_done:
    call compute_text_rows
    jmp process_next_byte

do_ctrl_x:
    mov byte ptr [should_quit], 1
    inc r13
    jmp read_input_done

do_ctrl_o:
    call save_file
    inc r13
    jmp process_next_byte

do_ctrl_c:
    call set_curpos_status
    inc r13
    jmp process_next_byte

do_ctrl_k:
    call cut_current_line
    inc r13
    jmp process_next_byte

do_ctrl_u:
    call paste_cut_buffer
    inc r13
    jmp process_next_byte

byte_skip:
    inc r13
    jmp process_next_byte

do_move_right:
    mov rax, [cursor_idx]
    cmp rax, [text_len]
    jge right_done
    inc qword ptr [cursor_idx]
    call recompute_cursor_coords
right_done:
    ret

do_move_left:
    cmp qword ptr [cursor_idx], 0
    jle left_done
    dec qword ptr [cursor_idx]
    call recompute_cursor_coords
left_done:
    ret

page_up_action:
    mov rcx, [text_rows]
pu_loop:
    push rcx
    call move_cursor_up
    pop rcx
    dec rcx
    jnz pu_loop
    ret

page_down_action:
    mov rcx, [text_rows]
pd_loop:
    push rcx
    call move_cursor_down
    pop rcx
    dec rcx
    jnz pd_loop
    ret

read_input_done:
    pop r14
    pop r13
    pop r12
    pop rbp
    ret

/* -------------------------------------------------------------
 * Buffer Editing Primitives
 * ------------------------------------------------------------- */
insert_char:
    push rbp
    mov rbp, rsp
    push r12
    push r13
    push r14

    /* Check buffer capacity */
    mov rax, [text_len]
    cmp rax, BUFFER_MAX - 16
    jge insert_done

    mov r12b, dil       /* char to insert */
    mov r13, [cursor_idx]
    mov r14, [text_len]

    /* Shift text from cursor_idx right by 1 byte */
    lea rsi, text_buffer
    add rsi, r14        /* rsi = &text_buffer[text_len] */
    dec rsi
shift_right_loop:
    lea rax, text_buffer
    add rax, r13
    cmp rsi, rax
    jl shift_right_done
    mov cl, [rsi]
    mov [rsi + 1], cl
    dec rsi
    jmp shift_right_loop

shift_right_done:
    /* Put char at cursor_idx */
    lea rax, text_buffer
    add rax, r13
    mov [rax], r12b

    inc qword ptr [text_len]
    inc qword ptr [cursor_idx]
    mov byte ptr [is_modified], 1
    mov qword ptr [status_msg_len], 0   /* Clear status message on typing */

    call recompute_cursor_coords

insert_done:
    pop r14
    pop r13
    pop r12
    pop rbp
    ret

delete_char_before_cursor:
    push rbp
    mov rbp, rsp

    cmp qword ptr [cursor_idx], 0
    jle del_before_done

    dec qword ptr [cursor_idx]
    call delete_char_at_cursor

del_before_done:
    pop rbp
    ret

delete_char_at_cursor:
    push rbp
    mov rbp, rsp
    push r12
    push r13
    push r14

    mov r12, [cursor_idx]
    mov r13, [text_len]
    cmp r12, r13
    jge del_at_done

    /* Shift bytes left from cursor_idx + 1 */
    lea rdi, text_buffer
    add rdi, r12        /* destination: &text_buffer[cursor_idx] */
    lea rsi, [rdi + 1]  /* source: &text_buffer[cursor_idx + 1] */
    mov rcx, r13
    sub rcx, r12
    dec rcx
    rep movsb

    dec qword ptr [text_len]
    mov byte ptr [is_modified], 1
    mov qword ptr [status_msg_len], 0

    call recompute_cursor_coords

del_at_done:
    pop r14
    pop r13
    pop r12
    pop rbp
    ret

/* -------------------------------------------------------------
 * Cursor Movement: Up, Down, Home, End
 * ------------------------------------------------------------- */
move_cursor_home:
    push rbp
    mov rbp, rsp

    mov rax, [cursor_idx]
find_home_loop:
    cmp rax, 0
    jle set_home_pos
    lea rbx, text_buffer
    cmp byte ptr [rbx + rax - 1], 10
    je set_home_pos
    dec rax
    jmp find_home_loop

set_home_pos:
    mov [cursor_idx], rax
    call recompute_cursor_coords
    pop rbp
    ret

move_cursor_end:
    push rbp
    mov rbp, rsp

    mov rax, [cursor_idx]
    mov rcx, [text_len]
find_end_loop:
    cmp rax, rcx
    jge set_end_pos
    lea rbx, text_buffer
    cmp byte ptr [rbx + rax], 10
    je set_end_pos
    inc rax
    jmp find_end_loop

set_end_pos:
    mov [cursor_idx], rax
    call recompute_cursor_coords
    pop rbp
    ret

move_cursor_up:
    push rbp
    mov rbp, rsp
    push r12
    push r13

    /* Find start of current line */
    mov rax, [cursor_idx]
cur_line_start:
    cmp rax, 0
    jle move_up_done    /* Already on line 0 */
    lea rbx, text_buffer
    cmp byte ptr [rbx + rax - 1], 10
    je prev_line_end
    dec rax
    jmp cur_line_start

prev_line_end:
    dec rax             /* Index of newline before current line */
    mov r12, rax        /* End of previous line */

    /* Find start of previous line */
prev_line_start:
    cmp rax, 0
    jle found_prev_start
    lea rbx, text_buffer
    cmp byte ptr [rbx + rax - 1], 10
    je found_prev_start
    dec rax
    jmp prev_line_start

found_prev_start:
    /* Start of prev line is in rax, end is in r12 */
    /* Target offset = rax + cursor_col */
    mov rdx, rax
    add rdx, [cursor_col]
    cmp rdx, r12
    jle set_up_idx
    mov rdx, r12
set_up_idx:
    mov [cursor_idx], rdx
    call recompute_cursor_coords

move_up_done:
    pop r13
    pop r12
    pop rbp
    ret

move_cursor_down:
    push rbp
    mov rbp, rsp
    push r12
    push r13

    mov rax, [cursor_idx]
    mov rcx, [text_len]

find_next_nl:
    cmp rax, rcx
    jge move_down_done
    lea rbx, text_buffer
    cmp byte ptr [rbx + rax], 10
    je found_next_nl
    inc rax
    jmp find_next_nl

found_next_nl:
    inc rax             /* Start of next line */
    cmp rax, rcx
    jg move_down_done   /* Past EOF */
    je land_at_eof      /* Exactly at EOF */

    mov r12, rax        /* Start of next line */

scan_next_end:
    cmp rax, rcx
    jge reached_next_end
    lea rbx, text_buffer
    cmp byte ptr [rbx + rax], 10
    je reached_next_end
    inc rax
    jmp scan_next_end

reached_next_end:
    /* r12 = start, rax = end */
    mov rdx, r12
    add rdx, [cursor_col]
    cmp rdx, rax
    jle set_down_idx
    mov rdx, rax
set_down_idx:
    mov [cursor_idx], rdx
    call recompute_cursor_coords
    jmp move_down_done

land_at_eof:
    mov [cursor_idx], rax
    call recompute_cursor_coords

move_down_done:
    pop r13
    pop r12
    pop rbp
    ret

/* -------------------------------------------------------------
 * Cut Line (^K) and Paste Line (^U)
 * ------------------------------------------------------------- */
cut_current_line:
    push rbp
    mov rbp, rsp
    push r12
    push r13

    /* Move to start of line */
    call move_cursor_home
    mov r12, [cursor_idx]       /* Line start */

    /* Find end of line (including newline if present) */
    mov rax, r12
    mov rcx, [text_len]
cut_find_end:
    cmp rax, rcx
    jge cut_found_end
    lea rbx, text_buffer
    cmp byte ptr [rbx + rax], 10
    je cut_include_nl
    inc rax
    jmp cut_find_end

cut_include_nl:
    inc rax                     /* Include '\n' in cut buffer */
cut_found_end:
    mov r13, rax                /* End offset */
    mov rcx, r13
    sub rcx, r12                /* Length to cut */
    test rcx, rcx
    jz cut_done

    /* Copy to cut_buf */
    lea rsi, text_buffer
    add rsi, r12
    lea rdi, cut_buf
    mov [cut_len], rcx
    rep movsb

    /* Delete range [r12 .. r13) from text_buffer */
    cld
    lea rdi, text_buffer
    add rdi, r12
    lea rsi, text_buffer
    add rsi, r13
    mov rcx, [text_len]
    sub rcx, r13
    rep movsb

    mov rax, r13
    sub rax, r12
    sub [text_len], rax
    mov [cursor_idx], r12
    mov byte ptr [is_modified], 1

    call recompute_cursor_coords

cut_done:
    pop r13
    pop r12
    pop rbp
    ret

paste_cut_buffer:
    push rbp
    mov rbp, rsp
    push r12
    push r13

    mov rcx, [cut_len]
    test rcx, rcx
    jz paste_done

    /* Check if fits in buffer */
    mov rax, [text_len]
    add rax, rcx
    cmp rax, BUFFER_MAX - 16
    jge paste_done

    /* Shift text right by cut_len bytes from cursor_idx */
    mov r12, [cursor_idx]
    mov r13, [text_len]

    /* Copy from back */
    lea rsi, text_buffer
    add rsi, r13
    dec rsi                     /* Source: text_buffer + text_len - 1 */

paste_shift_loop:
    lea rax, text_buffer
    add rax, r12
    cmp rsi, rax
    jl paste_shift_done
    mov dl, [rsi]
    mov [rsi + rcx], dl
    dec rsi
    jmp paste_shift_loop

paste_shift_done:
    /* Copy cut_buf into text_buffer at cursor_idx */
    cld
    lea rsi, cut_buf
    lea rdi, text_buffer
    add rdi, r12
    mov rcx, [cut_len]
    rep movsb

    mov rax, [cut_len]
    add [text_len], rax
    add [cursor_idx], rax
    mov byte ptr [is_modified], 1

    call recompute_cursor_coords

paste_done:
    pop r13
    pop r12
    pop rbp
    ret

/* -------------------------------------------------------------
 * Cursor Position Status (^C)
 * ------------------------------------------------------------- */
set_curpos_status:
    push rbp
    mov rbp, rsp

    /* Format string: "line X/Y (Z%), col A/B" */
    lea rdi, status_msg
    mov byte ptr [rdi], 'l'
    mov byte ptr [rdi+1], 'i'
    mov byte ptr [rdi+2], 'n'
    mov byte ptr [rdi+3], 'e'
    mov byte ptr [rdi+4], ' '
    add rdi, 5

    /* Convert cursor_row + 1 */
    mov rax, [cursor_row]
    inc rax
    xor rbx, rbx
    call write_num_to_rdi

    mov byte ptr [rdi], ','
    mov byte ptr [rdi+1], ' '
    mov byte ptr [rdi+2], 'c'
    mov byte ptr [rdi+3], 'o'
    mov byte ptr [rdi+4], 'l'
    mov byte ptr [rdi+5], ' '
    add rdi, 6

    mov rax, [cursor_col]
    inc rax
    xor rbx, rbx
    call write_num_to_rdi

    lea rax, status_msg
    mov rcx, rdi
    sub rcx, rax
    mov [status_msg_len], rcx

    pop rbp
    ret

write_num_to_rdi:
    push r12
    push r13
    lea r12, num_buf
    add r12, 30
    mov byte ptr [r12], 0
    mov rcx, 10
wn_loop:
    xor rdx, rdx
    div rcx
    add dl, '0'
    dec r12
    mov [r12], dl
    test rax, rax
    jnz wn_loop

wn_copy:
    mov al, [r12]
    test al, al
    jz wn_done
    mov [rdi], al
    inc rdi
    inc r12
    jmp wn_copy

wn_done:
    pop r13
    pop r12
    ret
