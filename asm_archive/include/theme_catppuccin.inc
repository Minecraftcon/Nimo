/*
 * theme_catppuccin.inc - Catppuccin Mocha Truecolor Palette (24-bit RGB ANSI)
 *
 * Base (Editor BG):       #1e1e2e -> \033[48;2;30;30;46m
 * Surface0 (Status/Lnum): #313244 -> \033[48;2;49;50;68m
 * Mantle (Nano Bar BG):   #181825 -> \033[48;2;24;24;37m
 * Crust (Darker accents): #11111b -> \033[48;2;17;17;27m
 *
 * Text (Foreground):      #cdd6f4 -> \033[38;2;205;214;244m
 * Subtext0 (Muted text):  #a6adc8 -> \033[38;2;166;173;200m
 * Mauve (Accent / Title): #cba6f7 -> \033[38;2;203;166;247m
 * Sapphire (Keywords):    #74c7ec -> \033[38;2;116;199;236m
 * Peach (Shortcuts/Keys): #fab387 -> \033[38;2;250;179;135m
 * Green (Modified/Saved): #a6e3a1 -> \033[38;2;166;227;161m
 * Red (Errors/Unsaved):   #f38ba8 -> \033[38;2;243;139;168m
 * Yellow (Warnings):      #f9e2af -> \033[38;2;249;226;175m
 */

.data

/* Reset code */
ansi_reset:
    .ascii "\033[0m"
ansi_reset_len = . - ansi_reset

/* Clear screen & home cursor */
ansi_clear_all:
    .ascii "\033[2J\033[H"
ansi_clear_len = . - ansi_clear_all

/* Hide & Show cursor */
ansi_hide_cursor:
    .ascii "\033[?25l"
ansi_hide_cursor_len = . - ansi_hide_cursor

ansi_show_cursor:
    .ascii "\033[?25h"
ansi_show_cursor_len = . - ansi_show_cursor

/* Enter & Leave Alternate Screen Buffer (xterm/vt100) */
ansi_enter_alt_screen:
    .ascii "\033[?1049h"
ansi_enter_alt_screen_len = . - ansi_enter_alt_screen

ansi_leave_alt_screen:
    .ascii "\033[?1049l"
ansi_leave_alt_screen_len = . - ansi_leave_alt_screen

/* Reset Cursor Shape to default */
ansi_reset_cursor:
    .ascii "\033[0 q"
ansi_reset_cursor_len = . - ansi_reset_cursor

/* Editor Text Area Style (Base BG + Text FG) */
style_editor_area:
    .ascii "\033[48;2;30;30;46m\033[38;2;205;214;244m"
style_editor_area_len = . - style_editor_area

/* Line Number Gutter Style (Surface0 BG + Subtext0 FG) */
style_gutter:
    .ascii "\033[48;2;36;37;53m\033[38;2;108;112;134m"
style_gutter_len = . - style_gutter

/* Current Line Gutter Active Style (Surface0 BG + Mauve FG) */
style_gutter_active:
    .ascii "\033[48;2;49;50;68m\033[38;2;203;166;247;1m"
style_gutter_active_len = . - style_gutter_active

/* Top Header Bar Style (Mauve BG + Crust FG bold) */
style_top_bar:
    .ascii "\033[48;2;203;166;247m\033[38;2;17;17;27;1m"
style_top_bar_len = . - style_top_bar

/* Status Bar Style (Surface0 BG + Text FG) */
style_status_bar:
    .ascii "\033[48;2;49;50;68m\033[38;2;205;214;244m"
style_status_bar_len = . - style_status_bar

/* Status Bar Modified Pill (Red BG + Crust FG) */
style_status_mod:
    .ascii "\033[48;2;243;139;168m\033[38;2;17;17;27;1m"
style_status_mod_len = . - style_status_mod

/* Nano Shortcut Key Pill (Peach FG + Crust BG or bold inverted) */
style_shortcut_key:
    .ascii "\033[48;2;250;179;135m\033[38;2;17;17;27;1m"
style_shortcut_key_len = . - style_shortcut_key

/* Nano Shortcut Label (Mantle BG + Text FG) */
style_shortcut_desc:
    .ascii "\033[48;2;24;24;37m\033[38;2;205;214;244m"
style_shortcut_desc_len = . - style_shortcut_desc

/* Nano Footer Bar background */
style_footer_bg:
    .ascii "\033[48;2;24;24;37m\033[38;2;166;173;200m"
style_footer_bg_len = . - style_footer_bg

/* Selection / Highlight Style */
style_highlight:
    .ascii "\033[48;2;88;91;112m\033[38;2;245;224;220;1m"
style_highlight_len = . - style_highlight
