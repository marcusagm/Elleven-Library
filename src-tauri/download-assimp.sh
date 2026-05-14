#!/bin/bash
# Script to download/setup Assimp binaries for Mundam
# This script ensures the local assimp directory is properly populated

set -e

ASSIMP_DIR="$(dirname "$0")/assimp"
mkdir -p "$ASSIMP_DIR"

echo "=== Assimp Download Script for Mundam ==="
echo ""

OS_TYPE=$(uname -s)
ARCH=$(uname -m)

case "${OS_TYPE}" in
    Darwin)
        PLATFORM="macos"
        BINARY_NAME="assimp"
        LIB_NAME="libassimp.dylib"
        echo "Detected: macOS"
        ;;
    Linux)
        PLATFORM="linux"
        BINARY_NAME="assimp"
        LIB_NAME="libassimp.so"
        echo "Detected: Linux"
        ;;
    MINGW*|MSYS*|CYGWIN*|Windows_NT)
        PLATFORM="windows-x64"
        BINARY_NAME="assimp.exe"
        LIB_NAME="assimp-vc143-mt.dll"
        echo "Detected: Windows"
        ;;
    *)
        echo "Unsupported OS: ${OS_TYPE}"
        exit 1
        ;;
esac

# Check if binaries already exist in the platform-specific folder
if [ -f "$ASSIMP_DIR/$PLATFORM/$BINARY_NAME" ] || [ -f "$ASSIMP_DIR/$PLATFORM/bin/$BINARY_NAME" ]; then
    echo "Assimp binary already exists in $ASSIMP_DIR/$PLATFORM"
else
    echo "Assimp binary not found in $ASSIMP_DIR/$PLATFORM"
    
    # Try to find in system and copy (similar to download-ffmpeg.sh)
    if command -v assimp &> /dev/null; then
        SYSTEM_ASSIMP=$(which assimp)
        echo "Found system Assimp at: $SYSTEM_ASSIMP"
        mkdir -p "$ASSIMP_DIR/$PLATFORM/bin"
        cp "$SYSTEM_ASSIMP" "$ASSIMP_DIR/$PLATFORM/bin/$BINARY_NAME"
        echo "Copied system Assimp to $ASSIMP_DIR/$PLATFORM/bin/$BINARY_NAME"
    else
        echo "Please install Assimp on your system or provide the binary manually."
        if [ "$PLATFORM" == "macos" ]; then
            echo "On macOS: brew install assimp"
        elif [ "$PLATFORM" == "linux" ]; then
            echo "On Linux: sudo apt install assimp"
        fi
    fi
fi

echo ""
echo "=== Assimp Download Complete ==="
