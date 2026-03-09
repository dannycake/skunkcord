#!/bin/bash
# Package Skunkcord with bundled Qt dependencies for distribution
set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Skunkcord - Self-Contained Packaging"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Detect Qt 6 installation (prefer qmake6)
QMAKE="${QMAKE:-qmake6}"
QT_LIBS=$($QMAKE -query QT_INSTALL_LIBS 2>/dev/null || echo "/usr/lib/x86_64-linux-gnu")
QT_PLUGINS=$($QMAKE -query QT_INSTALL_PLUGINS 2>/dev/null || echo "/usr/lib/x86_64-linux-gnu/qt6/plugins")
QT_QML=$($QMAKE -query QT_INSTALL_QML 2>/dev/null || echo "/usr/lib/x86_64-linux-gnu/qt6/qml")
# Use Qt 6 for cargo build when script runs cargo build --release
QT_INCLUDE_PATH="${QT_INCLUDE_PATH:-/usr/include/x86_64-linux-gnu/qt6}"
export QT_INCLUDE_PATH
export QT_LIBRARY_PATH="${QT_LIBRARY_PATH:-$QT_LIBS}"

echo "Qt Paths:"
echo "  Libraries: $QT_LIBS"
echo "  Plugins:   $QT_PLUGINS"
echo "  QML:       $QT_QML"
echo

# Build if needed
if [ ! -f "target/release/skunkcord" ]; then
    echo "→ Building release binary..."
    cargo build --release
    echo
fi

# Clean old package
echo "→ Cleaning old package..."
rm -rf skunkcord-bundle

# Create package directory structure
echo "→ Creating package structure..."
mkdir -p skunkcord-bundle/{lib,plugins/platforms,qml}

# Copy executable
echo "→ Copying executable..."
cp target/release/skunkcord skunkcord-bundle/

# Copy application QML files
echo "→ Copying application QML files..."
cp -r src/qml skunkcord-bundle/app-qml

# Copy Qt 6 libraries
echo "→ Copying Qt libraries..."
QT_LIBRARY_NAMES=(
    "libQt6Core.so.6"
    "libQt6Gui.so.6"
    "libQt6Widgets.so.6"
    "libQt6Quick.so.6"
    "libQt6Qml.so.6"
    "libQt6QmlModels.so.6"
    "libQt6Network.so.6"
    "libQt6DBus.so.6"
    "libQt6XcbQpa.so.6"
    "libQt6QuickControls2.so.6"
    "libQt6QuickTemplates2.so.6"
    "libQt6QuickLayouts.so.6"
)

for lib in "${QT_LIBRARY_NAMES[@]}"; do
    if [ -f "$QT_LIBS/$lib" ]; then
        echo "  - $lib"
        cp -P "$QT_LIBS/$lib"* skunkcord-bundle/lib/ 2>/dev/null || true
    fi
done

# Copy additional dependencies
echo "→ Copying additional dependencies..."
EXTRA_LIBS=(
    "libicui18n.so.??"
    "libicuuc.so.??"
    "libicudata.so.??"
    "libdouble-conversion.so.?"
    "libpcre2-16.so.?"
    "libzstd.so.?"
)

for lib_pattern in "${EXTRA_LIBS[@]}"; do
    for lib in $QT_LIBS/$lib_pattern; do
        if [ -f "$lib" ]; then
            echo "  - $(basename $lib)"
            cp -P "$lib"* skunkcord-bundle/lib/ 2>/dev/null || true
        fi
    done
done

# Copy Qt platform plugins
echo "→ Copying Qt platform plugins..."
if [ -d "$QT_PLUGINS/platforms" ]; then
    echo "  - libqxcb.so (X11 platform)"
    cp "$QT_PLUGINS/platforms/libqxcb.so" skunkcord-bundle/plugins/platforms/
    
    # Copy xcb dependencies
    mkdir -p skunkcord-bundle/plugins/xcbglintegrations
    if [ -d "$QT_PLUGINS/xcbglintegrations" ]; then
        cp "$QT_PLUGINS/xcbglintegrations/"*.so skunkcord-bundle/plugins/xcbglintegrations/ 2>/dev/null || true
    fi
fi

# Copy essential QML modules (Qt 6 layout)
echo "→ Copying QML modules..."
QML_MODULES=(
    "QtQuick"
    "QtQuick/Controls"
    "QtQuick/Layouts"
    "QtQuick/Window.2"
    "QtQml"
)
# Optional: QtWebEngine (captcha) if present
[ -d "$QT_QML/QtWebEngine" ] && QML_MODULES+=("QtWebEngine")

for module in "${QML_MODULES[@]}"; do
    if [ -d "$QT_QML/$module" ]; then
        echo "  - $module"
        cp -r "$QT_QML/$module" skunkcord-bundle/qml/
    fi
done

