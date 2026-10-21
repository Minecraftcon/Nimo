#!/usr/bin/env bash
# ==============================================================================
# Nimo Build & Dynamic Environment Setup Script
# Supports: Standard Linux Desktops/Servers & Android Termux
# ==============================================================================

set -e

# ANSI Color Palette (Catppuccin Mocha inspired)
CLR_RESET="\033[0m"
CLR_MAUVE="\033[38;2;203;166;247m"
CLR_BLUE="\033[38;2;137;180;250m"
CLR_GREEN="\033[38;2;166;227;161m"
CLR_YELLOW="\033[38;2;249;226;175m"
CLR_PEACH="\033[38;2;250;179;135m"
CLR_RED="\033[38;2;243;139;168m"
CLR_MUTED="\033[38;2;108;112;134m"
CLR_BOLD="\033[1m"

log_info()    { echo -e "${CLR_BLUE}[INFO]${CLR_RESET} $1"; }
log_step()    { echo -e "${CLR_MAUVE}[STEP]${CLR_RESET} ${CLR_BOLD}$1${CLR_RESET}"; }
log_success() { echo -e "${CLR_GREEN}[SUCCESS]${CLR_RESET} $1"; }
log_warn()    { echo -e "${CLR_YELLOW}[WARN]${CLR_RESET} $1"; }
log_error()   { echo -e "${CLR_RED}[ERROR]${CLR_RESET} $1"; }

banner() {
    echo -e "${CLR_MAUVE}${CLR_BOLD}"
    cat << "EOF"
    _   ___                 
   / | / (_)___ ___  ____   
  /  |/ / / __ `__ \/ __ \  
 / /|  / / / / / / / /_/ /  
/_/ |_/_/_/ /_/ /_/\____/   
EOF
    echo -e "${CLR_RESET}${CLR_MUTED}  Modern Terminal Text Editor in Rust${CLR_RESET}\n"
}

# ------------------------------------------------------------------------------
# 1. Detect Host Environment
# ------------------------------------------------------------------------------
detect_environment() {
    log_step "Detecting runtime environment..."

    IS_TERMUX=false
    if [ -n "$TERMUX_VERSION" ] || [ -d "/data/data/com.termux" ] || [[ "${PREFIX:-}" == *"com.termux"* ]]; then
        IS_TERMUX=true
        ENV_NAME="Android Termux"
        DEFAULT_INSTALL_DIR="${PREFIX:-/data/data/com.termux/files/usr}/bin"
    else
        ENV_NAME="Linux Desktop/Server ($(uname -s) $(uname -m))"
        DEFAULT_INSTALL_DIR="/usr/local/bin"
    fi

    log_info "Target environment: ${CLR_PEACH}${ENV_NAME}${CLR_RESET}"

    # Ensure cargo/rustup paths are in PATH
    for candidate in "$HOME/.cargo/bin" "${PREFIX:-/usr}/bin" "/usr/local/bin"; do
        if [ -d "$candidate" ] && [[ ":$PATH:" != *":$candidate:"* ]]; then
            export PATH="$candidate:$PATH"
        fi
    done
}

# ------------------------------------------------------------------------------
# 2. Dependency Verification & Auto-Configuration
# ------------------------------------------------------------------------------
check_dependencies() {
    log_step "Checking Rust & Cargo toolchain..."

    if ! command -v cargo &> /dev/null; then
        log_warn "Cargo was not found in your PATH."

        if [ "$IS_TERMUX" = true ]; then
            log_info "Termux detected: Attempting automatic installation via pkg..."
            if command -v pkg &> /dev/null; then
                pkg install -y rust binutils
            elif command -v apt &> /dev/null; then
                apt update && apt install -y rust binutils
            fi
        else
            log_info "Linux desktop detected: Checking for rustup..."
            if [ -x "$HOME/.cargo/bin/cargo" ]; then
                export PATH="$HOME/.cargo/bin:$PATH"
            else
                log_error "Rust toolchain is missing."
                echo -e "${CLR_YELLOW}Install Rust using standard rustup:${CLR_RESET}"
                echo -e "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
                echo -e "Or via your package manager (e.g., sudo apt install cargo / sudo pacman -S rust / sudo dnf install cargo)"
                exit 1
            fi
        fi
    fi

    CARGO_VER=$(cargo --version | head -n 1)
    RUSTC_VER=$(rustc --version | head -n 1)
    log_info "Using: ${CLR_GREEN}$CARGO_VER${CLR_RESET}"
    log_info "Using: ${CLR_GREEN}$RUSTC_VER${CLR_RESET}"
}

# ------------------------------------------------------------------------------
# 3. Compilation
# ------------------------------------------------------------------------------
build_project() {
    log_step "Compiling Nimo in release mode..."

    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    cd "$SCRIPT_DIR"

    # Compile with cargo
    cargo build --release

    # Safe atomic replacement of local ./nimo binary (avoids 'Text file busy' if running)
    TARGET_BIN="target/release/nimo"
    if [ -f "$TARGET_BIN" ]; then
        cp -f "$TARGET_BIN" ./nimo.tmp
        mv -f ./nimo.tmp ./nimo
        chmod +x ./nimo
        log_success "Binary built successfully at ${CLR_BOLD}./nimo${CLR_RESET}"
    else
        log_error "Expected binary was not found at $TARGET_BIN"
        exit 1
    fi
}

# ------------------------------------------------------------------------------
# 4. Optional Installation
# ------------------------------------------------------------------------------
install_binary() {
    local target_dir="${1:-$DEFAULT_INSTALL_DIR}"
    log_step "Installing Nimo to ${target_dir}..."

    if [ ! -f "./nimo" ]; then
        build_project
    fi

    if [ "$IS_TERMUX" = true ]; then
        mkdir -p "$target_dir"
        cp -f ./nimo "$target_dir/nimo"
        chmod +x "$target_dir/nimo"
    else
        if [ -w "$target_dir" ]; then
            cp -f ./nimo "$target_dir/nimo"
            chmod +x "$target_dir/nimo"
        else
            log_info "Root permissions required to install to $target_dir. Prompting sudo..."
            sudo cp -f ./nimo "$target_dir/nimo"
            sudo chmod +x "$target_dir/nimo"
        fi
    fi

    log_success "Nimo is installed! You can now run '${CLR_BOLD}nimo${CLR_RESET}' from anywhere."
}

clean_project() {
    log_step "Cleaning build artifacts..."
    cargo clean
    rm -f ./nimo ./nimo.tmp
    log_success "Clean completed."
}

# ------------------------------------------------------------------------------
# Main Dispatcher
# ------------------------------------------------------------------------------
banner
detect_environment

case "${1:-build}" in
    build)
        check_dependencies
        build_project
        ;;
    install)
        check_dependencies
        build_project
        install_binary "${2:-$DEFAULT_INSTALL_DIR}"
        ;;
    clean)
        clean_project
        ;;
    check)
        check_dependencies
        cargo check
        ;;
    *)
        echo -e "${CLR_BOLD}Usage:${CLR_RESET} $0 [build|install|clean|check]"
        echo -e "  ${CLR_PEACH}build${CLR_RESET}   - (Default) Checks toolchain & builds release binary to ./nimo"
        echo -e "  ${CLR_PEACH}install${CLR_RESET} - Builds and installs nimo to \$PREFIX/bin (Termux) or /usr/local/bin (Linux)"
        echo -e "  ${CLR_PEACH}clean${CLR_RESET}   - Cleans target/ and local build files"
        echo -e "  ${CLR_PEACH}check${CLR_RESET}   - Runs cargo check verification"
        exit 1
        ;;
esac
