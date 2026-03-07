#!/bin/bash
# Package Discord Qt for distribution
set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Discord Qt - Release Packaging"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Build if needed
if [ ! -f "target/release/discord_qt" ]; then
    echo "→ Building release binary..."
    cargo build --release
    echo
fi

# Clean old package
echo "→ Cleaning old package..."
rm -rf discord-qt-release

# Create package directory
echo "→ Creating package structure..."
mkdir -p discord-qt-release

# Copy executable
echo "→ Copying executable..."
cp target/release/discord_qt discord-qt-release/

# Copy QML files
echo "→ Copying QML files..."
cp -r src/qml discord-qt-release/

# Create README
echo "→ Creating README..."
cat > discord-qt-release/README.txt << 'EOF'
Discord Qt Client v0.2.0
========================

A lightweight Discord client built with Rust and Qt.

REQUIREMENTS
------------
Qt 5.15+ runtime libraries must be installed.

Ubuntu/Debian:
  sudo apt install qml-module-qtquick2 qml-module-qtquick-controls2 \
                   qml-module-qtquick-layouts qml-module-qtgraphicaleffects

Fedora/RHEL:
  sudo dnf install qt5-qtquickcontrols2

Arch Linux:
  sudo pacman -S qt5-declarative qt5-graphicaleffects

RUNNING
-------
  ./discord_qt

You can provide your Discord token via:
  - Environment variable: DISCORD_TOKEN=your_token ./discord_qt
  - Command line: ./discord_qt --token your_token
  - Login screen when no token is provided

DIRECTORY STRUCTURE
-------------------
The executable MUST be in the same directory as the 'qml' folder:

  discord_qt          <- executable
  qml/                <- QML files (required)
    main.qml
    Discord.js
    components/
      ...

Do not move the executable without the qml folder!

SUPPORT
-------
For issues and documentation, visit:
https://github.com/your-username/discord-qt

EOF

# Set executable permission
chmod +x discord-qt-release/discord_qt

echo
echo "✓ Package created successfully!"
echo
echo "Location: discord-qt-release/"
echo
echo "Next steps:"
echo "  1. Test the package:"
echo "     cd discord-qt-release && ./discord_qt"
echo
echo "  2. Create a distributable archive:"
echo "     tar -czf discord-qt-linux-$(uname -m).tar.gz discord-qt-release/"
echo
echo "  3. Transfer the archive to another machine and extract:"
echo "     tar -xzf discord-qt-linux-$(uname -m).tar.gz"
echo "     cd discord-qt-release && ./discord_qt"
echo
