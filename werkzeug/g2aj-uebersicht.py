#!/usr/bin/env python3
"""G2-AJ: Kontaktbogen aus Bildschirmfotos (Fensterausschnitt, 4 Spalten).

  python3 -I werkzeug/g2aj-uebersicht.py <ziel.png> <1920|1024> <bild> …
"""
import sys

import gi

gi.require_version("GdkPixbuf", "2.0")
from gi.repository import GdkPixbuf  # noqa: E402

ziel, breite, *bilder = sys.argv[1:]
x, y = (530, 226) if breite == "1920" else (82, 70)
w, h, s = 860, 628, 0.5
tw, th = int(w * s), int(h * s)
spalten = 4
zeilen = (len(bilder) + spalten - 1) // spalten
bogen = GdkPixbuf.Pixbuf.new(GdkPixbuf.Colorspace.RGB, False, 8, tw * spalten, th * zeilen)
bogen.fill(0xFFFFFFFF)
for k, datei in enumerate(bilder):
    p = GdkPixbuf.Pixbuf.new_from_file(datei)
    teil = p.new_subpixbuf(x, y, min(w, p.get_width() - x), min(h, p.get_height() - y))
    klein = teil.scale_simple(tw, th, GdkPixbuf.InterpType.BILINEAR)
    if klein.get_has_alpha():
        klein = klein.composite_color_simple(tw, th, GdkPixbuf.InterpType.NEAREST, 255, 8, 0xFFFFFF, 0xFFFFFF)
    klein.copy_area(0, 0, tw, th, bogen, (k % spalten) * tw, (k // spalten) * th)
bogen.savev(ziel, "png", [], [])
