#!/usr/bin/env python3
"""G2-AA: Auswertung der isolierten Fenstersymbol-Läufe (werkzeug/g2aa-gui-lauf.sh).

Je Lauf und Fenster (Hauptfenster, Unterdialog):
  * Quelle laut labwc -d: „app_id“ (Starter + Symbolthema) oder „client“
    (vom Programm per xdg-toplevel-icon geschickt),
  * Bild: größte Zahl orangefarbener Pixel (#F07746 ± Toleranz) in einem
    24×24-Fenster innerhalb des Suchbereichs der Titelleiste. Das Zahnrad hat
    über 60, der Schraubenschlüssel von gnome-human 0 („kein-zahnrad“).
Dazu Ausschnitte (4×, Nachbarpixel) nach nachweis/g2aa/bilder/.

  auswerten-fenstersymbol.py <ergebnis.tsv>
"""
import os
import sys

import gi

gi.require_version("GdkPixbuf", "2.0")
from gi.repository import GdkPixbuf  # noqa: E402

P = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
N = os.path.join(P, "nachweis")
BILDER = os.path.join(N, "g2aa", "bilder")
SCHWELLE = 60

# Suchbereiche der Titelleistensymbole (x0, y0, x1, y1); labwc setzt die
# Fenster mittig, je nach Theme um einige Pixel verschoben.
BEREICH = {
    ("1024", "haupt"): (70, 66, 140, 101),
    ("1024", "dialog"): (276, 190, 340, 224),
    ("1920", "haupt"): (515, 222, 580, 257),
    ("1920", "dialog"): (720, 344, 785, 380),
    ("1920", "hintergrund"): (590, 140, 660, 200),
}

# Lauf → (Auflösung, Szenario, Symbol Hauptfenster, Quelle Hauptfenster[, Symbol/Quelle Dialog])
# „kein-zahnrad“: Schraubenschlüssel (Hauptfenster) bzw. gar kein Symbol (0.0.5-Dialog
# im Altstand: GTK findet ohne ~/.local/share/icons kein Symbol, labwc keinen Starter).
LAEUFE = [
    ("f1-neu-frisch-1024", "1024", "kz", "zahnrad", "app_id"),
    ("f2-neu-frisch-1920", "1920", "kz", "zahnrad", "app_id"),
    ("f3-alt005-frisch-1024", "1024", "kz", "zahnrad", "app_id"),
    ("f4-alt005-frisch-1920", "1920", "kz", "zahnrad", "app_id"),
    ("f5-alt005-reconf-1920", "1920", "kz", "zahnrad", "app_id"),
    ("f6-neu-reconf-1920", "1920", "kz", "zahnrad", "app_id"),
    ("f7-alt005-altstand-1920", "1920", "kz", "kein-zahnrad", "app_id", "kein-zahnrad", "-"),
    ("f8-neu-altstand-1920", "1920", "kz", "kein-zahnrad", "app_id"),
    ("f9-alt005-client-1920", "1920", "kz", "kein-zahnrad", "app_id", "kein-zahnrad", "-"),
    ("f10-neu-client-1920", "1920", "kz", "zahnrad", "client"),
    ("f11-neu-client-1024", "1024", "kz", "zahnrad", "client"),
    ("f12-neu-hintergrund-1920", "1920", "hg", "zahnrad", "app_id"),
]


def orange(r, g, b):
    return r >= 200 and 85 <= g <= 155 and b <= 115


def zaehlen(pfad, bereich, name):
    pb = GdkPixbuf.Pixbuf.new_from_file(pfad)
    x0, y0, x1, y1 = bereich
    w, h = x1 - x0, y1 - y0
    teil = pb.new_subpixbuf(x0, y0, w, h)
    n, rs = teil.get_n_channels(), teil.get_rowstride()
    px = teil.get_pixels()
    maske = [[1 if orange(*px[y * rs + x * n:y * rs + x * n + 3]) else 0 for x in range(w)] for y in range(h)]
    best = 0
    for y in range(0, h - 23):
        for x in range(0, w - 23):
            s = sum(sum(maske[yy][x:x + 24]) for yy in range(y, y + 24))
            best = max(best, s)
    os.makedirs(BILDER, exist_ok=True)
    teil.scale_simple(w * 4, h * 4, GdkPixbuf.InterpType.NEAREST).savev(
        os.path.join(BILDER, name), "png", [], [])
    return best


