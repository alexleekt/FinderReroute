#!/bin/bash
set -e

APP_NAME="FinderReroute"
APP_BUNDLE="${APP_NAME}.app"
INSTALL_DIR="/Applications"
BUNDLE_ID="com.alexleekt.finder-reroute"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo -e "${RED}Error: This installer is for macOS only.${NC}"
    exit 1
fi

# Check if app bundle exists in current directory
if [ ! -d "${APP_BUNDLE}" ]; then
    echo -e "${RED}Error: ${APP_BUNDLE} not found in current directory.${NC}"
    echo "Please download the release and run this script from the same directory."
    exit 1
fi

echo "=== Installing ${APP_NAME} ==="

# Copy to /Applications
if [ -d "${INSTALL_DIR}/${APP_BUNDLE}" ]; then
    echo -e "${YELLOW}Existing installation found. Removing...${NC}"
    rm -rf "${INSTALL_DIR}/${APP_BUNDLE}"
fi

echo "Copying to ${INSTALL_DIR}..."
cp -R "${APP_BUNDLE}" "${INSTALL_DIR}/"

# Remove quarantine attribute (added by macOS when downloaded from internet)
if xattr -p com.apple.quarantine "${INSTALL_DIR}/${APP_BUNDLE}" &>/dev/null; then
    echo "Removing quarantine attribute..."
    xattr -d com.apple.quarantine "${INSTALL_DIR}/${APP_BUNDLE}"
fi

# Verify code signature
echo "Verifying signature..."
if codesign -v "${INSTALL_DIR}/${APP_BUNDLE}" 2>/dev/null; then
    echo -e "${GREEN}Signature valid.${NC}"
else
    echo -e "${YELLOW}Warning: Signature check failed. This is expected for ad-hoc signed apps.${NC}"
fi

echo ""
echo -e "${GREEN}=== Installation complete ===${NC}"
echo ""
echo "First launch:"
echo "  1. Open System Settings → Privacy & Security → Accessibility"
echo "  2. Add ${APP_NAME}.app and enable it"
echo "  3. Also add it under Input Monitoring"
echo "  4. Launch: ${INSTALL_DIR}/${APP_BUNDLE}/Contents/MacOS/${APP_NAME}"
echo ""
echo "Or right-click the app in /Applications and select 'Open' to bypass Gatekeeper."
echo ""
echo "Install auto-start LaunchAgent:"
echo "  ${INSTALL_DIR}/${APP_BUNDLE}/Contents/MacOS/finder-reroute --install"
