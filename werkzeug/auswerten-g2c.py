#!/usr/bin/env python3
"""G2-C: Zusatzprüfungen mit dem Release-Build – r13 (`querencia-einstellungen
--hintergrund` öffnet direkt „Querencia – Hintergrund“) und r14 (ohne Argument
öffnet sich das Kontrollzentrum, nicht der Hintergrund-Dialog).
Ergebnis: nachweis/g2c-direktstart.tsv (bzw. argv[1])."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from PIL import Image  # noqa: E402

N = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "nachweis")
erg = []


def pruefe(nr, test, ok, beleg):
    erg.append((nr, test, "PASS" if ok else "FAIL", beleg))
    print(f"{'PASS' if ok else 'FAIL'}  {nr:5s} {test}: {beleg}")


def bild(lauf, name):
    return Image.open(os.path.join(N, lauf, name)).convert("RGB")


def lesen(lauf, name):
    p = os.path.join(N, lauf, name)
    return open(p, encoding="utf-8", errors="replace").read() if os.path.exists(p) else ""


def vorschau_rechteck(im, x0=560, x1=1360, y0=140, y1=560):
    """Vorschau-Rahmen (64,64,64) des Hintergrund-Dialogs, wie auswerten.py."""
    px = im.load()
    zeilen = {}
    for y in range(y0, y1):
        xs = [x for x in range(x0, x1) if px[x, y] == (64, 64, 64)]
        if len(xs) > 200:
            zeilen[y] = (min(xs), max(xs))
    if len(zeilen) < 2:
        return None
    top, bottom = min(zeilen), max(zeilen)
    left, right = zeilen[top]
    return left + 2, top + 2, right - 2, bottom - 2


BLAU = (28, 78, 150)  # Querencia Blau, Standardfarbe der Fläche


def fensterfarbe(im, p):
    """Pixel gehört zu einem GTK-Fenster (hell, nicht Hintergrundblau)."""
    r, g, b = im.getpixel(p)
    return (r, g, b) != BLAU and min(r, g, b) > 180


L = "r13-hintergrund-direkt"
im = bild(L, "01-dialog-direkt.png")
r = vorschau_rechteck(im)
log = lesen(L, "logs/app-direkt.log")
pruefe("13.1", "`--hintergrund` öffnet direkt „Querencia – Hintergrund“ (Vorschau-Rahmen wie r12)",
       r is not None and abs(r[0] - 748) < 40 and abs(r[1] - 217) < 40, f"Vorschau-Rahmen {r}")
pruefe("13.2", "Dialog erkennt den QE-01-Desktop (Log: Farbe true)", "Farbe true" in log, log.strip().replace("\n", " | ")[-120:])
pruefe("13.3", "Kein Absturz, keine CRITICAL-Meldung", "panicked" not in log and "CRITICAL" not in log, "app-direkt.log sauber")

L = "r14-kontrollzentrum"
im = bild(L, "01-kontrollzentrum.png")
r = vorschau_rechteck(im)
log = lesen(L, "logs/app-direkt.log")
pruefe("14.1", "Ohne Argument: Fenster in der Mitte offen (Kontrollzentrum), kein Hintergrund-Dialog",
       r is None and fensterfarbe(im, (960, 540)) and fensterfarbe(im, (700, 400)),
       f"Vorschau-Rahmen {r}, (960,540) {im.getpixel((960, 540))}, (700,400) {im.getpixel((700, 400))}")
pruefe("14.2", "Kein Absturz, keine CRITICAL-Meldung", "panicked" not in log and "CRITICAL" not in log, "app-direkt.log sauber")

ziel = sys.argv[1] if len(sys.argv) > 1 else "g2c-direktstart.tsv"
with open(os.path.join(N, ziel), "w", encoding="utf-8") as f:
    f.write("Nr\tPrüfung\tErgebnis\tBeleg\n")
    for z in erg:
        f.write("\t".join(z) + "\n")
n_ok = sum(1 for z in erg if z[2] == "PASS")
print(f"ERGEBNIS g2c-direktstart: {n_ok}/{len(erg)} PASS")
sys.exit(0 if n_ok == len(erg) else 1)
