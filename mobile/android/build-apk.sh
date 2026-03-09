#!/bin/bash
# Build Skunkcord Android APK (Rust .so + Qt app + androiddeployqt).
#
# Prerequisites:
#   - Qt 6 for Android (e.g. Qt Online Installer, select Android component)
#   - Android SDK + NDK (ANDROID_SDK_ROOT or ANDROID_HOME, ANDROID_NDK_ROOT or ANDROID_NDK_HOME)
#   - cargo-ndk, Rust Android targets: aarch64-linux-android, armv7-linux-androideabi, x86_64-linux-android
#
# Usage:
#   QT_ANDROID_PATH=~/Qt/6.8.3/android_arm64_v8a ./mobile/android/build-apk.sh [--install]
#
# Optional: --install  Install APK to connected device/emulator after build (adb install).
#
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
QT_DIR=""
DO_INSTALL=false

for arg in "$@"; do
    case "$arg" in
        --install) DO_INSTALL=true ;;
        *) echo "Unknown option: $arg"; exit 1 ;;
    esac
done

if [ -z "$QT_ANDROID_PATH" ]; then
    # Try common layout from Qt Online Installer
    if [ -d "$HOME/Qt/6.8.3/android_arm64_v8a" ]; then
        QT_ANDROID_PATH="$HOME/Qt/6.8.3/android_arm64_v8a"
    elif [ -d "$HOME/Qt/6.8/android_arm64_v8a" ]; then
        QT_ANDROID_PATH="$HOME/Qt/6.8/android_arm64_v8a"
    else
        echo "Set QT_ANDROID_PATH to your Qt for Android directory (e.g. ~/Qt/6.8.3/android_arm64_v8a)"
        exit 1
    fi
fi

if [ ! -d "$QT_ANDROID_PATH" ]; then
    echo "QT_ANDROID_PATH is not a directory: $QT_ANDROID_PATH"
    exit 1
fi

# Infer ABI from Qt path (e.g. android_arm64_v8a -> arm64-v8a)
QT_BASE="$(basename "$QT_ANDROID_PATH")"
case "$QT_BASE" in
    android_arm64_v8a)  ABI=arm64-v8a; RUST_TARGET=aarch64-linux-android ;;
    android_armeabi_v7a|android_armv7) ABI=armeabi-v7a; RUST_TARGET=armv7-linux-androideabi ;;
    android_x86_64)    ABI=x86_64; RUST_TARGET=x86_64-linux-android ;;
    android_x86)       ABI=x86; RUST_TARGET=i686-linux-android ;;
    *) echo "Cannot infer ABI from QT_ANDROID_PATH ($QT_BASE). Use android_arm64_v8a, android_armeabi_v7a, or android_x86_64."; exit 1 ;;
esac

ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-$ANDROID_HOME}"
ANDROID_NDK_ROOT="${ANDROID_NDK_ROOT:-$ANDROID_NDK_HOME}"
if [ -z "$ANDROID_SDK_ROOT" ] || [ ! -d "$ANDROID_SDK_ROOT" ]; then
    echo "Set ANDROID_SDK_ROOT (or ANDROID_HOME) to your Android SDK path."
    exit 1
fi
if [ -z "$ANDROID_NDK_ROOT" ] || [ ! -d "$ANDROID_NDK_ROOT" ]; then
    echo "Set ANDROID_NDK_ROOT (or ANDROID_NDK_HOME) to your Android NDK path."
    exit 1
fi

FEATURES="--no-default-features --features mobile"

echo "Building Rust library for $RUST_TARGET..."
cd "$REPO_ROOT"
cargo ndk -t "$RUST_TARGET" build --release --lib $FEATURES

RUST_LIB="$REPO_ROOT/target/$RUST_TARGET/release/libskunkcord.so"
if [ ! -f "$RUST_LIB" ]; then
    echo "Rust library not found: $RUST_LIB"
    exit 1
fi

echo "Configuring Qt Android project..."
cd "$SCRIPT_DIR/qt"
QT_CMAKE="$QT_ANDROID_PATH/bin/qt-cmake"
if [ ! -x "$QT_CMAKE" ]; then
    echo "qt-cmake not found at $QT_CMAKE"
    exit 1
fi

"$QT_CMAKE" -B build -S . \
    -DRUST_LIB="$RUST_LIB" \
    -DANDROID_SDK_ROOT="$ANDROID_SDK_ROOT" \
    -DANDROID_NDK_ROOT="$ANDROID_NDK_ROOT" \
    -G Ninja

echo "Building APK..."
cmake --build build --target apk

APK_DIR="$SCRIPT_DIR/qt/build/android-build"
APK="$(find "$APK_DIR" -maxdepth 1 -name "*.apk" 2>/dev/null | head -1)"
if [ -z "$APK" ]; then
    # Some layouts put APK in build/outputs/apk
    APK="$(find "$APK_DIR" -name "*.apk" 2>/dev/null | head -1)"
fi
if [ -n "$APK" ]; then
    echo "APK: $APK"
    if [ "$DO_INSTALL" = true ]; then
        echo "Installing on device..."
        adb install -r "$APK"
    fi
else
    echo "APK not found under $APK_DIR; check build output."
fi
