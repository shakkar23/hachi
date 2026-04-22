#!/usr/bin/env python3
"""
Visualize a hachi game-tree dump.

Renders the entire tree to a PNG using Pillow, then opens a GUI that
pans/zooms that image.

Usage:
    python visualize_tree.py dump.json [--out tree.png] [--no-gui]

Controls:
    Drag          pan
    Scroll        zoom
    R             reset view
"""
import argparse
import json
import re
from collections import defaultdict

from PIL import Image, ImageDraw, ImageFont

BOARD_W = 10
BOARD_H = 20

# ---- sizing ----
CELL = 5
BOARD_PX_W = BOARD_W * CELL
BOARD_PX_H = BOARD_H * CELL
GAP = 8
NODE_W = BOARD_PX_W * 2 + GAP + 20
NODE_H = BOARD_PX_H + 38
H_SPACING = 32
V_SPACING = 60
MARGIN = 40

# ---- colors (RGB) ----
BG       = (244, 244, 244)
NODE_BG  = (255, 255, 255)
EDGE     = (153, 153, 153)
FRAME    = (119, 119, 119)
EMPTY    = (232, 232, 232)
ROW_FILL = (58, 141, 222)
COL_FILL = (224, 74, 74)
POS_COL  = (26, 158, 58)
NEG_COL  = (201, 48, 44)


COLS_RE = re.compile(r"cols:\s*\[([^\]]+)\]", re.S)


def parse_cols(debug_str):
    m = COLS_RE.search(debug_str)
    if not m:
        return (0,) * BOARD_W
    nums = re.findall(r"\d+", m.group(1))
    return tuple(int(n) for n in nums[:BOARD_W])


# ----------------------------------------------------------------------
# Layout
# ----------------------------------------------------------------------
def layout(nodes):
    by_id = {n["id"]: n for n in nodes}
    children = defaultdict(list)
    for n in nodes:
        if n["parent_id"] is not None:
            children[n["parent_id"]].append(n["id"])

    roots = [n["id"] for n in nodes if n["parent_id"] is None]

    width = {}

    def compute_width(nid):
        kids = children.get(nid, [])
        if not kids:
            width[nid] = 1
            return 1
        w = sum(compute_width(k) for k in kids)
        width[nid] = w
        return w

    for r in roots:
        compute_width(r)

    pos = {}

    def place(nid, x_left, depth):
        w = width[nid]
        cx = x_left + w / 2
        pos[nid] = (cx, depth)
        cursor = x_left
        for k in sorted(children.get(nid, [])):
            kw = width[k]
            place(k, cursor, depth + 1)
            cursor += kw

    cursor = 0
    for r in sorted(roots):
        place(r, cursor, 0)
        cursor += width[r]

    px_pos = {}
    for nid, (cx, d) in pos.items():
        x = cx * (NODE_W + H_SPACING)
        y = d * (NODE_H + V_SPACING)
        px_pos[nid] = (x, y)
    return by_id, children, px_pos


# ----------------------------------------------------------------------
# Pillow rendering
# ----------------------------------------------------------------------
def draw_board(draw, ox, oy, cols, fill_color):
    for y in range(BOARD_H):
        bit = BOARD_H - 1 - y
        for x in range(BOARD_W):
            color = fill_color if (cols[x] >> bit) & 1 else EMPTY
            x0 = ox + x * CELL
            y0 = oy + y * CELL
            draw.rectangle([x0, y0, x0 + CELL - 1, y0 + CELL - 1], fill=color)
    draw.rectangle([ox, oy, ox + BOARD_PX_W, oy + BOARD_PX_H],
                   outline=FRAME, width=1)


def get_font(size):
    for name in ("DejaVuSans-Bold.ttf", "Arial Bold.ttf", "Helvetica-Bold.ttf"):
        try:
            return ImageFont.truetype(name, size)
        except (OSError, IOError):
            continue
    try:
        return ImageFont.truetype("DejaVuSans.ttf", size)
    except (OSError, IOError):
        return ImageFont.load_default()


def render_tree(nodes):
    by_id, children, pos = layout(nodes)

    max_x = max(x for x, _ in pos.values()) + NODE_W
    max_y = max(y for _, y in pos.values()) + NODE_H
    W = int(max_x + 2 * MARGIN)
    H = int(max_y + 2 * MARGIN)

    img = Image.new("RGB", (W, H), BG)
    draw = ImageDraw.Draw(img)
    font = get_font(11)

    # edges first so nodes overlay them
    for pid, kids in children.items():
        if pid not in pos:
            continue
        px, py = pos[pid]
        parent_cx = px + NODE_W / 2 + MARGIN
        parent_by = py + NODE_H + MARGIN
        for k in kids:
            kx, ky = pos[k]
            child_cx = kx + NODE_W / 2 + MARGIN
            child_ty = ky + MARGIN
            draw.line([parent_cx, parent_by, child_cx, child_ty],
                      fill=EDGE, width=1)

    for nid, (x, y) in pos.items():
        node = by_id[nid]
        x += MARGIN
        y += MARGIN
        draw.rectangle([x, y, x + NODE_W, y + NODE_H],
                       fill=NODE_BG, outline=FRAME, width=1)

        row_cols = parse_cols(node["gamestate_row"])
        col_cols = parse_cols(node["gamestate_col"])

        draw_board(draw, x + 10, y + 6, row_cols, ROW_FILL)
        draw_board(draw, x + 10 + BOARD_PX_W + GAP, y + 6, col_cols, COL_FILL)

        val = node["value"]
        color = POS_COL if val > 0.5 else NEG_COL
        label = f"v = {val:+.3f}"
        bbox = draw.textbbox((0, 0), label, font=font)
        tw = bbox[2] - bbox[0]
        th = bbox[3] - bbox[1]
        tx = x + NODE_W / 2 - tw / 2
        ty = y + BOARD_PX_H + 14 - th / 2
        draw.text((tx, ty), label, fill=color, font=font)

    return img


