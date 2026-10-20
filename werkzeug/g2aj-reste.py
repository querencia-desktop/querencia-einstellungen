#!/usr/bin/env python3
"""G2-AJ: Zeichenfehler messen. Vergleicht in einem Lauf jedes Bild
w<NN>-….png mit seiner Referenz w<NN>-…-ref.png (nach erzwungenem vollem
Neuzeichnen) im Seitenbereich des Kontrollzentrums und meldet je Bild die
abweichenden Pixelzeilen. Exit 0 = keine Reste, 1 = Reste.

  python3 -I werkzeug/g2aj-reste.py <laufordner> <1920|1024>
"""
import pathlib
import sys

import gi

gi.require_version("GdkPixbuf", "2.0")
from gi.repository import GdkPixbuf  # noqa: E402

ordner = pathlib.Path(sys.argv[1])
dx, dy = (0, 0) if sys.argv[2] == "1920" else (-448, -156)
# Seite rechts der Seitenleiste, unterhalb der Kopfzeile, bis zum Fensterrand
X0, Y0, X1, Y1 = 762 + dx, 300 + dy, 1389 + dx, 853 + dy


def zeilen(a, b):
    pa, pb = a.get_pixels(), b.get_pixels()
    ra, rb = a.get_rowstride(), b.get_rowstride()
    na, nb = a.get_n_channels(), b.get_n_channels()
    ergebnis = []
    for y in range(Y0, Y1):
        n = sum(
            1
            for x in range(X0, X1)
            if pa[y * ra + x * na : y * ra + x * na + 3] != pb[y * rb + x * nb : y * rb + x * nb + 3]
        )
        if n:
            ergebnis.append((y, n))
    return ergebnis


fehler = 0
bilder = sorted(p for p in ordner.glob("w[0-9][0-9]-*.png") if not p.stem.endswith("-ref"))
for bild in bilder:
    ref = bild.with_name(bild.stem + "-ref.png")
    if not ref.exists():
        print(f"FEHLT\t{bild.name}\tkeine Referenz")
        fehler += 1
        continue
    z = zeilen(GdkPixbuf.Pixbuf.new_from_file(str(bild)), GdkPixbuf.Pixbuf.new_from_file(str(ref)))
    if z:
        fehler += 1
        text = ", ".join(f"y{y}:{n}" for y, n in z[:12])
        print(f"RESTE\t{bild.name}\t{len(z)} Zeilen, {sum(n for _, n in z)} Pixel\t{text}")
    else:
        print(f"SAUBER\t{bild.name}")
print(f"{len(bilder) - fehler}/{len(bilder)} sauber")
sys.exit(1 if fehler else 0)
