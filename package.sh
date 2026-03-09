#!/bin/bash
# Package Skunkcord for distribution
set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Skunkcord - Release Packaging"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Build if needed
if [ ! -f "target/release/skunkcord" ]; then
    echo "→ Building release binary..."
    cargo build --release
    echo
fi

# Clean old package
echo "→ Cleaning old package..."
rm -rf skunkcord-release

# Create package directory
echo "→ Creating package structure..."
mkdir -p skunkcord-release

# Copy executable
echo "→ Copying executable..."
cp target/release/skunkcord skunkcord-release/

# Copy QML files
echo "→ Copying QML files..."
cp -r src/qml skunkcord-release/

# Create README
echo "→ Creating README..."
cat > skunkcord-release/README.txt << 'EOF'
Skunkcord Client v0.2.0
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
  ./skunkcord

You can provide your Discord token via:
  - Environment variable: DISCORD_TOKEN=your_token ./skunkcord
  - Command line: ./skunkcord --token your_token
  - Login screen when no token is provided

DIRECTORY STRUCTURE
-------------------
The executable MUST be in the same directory as the 'qml' folder:

  skunkcord          <- executable
  qml/                <- QML files (required)
    main.qml
    Discord.js
    components/
      ...

Do not move the executable without the qml folder!

SUPPORT
-------
For issues and documentation, visit:
https://github.com/skunkllc/skunkcord

EOF

# Set executable permission
chmod +x skunkcord-release/skunkcord

echo
echo "✓ Package created successfully!"
echo
echo "Location: skunkcord-release/"
echo
echo "Next steps:"
echo "  1. Test the package:"
echo "     cd skunkcord-release && ./skunkcord"
echo
echo "  2. Create a distributable archive:"
echo "     tar -czf skunkcord-linux-$(uname -m).tar.gz skunkcord-release/"
echo
echo "  3. Transfer the archive to another machine and extract:"
echo "     tar -xzf skunkcord-linux-$(uname -m).tar.gz"
echo "     cd skunkcord-release && ./skunkcord"
echo
