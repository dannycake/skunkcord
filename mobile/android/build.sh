#!/bin/bash
# Build Discord Qt Rust library for Android
# Requires: cargo-ndk, Android NDK (ANDROID_NDK_HOME)
# Mobile build: no Qt/desktop UI, FFI-only (same as iOS)
set -e

cd "$(dirname "$0")/../.."

FEATURES="--no-default-features --features mobile"

echo "Building for Android ARM64..."
cargo ndk -t aarch64-linux-android build --release --lib $FEATURES

echo "Building for Android ARMv7..."
cargo ndk -t armv7-linux-androideabi build --release --lib $FEATURES

echo "Building for Android x86_64 (emulator)..."
cargo ndk -t x86_64-linux-android build --release --lib $FEATURES

echo ""
echo "Libraries built at:"
echo "  target/aarch64-linux-android/release/libdiscord_qt.so"
echo "  target/armv7-linux-androideabi/release/libdiscord_qt.so"
echo "  target/x86_64-linux-android/release/libdiscord_qt.so"
echo ""
echo "Copy these into your Android project's jniLibs/ directory:"
echo "  app/src/main/jniLibs/arm64-v8a/libdiscord_qt.so"
echo "  app/src/main/jniLibs/armeabi-v7a/libdiscord_qt.so"
echo "  app/src/main/jniLibs/x86_64/libdiscord_qt.so"
