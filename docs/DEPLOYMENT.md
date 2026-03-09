<!-- Copyright (c) Skunk Ventures LLC | Last modified: 2025-03-07 | SPDX-License-Identifier: MIT -->

# Deployment Guide

This guide explains how to build and deploy Skunkcord to other machines.

## Building for Release

Build the optimized release binary:

```bash
cargo build --release
```

The executable will be at:
- Linux/macOS: `target/release/skunkcord`
- Windows: `target\release\skunkcord.exe`

## Packaging Options

Skunkcord offers two packaging methods:

### Option 1: Standalone Bundle (Recommended)

**Self-contained package with Qt libraries included** - works on ANY Linux system without Qt installation.

```bash
./package-bundle.sh
tar -czf skunkcord-linux-standalone.tar.gz skunkcord-bundle/
```

- **Size:** ~34 MB
- **Requirements:** Basic system libraries (glibc, X11) - already on most systems
- **Pros:** No Qt installation needed, maximum compatibility
- **Cons:** Larger download size

### Option 2: Minimal Package

**Small package requiring Qt installation** on the target system.

```bash
./package.sh
tar -czf skunkcord-linux.tar.gz skunkcord-release/
```

- **Size:** ~3.6 MB
- **Requirements:** Qt 6 must be installed
- **Pros:** Small download, faster deployment
- **Cons:** Requires Qt installation on every target machine

## Quick Start (Standalone Bundle)

For most users, the standalone bundle is recommended:

```bash
# 1. Build and package
cargo build --release
./package-bundle.sh

# 2. Create archive
tar -czf skunkcord-linux-standalone.tar.gz skunkcord-bundle/

# 3. Transfer to target machine
scp skunkcord-linux-standalone.tar.gz user@remote:~/

# 4. On target machine - extract and run (no Qt needed!)
tar -xzf skunkcord-linux-standalone.tar.gz
cd skunkcord-bundle
./skunkcord.sh
```

## Packaging for Distribution (Detailed)

To deploy Skunkcord to another machine, you need **both**:

1. **The executable** (`skunkcord` or `skunkcord.exe`)
2. **The `qml` directory** with all QML and JavaScript files
3. **(Optional)** Qt libraries for standalone deployment

### Directory Structure

Your deployment package should look like this:

```
skunkcord/              # or skunkcord.exe on Windows
qml/
├── main.qml
├── test_ui.qml
├── Discord.js
└── components/
    ├── DAvatar.qml
    ├── DBadge.qml
    ├── DHoverRect.qml
    ├── DScrollBar.qml
    ├── DSeparator.qml
    ├── DText.qml
    ├── GuildIcon.qml
    ├── IconButton.qml
    ├── RolePill.qml
    ├── Twemoji.qml
    └── UserBadges.qml
```

### Copy QML Files

From your project root:

```bash
# Copy the QML directory to the release directory
cp -r src/qml target/release/

# Or create a deployment package
mkdir skunkcord-release
cp target/release/skunkcord skunkcord-release/
cp -r src/qml skunkcord-release/
```

On Windows (PowerShell):

```powershell
# Copy QML directory
Copy-Item -Recurse src\qml target\release\

# Or create a deployment package
New-Item -ItemType Directory -Path skunkcord-release
Copy-Item target\release\skunkcord.exe skunkcord-release\
Copy-Item -Recurse src\qml skunkcord-release\
```

## Standalone Bundle Details

The standalone bundle (`package-bundle.sh`) includes:

- **Qt 6 Libraries:** Core, Gui, Widgets, Quick, Qml, QmlModels, Network, DBus, XcbQpa, QuickControls2, QuickLayouts
- **ICU Libraries:** Internationalization support (i18n, uc, data)
- **Qt Plugins:** Platform plugins (xcb for X11), QML modules
- **QML Modules:** QtQuick, QtQuick/Controls, QtQuick/Layouts, QtQml (and optionally QtWebEngine for captcha)
- **Launcher Script:** Sets up library paths automatically

The bundle works on:
- Ubuntu 20.04+ / Debian 11+
- Fedora 35+ / RHEL 9+
- Arch Linux (current)
- Any modern Linux with glibc 2.31+ and X11

**No Qt installation required on target system!**

## System Requirements

### Standalone Bundle Requirements

Minimal requirements (already present on most Linux systems):
- Linux x86_64
- glibc 2.31+
- X11 display server (standard on most desktops)
- Basic system libraries (libstdc++, libm, libpthread)

### Minimal Package Requirements

**Qt 6 libraries** must be installed on the target system:

#### Linux (Ubuntu/Debian)
```bash
sudo apt install qt6-base-dev qt6-declarative-dev qt6-qmake6 \
                 qml6-module-qtquick qml6-module-qtquick-controls \
                 qml6-module-qtquick-layouts
```

