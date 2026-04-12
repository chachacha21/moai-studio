#!/bin/bash
set -euo pipefail
# ghostty-vt.xcframework 빌드 (Ghostty 1.3.0+에서 이름 변경)
# Requires: zig 0.15+, Metal Toolchain, Xcode

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

if ! command -v zig &> /dev/null; then
    echo "ERROR: zig is not installed. Run: brew install zig"
    exit 1
fi

if ! xcrun -sdk macosx metal --version &> /dev/null 2>&1; then
    echo "ERROR: Metal Toolchain not installed."
    echo "Run: xcodebuild -downloadComponent MetalToolchain"
    exit 1
fi

GHOSTTY_DIR="$PROJECT_ROOT/vendor/ghostty"
if [ ! -d "$GHOSTTY_DIR" ]; then
    echo "ERROR: vendor/ghostty not found. Run: git submodule update --init"
    exit 1
fi

cd "$GHOSTTY_DIR"
echo "Building ghostty-vt.xcframework (this may take several minutes)..."
zig build -Doptimize=ReleaseFast

XCFW="zig-out/lib/ghostty-vt.xcframework"
if [ ! -d "$XCFW" ]; then
    echo "ERROR: xcframework not found at $XCFW"
    echo "Check zig build output for errors."
    exit 1
fi

mkdir -p "$PROJECT_ROOT/app/Frameworks"
cp -R "$XCFW" "$PROJECT_ROOT/app/Frameworks/"
echo "ghostty-vt.xcframework ($(du -sh "$XCFW" | cut -f1)) copied to app/Frameworks/"
echo "Build successful."
