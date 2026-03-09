# Skunkcord Client — Makefile
# Convenience targets for common development tasks

# Qt paths (Ubuntu/Debian defaults; Qt 6)
export QT_INCLUDE_PATH ?= /usr/include/x86_64-linux-gnu/qt6
export QT_LIBRARY_PATH ?= /usr/lib/x86_64-linux-gnu
export QMAKE ?= /usr/bin/qmake6

.PHONY: build test check fmt clippy doc clean run install-hooks

# Build in debug mode
build:
	cargo build

# Build in release mode
release:
	cargo build --release

# Run all tests
test:
	cargo test --all

# Quick check (no linking)
check:
	cargo check

# Format all code
fmt:
	cargo fmt --all

# Check formatting without modifying
fmt-check:
	cargo fmt --all -- --check

# Run clippy linter
clippy:
	cargo clippy --all-targets

# Generate documentation
doc:
	cargo doc --no-deps --document-private-items --open

# Clean build artifacts
clean:
	cargo clean

# Run the application
run:
	cargo run

# Run UI test mode (mock data, no Discord connection)
ui-test:
	cargo run --bin ui_test

# Run with a token
run-token:
	@echo "Usage: DISCORD_TOKEN=your_token make run"

# Install pre-commit hooks
install-hooks:
	cp scripts/pre-commit.sh .git/hooks/pre-commit
	chmod +x .git/hooks/pre-commit
	@echo "Pre-commit hook installed."

# Full CI check (same as CI pipeline)
ci: fmt-check clippy test doc
	@echo "All CI checks passed."

# Build Rust lib for Android only (requires cargo-ndk + Android NDK)
android:
	./mobile/android/build.sh

# Build full Android APK (requires Qt 6 for Android, SDK/NDK; optional: make android-apk install)
android-apk:
	./mobile/android/build-apk.sh

# Build for iOS (requires Xcode + iOS targets)
ios:
	./mobile/ios/build.sh

# Build and run mobile code path on Linux (no emulator; requires system Qt 6)
mobile-linux:
	./mobile/linux-test/build.sh

# Show project stats
stats:
	@echo "Files:    $$(find src -name '*.rs' | wc -l) Rust + $$(find tests -name '*.rs' | wc -l) test"
	@echo "Lines:    $$(find src tests -name '*.rs' -exec cat {} + | wc -l) Rust + $$(wc -l < src/qml/main.qml) QML"
	@echo "Tests:    $$(cargo test 2>&1 | grep '^test ' | wc -l)"
	@echo "Modules:  $$(find src/client -name '*.rs' ! -name 'mod.rs' | wc -l) client"
