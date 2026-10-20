#!/usr/bin/env python3
"""G2-AF: Auswertung der isolierten Läufe nachweis/g2af-*/.

Je Lauf: (1) Gruppe [Background] in desktop.ini.nachher (Dateiname der
Fassung, Mode), (2) Bildschirmfoto des Desktops gegen das erwartete Bild,
wie der Desktop es zeichnet (zoom/centered/tiled; mittlere Abweichung je
Kanal an einem Raster außerhalb von Panels und Symbolspalte), (3) bei
Logo-Fassungen: Logo-Bereich näher an der Fassung mit Logo als an der ohne.
Bilder kommen aus dem Datenordner des Laufs (Profil share/).

Aufruf: python3 -I werkzeug/auswerten-g2af.py <ausgabe.tsv>
"""
import sys
from pathlib import Path

import gi

gi.require_version("GdkPixbuf", "2.0")
from gi.repository import GdkPixbuf  # noqa: E402

P = Path(__file__).resolve().parents[1]
N = P / "nachweis"
# Logo-Rechteck im Bild (aus erzeugen.py, docs/g2af-hintergruende.md)
LOGO = {(2560, 1440): (2177, 1165, 180, 176), (1600, 1200): (1338, 904, 150, 147)}
GRENZE = 10.0

# Lauf -> (Bildschirm, Foto, erwartete Datei ("" = File= leer), Mode, Bild auf dem Desktop)
LAEUFE = {
    "f1-abend-1920": ((1920, 1080), "04-desktop.png", "querencia-abend.png", "zoom", None),
    "f2-erde-logo-1024": ((1024, 768), "04-desktop.png", "querencia-erde-logo-4x3.png", "zoom", None),
    "f3-nacht-logo-2560": ((2560, 1440), "04-desktop.png", "querencia-nacht-logo.png", "zoom", None),
    "f4-zentrieren-1920": ((1920, 1080), "04-desktop.png", "querencia-blau2.png", "centered", None),
    "f5-kacheln-1024": ((1024, 768), "04-desktop.png", "querencia-nacht-4x3.png", "tiled", None),
    "f6-vorschau-1920": ((1920, 1080), "05-desktop.png", "querencia-abend-logo.png", "zoom", None),
    "f7-erkennen-1920": ((1920, 1080), "04-desktop.png", "", "zoom", "querencia-blau.png"),
    "f9-tastatur-1920": ((1920, 1080), "04-desktop.png", "querencia-abend-logo.png", "zoom", None),
}


def hintergrund(ini):
    werte, gruppe = {}, False
    for z in ini.read_text().splitlines() if ini.is_file() else []:
        z = z.strip()
        if z.startswith("["):
            gruppe = z == "[Background]"
        elif gruppe and "=" in z:
            k, v = z.split("=", 1)
            werte[k] = v
    return werte


def laden(p):
    pb = GdkPixbuf.Pixbuf.new_from_file(str(p))
    return pb, pb.get_pixels(), pb.get_rowstride(), pb.get_n_channels()


def pixel(b, x, y):
    pb, px, rs, n = b
    i = y * rs + x * n
    return px[i], px[i + 1], px[i + 2]


def erwartet(bild, mode, schirm, x, y):
    pb = bild[0]
    iw, ih = pb.get_width(), pb.get_height()
    w, h = schirm
    if mode == "zoom":
        s = max(w / iw, h / ih)
        ix, iy = (x + (iw * s - w) / 2) / s, (y + (ih * s - h) / 2) / s
    elif mode == "centered":
        ix, iy = x - (w - iw) / 2, y - (h - ih) / 2
    else:
        ix, iy = x % iw, y % ih
    ix, iy = int(ix), int(iy)
    if not (0 <= ix < iw and 0 <= iy < ih):
        return None
    return pixel(bild, ix, iy)


def abweichung(foto, bild, mode, schirm, punkte):
    summe = anzahl = 0
    for x, y in punkte:
        e = erwartet(bild, mode, schirm, x, y)
        if e is None:
            continue
        f = pixel(foto, x, y)
        summe += sum(abs(a - b) for a, b in zip(e, f)) / 3
        anzahl += 1
    return summe / anzahl if anzahl else 999.0


def raster(schirm):
    w, h = schirm
    return [(x, y) for x in range(200, w - 200, 37) for y in range(90, h - 60, 37)]


def logo_punkte(name, bild, schirm):
    """Punkte im Logo-Rechteck, in Bildschirmkoordinaten (zoom)."""
    pb = bild[0]
    iw, ih = pb.get_width(), pb.get_height()
    lx, ly, lb, lh = LOGO[(iw, ih)]
    w, h = schirm
    s = max(w / iw, h / ih)
    ox, oy = (iw * s - w) / 2, (ih * s - h) / 2
    pts = []
    for yy in range(ly, ly + lh, 3):
        for xx in range(lx, lx + lb, 3):
            x, y = int(xx * s - ox), int(yy * s - oy)
            if 0 <= x < w and 32 <= y < h - 32:
                pts.append((x, y))
    return pts


