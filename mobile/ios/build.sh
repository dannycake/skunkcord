#!/bin/bash
# Build Discord Qt Rust library for iOS
# Requires: macOS with Xcode, Rust iOS targets
#   rustup target add aarch64-apple-ios aarch64-apple-ios-sim
#
# iOS build does NOT work on Linux (needs xcrun/iOS SDK).
set -e

cd "$(dirname "$0")/../.."

# Check for Xcode (required for iOS toolchain)
if ! command -v xcrun &>/dev/null; then
    echo "Error: iOS build requires macOS with Xcode (xcrun not found)."
    echo "Run this script on a Mac with Xcode installed."
    exit 1
fi

# Mobile build: no Qt/desktop UI, FFI-only
FEATURES="--no-default-features --features mobile"

echo "Building for iOS ARM64 (device)..."
cargo build --release --target aarch64-apple-ios --lib $FEATURES

echo "Building for iOS ARM64 Simulator..."
cargo build --release --target aarch64-apple-ios-sim --lib $FEATURES

echo ""
echo "Libraries built at:"
echo "  target/aarch64-apple-ios/release/libdiscord_qt.a"
echo "  target/aarch64-apple-ios-sim/release/libdiscord_qt.a"
echo ""
echo "Link into your Xcode project as a static library."
