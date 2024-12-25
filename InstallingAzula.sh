#!/bin/bash

function check_command() {
    command -v "$1" >/dev/null 2>&1
}

function install_rust() {
    echo "Rust is not installed. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    echo "Rust installation complete. Adding Rust to PATH."
    export PATH="$HOME/.cargo/bin:$PATH"
    echo "export PATH=\"$HOME/.cargo/bin:\$PATH\"" >> ~/.bashrc
    echo "Rust has been added to PATH. Please restart your terminal if this is the first time."
}

function setup_azula() {
    echo "Setting up Azula project..."

    if [ ! -d "azula" ]; then
        echo "Cloning Azula from GitHub..."
        git clone https://github.com/AIAnsar1/Azula.git
    fi

    cd azula || exit
    echo "Building Azula..."
    cargo build --release
    echo "Installing Azula binary..."
    sudo cp target/release/azula /usr/local/bin/azula
    echo "Azula installed successfully. You can now run it using the 'azula' command."
    echo "Running Azula..."
    azula
}

function detect_os() {
    case "$(uname -s)" in
        Linux*)     os="Linux";;
        Darwin*)    os="Mac";;
        CYGWIN*|MINGW*|MSYS*) os="Windows";;
        *)          os="Unknown";;
    esac
    echo "Operating System detected: $os"
}

detect_os

if ! check_command "rustc"; then
    install_rust
else
    echo "Rust is already installed."
fi

setup_azula