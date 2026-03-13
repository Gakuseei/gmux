#!/bin/bash
set -e

BINARY_NAME="gmux-native"
INSTALL_DIR="/usr/local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"

echo "Building gmux-native (release)..."
cargo build --release

echo "Installing binary..."
sudo install -m 755 target/release/gmux "$INSTALL_DIR/$BINARY_NAME"

echo "Installing desktop file..."
mkdir -p "$DESKTOP_DIR"
cat > "$DESKTOP_DIR/gmux-native.desktop" << EOF
[Desktop Entry]
Name=gmux (Native)
Comment=GPU-accelerated terminal multiplexer
Exec=$INSTALL_DIR/$BINARY_NAME
Icon=gmux
Type=Application
Categories=System;TerminalEmulator;
Terminal=false
StartupWMClass=gmux
EOF

echo "Installing icon..."
mkdir -p "$ICON_DIR"
if [ -f "../src-tauri/icons/icon.png" ]; then
    cp "../src-tauri/icons/icon.png" "$ICON_DIR/gmux.png"
elif [ -f "../src-tauri/icons/128x128.png" ]; then
    cp "../src-tauri/icons/128x128.png" "$ICON_DIR/gmux.png"
fi

echo "Updating desktop database..."
update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true

echo "Done! Run '$BINARY_NAME' or find 'gmux (Native)' in your application menu."
