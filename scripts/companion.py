import json
import math
import sys


def render_braille_canvas(width_px: int, height_px: int, draw_fn) -> list[str]:
    """Renders a 2x4-dot Braille bitmap into terminal text lines."""
    cols = (width_px + 1) // 2
    rows = (height_px + 3) // 4
    grid = [[False for _ in range(cols * 2)] for _ in range(rows * 4)]

    draw_fn(grid, cols * 2, rows * 4)

    # Standard Unicode Braille bit offsets for (dx, dy) in 2x4 cell
    braille_map = {
        (0, 0): 0x01,
        (0, 1): 0x02,
        (0, 2): 0x04,
        (1, 0): 0x08,
        (1, 1): 0x10,
        (1, 2): 0x20,
        (0, 3): 0x40,
        (1, 3): 0x80,
    }

    lines = []
    for r in range(rows):
        chars = []
        for c in range(cols):
            mask = 0
            for (dx, dy), bit in braille_map.items():
                px = c * 2 + dx
                py = r * 4 + dy
                if 0 <= py < len(grid) and 0 <= px < len(grid[0]) and grid[py][px]:
                    mask |= bit
            chars.append(chr(0x2800 + mask))
        lines.append("".join(chars).rstrip())

    while lines and lines[-1] == "":
        lines.pop()
    return lines if lines else ["⠿"]


def draw_filled_circle(grid, cx: float, cy: float, rx: float, ry: float, val: bool = True):
    h = len(grid)
    w = len(grid[0]) if h else 0
    for y in range(max(0, int(cy - ry - 1)), min(h, int(cy + ry + 2))):
        for x in range(max(0, int(cx - rx - 1)), min(w, int(cx + rx + 2))):
            nx = (x - cx) / max(rx, 0.5)
            ny = (y - cy) / max(ry, 0.5)
            if nx * nx + ny * ny <= 1.0:
                grid[y][x] = val


def draw_line(grid, x0: int, y0: int, x1: int, y1: int, val: bool = True):
    h = len(grid)
    w = len(grid[0]) if h else 0
    steps = max(abs(x1 - x0), abs(y1 - y0), 1)
    for i in range(steps + 1):
        x = round(x0 + (x1 - x0) * i / steps)
        y = round(y0 + (y1 - y0) * i / steps)
        if 0 <= y < h and 0 <= x < w:
            grid[y][x] = val


def build_big_sprite(emotion: str, level: int, frame_idx: int) -> list[str]:
    emotion = emotion.lower()
    phase = frame_idx % 4
    bob = [0, -2, 0, 2][phase]
    scale_boost = 2.0 if level >= 20 else (1.0 if level >= 5 else 0.0)

    def draw(grid, w, h):
        cx = w / 2.0
        cy = h / 2.0 + bob * 0.6
        rx = 11.0 + scale_boost
        ry = 8.5 + scale_boost * 0.7

        # Outer aura / sparkles for higher levels or excited/proud states
        if emotion in ("excited", "proud") or level >= 15:
            ring_rx = rx + 3.5 + (phase % 2) * 1.5
            ring_ry = ry + 2.5 + (phase % 2) * 1.0
            for deg in range(0, 360, 45):
                rad = math.radians(deg + phase * 22)
                sx = round(cx + math.cos(rad) * ring_rx)
                sy = round(cy + math.sin(rad) * ring_ry)
                draw_filled_circle(grid, sx, sy, 1.1, 1.1, True)

        # Main big body silhouette
        body_ry = ry * (0.88 if emotion == "sleepy" else 1.0)
        draw_filled_circle(grid, cx, cy, rx, body_ry, True)

        # Ears / horns / side appendages
        ear_lift = -3 if phase in (1, 2) else -1
        if emotion == "angry":
            # Sharp angled brows / horns
            draw_line(grid, int(cx - rx + 1), int(cy - ry + 1), int(cx - 3), int(cy - 3), False)
            draw_line(grid, int(cx + rx - 1), int(cy - ry + 1), int(cx + 3), int(cy - 3), False)
        else:
            # Cute rounded ears/tufts
            draw_filled_circle(grid, cx - rx * 0.6, cy - body_ry + ear_lift, 2.6, 2.6, True)
            draw_filled_circle(grid, cx + rx * 0.6, cy - body_ry + ear_lift, 2.6, 2.6, True)

        # Eyes (carved out of body)
        eye_y = int(cy - 1.5)
        left_eye_x = int(cx - rx * 0.36)
        right_eye_x = int(cx + rx * 0.36)

        if emotion == "sleepy" or phase == 3:
            # Closed / resting eyes
            draw_line(grid, left_eye_x - 2, eye_y, left_eye_x + 2, eye_y, False)
            draw_line(grid, right_eye_x - 2, eye_y, right_eye_x + 2, eye_y, False)
        elif emotion == "excited":
            # Big bright eyes with inner sparkle cutout
            draw_filled_circle(grid, left_eye_x, eye_y, 2.4, 2.4, False)
            draw_filled_circle(grid, right_eye_x, eye_y, 2.4, 2.4, False)
            draw_filled_circle(grid, left_eye_x, eye_y, 0.9, 0.9, True)
            draw_filled_circle(grid, right_eye_x, eye_y, 0.9, 0.9, True)
        else:
            draw_filled_circle(grid, left_eye_x, eye_y, 1.9, 2.1, False)
            draw_filled_circle(grid, right_eye_x, eye_y, 1.9, 2.1, False)

        # Mouth expression
        mouth_y = int(cy + 3.0)
        if emotion in ("happy", "excited", "proud"):
            if phase % 2 == 0:
                draw_filled_circle(grid, cx, mouth_y, 2.6, 1.8, False)
            else:
                draw_line(grid, int(cx - 3), mouth_y - 1, int(cx), mouth_y + 2, False)
                draw_line(grid, int(cx), mouth_y + 2, int(cx + 3), mouth_y - 1, False)
        elif emotion in ("sad", "lonely"):
            draw_line(grid, int(cx - 3), mouth_y + 2, int(cx), mouth_y - 1, False)
            draw_line(grid, int(cx), mouth_y - 1, int(cx + 3), mouth_y + 2, False)
        elif emotion == "angry":
            draw_line(grid, int(cx - 3), mouth_y + 1, int(cx + 3), mouth_y - 1, False)
        else:
            draw_line(grid, int(cx - 2), mouth_y, int(cx + 2), mouth_y, False)

    return render_braille_canvas(36, 28, draw)


def main() -> None:
    emotion = sys.argv[1] if len(sys.argv) > 1 else "Happy"
    level = int(sys.argv[2]) if len(sys.argv) > 2 else 1
    frame_idx = int(sys.argv[3]) if len(sys.argv) > 3 else 0

    lines = build_big_sprite(emotion, level, frame_idx)
    print(json.dumps({"lines": lines}, ensure_ascii=False))


if __name__ == "__main__":
    main()
