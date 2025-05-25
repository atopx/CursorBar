#!/bin/bash

# CursorBar Packaging Script
# This script creates platform-specific packages from pre-built binaries

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VERSION="${1:-v0.1.0}"
TARGET="${2:-x86_64-unknown-linux-gnu}"

echo "Packaging CursorBar $VERSION for $TARGET"

cd "$PROJECT_ROOT"

# Verify binary exists
case "$TARGET" in
    *pc-windows*)
        BINARY_PATH="target/$TARGET/release/cursor_bar.exe"
        ;;
    *)
        BINARY_PATH="target/$TARGET/release/cursor_bar"
        ;;
esac

if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary not found at $BINARY_PATH"
    echo "Please build the project first with: cargo build --release --target $TARGET"
    exit 1
fi

case "$TARGET" in
    *apple-darwin*)
        echo "Creating macOS App Bundle..."
        
        # Create app bundle structure
        APP_NAME="CursorBar.app"
        rm -rf "$APP_NAME"
        mkdir -p "$APP_NAME/Contents/MacOS"
        mkdir -p "$APP_NAME/Contents/Resources"
        
        # Copy binary
        cp "$BINARY_PATH" "$APP_NAME/Contents/MacOS/"
        
        # Create Info.plist
        cat > "$APP_NAME/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>cursor_bar</string>
    <key>CFBundleIdentifier</key>
    <string>com.atopx.cursorbar</string>
    <key>CFBundleName</key>
    <string>CursorBar</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSUIElement</key>
    <true/>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSRequiresAquaSystemAppearance</key>
    <false/>
    <key>NSAppTransportSecurity</key>
    <dict>
        <key>NSAllowsArbitraryLoads</key>
        <true/>
    </dict>
</dict>
</plist>
EOF
        
        # Create DMG
        echo "Creating DMG..."
        rm -rf dmg-temp
        mkdir dmg-temp
        cp -R "$APP_NAME" dmg-temp/
        ln -s /Applications dmg-temp/Applications
        
        # Create a nice background and layout for DMG (optional)
        mkdir -p dmg-temp/.background
        
        DMG_NAME="cursor_bar-macos-$(echo $TARGET | cut -d'-' -f1).dmg"
        hdiutil create -volname "CursorBar $VERSION" -srcfolder dmg-temp -ov -format UDZO "$DMG_NAME"
        
        echo "Created: $DMG_NAME"
        ;;
        
    *pc-windows*)
        echo "Creating Windows installer..."
        
        # Create WiX installer (requires WiX Toolset)
        if command -v candle.exe >/dev/null 2>&1; then
            VERSION_NUM=$(echo "$VERSION" | sed 's/v//')
            
            cat > installer.wxs << EOF
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*" Name="CursorBar" Language="1033" Version="$VERSION_NUM.0" Manufacturer="atopx" UpgradeCode="12345678-1234-1234-1234-123456789012">
    <Package InstallerVersion="200" Compressed="yes" InstallScope="perMachine" />
    
    <MajorUpgrade DowngradeErrorMessage="A newer version of [ProductName] is already installed." />
    <MediaTemplate EmbedCab="yes" />
    
    <Feature Id="ProductFeature" Title="CursorBar" Level="1">
      <ComponentGroupRef Id="ProductComponents" />
    </Feature>
    
    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="ProgramFilesFolder">
        <Directory Id="INSTALLFOLDER" Name="CursorBar" />
      </Directory>
      <Directory Id="ProgramMenuFolder">
        <Directory Id="ApplicationProgramsFolder" Name="CursorBar"/>
      </Directory>
      <Directory Id="StartupFolder" />
    </Directory>
    
    <ComponentGroup Id="ProductComponents" Directory="INSTALLFOLDER">
      <Component Id="MainExecutable" Guid="*">
        <File Id="CursorBarExe" Source="$BINARY_PATH" KeyPath="yes">
          <Shortcut Id="ApplicationStartMenuShortcut" Directory="ApplicationProgramsFolder" Name="CursorBar" WorkingDirectory="INSTALLFOLDER" Icon="CursorBar.exe" IconIndex="0" Advertise="yes" />
          <Shortcut Id="ApplicationStartupShortcut" Directory="StartupFolder" Name="CursorBar" WorkingDirectory="INSTALLFOLDER" Arguments="" />
        </File>
      </Component>
    </ComponentGroup>
    
    <Icon Id="CursorBar.exe" SourceFile="$BINARY_PATH" />
    <Property Id="ARPPRODUCTICON" Value="CursorBar.exe" />
    <Property Id="ARPURLINFOABOUT" Value="https://github.com/atopx/CursorBar" />
  </Product>
