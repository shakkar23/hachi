#!/usr/bin/env python3
"""
Visualize a hachi game-tree dump.

Usage:
    python visualize_tree.py dump.json

Controls:
    Drag          pan
    Scroll        zoom
    R             reset view
"""
import json
import re
import sys
import tkinter as tk
from collections import defaultdict

BOARD_W = 10
BOARD_H = 20

# ---- sizing ----
CELL = 5                       # base pixels per board cell
BOARD_PX_W = BOARD_W * CELL    # 50
BOARD_PX_H = BOARD_H * CELL    # 100
GAP = 8
NODE_W = BOARD_PX_W * 2 + GAP + 20
NODE_H = BOARD_PX_H + 38
H_SPACING = 32
V_SPACING = 60

# ---- colors ----
BG       = "#f4f4f4"
NODE_BG  = "#ffffff"
EDGE     = "#999999"
FRAME    = "#777777"
EMPTY    = "#e8e8e8"
ROW_FILL = "#3a8dde"
COL_FILL = "#e04a4a"


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
# Board -> PhotoImage at base scale (CELL px per cell)
# ----------------------------------------------------------------------
def build_board_image(cols, fill_color):
    img = tk.PhotoImage(width=BOARD_PX_W, height=BOARD_PX_H)
    rows = []
    for y in range(BOARD_H):
        bit = BOARD_H - 1 - y
        row_cells = []
        for x in range(BOARD_W):
            color = fill_color if (cols[x] >> bit) & 1 else EMPTY
            row_cells.extend([color] * CELL)
        row_str = "{" + " ".join(row_cells) + "}"
        rows.extend([row_str] * CELL)
    img.put(" ".join(rows), to=(0, 0))
    return img


