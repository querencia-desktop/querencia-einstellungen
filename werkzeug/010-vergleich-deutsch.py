#!/usr/bin/python3 -I
"""Einstellungen 0.1.0: Pixelvergleich der deutschen Seiten 0.0.9 ↔ 0.1.0.

Vergleicht den Inhaltsbereich rechts der Seitenleiste (ohne Panels und
Seitenleiste, die in 0.1.0 eine Zeile mehr hat) der Bilder aus
 mit nachweis/010-a1-de-<b>/.
Gleiche Pixel = deutsche Oberfläche unverändert. Pfadtexte (Profilordner)
weichen ab, deshalb wird der Anteil abweichender Pixel gemeldet.

  python3 -I werkzeug/010-vergleich-deutsch.py <1920|1024>
"""
import os
import sys

import gi

gi.require_version("GdkPixbuf", "2.0")
from gi.repository import GdkPixbuf  # noqa: E402

B = sys.argv[1]
ALT = os.path.expanduser(f")
P = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
NEU = os.path.join(P, "nachweis", f"010-a1-de-{B}")
dx, dy = (0, 0) if B == "1920" else (-448, -156)
# Fensterinhalt rechts der Seitenleiste (Seite und Fußleiste)
X0, Y0, X1, Y1 = 770 + dx, 298 + dy, 1388 + dx, 852 + dy
PAARE = [(f"{i:02d}-{n}.png", f"{i:02d}-{n}.png") for i, n in enumerate(
    ["erscheinungsbild", "hintergrundbild", "symbole", "fensterdesign", "schriften", "panels",
     "tastatur", "maus", "bildschirm", "arbeitsflaechen", "standardanwendungen"], 1)]
PAARE.append(("12-informationen.png", "13-informationen.png"))
fehler = 0
for a, n in PAARE:
    pa = GdkPixbuf.Pixbuf.new_from_file(os.path.join(ALT, a))
    pn = GdkPixbuf.Pixbuf.new_from_file(os.path.join(NEU, n))
    da, dn = pa.get_pixels(), pn.get_pixels()
    sa, sn, ca, cn = pa.get_rowstride(), pn.get_rowstride(), pa.get_n_channels(), pn.get_n_channels()
    anders = 0
    gesamt = 0
    for y in range(Y0, Y1):
        for x in range(X0, X1):
            gesamt += 1
            o, p = y * sa + x * ca, y * sn + x * cn
            if da[o:o + 3] != dn[p:p + 3]:
                anders += 1
    anteil = 100.0 * anders / gesamt
    print(f"{n}\t{anders}\t{anteil:.3f} %")
