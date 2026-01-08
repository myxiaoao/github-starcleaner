#!/bin/bash
set -e

# Create iconset directory
ICONSET_DIR="resources/AppIcon.iconset"
mkdir -p "${ICONSET_DIR}"

# Check if source icon exists
if [ ! -f "resources/icon.png" ]; then
    echo "Error: Please provide a 1024x1024 PNG icon at resources/icon.png"
    echo ""
    echo "You can create one or download a star icon, then run this script again."
    exit 1
fi

echo "Generating icon sizes..."

# Generate all required sizes
sips -z 16 16     resources/icon.png --out "${ICONSET_DIR}/icon_16x16.png"
sips -z 32 32     resources/icon.png --out "${ICONSET_DIR}/icon_16x16@2x.png"
sips -z 32 32     resources/icon.png --out "${ICONSET_DIR}/icon_32x32.png"
sips -z 64 64     resources/icon.png --out "${ICONSET_DIR}/icon_32x32@2x.png"
sips -z 128 128   resources/icon.png --out "${ICONSET_DIR}/icon_128x128.png"
sips -z 256 256   resources/icon.png --out "${ICONSET_DIR}/icon_128x128@2x.png"
sips -z 256 256   resources/icon.png --out "${ICONSET_DIR}/icon_256x256.png"
sips -z 512 512   resources/icon.png --out "${ICONSET_DIR}/icon_256x256@2x.png"
sips -z 512 512   resources/icon.png --out "${ICONSET_DIR}/icon_512x512.png"
sips -z 1024 1024 resources/icon.png --out "${ICONSET_DIR}/icon_512x512@2x.png"

echo "Creating icns file..."
iconutil -c icns "${ICONSET_DIR}" -o resources/AppIcon.icns

echo "Icon created at: resources/AppIcon.icns"