# ----------------------------------------------------------------------
# Tk viewer: pan/zoom a single PIL image
# ----------------------------------------------------------------------
class ImageViewer:
    def __init__(self, root, pil_img):
        self.root = root
        self.base = pil_img
        self._zoom_levels = [1/8, 1/6, 1/4, 1/3, 1/2, 2/3, 1.0,
                             1.5, 2.0, 3.0, 4.0, 6.0, 8.0]

        # Start at a zoom that fits roughly the initial window width.
        fit = 1400 / pil_img.width if pil_img.width else 1.0
        self._zoom_idx = min(range(len(self._zoom_levels)),
                             key=lambda i: abs(self._zoom_levels[i] - fit))
        self.scale = self._zoom_levels[self._zoom_idx]

        self.canvas = tk.Canvas(root, bg="#f4f4f4", highlightthickness=0)
        self.canvas.pack(fill="both", expand=True)

        self._tk_img = None
        self._img_id = None
        self._redraw()

        self.canvas.bind("<ButtonPress-1>", self._on_press)
        self.canvas.bind("<B1-Motion>", self._on_drag)
        self.canvas.bind("<MouseWheel>", self._zoom)
        self.canvas.bind("<Button-4>", lambda e: self._zoom_step(e, +1))
        self.canvas.bind("<Button-5>", lambda e: self._zoom_step(e, -1))
        root.bind("r", lambda e: self._reset())
        root.bind("R", lambda e: self._reset())

    def _redraw(self, anchor_canvas=None, anchor_img=None):
        w = max(1, int(self.base.width * self.scale))
        h = max(1, int(self.base.height * self.scale))
        resample = Image.NEAREST if self.scale >= 1 else Image.BILINEAR
        resized = self.base.resize((w, h), resample)
        self._tk_img = ImageTk.PhotoImage(resized)

        if self._img_id is None:
            self._img_id = self.canvas.create_image(0, 0, anchor="nw",
                                                    image=self._tk_img)
        else:
            self.canvas.itemconfigure(self._img_id, image=self._tk_img)

        self.canvas.configure(scrollregion=(0, 0, w, h))

        if anchor_canvas is not None and anchor_img is not None:
            target_x = anchor_img[0] * self.scale
            target_y = anchor_img[1] * self.scale
            new_origin_x = target_x - anchor_canvas[0]
            new_origin_y = target_y - anchor_canvas[1]
            if w > 0:
                self.canvas.xview_moveto(max(0, new_origin_x) / w)
            if h > 0:
                self.canvas.yview_moveto(max(0, new_origin_y) / h)

    def _on_press(self, e):
        self._drag = (e.x, e.y)

    def _on_drag(self, e):
        dx = e.x - self._drag[0]
        dy = e.y - self._drag[1]
        self.canvas.xview_scroll(-dx, "units")
        self.canvas.yview_scroll(-dy, "units")
        self._drag = (e.x, e.y)

    def _zoom(self, event):
        self._zoom_step(event, +1 if event.delta > 0 else -1)

    def _zoom_step(self, event, direction):
        new_idx = max(0, min(len(self._zoom_levels) - 1,
                             self._zoom_idx + direction))
        if new_idx == self._zoom_idx:
            return
        cx = self.canvas.canvasx(event.x)
        cy = self.canvas.canvasy(event.y)
        ix = cx / self.scale
        iy = cy / self.scale

        self._zoom_idx = new_idx
        self.scale = self._zoom_levels[new_idx]
        self._redraw(anchor_canvas=(event.x, event.y), anchor_img=(ix, iy))

    def _reset(self):
        if 1.0 in self._zoom_levels:
            self._zoom_idx = self._zoom_levels.index(1.0)
        else:
            self._zoom_idx = len(self._zoom_levels) // 2
        self.scale = self._zoom_levels[self._zoom_idx]
        self._redraw()
        self.canvas.xview_moveto(0)
        self.canvas.yview_moveto(0)


# ----------------------------------------------------------------------
# Main
# ----------------------------------------------------------------------
def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dump", nargs="?", default="dump.json")
    ap.add_argument("--out", default=None,
                    help="PNG output path (default: <dump>.png)")
    ap.add_argument("--no-gui", action="store_true")
    args = ap.parse_args()

    with open(args.dump) as f:
        nodes = json.load(f)

    img = render_tree(nodes)
    out = args.out or args.dump.rsplit(".", 1)[0] + ".png"
    img.save(out)
    print(f"saved {out} ({img.width}x{img.height})")

    if args.no_gui:
        return

    import tkinter as tk
    from PIL import ImageTk  # noqa: F401 — used by ImageViewer

    globals()["tk"] = tk
    globals()["ImageTk"] = ImageTk

    root = tk.Tk()
    root.title(f"hachi tree — {args.dump}")
    root.geometry("1400x900")
    ImageViewer(root, img)
    root.mainloop()


if __name__ == "__main__":
    main()