def pruefen(name, schirm, foto_name, datei, mode, bild_name, zeilen):
    lauf = N / f"g2af-{name}"
    hg = hintergrund(lauf / "desktop.ini.nachher")
    ist_datei = Path(hg.get("File", "")).name if hg.get("File") else ""
    ok_ini = ist_datei == datei and hg.get("Mode") == mode and (
        not datei or hg["File"].endswith(f"querencia/hintergruende/{datei}"))
    zeilen.append((name, "desktop.ini", f"File={hg.get('File', '-')} Mode={hg.get('Mode', '-')}",
                   "PASS" if ok_ini else "FAIL"))
    foto = laden(lauf / foto_name)
    daten = lauf / "profil" / "share" / "querencia" / "hintergruende"
    bild = laden(daten / (bild_name or datei))
    d = abweichung(foto, bild, mode, schirm, raster(schirm))
    zeilen.append((name, "Desktop zeigt das Bild", f"Abweichung {d:.1f}",
                   "PASS" if d < GRENZE else "FAIL"))
    if "-logo" in (bild_name or datei):
        ohne = laden(daten / (bild_name or datei).replace("-logo", ""))
        pts = logo_punkte(name, bild, schirm)
        mit_d = abweichung(foto, bild, mode, schirm, pts)
        ohne_d = abweichung(foto, ohne, mode, schirm, pts)
        zeilen.append((name, "Logo sichtbar unten rechts", f"mit {mit_d:.1f} / ohne {ohne_d:.1f}, {len(pts)} Punkte",
                       "PASS" if mit_d < ohne_d and mit_d < GRENZE else "FAIL"))


def main():
    zeilen = []
    for name, (schirm, foto, datei, mode, bild) in LAEUFE.items():
        if (N / f"g2af-{name}").is_dir():
            pruefen(name, schirm, foto, datei, mode, bild, zeilen)
        else:
            zeilen.append((name, "Lauf vorhanden", "fehlt", "FAIL"))
    # f8: fremdes Bild bleibt unverändert
    f8 = N / "g2af-f8-fremd-1920"
    vorher, nachher = f8 / "desktop.ini.vorher", f8 / "desktop.ini.nachher"
    gleich = vorher.is_file() and nachher.is_file() and vorher.read_bytes() == nachher.read_bytes()
    zeilen.append(("f8-fremd-1920", "desktop.ini unverändert (Abbrechen)", "", "PASS" if gleich else "FAIL"))
    # Lesbarkeits- und Logo-Läufe: nur der Desktop mit vorgegebener desktop.ini
    for lauf in sorted(N.glob("g2af-l-*")):
        teile = lauf.name.split("-")
        w, h = (int(v) for v in teile[-1].split("x"))
        hg = hintergrund(lauf / "desktop.ini.vorher")
        datei = Path(hg["File"]).name
        name = lauf.name[len("g2af-"):]
        foto = laden(lauf / "01-desktop.png")
        bild = laden(lauf / "profil" / "share" / "querencia" / "hintergruende" / datei)
        d = abweichung(foto, bild, "zoom", (w, h), raster((w, h)))
        zeilen.append((name, f"Desktop zeigt {datei}", f"Abweichung {d:.1f}", "PASS" if d < GRENZE else "FAIL"))
        if "-logo" in datei:
            ohne = laden(lauf / "profil" / "share" / "querencia" / "hintergruende" / datei.replace("-logo", ""))
            pts = logo_punkte(name, bild, (w, h))
            mit_d = abweichung(foto, bild, "zoom", (w, h), pts)
            ohne_d = abweichung(foto, ohne, "zoom", (w, h), pts)
            zeilen.append((name, "Logo sichtbar, über dem unteren Panel",
                           f"mit {mit_d:.1f} / ohne {ohne_d:.1f}, {len(pts)} Punkte",
                           "PASS" if mit_d < ohne_d and mit_d < GRENZE and len(pts) > 100 else "FAIL"))
    with open(sys.argv[1], "w") as f:
        f.write("lauf\tpruefung\twert\turteil\n")
        for z in zeilen:
            f.write("\t".join(z) + "\n")
    fail = sum(z[3] == "FAIL" for z in zeilen)
    print(f"{len(zeilen) - fail}/{len(zeilen)} PASS")
    return 1 if fail else 0


if __name__ == "__main__":
    sys.exit(main())
