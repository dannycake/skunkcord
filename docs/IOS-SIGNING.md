<!-- Copyright (c) Skunk Ventures LLC | Last modified: 2025-03-07 | SPDX-License-Identifier: MIT -->

# iOS Code Signing and TestFlight (GitHub Actions)

The iOS app is built on GitHub Actions macOS runners and can be uploaded to TestFlight so you can install it on your device without a Mac. This document describes the one-time Apple Developer setup and the GitHub secrets you need.

## Overview

1. **Develop on Linux** — edit Rust, QML, C++ in `mobile/ios/qt/`.
2. **Push to GitHub** — the `.github/workflows/ios.yml` workflow runs on `macos-15`.
3. **Workflow** — installs Qt for iOS, builds the Rust static lib, builds the Qt app with CMake, signs it, and (if secrets are set) uploads the IPA to TestFlight.
4. **Install** — open the TestFlight app on your iPhone and install the build.

## One-Time Apple Developer Setup

Do this from a browser at [developer.apple.com](https://developer.apple.com) and [appstoreconnect.apple.com](https://appstoreconnect.apple.com).

### 1. App ID and provisioning

1. Go to **Certificates, Identifiers & Profiles** → **Identifiers**.
2. Click **+** to add a new identifier.
3. Choose **App IDs** → **App** → Continue.
4. **Description:** e.g. “Skunkcord”.
5. **Bundle ID:** choose **Explicit** and enter your bundle ID (the project uses `ink.danny.skunkcord` — see `mobile/ios/qt/CMakeLists.txt`).
6. **Capabilities** — enable what you need:
   - **Push Notifications** — required for remote (APNs) and local notification support. Enable it for the main app ID.
   - **App Groups** — required if you add a **Share Extension** (share sheet / “share grid”) or Notification Service/Content extensions; the main app and extension share a group (e.g. `group.com.yourname.discordqt`). Create the group under **Identifiers** → **App Groups** first, then enable “App Groups” on the App ID and select that group.
7. Register the App ID.
8. **Certificates** — see [Create the certificate (CSR and .p12)](#create-the-certificate-csr-and-p12) below to generate a CSR and then the certificate.
9. **Profiles** → create a **Distribution** provisioning profile (App Store), select this App ID and certificate, then download the `.mobileprovision` file.

If you add a **Share Extension** later, create a **second** App ID with bundle ID `ink.danny.skunkcord.ShareExtension`, enable **App Groups** (same group as the main app), and add a separate provisioning profile for that ID.

**Summary:** The project bundle ID is `ink.danny.skunkcord`. Create the same App ID in the portal and enable **Push Notifications** and **App Groups** (create the group under Identifiers → App Groups first) as needed.

### 2. Create the certificate (CSR and .p12)

Apple asks you to **upload a Certificate Signing Request (CSR)** to create the certificate. You can generate the CSR on **Linux** with OpenSSL (no Mac needed).

#### Option A: Linux (OpenSSL)

1. **Generate a private key and CSR** (run in a secure directory, keep `ios_distribution.key` private):

   ```bash
   openssl genrsa -out ios_distribution.key 2048
   openssl req -new -key ios_distribution.key -out ios_distribution.csr -subj "/CN=Your Name/emailAddress=danny@skunk.so/O=Your Org/C=US"
   ```
   Replace the subject with your name, email, and organization. Use a real email that matches your Apple account if you like.

2. **In Apple Developer portal:** Certificates → **+** → **Apple Distribution** (iOS) → Continue. Choose **"Upload a Certificate Signing Request"**, upload `ios_distribution.csr`, then **Continue** and **Download** the `.cer` file.

3. **Build the .p12** (certificate + your private key) on your machine:

   ```bash
   openssl x509 -inform DER -in AppleDistribution.cer -out certificate.pem
   openssl pkcs12 -export -out build_certificate.p12 -inkey ios_distribution.key -in certificate.pem -password pass:YOUR_P12_PASSWORD
   ```
   Use a strong password for `YOUR_P12_PASSWORD`; store it as the `P12_PASSWORD` GitHub secret. The file `build_certificate.p12` is what you base64-encode for `BUILD_CERTIFICATE_BASE64`.

4. **Securely store or discard** `ios_distribution.key` and `certificate.pem`. You only need `build_certificate.p12` and its password for CI.

#### Option B: macOS (Keychain)

1. On a Mac: **Keychain Access** → menu **Keychain Access** → **Certificate Assistant** → **Request a Certificate From a Certificate Authority**. Enter your email and name, choose "Saved to disk", save the `.certSigningRequest`.
2. In the developer portal: create **Apple Distribution**, upload that CSR, download the `.cer`. Double-click the `.cer` to add it to Keychain.
3. In Keychain Access, find "Apple Distribution", right-click → **Export** → save as `.p12` and set a password. Use that file and password for the GitHub secrets.


### 3. App Store Connect API key

1. Go to **App Store Connect** → **Users and Access** → **Integrations** → **App Store Connect API**.
2. Create a new key with **App Manager** (or **Admin**) role.
3. Download the `.p8` file (only available once). Note the **Key ID** and **Issuer ID**.

### 4. Create the app in App Store Connect

1. **App Store Connect** → **My Apps** → **+** → **New App**.
2. Platform: iOS, name e.g. “Skunkcord”, select the App ID from step 1, set a SKU.
3. You don’t need to submit for review; this creates the app and TestFlight.

## GitHub Secrets and Variables

In your repo: **Settings** → **Secrets and variables** → **Actions**.

### Secrets (required for signing and TestFlight)

**New repository secret** for each:

| Secret | Description |
|--------|-------------|
| `BUILD_CERTIFICATE_BASE64` | Base64 of the `.p12` file: `base64 -i certificate.p12 \| pbcopy` (macOS) or output to a file and paste. |
| `P12_PASSWORD` | Password you set when exporting the `.p12`. |
| `BUILD_PROVISION_PROFILE_BASE64` | Base64 of the `.mobileprovision` file. |
| `KEYCHAIN_PASSWORD` | Any random string; used only to create a temporary keychain on the runner. |
| `APPLE_TEAM_ID` | Your 10‑character Team ID (Membership details in the developer portal). |
| `APP_STORE_CONNECT_KEY_ID` | Key ID from the App Store Connect API key. |
| `APP_STORE_CONNECT_ISSUER_ID` | Issuer ID from App Store Connect API. |
| `APP_STORE_CONNECT_P8` | **Full contents** of the `.p8` file (the private key text). |

### Variables (optional)

You can add **repository variables** for non-sensitive values so they appear in the Variables tab. The workflow uses secrets for all of the above; if you prefer to use a variable for `APPLE_TEAM_ID`, add it under **Variables** and reference it in the workflow as `vars.APPLE_TEAM_ID` (the current workflow uses `secrets.APPLE_TEAM_ID`).

## Behaviour of the workflow

- **Without signing secrets** — the workflow still runs: it builds the Rust lib and the Qt app. Archive/export and TestFlight upload are skipped.
- **With signing secrets** — after a successful build it archives, exports an IPA, and uploads it with `xcrun altool --upload-package` to TestFlight.
- **TestFlight** — after processing (a few minutes), the build appears in App Store Connect → TestFlight. Add yourself as an internal tester (or use a group) and install via the TestFlight app.

## Optional: Build and run on a Mac

If you have a Mac and want to build and run locally:

1. Install Xcode and Qt for iOS (e.g. via [Qt Online Installer](https://www.qt.io/download-qt-installer)).
2. Build the Rust lib:  
   `cargo build --release --target aarch64-apple-ios --lib --no-default-features --features mobile`
3. From `mobile/ios/qt/`:  
   `cmake -B build -G Xcode -DCMAKE_PREFIX_PATH=~/Qt/6.8.3/ios -DCMAKE_TOOLCHAIN_FILE=~/Qt/6.8.3/ios/lib/cmake/Qt6/qt.toolchain.cmake -DRUST_LIB=$PWD/../../target/aarch64-apple-ios/release/libskunkcord.a -DAPPLE_TEAM_ID=YOUR_TEAM_ID`  
   then open `build/Skunkcord.xcodeproj` in Xcode and run on a device or simulator.
