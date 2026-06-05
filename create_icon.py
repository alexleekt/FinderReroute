#!/usr/bin/env python3
"""Create a folder icon for the FinderReroute app."""

import os
import shutil
import subprocess

from PIL import Image, ImageDraw  # type: ignore[import]

# Create a simple folder icon
# We'll make it a nice blue folder with a subtle design


def create_folder_icon(size):
    """Create a folder icon at the given size."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Colors
    folder_color = (59, 130, 246)  # Blue-500
    folder_dark = (37, 99, 235)  # Blue-600
    folder_light = (96, 165, 250)  # Blue-400

    # Scale factor
    s = size / 1024

    # Folder body (rounded rectangle)
    body_left = int(100 * s)
    body_top = int(300 * s)
    body_right = int(924 * s)
    body_bottom = int(850 * s)
    body_radius = int(80 * s)

    # Folder tab (top part)
    tab_left = int(100 * s)
    tab_top = int(150 * s)
    tab_right = int(500 * s)
    tab_bottom = int(400 * s)
    tab_radius = int(60 * s)

    # Draw tab
    draw.rounded_rectangle(
        [tab_left, tab_top, tab_right, tab_bottom], radius=tab_radius, fill=folder_dark
    )

    # Draw body
    draw.rounded_rectangle(
        [body_left, body_top, body_right, body_bottom],
        radius=body_radius,
        fill=folder_color,
    )

    # Draw a subtle inner highlight
    highlight_left = int(140 * s)
    highlight_top = int(340 * s)
    highlight_right = int(884 * s)
    highlight_bottom = int(810 * s)
    highlight_radius = int(60 * s)

    draw.rounded_rectangle(
        [highlight_left, highlight_top, highlight_right, highlight_bottom],
        radius=highlight_radius,
        fill=folder_light,
    )

    # Draw a small arrow or indicator
    arrow_size = int(200 * s)
    arrow_x = size // 2
    arrow_y = int(580 * s)

    # Simple arrow pointing right
    arrow_points = [
        (arrow_x - arrow_size // 3, arrow_y - arrow_size // 2),
        (arrow_x + arrow_size // 3, arrow_y),
        (arrow_x - arrow_size // 3, arrow_y + arrow_size // 2),
    ]

    draw.polygon(arrow_points, fill=(255, 255, 255, 200))

    return img


# Determine project root from script location
script_dir = os.path.dirname(os.path.abspath(__file__))
iconset_dir = os.path.join(
    script_dir, "FinderReroute.app", "Contents", "Resources", "AppIcon.iconset"
)
os.makedirs(iconset_dir, exist_ok=True)

# Standard macOS icon sizes
sizes = [
    (16, "16x16"),
    (32, "16x16@2x"),
    (32, "32x32"),
    (64, "32x32@2x"),
    (128, "128x128"),
    (256, "128x128@2x"),
    (256, "256x256"),
    (512, "256x256@2x"),
    (512, "512x512"),
    (1024, "512x512@2x"),
]

print("Creating icon sizes...")
for size, name in sizes:
    icon = create_folder_icon(size)
    icon.save(f"{iconset_dir}/icon_{name}.png")
    print(f"  Created {name}.png ({size}x{size})")

# Create ICNS file using iconutil
print("\nCreating ICNS file...")
icns_path = os.path.join(
    script_dir, "FinderReroute.app", "Contents", "Resources", "AppIcon.icns"
)
result = subprocess.run(
    ["iconutil", "-c", "icns", "-o", icns_path, iconset_dir],
    capture_output=True,
)

if result.returncode == 0:
    print(f"✅ Created {icns_path}")
    # Clean up iconset directory
    shutil.rmtree(iconset_dir, ignore_errors=True)
    print("Cleaned up iconset directory")
else:
    print("❌ Failed to create ICNS file")
    print("Falling back to PNG...")
    # Use the largest PNG as the icon
    fallback_png = os.path.join(
        script_dir, "FinderReroute.app", "Contents", "Resources", "AppIcon.png"
    )
    shutil.copy(f"{iconset_dir}/icon_512x512@2x.png", fallback_png)
    shutil.rmtree(iconset_dir, ignore_errors=True)
