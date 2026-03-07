<!-- Copyright (c) Skunk Ventures LLC | Last modified: 2025-03-07 | SPDX-License-Identifier: MIT -->

# Mobile Builds

Shared C++ (main.cpp, AppController) for iOS, Android, and Linux host testing lives in **mobile/shared/**.
The Qt iOS app in **mobile/ios/qt/** and the Qt Android app in **mobile/android/qt/** both use it and the same QML UI as desktop.

## Android

### Option A: Build full APK (Qt + QML, like iOS)

Build the Qt Android app and produce an APK you can run on device or emulator.

**Prerequisites**

- **Qt 6 for Android** — Install via [Qt Online Installer](https://www.qt.io/download-qt-installer) (select Android, e.g. `android_arm64_v8a`).
- **Android SDK** — Set `ANDROID_SDK_ROOT` (or `ANDROID_HOME`) to your SDK path.
- **Android NDK** — Set `ANDROID_NDK_ROOT` (or `ANDROID_NDK_HOME`) to your NDK path.
- **Rust Android targets** — `rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`
- **cargo-ndk** — `cargo install cargo-ndk`

**Build APK**

```bash
# From project root. Qt path is auto-detected under ~/Qt/6.8.3/android_arm64_v8a or set explicitly:
export QT_ANDROID_PATH=~/Qt/6.8.3/android_arm64_v8a   # optional if using default layout
export ANDROID_SDK_ROOT=~/Android/Sdk
export ANDROID_NDK_ROOT=~/Android/Sdk/ndk/<version>

make android-apk
# Or: ./mobile/android/build-apk.sh
```

**Install on device/emulator**

```bash
./mobile/android/build-apk.sh --install
```

The script builds the Rust library for the ABI implied by `QT_ANDROID_PATH` (e.g. `android_arm64_v8a` → arm64-v8a), then configures and builds the Qt app with qt-cmake and creates the APK via CMake’s `apk` target.

### Option B: Build Rust library only

If you only need the Rust `.so` for another Android project (e.g. JNI):

**Prerequisites**

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
cargo install cargo-ndk
export ANDROID_NDK_HOME=$HOME/Android/Sdk/ndk/<version>
```

**Build**

```bash
make android
# Or: ./mobile/android/build.sh
```

This builds `libdiscord_qt.so` for ARM64, ARMv7, and x86_64 (emulator). Use `--no-default-features --features mobile` so desktop Qt dependencies are not built.

Outputs:

- `target/aarch64-linux-android/release/libdiscord_qt.so`
- `target/armv7-linux-androideabi/release/libdiscord_qt.so`
- `target/x86_64-linux-android/release/libdiscord_qt.so`

Copy into an Android project’s `jniLibs/` (e.g. `app/src/main/jniLibs/arm64-v8a/libdiscord_qt.so`) or use the Qt Android app in `mobile/android/qt/` as above.

## iOS

### Option A: GitHub Actions (no Mac required)

Push to `main` (or run the workflow manually). The workflow builds the Qt iOS app and, if [secrets are configured](../docs/IOS-SIGNING.md), signs it and uploads to TestFlight. Install via the TestFlight app on your device.

See **[docs/IOS-SIGNING.md](../docs/IOS-SIGNING.md)** for Apple Developer setup and GitHub secrets.

### Option B: Build on macOS

**Requires macOS with Xcode.** iOS build does not work on Linux (needs xcrun and iOS SDK).

#### Prerequisites

```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
```

Install Qt for iOS (e.g. Qt 6.8 with iOS component from the [Qt Online Installer](https://www.qt.io/download-qt-installer)).

#### Build Rust library

```bash
# From project root (on macOS)
make ios
# Or: ./mobile/ios/build.sh
```

Or manually:

```bash
cargo build --release --target aarch64-apple-ios --lib --no-default-features --features mobile
cargo build --release --target aarch64-apple-ios-sim --lib --no-default-features --features mobile
```

#### Build Qt iOS app

From project root, with Qt for iOS installed (e.g. `~/Qt/6.8.3/ios`):

```bash
cd mobile/ios/qt
cmake -B build -G Xcode \
  -DCMAKE_PREFIX_PATH=~/Qt/6.8.3/ios \
  -DCMAKE_TOOLCHAIN_FILE=~/Qt/6.8.3/ios/lib/cmake/Qt6/qt.toolchain.cmake \
  -DRUST_LIB="$(pwd)/../../../target/aarch64-apple-ios/release/libdiscord_qt.a" \
  -DAPPLE_TEAM_ID=YOUR_TEAM_ID
cmake --build build --config Release
open build/DiscordQt.xcodeproj
```

Then run on a device or simulator from Xcode. The iOS app uses shared C++ from **mobile/shared/** and the same QML as desktop.

## Linux: test mobile code path (no emulator)

You can build and run the same mobile C++/QML/FFI stack on your Linux desktop to test without an Android emulator or device.

**Prerequisites**

- System Qt 6 dev packages (e.g. `libqt6core6-dev`, `libqt6quick6-dev`, `libqt6qml6-dev` on Debian/Ubuntu).
- CMake, Ninja.

**Build and run**

```bash
make mobile-linux
# Or: ./mobile/linux-test/build.sh
```

This builds the Rust library with `--no-default-features --features mobile`, then builds the shared C++ app in **mobile/linux-test/** and runs it. The binary is `mobile/linux-test/build/discord_qt_mobile_test`; set `LD_LIBRARY_PATH` to `target/release` if you run it manually.

## Shared Architecture

```
┌──────────────────────────┐
│    Native UI Layer       │
│  Android: Qt/QML (APK)   │
│  iOS: Qt/QML             │
│  Desktop: Qt/QML         │
├──────────────────────────┤
│    Rust Core Library     │  ← Shared across ALL platforms
│  - Discord API client    │
│  - Gateway WebSocket     │
│  - Voice chat            │
│  - Feature flags         │
│  - Security              │
│  - Message logger        │
│  - All business logic    │
├──────────────────────────┤
│    Platform Adapters     │
│  - C FFI (Android/iOS)   │
│  - Qt bindings (Desktop) │
└──────────────────────────┘
```

## Mobile-Specific Considerations

- **Fingerprint**: Use mobile super properties (Android/iOS) instead of Chrome
- **Notifications**: Use platform push notification APIs
- **Voice**: Use platform audio APIs (AAudio on Android, AVAudioEngine on iOS)
- **Storage**: Use platform-appropriate data directories
- **Background**: Handle app lifecycle (pause/resume gateway on background/foreground)
