"""Generate Rime Studio icons at all required sizes."""
from PIL import Image, ImageDraw, ImageFont
import os
import math

OUT_DIR = "src-tauri/icons"
SIZES = {
    "32x32.png": 32,
    "128x128.png": 128,
    "128x128@2x.png": 256,
    "icon.png": 512,
    "Square30x30Logo.png": 30,
    "Square44x44Logo.png": 44,
    "Square71x71Logo.png": 71,
    "Square89x89Logo.png": 89,
    "Square107x107Logo.png": 107,
    "Square142x142Logo.png": 142,
    "Square150x150Logo.png": 150,
    "Square284x284Logo.png": 284,
    "Square310x310Logo.png": 310,
    "StoreLogo.png": 50,
}

# Blue gradient colors matching the app brand
GRADIENT_TOP = (0x60, 0xA5, 0xFA)    # #60a5fa
GRADIENT_BOTTOM = (0x25, 0x63, 0xEB)  # #2563eb
TEXT_COLOR = (0xFF, 0xFF, 0xFF, 255)
SHADOW_COLOR = (0x1D, 0x4E, 0xD8, 80)  # semi-transparent dark blue


def draw_icon(size: int) -> Image.Image:
    """Draw the Rime Studio icon at given size."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Rounded rect background with gradient
    corner = max(8, int(size * 0.19))
    margin = max(2, int(size * 0.04))

    # Draw rounded rectangle with gradient (vertical)
    for y in range(margin, size - margin):
        t = (y - margin) / (size - 2 * margin)
        r = int(GRADIENT_TOP[0] + (GRADIENT_BOTTOM[0] - GRADIENT_TOP[0]) * t)
        g = int(GRADIENT_TOP[1] + (GRADIENT_BOTTOM[1] - GRADIENT_TOP[1]) * t)
        b = int(GRADIENT_TOP[2] + (GRADIENT_BOTTOM[2] - GRADIENT_TOP[2]) * t)

        x_start = margin + corner
        x_end = size - margin - corner

        # Draw the line with rounded ends (approximate)
        draw.line([(x_start, y), (x_end, y)], fill=(r, g, b, 255), width=1)

    # Draw rounded corners properly using pieslice
    # Top-left corner
    draw.pieslice(
        [margin, margin, margin + corner * 2, margin + corner * 2],
        180, 270, fill=GRADIENT_TOP + (255,)
    )
    # Top-right corner
    draw.pieslice(
        [size - margin - corner * 2, margin, size - margin, margin + corner * 2],
        270, 360, fill=GRADIENT_TOP + (255,)
    )
    # Bottom-left corner
    draw.pieslice(
        [margin, size - margin - corner * 2, margin + corner * 2, size - margin],
        90, 180, fill=GRADIENT_BOTTOM + (255,)
    )
    # Bottom-right corner
    draw.pieslice(
        [size - margin - corner * 2, size - margin - corner * 2, size - margin, size - margin],
        0, 90, fill=GRADIENT_BOTTOM + (255,)
    )

    # Fill corner gaps
    # Top edge flat part
    draw.rectangle([margin + corner, margin, size - margin - corner, margin + corner], fill=GRADIENT_TOP + (255,))
    # Bottom edge flat part
    draw.rectangle(
        [margin + corner, size - margin - corner, size - margin - corner, size - margin],
        fill=GRADIENT_BOTTOM + (255,)
    )
    # Left edge
    draw.rectangle([margin, margin + corner, margin + corner, size - margin - corner], fill=(r, g, b, 255))
    # Right edge
    draw.rectangle(
        [size - margin - corner, margin + corner, size - margin, size - margin - corner],
        fill=(r, g, b, 255)
    )

    # Actually, the gradient fill approach above is complex and imperfect.
    # Let's use a simpler approach: solid rounded rectangle with gradient overlay.

    # --- Simpler approach: recreate the image ---
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    corner = max(6, int(size * 0.188))
    margin = 0

    # Draw rounded rectangle
    draw.rounded_rectangle(
        [margin, margin, size - margin, size - margin],
        radius=corner,
        fill=(0, 0, 0, 0),
        width=0,
    )

    # Draw gradient fill as a series of horizontal lines within the rounded rect
    for y in range(size):
        t = y / size
        r = int(GRADIENT_TOP[0] + (GRADIENT_BOTTOM[0] - GRADIENT_TOP[0]) * t)
        g = int(GRADIENT_TOP[1] + (GRADIENT_BOTTOM[1] - GRADIENT_TOP[1]) * t)
        b = int(GRADIENT_TOP[2] + (GRADIENT_BOTTOM[2] - GRADIENT_TOP[2]) * t)

        # Calculate how much to inset this line based on the corner radius
        # For y within the top corner region: inset based on circle equation
        if y < corner:
            dy = corner - y
            dx = int(math.sqrt(corner * corner - dy * dy))
            inset = corner - dx
        elif y > size - corner:
            dy = y - (size - corner)
            dx = int(math.sqrt(corner * corner - dy * dy))
            inset = corner - dx
        else:
            inset = 0

        x0 = inset
        x1 = size - inset
        if x1 > x0:
            draw.line([(x0, y), (x1, y)], fill=(r, g, b, 255), width=1)

    # Draw subtle inner highlight at top
    for y in range(int(size * 0.05)):
        alpha = int(80 * (1 - y / (size * 0.05)))
        if alpha > 0:
            # Recalculate inset for highlight
            if y < corner:
                dy = corner - y
                dx = int(math.sqrt(max(0, corner * corner - dy * dy)))
                inset = corner - dx
            else:
                inset = 0
            x0 = inset + 2
            x1 = size - inset - 2
            if x1 > x0:
                draw.line([(x0, y), (x1, y)], fill=(255, 255, 255, alpha), width=1)

    # Draw the "R" letter
    # Use a bold geometric R built from rectangles
    # The R is sized to ~52% of the icon
    letter_w = int(size * 0.52)
    letter_h = int(size * 0.58)
    letter_x = (size - letter_w) // 2
    letter_y = (size - letter_h) // 2
    stroke_w = max(3, int(size * 0.09))

    # Vertical stem
    stem_x0 = letter_x
    stem_x1 = letter_x + stroke_w
    stem_top = letter_y
    stem_bot = letter_y + letter_h
    draw.rectangle([stem_x0, stem_top, stem_x1, stem_bot], fill=TEXT_COLOR)

    # Top horizontal bar
    bar_right = letter_x + letter_w
    bar_top = letter_y
    bar_bot = letter_y + stroke_w
    draw.rectangle([stem_x0, bar_top, bar_right, bar_bot], fill=TEXT_COLOR)

    # Right vertical (top-right of R)
    right_x0 = bar_right - stroke_w
    right_x1 = bar_right
    right_top = letter_y
    right_mid = letter_y + letter_h // 2
    draw.rectangle([right_x0, right_top, right_x1, right_mid], fill=TEXT_COLOR)

    # Middle horizontal bar
    mid_y = letter_y + letter_h // 2 - stroke_w // 2
    draw.rectangle([stem_x0, mid_y, right_x1, mid_y + stroke_w], fill=TEXT_COLOR)

    # Diagonal leg
    leg_start_x = stem_x1
    leg_start_y = letter_y + letter_h // 2 + stroke_w // 2
    leg_end_x = letter_x + letter_w
    leg_end_y = letter_y + letter_h

    # Draw diagonal as filled polygon
    leg_points = [
        (leg_start_x, leg_start_y),
        (leg_start_x + stroke_w, leg_start_y),
        (leg_end_x, leg_end_y),
        (leg_end_x - stroke_w, leg_end_y),
    ]
    draw.polygon(leg_points, fill=TEXT_COLOR)

    return img


def create_ico(png_path: str, ico_path: str):
    """Convert a 256x256 PNG to ICO format."""
    img = Image.open(png_path)
    # Save as ICO with multiple sizes
    sizes = [(256, 256), (128, 128), (64, 64), (48, 48), (32, 32), (16, 16)]
    ico_images = []
    for s in sizes:
        if s[0] <= img.width:
            ico_images.append(img.resize(s, Image.LANCZOS))
    ico_images[0].save(
        ico_path,
        format="ICO",
        sizes=[(im.width, im.height) for im in ico_images],
        append_images=ico_images[1:],
    )
    print(f"  Created: {ico_path}")


def main():
    os.makedirs(OUT_DIR, exist_ok=True)

    # Generate PNGs at all sizes
    for filename, size in SIZES.items():
        path = os.path.join(OUT_DIR, filename)
        img = draw_icon(size)
        img.save(path, "PNG")
        print(f"  {filename} ({size}x{size})")

    # Generate ICO from the 256x256 source
    src_png = os.path.join(OUT_DIR, "128x128@2x.png")
    ico_path = os.path.join(OUT_DIR, "icon.ico")
    create_ico(src_png, ico_path)

    print(f"\nDone! {len(SIZES)} PNGs + 1 ICO generated in {OUT_DIR}/")


if __name__ == "__main__":
    main()
