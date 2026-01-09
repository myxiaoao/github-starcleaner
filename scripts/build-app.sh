#!/bin/bash
set -e

APP_NAME="GitHub Star Cleaner"
CONFIGURATION="${1:-Release}"

echo "Building ${APP_NAME} (${CONFIGURATION})..."

# Use xcodebuild for native macOS app bundle
xcodebuild -project GitHubStarCleaner.xcodeproj \
    -scheme "GitHub Star Cleaner" \
    -configuration "${CONFIGURATION}" \
    build

# Get the build products directory
BUILD_DIR=$(xcodebuild -project GitHubStarCleaner.xcodeproj \
    -scheme "GitHub Star Cleaner" \
    -configuration "${CONFIGURATION}" \
    -showBuildSettings 2>/dev/null | grep -m 1 "BUILT_PRODUCTS_DIR" | awk '{print $3}')

APP_PATH="${BUILD_DIR}/${APP_NAME}.app"

echo ""
echo "Build succeeded!"
echo "App bundle created at: ${APP_PATH}"
echo ""
echo "To install:"
echo "  cp -r \"${APP_PATH}\" /Applications/"
echo ""
echo "Or open directly:"
echo "  open \"${APP_PATH}\""
