#!/bin/bash
# Pre-commit hook: run formatting, linting, and tests before committing.
# Install: cp scripts/pre-commit.sh .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit
set -e

echo "=== Pre-commit checks ==="

echo "[1/4] cargo fmt..."
cargo fmt --all -- --check
if [ $? -ne 0 ]; then
    echo "FAIL: Code is not formatted. Run: cargo fmt --all"
    exit 1
fi

echo "[2/4] cargo clippy..."
cargo clippy --all-targets -- -D warnings 2>/dev/null
if [ $? -ne 0 ]; then
    echo "FAIL: Clippy warnings found."
    exit 1
fi

echo "[3/4] cargo check (zero warnings)..."
OUTPUT=$(cargo check 2>&1)
if echo "$OUTPUT" | grep -q "^warning:"; then
    echo "FAIL: Compiler warnings found."
    echo "$OUTPUT" | grep "^warning:"
    exit 1
fi

echo "[4/4] cargo test..."
cargo test --quiet
if [ $? -ne 0 ]; then
    echo "FAIL: Tests failed."
    exit 1
fi

echo "=== All checks passed ==="