#### Linux (Fedora/RHEL)
```bash
sudo dnf install qt6-qtbase-devel qt6-qtdeclarative-devel \
                 qt6-qtquickcontrols2
```

#### macOS
```bash
brew install qt@6
```

#### Windows

Download and install Qt 6 from:
- https://www.qt.io/download-qt-installer

Or install via vcpkg:
```cmd
vcpkg install qt6-base:x64-windows qt6-declarative:x64-windows
```

## Automated Packaging Script

Create a deployment script to automate packaging:

### Linux/macOS (`package.sh`)

```bash
#!/bin/bash
set -e

echo "Building release..."
cargo build --release

echo "Creating package..."
rm -rf skunkcord-release
mkdir -p skunkcord-release

# Copy executable
cp target/release/skunkcord skunkcord-release/

# Copy QML files
cp -r src/qml skunkcord-release/

# Create README
cat > skunkcord-release/README.txt << 'EOF'
Skunkcord Client
=================

Requirements:
- Qt 6 runtime libraries

On Ubuntu/Debian:
  sudo apt install qml6-module-qtquick qml6-module-qtquick-controls

Run:
  ./skunkcord

For support, visit: https://github.com/your-username/skunkcord
EOF

echo "Package created at skunkcord-release/"
echo "Create archive with: tar -czf skunkcord-linux.tar.gz skunkcord-release/"
```

### Windows (`package.ps1`)

```powershell
Write-Host "Building release..."
cargo build --release

Write-Host "Creating package..."
Remove-Item -Recurse -Force skunkcord-release -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path skunkcord-release

# Copy executable
Copy-Item target\release\skunkcord.exe skunkcord-release\

# Copy QML files
Copy-Item -Recurse src\qml skunkcord-release\

# Create README
@"
Skunkcord Client
=================

Requirements:
- Qt 6 runtime libraries

Download Qt 6 from: https://www.qt.io/download-qt-installer

Run:
  skunkcord.exe

For support, visit: https://github.com/your-username/skunkcord
"@ | Out-File -Encoding UTF8 skunkcord-release\README.txt

Write-Host "Package created at skunkcord-release\"
Write-Host "Create archive with: Compress-Archive skunkcord-release skunkcord-windows.zip"
```

Make the script executable (Linux/macOS):
```bash
chmod +x package.sh
./package.sh
```

## Troubleshooting

### "QQmlApplicationEngine failed to load component"

This error means the QML files are missing. Ensure:
1. The `qml/` directory is in the same directory as the executable
2. All QML files and the `components/` subdirectory are present
3. File permissions are correct (files should be readable)

### "Cannot find Qt libraries"

Install Qt runtime libraries on the target system (see Runtime Dependencies above).

### Testing the Package

Before distributing, test the package on a clean system:

```bash
# Extract package to temporary location
cd /tmp
tar -xzf skunkcord-linux.tar.gz
cd skunkcord-release

# Run
./skunkcord
```

## Static Linking (Advanced)

To create a fully standalone executable without Qt dependencies, you can statically link Qt. This requires building Qt from source with static configuration.

See: https://doc.qt.io/qt-5/linux-deployment.html

## Distribution Checklist

### For Standalone Bundle (Recommended)

- [ ] Run `./package-bundle.sh` to create self-contained package
- [ ] Test on clean system without Qt installed
- [ ] Verify launcher script (`skunkcord.sh`) works
- [ ] Check all QML files load correctly
- [ ] Create archive: `tar -czf skunkcord-standalone.tar.gz skunkcord-bundle/`
- [ ] Include README with launch instructions

### For Minimal Package

- [ ] Run `./package.sh` to create minimal package
- [ ] Copy QML directory alongside executable  
- [ ] Test on system with Qt 6 installed
- [ ] Include README with Qt installation instructions
- [ ] Test with different Qt versions if possible
- [ ] Create archive: `tar -czf skunkcord.tar.gz skunkcord-release/`

## Comparison: Standalone vs Minimal

| Feature | Standalone Bundle | Minimal Package |
|---------|------------------|-----------------|
| **Size** | ~34 MB | ~3.6 MB |
| **Qt Required** | ❌ No | ✅ Yes (Qt 6) |
| **Compatibility** | Works everywhere | Requires Qt install |
| **Setup Time** | Extract & run | Install Qt + extract |
| **Best For** | End users, distribution | Developers, Qt users |
| **Script** | `package-bundle.sh` | `package.sh` |

## GitHub Actions (CI/CD)

For automated builds, see `.github/workflows/` for example build configurations that package the QML files automatically.
