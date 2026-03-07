#!/bin/bash
# Build and run the mobile code path on Linux (same C++/QML/FFI as iOS/Android, no emulator).
# Requires: system Qt 6 dev packages (e.g. libqt6core6-dev, libqt6quick6-dev, libqt6qml6-dev).
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "Building Rust library (mobile features)..."
cd "$REPO_ROOT"
cargo build --release --lib --no-default-features --features mobile

# cdylib output on Linux is libdiscord_qt.so
RUST_LIB="$REPO_ROOT/target/release/libdiscord_qt.so"
if [ ! -f "$RUST_LIB" ]; then
    echo "Rust library not found: $RUST_LIB"
    exit 1
fi

echo "Configuring CMake..."
cd "$SCRIPT_DIR"
cmake -B build -S . -DRUST_LIB="$RUST_LIB" -G Ninja

echo "Building..."
cmake --build build

BIN="$SCRIPT_DIR/build/discord_qt_mobile_test"
export LD_LIBRARY_PATH="$REPO_ROOT/target/release${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
# So the QML engine finds QtQuick, QtQuick.Controls, etc. (system Qt6)
QT_QML="$(qmake6 -query QT_INSTALL_QML 2>/dev/null)"
if [ -z "$QT_QML" ] && [ -d /usr/lib/x86_64-linux-gnu/qt6/qml ]; then
    QT_QML=/usr/lib/x86_64-linux-gnu/qt6/qml
fi
if [ -n "$QT_QML" ]; then
    export QT_QML_IMPORT_PATH="$QT_QML${QT_QML_IMPORT_PATH:+:$QT_QML_IMPORT_PATH}"
fi
echo "Run with: $BIN"
exec "$BIN" "$@"