# Create a symlink so the app can find QML at the expected location
ln -sf app-qml skunkcord-bundle/qml/app

# Create launcher script
echo "→ Creating launcher script..."
cat > skunkcord-bundle/skunkcord.sh << 'LAUNCHER_EOF'
#!/bin/bash
# Skunkcord Launcher - Sets up Qt environment

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Set library path to use bundled Qt libraries
export LD_LIBRARY_PATH="$SCRIPT_DIR/lib:$LD_LIBRARY_PATH"

# Set Qt plugin path
export QT_PLUGIN_PATH="$SCRIPT_DIR/plugins"

# Set QML import path (Qt 6 uses QML_IMPORT_PATH)
export QML_IMPORT_PATH="$SCRIPT_DIR/qml"
export QML2_IMPORT_PATH="$SCRIPT_DIR/qml"

# Use bundled platform plugin
export QT_QPA_PLATFORM_PLUGIN_PATH="$SCRIPT_DIR/plugins/platforms"

# Debug: Uncomment to see Qt debug messages
# export QT_DEBUG_PLUGINS=1
# export QT_LOGGING_RULES="qt.qpa.*=true"

# Run the application
exec "$SCRIPT_DIR/skunkcord" "$@"
LAUNCHER_EOF

chmod +x skunkcord-bundle/skunkcord.sh

# Update the executable's rpath (optional, for cleaner ldd output)
echo "→ Patching executable RPATH..."
if command -v patchelf &> /dev/null; then
    patchelf --set-rpath '$ORIGIN/lib' skunkcord-bundle/skunkcord || echo "  (patchelf failed, will use LD_LIBRARY_PATH instead)"
else
    echo "  (patchelf not found, using LD_LIBRARY_PATH fallback)"
fi

# Create README
echo "→ Creating README..."
cat > skunkcord-bundle/README.txt << 'README_EOF'
Skunkcord Client v0.2.0 - Standalone Bundle
============================================

This is a self-contained package with all Qt dependencies included.
NO Qt installation required on the target system!

SYSTEM REQUIREMENTS
-------------------
- Linux x86_64 (Ubuntu 20.04+, Debian 11+, Fedora 35+, or similar)
- X11 display server (standard on most Linux desktops)
- Basic system libraries (glibc, libstdc++, X11 libs)

Most modern Linux systems have these by default.

RUNNING
-------
Use the launcher script (recommended):
  ./skunkcord.sh

Or run directly:
  ./skunkcord

You can provide your Discord token via:
  - Environment variable: DISCORD_TOKEN=your_token ./skunkcord.sh
  - Command line: ./skunkcord.sh --token your_token
  - Login screen when no token is provided

DIRECTORY STRUCTURE
-------------------
skunkcord.sh       <- Launch script (use this!)
skunkcord          <- Main executable
lib/                <- Bundled Qt libraries
plugins/            <- Qt platform plugins
qml/                <- Qt QML modules
app-qml/            <- Application QML files

Do not separate files - keep this directory structure intact!

TROUBLESHOOTING
---------------
1. If you see "cannot execute binary file":
   chmod +x skunkcord.sh skunkcord

2. If you see Qt platform plugin errors:
   - Make sure you're using X11 (not Wayland)
   - Try: QT_QPA_PLATFORM=xcb ./skunkcord.sh

3. If you see missing library errors:
   ldd ./skunkcord
   Install any missing system libraries (not Qt libraries)

4. For debug output:
   QT_DEBUG_PLUGINS=1 ./skunkcord.sh

SUPPORT
-------
For issues and documentation, visit:
https://github.com/skunkllc/skunkcord

README_EOF

# Create version info
cat > skunkcord-bundle/VERSION << EOF
Skunkcord v0.2.0
Build date: $(date -u +"%Y-%m-%d %H:%M:%S UTC")
Architecture: $(uname -m)
Qt version: $(${QMAKE:-qmake6} -query QT_VERSION 2>/dev/null || echo "6.x")
Bundled: Yes (self-contained)
EOF

echo
echo "✓ Self-contained package created successfully!"
echo
echo "Package details:"
du -sh skunkcord-bundle
echo
echo "Contents:"
tree skunkcord-bundle -L 2 -h 2>/dev/null || find skunkcord-bundle -maxdepth 2 -type f -o -type d | head -20
echo
echo "Next steps:"
echo "  1. Test the package:"
echo "     cd skunkcord-bundle && ./skunkcord.sh"
echo
echo "  2. Create a distributable archive:"
echo "     tar -czf skunkcord-linux-$(uname -m)-standalone.tar.gz skunkcord-bundle/"
echo
echo "  3. Transfer to ANY Linux machine (no Qt required!):"
echo "     tar -xzf skunkcord-linux-$(uname -m)-standalone.tar.gz"
echo "     cd skunkcord-bundle && ./skunkcord.sh"
echo
