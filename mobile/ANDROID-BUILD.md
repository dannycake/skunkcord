<!-- Copyright (c) Skunk Ventures LLC | Last modified: 2025-03-07 | SPDX-License-Identifier: MIT -->

# Android Qt Build Guide

## Quick Summary

Android build now mirrors the iOS setup: shared C++ (`mobile/shared/`) + QML resources + Rust FFI → APK or Linux test.

**Validated on Linux**: `make mobile-linux` builds and runs the mobile C++/FFI/QML stack using your system Qt (Qt5/Qt6).

## Build Targets

### 1. Full Android APK (on device/emulator)

```bash
# Prerequisites: Qt for Android, Android SDK/NDK, cargo-ndk, Rust targets
export QT_ANDROID_PATH=~/Qt/6.8.3/android_arm64_v8a
export ANDROID_SDK_ROOT=~/Android/Sdk
export ANDROID_NDK_ROOT=~/Android/Sdk/ndk/<version>

make android-apk
# Or: ./mobile/android/build-apk.sh --install  (to install on device)
```

**What it does**:
1. Builds `libskunkcord.so` for the target ABI (aarch64-linux-android, etc.) with `--no-default-features --features mobile`
2. Runs `qt-cmake` from Qt for Android with the NDK toolchain
3. Links shared C++ (`mobile/shared/main.cpp`, `AppController.cpp`) + Rust `.so`
4. Bundles QML (main.qml, components, Discord.js) as Qt resources
5. Builds the APK via CMake's `apk` target
6. Optionally installs with `adb`

### 2. Linux Mobile Test (fastest iteration)

```bash
make mobile-linux
# Or: ./mobile/linux-test/build.sh
```

**What it does**:
1. Builds `libskunkcord.so` (x86_64) with mobile features
2. Builds the same shared C++ + QML (as resources) using your system Qt (Qt5 or Qt6)
3. Runs the binary: validates **Rust FFI → C++ AppController → QML UI** without an emulator

**Binary**: `mobile/linux-test/build/skunkcord_mobile_test`

**Tested on this machine**: ✓ Builds with Qt5, loads QML from `qrc:/`, C FFI connects.

### 3. Rust library only

```bash
make android
# Or: ./mobile/android/build.sh
```

Builds `libskunkcord.so` for ARM64, ARMv7, x86_64. Use if you have a separate Android project (JNI/Kotlin).

## Architecture

```
mobile/
├── shared/              ← C++ (main.cpp, AppController.h/cpp) shared by all platforms
├── android/
│   ├── build.sh        ← Rust .so only (3 ABIs)
│   ├── build-apk.sh    ← Full APK build (Rust + Qt + androiddeployqt)
│   └── qt/             ← Qt Android project (CMakeLists, AndroidManifest.xml)
├── ios/
│   ├── build.sh        ← Rust .a (device + sim)
│   └── qt/             ← Qt iOS project (now uses ../../shared/)
└── linux-test/
    ├── build.sh        ← Rust + Qt on Linux host (mobile code path)
    └── CMakeLists.txt  ← Uses shared C++, Qt5/Qt6, QML resources
```

## Mobile FFI vs Desktop

| Layer | Desktop | Mobile (iOS/Android/linux-test) |
|-------|---------|----------------------------------|
| **Rust** | Full with `qmetaobject` | C FFI only (`mobile_ffi.rs`) |
| **C++** | None (Rust→Qt direct) | `AppController.cpp` (C FFI→Qt) |
| **QML** | Filesystem (`src/qml/`) | Bundled resources (`qrc:/qml/`) |
| **Features** | `desktop` (default) | `mobile` (`--no-default-features`) |

## CI

- **iOS**: `.github/workflows/ios.yml` → TestFlight
- **Android**: `.github/workflows/android.yml` → APK artifact (no signing; add later)

## Next Steps for Real Android Testing

1. Install Qt for Android: [Qt Online Installer](https://www.qt.io/download-qt-installer) → select Android component (6.8+)
2. Install Android Studio or SDK tools → set `ANDROID_SDK_ROOT`, `ANDROID_NDK_ROOT`
3. `rustup target add aarch64-linux-android`; `cargo install cargo-ndk`
4. Run `./mobile/android/build-apk.sh` → produces unsigned debug APK
5. `adb install` or `./mobile/android/build-apk.sh --install`

For now: **use `make mobile-linux` for fast iteration** (same C++/FFI/QML as Android/iOS, no emulator).
