<!-- Copyright (c) Skunk Ventures LLC | Last modified: 2025-03-07 | SPDX-License-Identifier: MIT -->

# Building for Windows

This guide covers three methods for building Discord Qt on Windows.

## Option 1: GitHub Actions (Easiest)

The project now includes a Windows build workflow at `.github/workflows/build-windows.yml`.

### Automatic builds

Every push to `main` triggers a Windows build. Download artifacts from:
1. Go to your GitHub repository → Actions tab
2. Click on the latest workflow run
3. Download `discord-qt-windows-x64` artifact

### Manual trigger

1. Go to Actions tab in GitHub
2. Select "Windows Build" workflow
3. Click "Run workflow"
4. Download the artifact when complete

## Option 2: Native Windows Build

Build directly on a Windows machine (most reliable method).

### Prerequisites

1. **Install Visual Studio 2019 or 2022**
   - Download from: https://visualstudio.microsoft.com/
   - Install "Desktop development with C++" workload
   - Includes MSVC compiler and Windows SDK

2. **Install Qt 6**
   
   **Option A: Qt Online Installer (Recommended)**
   - Download: https://www.qt.io/download-qt-installer
   - Install Qt 6.5.x or 6.6.x (or latest 6.x)
   - Select: `MSVC 2019 64-bit` or `MSVC 2022 64-bit` component
   - Default path: `C:\Qt\6.5.3\msvc2019_64` (or similar)
   - Include Qt WebEngine if you need in-app captcha (login).
   
   **Option B: Chocolatey**
   ```powershell
   choco install qt6
   ```

3. **Install Rust**
   ```powershell
   # Download from https://rustup.rs/ or use:
   winget install Rustlang.Rustup
   
   # Add MSVC target (should be default)
   rustup target add x86_64-pc-windows-msvc
   ```

4. **Install OpenSSL** (for HTTPS)
   
   **Option A: vcpkg (Recommended)**
   ```powershell
   git clone https://github.com/Microsoft/vcpkg.git
   cd vcpkg
   .\bootstrap-vcpkg.bat
   .\vcpkg integrate install
   .\vcpkg install openssl:x64-windows-static
   ```
   
   **Option B: Pre-built binaries**
   - Download: https://slproweb.com/products/Win32OpenSSL.html
   - Install "Win64 OpenSSL v3.x.x"
   - Set environment variable: `OPENSSL_DIR=C:\Program Files\OpenSSL-Win64`

### Build steps

```powershell
# Open "x64 Native Tools Command Prompt for VS 2022" (or 2019)

cd path\to\discord-qt

# Set Qt 6 environment variables
$env:Qt6_DIR = "C:\Qt\6.5.3\msvc2019_64"
$env:QT_INCLUDE_PATH = "$env:Qt6_DIR\include"
$env:QT_LIBRARY_PATH = "$env:Qt6_DIR\lib"
$env:QMAKE = "$env:Qt6_DIR\bin\qmake.exe"
$env:PATH = "$env:Qt6_DIR\bin;$env:PATH"

# Build
cargo build --release

# Binary will be at: target\release\discord_qt.exe
```

### Collecting dependencies for distribution

After building, you need to bundle Qt DLLs with your executable:

```powershell
# Create distribution folder
mkdir release-windows
copy target\release\discord_qt.exe release-windows\

# Copy Qt 6 DLLs
$qtBin = "C:\Qt\6.5.3\msvc2019_64\bin"
$qtDlls = @(
    "Qt6Core.dll",
    "Qt6Gui.dll",
    "Qt6Qml.dll",
    "Qt6Quick.dll",
    "Qt6Network.dll",
    "Qt6Widgets.dll",
    "Qt6QuickControls2.dll",
    "Qt6QuickTemplates2.dll",
    "Qt6QuickLayouts.dll"
)
foreach ($dll in $qtDlls) {
    if (Test-Path "$qtBin\$dll") { copy "$qtBin\$dll" release-windows\ }
}

# Copy Qt platform plugin
mkdir release-windows\platforms
copy "C:\Qt\6.5.3\msvc2019_64\plugins\platforms\qwindows.dll" release-windows\platforms\

# Copy QML modules (Qt 6 layout)
$qtQml = "C:\Qt\6.5.3\msvc2019_64\qml"
xcopy "$qtQml\QtQuick" release-windows\QtQuick\ /E /I
if (Test-Path "$qtQml\QtQml") { xcopy "$qtQml\QtQml" release-windows\QtQml\ /E /I }

# Create qt.conf (Qt 6 uses QmlImports)
@"
[Paths]
Plugins = .
QmlImports = .
"@ | Out-File -FilePath release-windows\qt.conf -Encoding UTF8

# Optional: Use Qt's windeployqt tool (automatic)
C:\Qt\6.5.3\msvc2019_64\bin\windeployqt.exe release-windows\discord_qt.exe --qmldir src\qml
```

