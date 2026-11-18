#!/usr/bin/env python3
"""Makes the app's icon from the game's own files: SpongeBob's round badge from the menus
(a 128x128 paletted texture in files/MN/mnui.HIP), on a plain background, as a PNG.
It is game data, so it is not in the repository; tools/ios.sh calls this with the disc.

usage: icon.py <extracted disc> <out.png>
"""
import struct
import sys
import zlib

SIZE = 128
# The badge is the third 128x128 CI8 texture with an RGB5A3 palette in the file, in the
# American and the European versions alike.
WHICH = 2
BACKGROUND = (38, 132, 196)


def rgb5a3(v):
    if v & 0x8000:
        return ((v >> 10 & 31) * 255 // 31, (v >> 5 & 31) * 255 // 31, (v & 31) * 255 // 31, 255)
    return ((v >> 8 & 15) * 17, (v >> 4 & 15) * 17, (v & 15) * 17, (v >> 12 & 7) * 255 // 7)


def badge(data):
    found = []
    for at in range(0, len(data) - 200, 4):
        head = at + 24  # past the texture and struct chunk headers
        if (data[at:at + 4] == b"\x15\0\0\0" and data[head:head + 4] == b"\0\0\0\x06"
                and data[head + 92:head + 100] == bytes([0, 0x80, 0, 0x80, 8, 1, 9, 2])):
            found.append(head)
    if len(found) <= WHICH:
        return None
    head = found[WHICH]
    palette = [rgb5a3(v) for v in struct.unpack(">256H", data[head + 104:head + 104 + 512])]
    at = head + 104 + 512 + 4
    pixels = [None] * (SIZE * SIZE)
    for tile_y in range(0, SIZE, 4):  # 8x4 texel tiles
        for tile_x in range(0, SIZE, 8):
            for y in range(4):
                for x in range(8):
                    pixels[(tile_y + y) * SIZE + tile_x + x] = palette[data[at]]
                    at += 1
    return pixels


def main():
    disc, out = sys.argv[1], sys.argv[2]
    with open(disc + "/files/MN/mnui.HIP", "rb") as f:
        pixels = badge(f.read())
    if pixels is None:
        sys.exit("the badge texture is not where it should be")
    # An iOS icon has no transparency: the badge goes on a plain square, a little inside it.
    scale, canvas = 4, 640
    offset = (canvas - SIZE * scale) // 2
    rows = []
    for y in range(canvas):
        row = bytearray([0])
        for x in range(canvas):
            sx, sy = (x - offset) // scale, (y - offset) // scale
            r, g, b, a = pixels[sy * SIZE + sx] if 0 <= sx < SIZE and 0 <= sy < SIZE else (0, 0, 0, 0)
            row += bytes((c * a + bg * (255 - a)) // 255 for c, bg in zip((r, g, b), BACKGROUND))
        rows.append(bytes(row))

    def chunk(kind, body):
        return struct.pack(">I", len(body)) + kind + body + struct.pack(">I", zlib.crc32(kind + body))

    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", canvas, canvas, 8, 2, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(b"".join(rows), 9)) + chunk(b"IEND", b"")
    with open(out, "wb") as f:
        f.write(png)


if __name__ == "__main__":
    main()
