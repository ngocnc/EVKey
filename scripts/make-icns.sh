#!/usr/bin/env bash
# Builds crates/app/icons/icon.icns from the biggest frame of icon.ico, and the macOS tray template
# PNGs (tray-v-template.png, tray-e-template.png + @2x) from the coloured tray-v/e.png.
# Needs macOS (sips, iconutil) and python3 (Command Line Tools ships one). No ImageMagick/PIL required.
# Note: icon.ico tops out at 256px, so the 512/1024 iconset slots are upscaled by sips.
set -euo pipefail

icons="$(cd "$(dirname "$0")/../crates/app/icons" && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# 1. Largest ico frame -> png (frames in icon.ico are PNG-encoded).
python3 - "$icons/icon.ico" "$work/big.png" <<'PY'
import struct, sys
data = open(sys.argv[1], "rb").read()
count = struct.unpack_from("<H", data, 4)[0]
best = None
for i in range(count):
    w, _h, _c, _r, _p, _b, size, off = struct.unpack_from("<BBBBHHII", data, 6 + 16 * i)
    w = w or 256
    if best is None or w > best[0]:
        best = (w, size, off)
_w, size, off = best
frame = data[off:off + size]
assert frame[:8] == b"\x89PNG\r\n\x1a\n", "largest ico frame is not PNG"
open(sys.argv[2], "wb").write(frame)
PY

# 2. iconset -> icns.
set_dir="$work/icon.iconset"
mkdir "$set_dir"
for s in 16 32 128 256 512; do
  sips -z "$s" "$s" "$work/big.png" --out "$set_dir/icon_${s}x${s}.png" >/dev/null
  sips -z "$((s * 2))" "$((s * 2))" "$work/big.png" --out "$set_dir/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$set_dir" -o "$icons/icon.icns"
echo "wrote $icons/icon.icns"

# 3. Tray templates: keep the rounded-rect silhouette in black and punch the bright glyph
# (V: yellow star, E: white letter) out of it, so macOS can tint it for light/dark menu bars.
python3 - "$icons" "$work" <<'PY'
import struct, sys, zlib

def read_png(path):
    d = open(path, "rb").read(); p = 8; idat = b""
    while p < len(d):
        n, t = struct.unpack(">I4s", d[p:p + 8]); c = d[p + 8:p + 8 + n]
        if t == b"IHDR":
            w, h, bd, ct = struct.unpack(">IIBB", c[:10]); assert (bd, ct) == (8, 6)
        elif t == b"IDAT":
            idat += c
        p += 12 + n
    raw = zlib.decompress(idat); s = w * 4; rows = []; prev = bytearray(s)
    for y in range(h):
        f = raw[y * (s + 1)]; r = bytearray(raw[y * (s + 1) + 1:(y + 1) * (s + 1)])
        for i in range(s):
            a = r[i - 4] if i >= 4 else 0; b = prev[i]; c = prev[i - 4] if i >= 4 else 0
            if f == 1: r[i] = (r[i] + a) & 255
            elif f == 2: r[i] = (r[i] + b) & 255
            elif f == 3: r[i] = (r[i] + (a + b) // 2) & 255
            elif f == 4:
                pa, pb, pc = abs(b - c), abs(a - c), abs(a + b - 2 * c)
                r[i] = (r[i] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        rows.append(r); prev = r
    return w, h, rows

def write_png(path, w, h, rows):
    def chunk(t, c):
        x = struct.pack(">I", len(c)) + t + c
        return x + struct.pack(">I", zlib.crc32(t + c))
    raw = b"".join(b"\0" + bytes(r) for r in rows)
    open(path, "wb").write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
                           + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b""))

icons, work = sys.argv[1:3]
for name in ("v", "e"):
    w, h, rows = read_png(f"{icons}/tray-{name}.png")
    out = []
    for r in rows:
        o = bytearray(w * 4)
        for x in range(w):
            R, G, B, A = r[x * 4:x * 4 + 4]
            lum = 0.299 * R + 0.587 * G + 0.114 * B
            glyph = min(1.0, max(0.0, (lum - 120) / 60))
            o[x * 4 + 3] = round(A * (1 - glyph))
        out.append(o)
    write_png(f"{work}/tray-{name}-template.png", w, h, out)
PY
for n in v e; do
  # Menu bar icons are 22pt: @1x = 22px, @2x = 44px (upscaled from the 32px source).
  sips -z 22 22 "$work/tray-$n-template.png" --out "$icons/tray-$n-template.png" >/dev/null
  sips -z 44 44 "$work/tray-$n-template.png" --out "$icons/tray-$n-template@2x.png" >/dev/null
done
echo "wrote tray templates in $icons"