## Option 3: Cross-Compilation from Linux (Advanced)

⚠️ **Warning**: Complex setup, native build recommended instead.

Cross-compiling Qt applications from Linux to Windows requires:
1. MinGW-w64 toolchain
2. Windows Qt libraries built with MinGW
3. Wine (for build tools)

### Prerequisites

```bash
# Install MinGW-w64
sudo apt-get install -y \
    mingw-w64 \
    g++-mingw-w64-x86-64 \
    wine64

# Add Windows target
rustup target add x86_64-pc-windows-gnu

# Download Qt 6 for Windows (MinGW build)
# Extract to /opt/qt6-windows-mingw or similar
```

### Configure Cargo for cross-compilation

Create or edit `~/.cargo/config.toml`:

```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
ar = "x86_64-w64-mingw32-ar"

[target.x86_64-pc-windows-gnu.env]
# Update these paths to match your Qt 6 installation
QT_INCLUDE_PATH = "/opt/qt6-windows-mingw/include"
QT_LIBRARY_PATH = "/opt/qt6-windows-mingw/lib"
QMAKE = "/usr/bin/x86_64-w64-mingw32-qmake6"
```

### Build

```bash
cd discord-qt

# Set up Qt 6 paths for Windows
export QT_INCLUDE_PATH="/opt/qt6-windows-mingw/include"
export QT_LIBRARY_PATH="/opt/qt6-windows-mingw/lib"

# Build
cargo build --release --target x86_64-pc-windows-gnu

# Binary: target/x86_64-pc-windows-gnu/release/discord_qt.exe
```

### Known issues with cross-compilation

- Qt libraries must be built with the same MinGW version
- C++ ABI compatibility issues between Linux and Windows builds
- Missing Windows-specific dependencies
- Debug builds may fail (release builds more reliable)

## Troubleshooting

### "Qt6Core.dll not found" when running

- Ensure Qt 6 DLLs are in the same directory as `discord_qt.exe`
- Or add Qt bin directory to system PATH
- Or use `windeployqt` to auto-collect dependencies

### "Cannot find -lQt6Core" during build

- Verify `Qt6_DIR`, `QT_INCLUDE_PATH`, `QT_LIBRARY_PATH`, and `QMAKE` environment variables
- Check Qt 6 installation path matches your environment variables
- Ensure you're using "x64 Native Tools Command Prompt" on Windows

### qmetaobject build failures

- Ensure MSVC toolchain is properly installed
- Check C++ compiler is in PATH
- Try cleaning build: `cargo clean`

### OpenSSL errors

```powershell
# Set OpenSSL path if using pre-built binaries
$env:OPENSSL_DIR = "C:\Program Files\OpenSSL-Win64"

# Or use vcpkg integration (recommended)
vcpkg integrate install
```

### Qt version mismatches

- Use Qt 6.4+ (project targets Qt 6)
- Ensure all Qt modules are from the same version
- QML imports in `src/qml` use Qt 6 (QtQuick 6, etc.)

## Creating an installer

Use **Inno Setup** or **NSIS** to create a Windows installer:

### Inno Setup script example

```ini
[Setup]
AppName=Discord Qt
AppVersion=0.2.0
DefaultDirName={pf}\DiscordQt
OutputBaseFilename=discord-qt-setup

[Files]
Source: "release-windows\*"; DestDir: "{app}"; Flags: recursesubdirs

[Icons]
Name: "{commondesktop}\Discord Qt"; Filename: "{app}\discord_qt.exe"
```

Compile with Inno Setup Compiler to create `discord-qt-setup.exe`.

## Distribution checklist

Before distributing your Windows build:

- [ ] Test on a clean Windows machine (no Qt installed)
- [ ] Include all Qt 6 DLLs (Core, Gui, Qml, Quick, Network, Widgets, QuickControls2, QuickLayouts)
- [ ] Include `platforms/qwindows.dll` plugin
- [ ] Include QML modules (QtQuick, QtQml, and QtQuick submodules)
- [ ] Include `qt.conf` with `QmlImports = .`
- [ ] Test with different Windows versions (10, 11)
- [ ] Include Visual C++ Redistributable if needed
- [ ] Sign executable with code signing certificate (optional but recommended)

## CI/CD Integration

The GitHub Actions workflow automatically:
- ✅ Builds on Windows Server 2022
- ✅ Installs Qt 6.5.3
- ✅ Compiles with MSVC
- ✅ Collects all dependencies
- ✅ Creates distributable package
- ✅ Uploads as artifact
- ✅ Attaches to releases (if tagged)

To create a release:

```bash
git tag v0.2.0
git push origin v0.2.0
```

The workflow will automatically build and attach `discord-qt-windows-x64.zip` to the release.