</Wix>
EOF
            
            candle.exe installer.wxs
            light.exe installer.wixobj -out "cursor_bar-windows-x86_64.msi"
            echo "Created: cursor_bar-windows-x86_64.msi"
        else
            echo "WiX Toolset not found. Creating simple zip package..."
            mkdir -p "cursor_bar-windows-$VERSION"
            cp "$BINARY_PATH" "cursor_bar-windows-$VERSION/"
            
            # Create install script
            cat > "cursor_bar-windows-$VERSION/install.bat" << 'EOF'
@echo off
echo Installing CursorBar...
if not exist "%ProgramFiles%\CursorBar" mkdir "%ProgramFiles%\CursorBar"
copy cursor_bar.exe "%ProgramFiles%\CursorBar\"
echo.
echo Installation complete!
echo You can now run CursorBar from: %ProgramFiles%\CursorBar\cursor_bar.exe
echo.
pause
EOF
            
            # Create uninstall script
            cat > "cursor_bar-windows-$VERSION/uninstall.bat" << 'EOF'
@echo off
echo Uninstalling CursorBar...
taskkill /f /im cursor_bar.exe 2>nul
rmdir /s /q "%ProgramFiles%\CursorBar"
echo Uninstallation complete!
pause
EOF
            
            zip -r "cursor_bar-windows-x86_64.zip" "cursor_bar-windows-$VERSION"
            echo "Created: cursor_bar-windows-x86_64.zip"
        fi
        ;;
        
    *linux*)
        echo "Creating DEB package..."
        
        PACKAGE_NAME="cursor-bar_${VERSION}_amd64"
        rm -rf "$PACKAGE_NAME"
        
        # Create package structure
        mkdir -p "$PACKAGE_NAME/DEBIAN"
        mkdir -p "$PACKAGE_NAME/usr/bin"
        mkdir -p "$PACKAGE_NAME/usr/share/applications"
        mkdir -p "$PACKAGE_NAME/usr/share/doc/cursor-bar"
        mkdir -p "$PACKAGE_NAME/usr/share/pixmaps"
        
        # Copy binary
        cp "$BINARY_PATH" "$PACKAGE_NAME/usr/bin/"
        
        # Create control file (updated for Ubuntu 22.04)
        cat > "$PACKAGE_NAME/DEBIAN/control" << EOF
Package: cursor-bar
Version: $VERSION
Section: utils
Priority: optional
Architecture: amd64
Depends: libgtk-3-0, libwebkit2gtk-4.0-37, libappindicator3-1, libc6
Maintainer: atopx <atopx@example.com>
Homepage: https://github.com/atopx/CursorBar
Description: Real-time Cursor AI usage monitoring
 A modern, cross-platform system tray application for monitoring
 Cursor AI usage metrics in real-time.
 .
 Features:
  - Real-time usage tracking with configurable intervals
  - System tray integration with color-coded status
  - Bilingual interface (English/Chinese)
  - Minimal resource usage (~5MB RAM)
  - UPX compressed binary for smaller size
  - Secure local token storage
 .
 System Requirements:
  - Ubuntu 22.04+ or equivalent Linux distribution
  - GTK 3.0+ with system tray support
  - Cursor AI installed and logged in
EOF
        
        # Create desktop file
        cat > "$PACKAGE_NAME/usr/share/applications/cursor-bar.desktop" << EOF
[Desktop Entry]
Name=CursorBar
Comment=Cursor AI usage monitoring
Exec=/usr/bin/cursor_bar
Icon=cursor-bar
Terminal=false
Type=Application
Categories=Utility;System;Monitor;
StartupNotify=false
NoDisplay=true
X-GNOME-Autostart-enabled=true
Keywords=cursor;ai;usage;monitor;tray;
EOF
        
        # Create copyright file
        cat > "$PACKAGE_NAME/usr/share/doc/cursor-bar/copyright" << EOF
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: cursor-bar
Source: https://github.com/atopx/CursorBar

Files: *
Copyright: 2024 atopx
License: MIT