def quellen(lauf):
    """Quellen der Fenstersymbole laut labwc -d, in Reihenfolge."""
    erg = []
    letzte_app = None
    with open(os.path.join(N, "g2aa-" + lauf, "logs", "labwc.log"), errors="replace") as f:
        for z in f:
            if "via exact match" in z or "failed to find .desktop file" in z or "via fuzzy" in z:
                letzte_app = z.split("'")[1] if "'" in z else z.strip()
            elif "loaded icon by app_id" in z:
                erg.append(("app_id", letzte_app))
            elif "loaded icon from client" in z:
                erg.append(("client", letzte_app))
    return erg


def main():
    ziel = sys.argv[1]
    zeilen = ["lauf\tfenster\tbild\torange\tsymbol\terwartet\tquelle_labwc\tergebnis"]
    gesamt = fehler = 0
    for lauf, aufl, sz, soll, soll_quelle, *dsoll in LAEUFE:
        d_symbol, d_quelle = dsoll if dsoll else ("zahnrad", "client")
        d = os.path.join(N, "g2aa-" + lauf)
        if not os.path.isdir(d):
            zeilen.append(f"{lauf}\t-\t-\t-\t-\t{soll}\t-\tFAIL (Lauf fehlt)")
            gesamt += 1; fehler += 1
            continue
        q = quellen(lauf)
        eingabe = open(os.path.join(d, "logs", "eingabe.log"), errors="replace").read()
        app = open(os.path.join(d, "logs", "app-direkt.log"), errors="replace").read()
        absturz = any(w in app for w in ("Segmentation", "double free", "Aborted", "free(): ", "CRITICAL"))
        if sz == "kz":
            pruef = [("01-kz-%s.png" % aufl, "haupt", soll, "haupt"),
                     ("03-dialog-%s.png" % aufl, "dialog", d_symbol, "dialog"),
                     ("04-ende-%s.png" % aufl, "haupt", soll, "haupt-ende")]
        else:
            pruef = [("01-hintergrund-1920.png", "hintergrund", soll, "hintergrund")]
        for bild, ber, erwartet, fenster in pruef:
            pfad = os.path.join(d, bild)
            gesamt += 1
            if not os.path.exists(pfad):
                zeilen.append(f"{lauf}\t{fenster}\t{bild}\t-\t-\t{erwartet}\t-\tFAIL (Bild fehlt)")
                fehler += 1
                continue
            o = zaehlen(pfad, BEREICH[(aufl, ber)], f"{lauf}-{fenster}.png")
            ist = "zahnrad" if o >= SCHWELLE else "kein-zahnrad"
            if fenster == "dialog" and soll_quelle == "client":
                # Mit iconPriority="client" sucht labwc gar keinen Starter,
                # die Ladevorgänge sind keinem Fenster zuzuordnen: alle „client“.
                ql = ",".join(sorted({k for k, _ in q})) or "-"
                soll_q = "client"
            elif fenster == "dialog":
                qd = [k for k, a in q if a == "querencia-einstellungen"]
                ql = ",".join(sorted(set(qd))) or "-"
                soll_q = d_quelle
            else:
                qh = [k for k, a in q if a != "querencia-einstellungen"]
                ql = ",".join(sorted(set(qh))) or "-"
                soll_q = soll_quelle
            ok = ist == erwartet and ql == soll_q and "szenario exit=0" in eingabe and not absturz
            if not ok:
                fehler += 1
            zeilen.append(f"{lauf}\t{fenster}\t{bild}\t{o}\t{ist}\t{erwartet}\t{ql} (soll {soll_q})\t{'PASS' if ok else 'FAIL'}")
    zeilen.append(f"# {gesamt - fehler}/{gesamt} PASS")
    with open(ziel, "w") as f:
        f.write("\n".join(zeilen) + "\n")
    print("\n".join(zeilen))
    return 1 if fehler else 0


if __name__ == "__main__":
    sys.exit(main())