# ----------------------------------------------------------------------
# Rendering
# ----------------------------------------------------------------------
class TreeView:
    def __init__(self, root, nodes):
        self.by_id, self.children, self.pos = layout(nodes)

        self.canvas = tk.Canvas(
            root, bg=BG, highlightthickness=0,
            xscrollincrement=1, yscrollincrement=1,
        )
        self.canvas.pack(fill="both", expand=True)

        xs = [p[0] for p in self.pos.values()]
        ys = [p[1] for p in self.pos.values()]
        pad = 100
        self.canvas.configure(scrollregion=(
            min(xs) - pad,
            min(ys) - pad,
            max(xs) + NODE_W + pad,
            max(ys) + NODE_H + pad,
        ))

        # Cache: (cols_tuple, color) -> base PhotoImage (shared across nodes)
        self._base_img_cache = {}
        # Currently-displayed scaled variant per key
        self._scaled_img_cache = {}
        # (item_id, key) pairs so we can swap images on zoom
        self._image_items = []
        # (item_id, base_font_size) pairs so we can rescale text on zoom
        self._text_items = []

        # Discrete zoom levels. 1.0 = base. Keep sorted ascending.
        # Fractional entries use subsample; integers use zoom.
        self._zoom_levels = [1/4, 1/3, 1/2, 1.0, 2.0, 3.0, 4.0, 6.0, 8.0]
        self._zoom_idx = self._zoom_levels.index(1.0)
        self.scale = 1.0

        self._draw()

        self.canvas.bind("<ButtonPress-1>", self._on_press)
        self.canvas.bind("<B1-Motion>", self._on_drag)
        self.canvas.bind("<MouseWheel>", self._zoom)
        self.canvas.bind("<Button-4>", lambda e: self._zoom_step(e, 1.1))
        self.canvas.bind("<Button-5>", lambda e: self._zoom_step(e, 1 / 1.1))

        root.bind("r", lambda e: self._reset_view())
        root.bind("R", lambda e: self._reset_view())

    # -- interaction ----------------------------------------------------
    def _on_press(self, e):
        self._drag = (e.x, e.y)

    def _on_drag(self, e):
        dx = e.x - self._drag[0]
        dy = e.y - self._drag[1]
        self.canvas.xview_scroll(-dx, "units")
        self.canvas.yview_scroll(-dy, "units")
        self._drag = (e.x, e.y)

    def _zoom(self, event):
        direction = 1 if event.delta > 0 else -1
        self._zoom_to(event, direction)

    def _zoom_step(self, event, factor):
        direction = 1 if factor > 1 else -1
        self._zoom_to(event, direction)

    def _zoom_to(self, event, direction):
        new_idx = max(0, min(len(self._zoom_levels) - 1, self._zoom_idx + direction))
        if new_idx == self._zoom_idx:
            return
        new_scale = self._zoom_levels[new_idx]
        factor = new_scale / self.scale
        x = self.canvas.canvasx(event.x)
        y = self.canvas.canvasy(event.y)
        self.canvas.scale("all", x, y, factor, factor)
        self.scale = new_scale
        self._zoom_idx = new_idx
        self._update_image_scale()
        self._update_text_scale()
        self._update_scrollregion()

    def _reset_view(self):
        if self.scale != 1.0:
            factor = 1.0 / self.scale
            self.canvas.scale("all", 0, 0, factor, factor)
            self.scale = 1.0
            self._zoom_idx = self._zoom_levels.index(1.0)
            self._update_image_scale()
            self._update_text_scale()
            self._update_scrollregion()
        self.canvas.xview_moveto(0)
        self.canvas.yview_moveto(0)

    def _update_scrollregion(self):
        bbox = self.canvas.bbox("all")
        if bbox is None:
            return
        pad = 100
        x1, y1, x2, y2 = bbox
        self.canvas.configure(scrollregion=(x1 - pad, y1 - pad, x2 + pad, y2 + pad))

    # -- image scaling --------------------------------------------------
    def _get_scaled(self, key):
        """Return a PhotoImage for `key` at current zoom level."""
        cached = self._scaled_img_cache.get(key)
        if cached is not None:
            return cached

        base = self._base_img_cache[key]
        s = self.scale
        if s == 1.0:
            img = base
        elif s > 1.0:
            img = base.zoom(int(round(s)), int(round(s)))
        else:
            img = base.subsample(int(round(1.0 / s)), int(round(1.0 / s)))
        self._scaled_img_cache[key] = img
        return img

    def _update_image_scale(self):
        self._scaled_img_cache.clear()
        for item_id, key in self._image_items:
            img = self._get_scaled(key)
            self.canvas.itemconfigure(item_id, image=img)

    def _update_text_scale(self):
        for item_id, base_size in self._text_items:
            new_size = max(1, int(round(base_size * self.scale)))
            # Font spec: use current font family/weight, swap size.
            current = self.canvas.itemcget(item_id, "font")
            parts = current.rsplit(" ", 2)  # "family size weight" or similar
            if len(parts) == 3 and parts[1].lstrip("-").isdigit():
                new_font = f"{parts[0]} {new_size} {parts[2]}"
            else:
                new_font = ("Helvetica", new_size, "bold")
            self.canvas.itemconfigure(item_id, font=new_font)

    # -- drawing --------------------------------------------------------
    def _get_base(self, cols, color):
        key = (cols, color)
        if key not in self._base_img_cache:
            self._base_img_cache[key] = build_board_image(cols, color)
        return key

    def _draw(self):
        for pid, kids in self.children.items():
            if pid not in self.pos:
                continue
            px, py = self.pos[pid]
            parent_cx = px + NODE_W / 2
            for k in kids:
                kx, ky = self.pos[k]
                child_cx = kx + NODE_W / 2
                self.canvas.create_line(
                    parent_cx, py + NODE_H,
                    child_cx, ky,
                    fill=EDGE, width=1,
                )

        for nid, (x, y) in self.pos.items():
            self._draw_node(nid, x, y)

    def _draw_node(self, nid, x, y):
        node = self.by_id[nid]
        self.canvas.create_rectangle(
            x, y, x + NODE_W, y + NODE_H,
            fill=NODE_BG, outline=FRAME,
        )

        row_cols = parse_cols(node["gamestate_row"])
        col_cols = parse_cols(node["gamestate_col"])

        key_row = self._get_base(row_cols, ROW_FILL)
        key_col = self._get_base(col_cols, COL_FILL)

        img_row = self._get_scaled(key_row)
        img_col = self._get_scaled(key_col)

        item_row = self.canvas.create_image(x + 10, y + 6, anchor="nw", image=img_row)
        self._image_items.append((item_row, key_row))
        self.canvas.create_rectangle(
            x + 10, y + 6, x + 10 + BOARD_PX_W, y + 6 + BOARD_PX_H,
            outline=FRAME,
        )

        x2 = x + 10 + BOARD_PX_W + GAP
        item_col = self.canvas.create_image(x2, y + 6, anchor="nw", image=img_col)
        self._image_items.append((item_col, key_col))
        self.canvas.create_rectangle(
            x2, y + 6, x2 + BOARD_PX_W, y + 6 + BOARD_PX_H,
            outline=FRAME,
        )

        val = node["value"]
        color = "#1a9e3a" if val > 0.5 else "#c9302c"
        label = f"v = {val:+.3f}"
        base_font_size = 10
        text_id = self.canvas.create_text(
            x + NODE_W / 2, y + BOARD_PX_H + 24,
            text=label, fill=color,
            font=("Helvetica", base_font_size, "bold"),
        )
        self._text_items.append((text_id, base_font_size))


def main():
    path = sys.argv[1] if len(sys.argv) > 1 else "dump.json"
    with open(path) as f:
        nodes = json.load(f)

    root = tk.Tk()
    root.title(f"hachi tree — {path}")
    root.geometry("1400x900")
    TreeView(root, nodes)
    root.mainloop()


if __name__ == "__main__":
    main()