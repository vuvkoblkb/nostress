#!/bin/bash

# Rust Super Powerful Full Setup Script
# Supports: Ubuntu/Debian, CentOS/RHEL/Fedora, Arch Linux, macOS

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Ubuntu only - simplified OS detection
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]] && [ -f /etc/debian_version ]; then
        OS="ubuntu"
        print_success "Ubuntu detected - ready to install!"
    else
        print_error "This script is optimized for Ubuntu only!"
        print_error "Current OS: $OSTYPE"
        exit 1
    fi
}

# Update Ubuntu system first
update_ubuntu_system() {
    print_status "Updating Ubuntu system packages..."
    
    # Update package lists
    sudo apt update
    
    # Upgrade existing packages
    print_status "Upgrading existing packages (this may take a while)..."
    sudo apt upgrade -y
    
    # Clean up
    sudo apt autoremove -y
    sudo apt autoclean
    
    print_success "Ubuntu system updated and cleaned!"
}

# Install Ubuntu system dependencies
install_system_deps() {
    print_status "Installing Ubuntu system dependencies..."
    
    # Install essential build tools and dependencies
    sudo apt install -y \
        build-essential \
        curl \
        git \
        pkg-config \
        libssl-dev \
        libudev-dev \
        cmake \
        libsqlite3-dev \
        wget \
        unzip \
        ca-certificates \
        gnupg \
        lsb-release
    
    print_success "Ubuntu system dependencies installed!"
}

# Install Rust 1.87 Stable
install_rust() {
    print_status "Installing Rust 1.87 stable..."
    
    if command -v rustc &> /dev/null; then
        print_warning "Rust is already installed. Updating to 1.87..."
        rustup update stable
    else
        # Install rustup first
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
        source ~/.cargo/env
        
        # Add cargo to PATH for current session
        export PATH="$HOME/.cargo/bin:$PATH"
        
        # Install specific Rust 1.87 stable
        rustup toolchain install 1.87.0
        rustup default 1.87.0
    fi
    
    # Verify we have the correct version
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    if [[ "$RUST_VERSION" == "1.87.0" ]]; then
        print_success "Rust 1.87.0 installed successfully!"
    else
        print_warning "Current Rust version: $RUST_VERSION"
        print_status "Attempting to switch to 1.87.0..."
        rustup toolchain install 1.87.0
        rustup default 1.87.0
    fi
}

# Install stable toolchains only (no experimental features)
install_toolchains() {
    print_status "Installing stable Rust components..."
    
    # Add useful stable targets
    rustup target add wasm32-unknown-unknown
    rustup target add x86_64-unknown-linux-musl
    
    # Add stable components only
    rustup component add rustfmt
    rustup component add clippy
    rustup component add rust-src
    
    print_success "Stable Rust components installed!"
}

# Install stable cargo tools only
install_cargo_tools() {
    print_status "Installing stable cargo tools (this may take a while)..."
    
    # Essential stable tools only
    TOOLS=(
        "cargo-watch"          # Auto-rebuild on file changes
        "cargo-edit"           # Add/remove dependencies easily
        "cargo-audit"          # Security audit
        "cargo-outdated"       # Check for outdated dependencies
        "cargo-tree"           # Dependency tree visualization
        "cargo-update"         # Update installed cargo tools
        "cargo-cache"          # Cargo cache management
        "tokei"                # Code statistics
        "hyperfine"            # Command-line benchmarking
        "cargo-generate"       # Project templates
        "cargo-nextest"        # Modern test runner
        "mdbook"               # Documentation generator
    )
    
    for tool in "${TOOLS[@]}"; do
        print_status "Installing $tool..."
        if cargo install "$tool"; then
            print_success "$tool installed!"
        else
            print_warning "Failed to install $tool, continuing..."
        fi
    done
    
    print_success "Stable cargo tools installation completed!"
}

