#!/usr/bin/env python3
"""Einstellungen 0.1.2: WCAG-2.x-Kontrast der eigenen Farben der
Anordnungsfläche (Seite Bildschirm, bildschirm.rs `HELL`/`DUNKEL`).
Text muss AA für normalen Text erreichen (4,5:1), Flächen und Ränder
gegen den Grund 3:1 (WCAG 1.4.11, Bedienelemente/Grafik).
Die Werte stehen hier und in bildschirm.rs; das Skript liest bildschirm.rs
und bricht ab, wenn eine Farbe dort fehlt.

  python3 -I werkzeug/kontrast-012.py [--markdown]
Exit 1, wenn ein Pflichtpaar verfehlt oder eine Farbe fehlt.
"""
import sys
from pathlib import Path

QUELLE = (Path(__file__).resolve().parent.parent / "crates/querencia-einstellungen/src/bildschirm.rs").read_text()

DUNKEL = {"flaeche": "2b2a27", "leer": "b3afa6", "gewaehlt": "1f5fae", "ausgabe": "5b5955",
          "rand": "bfbbb2", "leiste": "141414", "text_gewaehlt": "ffffff", "text": "f2f0eb"}
# (Element, Vordergrund, Grund, Mindestwert)
PAARE = [
    ("„Keine Ausgabe eingeschaltet“ auf der Fläche", "leer", "flaeche", 4.5),
    ("Name/Größe der gewählten Ausgabe", "text_gewaehlt", "gewaehlt", 4.5),
    ("Name/Größe einer anderen Ausgabe", "text", "ausgabe", 4.5),
    ("gewählte Ausgabe gegen die Fläche", "gewaehlt", "flaeche", 1.0),
    ("andere Ausgabe gegen die Fläche", "ausgabe", "flaeche", 1.0),
    ("Rand der Ausgabe gegen die Fläche", "rand", "flaeche", 3.0),
    ("Leiste des Hauptmonitors auf der Ausgabe", "leiste", "ausgabe", 1.5),
]


def lum(h):
    def k(c):
        c = int(c, 16) / 255
        return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4
    r, g, b = (k(h[i:i + 2]) for i in (0, 2, 4))
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def kontrast(a, b):
    x, y = sorted((lum(a), lum(b)), reverse=True)
    return (x + 0.05) / (y + 0.05)


def main():
    fehlt = [n for n, h in DUNKEL.items()
             if f"{n}: rgb(0x{h[0:2]}, 0x{h[2:4]}, 0x{h[4:6]})" not in QUELLE]
    md = "--markdown" in sys.argv
    schlecht = 0
    if md:
        print("| Paar | Vordergrund | Grund | Kontrast | Mindestens | Ergebnis |\n|---|---|---|---|---|---|")
    for name, v, g, mind in PAARE:
        k = kontrast(DUNKEL[v], DUNKEL[g])
        ok = k >= mind
        schlecht += not ok
        if md:
            print(f"| {name} | #{DUNKEL[v]} | #{DUNKEL[g]} | {k:.2f}:1 | {mind}:1 | {'PASS' if ok else 'FAIL'} |")
        else:
            print(f"{'PASS' if ok else 'FAIL'} {k:5.2f}:1 (≥ {mind}) {name}")
    for n in fehlt:
        print(f"FAIL Farbe {n} #{DUNKEL[n]} nicht in bildschirm.rs")
    return 1 if schlecht or fehlt else 0


if __name__ == "__main__":
    sys.exit(main())
