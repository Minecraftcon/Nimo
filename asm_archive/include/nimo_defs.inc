/*
 * nimo_defs.inc - System call numbers and terminal constants for Nimo
 * Architecture: x86_64 Linux
 */

.set SYS_read,    0
.set SYS_write,   1
.set SYS_open,    2
.set SYS_close,   3
.set SYS_ioctl,   16
.set SYS_exit,    60

.set STDIN_FILENO,  0
.set STDOUT_FILENO, 1
.set STDERR_FILENO, 2

/* Termios / ioctl constants */
.set TCGETS,      0x5401
.set TCSETSF,     0x5404
.set TIOCGWINSZ,  0x5413

/* termios offsets */
.set TERMIOS_IFLAG, 0
.set TERMIOS_OFLAG, 4
.set TERMIOS_CFLAG, 8
.set TERMIOS_LFLAG, 12
.set TERMIOS_CC,    17

/* Termios flags to mask */
/* iflag: ~(BRKINT | ICRNL | INPCK | ISTRIP | IXON) */
/* 0x2 | 0x100 | 0x10 | 0x20 | 0x400 = 0x532 */
.set IFLAG_MASK, 0x532

/* oflag: ~(OPOST) */
.set OFLAG_MASK, 0x1

/* cflag: CS8 (0x30) */
.set CS8, 0x30

/* lflag: ~(ECHO | ICANON | IEXTEN | ISIG) */
/* 0x8 | 0x2 | 0x8000 | 0x1 = 0x800b */
.set LFLAG_MASK, 0x800b

/* c_cc offsets */
.set VMIN_IDX,  6
.set VTIME_IDX, 5

/* Text buffer capacity: 1MB for smooth fast editing */
.set BUFFER_MAX, 1048576
.set RENDER_BUF_MAX, 262144