# Create sample project with powerful setup
create_sample_project() {
    print_status "Creating sample project..."
    
    PROJECT_DIR="$HOME/rust_super_project"
    
    if [ -d "$PROJECT_DIR" ]; then
        print_warning "Project directory already exists. Skipping..."
        return
    fi
    
    cargo new "$PROJECT_DIR"
    cd "$PROJECT_DIR"
    
    # Add powerful dependencies
    cargo add tokio --features full
    cargo add serde --features derive
    cargo add serde_json
    cargo add anyhow
    cargo add thiserror
    cargo add tracing
    cargo add tracing-subscriber
    cargo add clap --features derive
    cargo add reqwest --features json
    
    # Create optimized Cargo.toml
    cat >> Cargo.toml << 'EOF'

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true

[profile.dev]
opt-level = 0
debug = true
split-debuginfo = "unpacked"

[profile.bench]
opt-level = 3
debug = false
rpath = false
lto = true
debug-assertions = false
codegen-units = 1
panic = "abort"
EOF
    
    # Create a sample main.rs with modern Rust features
    cat > src/main.rs << 'EOF'
use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use tokio;
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name to greet
    #[arg(short, long, default_value = "World")]
    name: String,
    
    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct GreetingResponse {
    message: String,
    timestamp: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    info!("Starting Rust Super Powerful App!");
    
    for i in 1..=args.count {
        let greeting = GreetingResponse {
            message: format!("Hello, {}! ({})", args.name, i),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        println!("{}", serde_json::to_string_pretty(&greeting)?);
        
        if i < args.count {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }
    
    warn!("App completed successfully!");
    Ok(())
}
EOF

    # Add chrono dependency
    cargo add chrono --features serde
    
    print_success "Sample project created at $PROJECT_DIR"
}

# Setup development environment
setup_dev_environment() {
    print_status "Setting up development environment..."
    
    # Create useful aliases
    ALIAS_FILE="$HOME/.rust_aliases"
    cat > "$ALIAS_FILE" << 'EOF'
# Rust Development Aliases
alias cr='cargo run'
alias cb='cargo build'
alias ct='cargo test'
alias cc='cargo check'
alias cf='cargo fmt'
alias ccl='cargo clippy'
alias cw='cargo watch -x run'
alias cwt='cargo watch -x test'
alias cwc='cargo watch -x check'
alias crel='cargo build --release'
alias cau='cargo audit'
alias cud='cargo udeps'
alias cup='cargo update'
alias ctr='cargo tree'
alias cbl='cargo bloat --release'
alias ccr='cargo criterion'
EOF
    
    # Add to shell profile
    SHELL_PROFILE=""
    if [ -f "$HOME/.bashrc" ]; then
        SHELL_PROFILE="$HOME/.bashrc"
    elif [ -f "$HOME/.zshrc" ]; then
        SHELL_PROFILE="$HOME/.zshrc"
    fi
    
    if [ -n "$SHELL_PROFILE" ]; then
        if ! grep -q "source $ALIAS_FILE" "$SHELL_PROFILE"; then
            echo "source $ALIAS_FILE" >> "$SHELL_PROFILE"
            print_success "Rust aliases added to $SHELL_PROFILE"
        fi
    fi
    
    # Create .cargo/config.toml for global settings
    mkdir -p "$HOME/.cargo"
    cat > "$HOME/.cargo/config.toml" << 'EOF'
[build]
jobs = 0  # Use all CPU cores

[cargo-new]
name = "Your Name"
email = "your.email@example.com"

[net]
retry = 3

[profile.dev]
split-debuginfo = "unpacked"

[profile.release]
strip = true
lto = true
codegen-units = 1
EOF
    
    print_success "Development environment configured!"
}

# Verify installation
verify_installation() {
    print_status "Verifying installation..."
    
    echo "Rust version:"
    rustc --version
    echo ""
    
    echo "Cargo version:"
    cargo --version
    echo ""
    
    echo "Installed targets:"
    rustup target list --installed
    echo ""
    
    echo "Installed stable components:"
    rustup component list --installed
    echo ""
    
    echo "Installed cargo tools:"
    cargo install --list | head -20
    
    print_success "Installation verified!"
}

# Main execution
main() {
    echo "========================================="
    echo "🚀 Ubuntu Rust 1.87 Stable Setup 🚀"
    echo "========================================="
    echo ""
    
    detect_os
    print_status "Ubuntu Rust 1.87 Super Setup Starting..."
    
    update_ubuntu_system
    install_system_deps
    install_rust
    install_toolchains
    install_cargo_tools
    create_sample_project
    setup_dev_environment
    verify_installation
    
    echo ""
    echo "=================================="
    print_success "🎉 SETUP COMPLETED! 🎉"
    echo "=================================="
    echo ""
    echo "Next steps:"
    echo "1. Restart your terminal or run: source ~/.cargo/env"
    echo "2. Try the sample project: cd ~/rust_super_project && cargo run"
    echo "3. Use rust aliases: cr (cargo run), cb (cargo build), etc."
    echo "4. Happy coding with Rust! 🦀"
    echo "@liqga lucu"
}

# Run main function
main "$@"