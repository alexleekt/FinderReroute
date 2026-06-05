#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_NAME="FinderReroute"
APP_BUNDLE="${SCRIPT_DIR}/${APP_NAME}.app"
MACOS_DIR="${APP_BUNDLE}/Contents/MacOS"
RESOURCES_DIR="${APP_BUNDLE}/Contents/Resources"

echo "=== Building ${APP_NAME} ==="

# Build Rust release binary
echo "Building Rust binary..."
cd "${SCRIPT_DIR}"
cargo build --release

# Build SwiftUI binary
echo "Building SwiftUI binary..."
cd "${SCRIPT_DIR}/ui"
swift build -c release

# Create app bundle structure
echo "Creating app bundle..."
mkdir -p "${MACOS_DIR}"
mkdir -p "${RESOURCES_DIR}"

# Copy binaries
echo "Copying binaries..."
cp "${SCRIPT_DIR}/target/release/finder-reroute" "${MACOS_DIR}/"
cp "${SCRIPT_DIR}/ui/.build/release/${APP_NAME}UI" "${MACOS_DIR}/${APP_NAME}"
chmod +x "${MACOS_DIR}/finder-reroute"
chmod +x "${MACOS_DIR}/${APP_NAME}"

# Create app icon
echo "Creating app icon..."
if command -v python3 &> /dev/null && python3 -c "from PIL import Image" 2>/dev/null; then
    python3 "${SCRIPT_DIR}/create_icon.py"
else
    echo "Warning: Python3 with PIL not available. Skipping icon creation."
fi

# Create Info.plist if not exists
if [ ! -f "${APP_BUNDLE}/Contents/Info.plist" ]; then
    echo "Creating Info.plist..."
    cat > "${APP_BUNDLE}/Contents/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>FinderReroute</string>
    <key>CFBundleIdentifier</key>
    <string>com.alexleekt.finder-reroute</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>FinderReroute</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>LSMinimumSystemVersion</key>
    <string>13.0</string>
    <key>LSUIElement</key>
    <true/>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright © 2026. All rights reserved.</string>
</dict>
</plist>
EOF
fi

echo ""
echo "=== Build complete ==="
echo "App bundle: ${APP_BUNDLE}"
echo ""
echo "To install:"
echo "  cp -R ${APP_BUNDLE} /Applications/"
echo ""
echo "To run:"
echo "  /Applications/${APP_NAME}.app/Contents/MacOS/${APP_NAME}"
echo ""
echo "To install LaunchAgent:"
echo "  /Applications/${APP_NAME}.app/Contents/MacOS/finder-reroute --install"
