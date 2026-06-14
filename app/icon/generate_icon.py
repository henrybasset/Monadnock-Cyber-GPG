#!/usr/bin/env python3
"""Generate icon.png (1024x1024) — a padlock on an indigo gradient. No deps."""
import os, zlib, struct, math

W = H = 1024
buf = bytearray(W * H * 4)


def blend(x, y, r, g, b, a):
    if a <= 0 or x < 0 or y < 0 or x >= W or y >= H:
        return
    i = (y * W + x) * 4
    ba = buf[i + 3] / 255.0
    oa = a + ba * (1 - a)
    if oa <= 0:
        return
    for k, c in enumerate((r, g, b)):
        buf[i + k] = int((c * a + buf[i + k] * ba * (1 - a)) / oa)
    buf[i + 3] = int(oa * 255 + 0.5)


def rbox_sdf(px, py, cx, cy, hx, hy, rad):
    qx = abs(px - cx) - (hx - rad)
    qy = abs(py - cy) - (hy - rad)
    return math.hypot(max(qx, 0), max(qy, 0)) + min(max(qx, qy), 0) - rad


def fill_rbox(cx, cy, hx, hy, rad, color_fn):
    for y in range(max(0, int(cy - hy - 2)), min(H, int(cy + hy + 2))):
        for x in range(max(0, int(cx - hx - 2)), min(W, int(cx + hx + 2))):
            d = rbox_sdf(x + 0.5, y + 0.5, cx, cy, hx, hy, rad)
            cov = min(max(0.5 - d, 0.0), 1.0)
            if cov > 0:
                r, g, b, a = color_fn(x, y)
                blend(x, y, r, g, b, a * cov)


def ring(cx, cy, r_out, r_in, color, y_max):
    r, g, b = color
    for y in range(max(0, int(cy - r_out - 2)), min(H, int(cy + r_out + 2))):
        if y > y_max:
            continue
        for x in range(max(0, int(cx - r_out - 2)), min(W, int(cx + r_out + 2))):
            d = math.hypot(x + 0.5 - cx, y + 0.5 - cy)
            cov = min(max(0.5 - (d - r_out), 0.0), 1.0) * min(max(0.5 - (r_in - d), 0.0), 1.0)
            if cov > 0:
                blend(x, y, r, g, b, cov)


def fill_circle(cx, cy, rad, color):
    r, g, b = color
    for y in range(max(0, int(cy - rad - 2)), min(H, int(cy + rad + 2))):
        for x in range(max(0, int(cx - rad - 2)), min(W, int(cx + rad + 2))):
            cov = min(max(0.5 - (math.hypot(x + 0.5 - cx, y + 0.5 - cy) - rad), 0.0), 1.0)
            if cov > 0:
                blend(x, y, r, g, b, cov)


def lerp(a, b, t):
    return tuple(int(a[k] + (b[k] - a[k]) * t) for k in range(3))


def main():
    top, bot = (0x6D, 0x5E, 0xF6), (0x39, 0x2C, 0xB0)
    ry0, ry1 = 96, 928

    def grad(x, y):
        t = min(max((y - ry0) / (ry1 - ry0), 0.0), 1.0)
        r, g, b = lerp(top, bot, t)
        return (r, g, b, 1.0)

    fill_rbox(512, 512, 416, 416, 200, grad)            # background tile

    white = lambda x, y: (255, 255, 255, 1.0)
    ring(512, 452, 156, 100, (255, 255, 255), 470)      # shackle (top arch)
    fill_rbox(512, 600, 216, 188, 44, white)            # padlock body

    keyhole = (0x39, 0x2C, 0xB0)
    fill_circle(512, 568, 40, keyhole)                  # keyhole circle
    for y in range(568, 694):                           # keyhole stem (trapezoid)
        t = (y - 568) / 126.0
        hw = 14 + t * 26
        for x in range(int(512 - hw), int(512 + hw)):
            blend(x, y, *keyhole, 1.0)

    out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "icon.png")
    write_png(out)
    print("wrote", out)


def write_png(path):
    raw = bytearray()
    for y in range(H):
        raw.append(0)
        raw += buf[y * W * 4:(y + 1) * W * 4]
    comp = zlib.compress(bytes(raw), 9)

    def chunk(typ, data):
        return struct.pack(">I", len(data)) + typ + data + struct.pack(">I", zlib.crc32(typ + data) & 0xffffffff)

    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n"
                + chunk(b"IHDR", struct.pack(">IIBBBBB", W, H, 8, 6, 0, 0, 0))
                + chunk(b"IDAT", comp) + chunk(b"IEND", b""))


if __name__ == "__main__":
    main()
