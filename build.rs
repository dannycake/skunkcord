// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

fn main() {
    // Skip Qt/C++ build for mobile (iOS/Android use shared C++ in mobile/shared/)
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("apple-ios") || target.contains("android") {
        return;
    }
    cpp_build::build("src/lib.rs");
}