License: MIT
 Permission is hereby granted, free of charge, to any person obtaining a
 copy of this software and associated documentation files (the "Software"),
 to deal in the Software without restriction, including without limitation
 the rights to use, copy, modify, merge, publish, distribute, sublicense,
 and/or sell copies of the Software, and to permit persons to whom the
 Software is furnished to do so, subject to the following conditions:
 .
 The above copyright notice and this permission notice shall be included
 in all copies or substantial portions of the Software.
 .
 THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
 OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
 THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 DEALINGS IN THE SOFTWARE.
EOF
        
        # Create changelog
        cat > "$PACKAGE_NAME/usr/share/doc/cursor-bar/changelog.Debian" << EOF
cursor-bar ($VERSION) unstable; urgency=medium

  * Initial release
  * Real-time Cursor AI usage monitoring
  * Cross-platform system tray support
  * Bilingual interface (English/Chinese)
  * UPX compressed binary
  * Optimized for Ubuntu 22.04+

 -- atopx <atopx@example.com>  $(date -R)
EOF
        
        # Compress changelog
        gzip -9 "$PACKAGE_NAME/usr/share/doc/cursor-bar/changelog.Debian"
        
        # Create postinst script
        cat > "$PACKAGE_NAME/DEBIAN/postinst" << 'EOF'
#!/bin/bash
set -e

case "$1" in
    configure)
        # Update desktop database
        if command -v update-desktop-database >/dev/null 2>&1; then
            update-desktop-database -q /usr/share/applications
        fi

        # Update icon cache
        if command -v gtk-update-icon-cache >/dev/null 2>&1; then
            gtk-update-icon-cache -q -t -f /usr/share/pixmaps 2>/dev/null || true
        fi

        echo "CursorBar installed successfully!"
        echo "You can start it by running 'cursor_bar' in terminal."
        echo "It will appear in your system tray."
        ;;
esac

exit 0
EOF
        
        # Create prerm script
        cat > "$PACKAGE_NAME/DEBIAN/prerm" << 'EOF'
#!/bin/bash
set -e

case "$1" in
    remove|upgrade|deconfigure)
        # Kill any running instances
        pkill -f cursor_bar || true
        ;;
esac

exit 0
EOF
        
        # Create postrm script
        cat > "$PACKAGE_NAME/DEBIAN/postrm" << 'EOF'
#!/bin/bash
set -e

case "$1" in
    remove)
        # Update desktop database
        if command -v update-desktop-database >/dev/null 2>&1; then
            update-desktop-database -q /usr/share/applications
        fi
        ;;
esac

exit 0
EOF
        
        # Set permissions
        chmod 755 "$PACKAGE_NAME/usr/bin/cursor_bar"
        chmod 644 "$PACKAGE_NAME/usr/share/applications/cursor-bar.desktop"
        chmod 755 "$PACKAGE_NAME/DEBIAN/postinst"
        chmod 755 "$PACKAGE_NAME/DEBIAN/prerm"
        chmod 755 "$PACKAGE_NAME/DEBIAN/postrm"
        
        # Build DEB package
        if command -v dpkg-deb >/dev/null 2>&1; then
            fakeroot dpkg-deb --build "$PACKAGE_NAME"
            echo "Created: $PACKAGE_NAME.deb"
            
            # Verify package
            echo "Package info:"
            dpkg-deb --info "$PACKAGE_NAME.deb"
        else
            echo "dpkg-deb not found. Creating tar.gz instead..."
            tar -czf "$PACKAGE_NAME.tar.gz" "$PACKAGE_NAME"
            echo "Created: $PACKAGE_NAME.tar.gz"
        fi
        ;;
        
    *)
        echo "Unknown target: $TARGET"
        echo "Creating generic tar.gz package..."
        mkdir -p "cursor_bar-$VERSION-$TARGET"
        cp "$BINARY_PATH" "cursor_bar-$VERSION-$TARGET/"
        
        # Create simple README
        cat > "cursor_bar-$VERSION-$TARGET/README.txt" << EOF
CursorBar $VERSION

A real-time Cursor AI usage monitoring application.

Installation:
1. Copy the cursor_bar binary to a directory in your PATH
2. Run: ./cursor_bar

For more information, visit:
https://github.com/atopx/CursorBar
EOF
        
        tar -czf "cursor_bar-$VERSION-$TARGET.tar.gz" "cursor_bar-$VERSION-$TARGET"
        echo "Created: cursor_bar-$VERSION-$TARGET.tar.gz"
        ;;
esac

echo "Packaging complete!"
