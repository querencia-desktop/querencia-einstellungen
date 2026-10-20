#!/usr/bin/python3
"""Einstellungen 0.1.2: Auswertung von werkzeug/012-gui-lauf.sh.

Je Breite (1920/1024) und Sprache (de/en):
  t01  installiertes Querencia-Klassisch (ohne dunkle Fassung): 0.1.2 ist in
       allen Bildern pixelgleich mit 0.1.1, hell wie dunkel eingestellt
  t02  Thema 0.2 hell: pixelgleich bis auf den Hinweis unter „Dunkle
       Darstellung bevorzugen“ (nur die Bilder der Seite Erscheinungsbild,
       Unterschied auf den Bereich ab dem Hinweis begrenzt)
  t02  dunkel ab dem Start: 0.1.2 dunkel (mittlere Helligkeit des Fensters),
       Anordnungsfläche der Seite Bildschirm #2b2a27; 0.1.1 bleibt hell (QD-081)
  w1   Wechsel im Lauf (0.1.2): hell → dunkel → hell; das letzte Hell ist
       pixelgleich mit dem ersten, das Dunkel gleich dem Start unter dunkel
Schreibt eine TSV; Exit 0 nur ohne FAIL.

  . ~/projects/querencia-boot-splash/werkzeug/pil-umgebung.sh  # Pillow (PYTHONPATH)
  python3 werkzeug/auswerten-012.py <ziel.tsv>
"""
import os
import sys

from PIL import Image, ImageChops, ImageStat

P = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
N = os.path.join(P, "nachweis")
SEITEN = ["erscheinungsbild", "hintergrundbild", "symbole", "fensterdesign", "schriften", "panels",
          "tastatur", "maus", "bildschirm", "arbeitsflaechen", "standardanwendungen", "sprache",
          "informationen"]
BILDER = [f"{n:02d}-{s}.png" for n, s in enumerate(SEITEN, 1)] + ["14-erscheinungsbild-unten.png"]
zeilen = []


def pruef(lauf, fall, ok, detail=""):
    zeilen.append((lauf, fall, "PASS" if ok else "FAIL", str(detail).replace("\t", " ")[:300]))


def fenster(b):
    """Innenfläche des Kontrollzentrums ohne Titelleiste, im Ausschnitt von bild()."""
    return (20, 50, 860, 630)


def bild(lauf, datei):
    """Nur das Fenster samt Titelleiste (Panel-Uhr und Desktop außen vor)."""
    p = os.path.join(N, lauf, datei)
    if not os.path.exists(p):
        return None
    b = int(lauf.rsplit("-", 1)[1])
    dx, dy = (0, 0) if b == 1920 else (-448, -156)
    return Image.open(p).convert("RGB").crop((520 + dx, 200 + dy, 1400 + dx, 850 + dy))


def unterschied(a, b):
    """None = gleich, sonst Begrenzungsrechteck der Abweichung."""
    if a is None or b is None or a.size != b.size:
        return "fehlt"
    return ImageChops.difference(a, b).getbbox()


def helligkeit(im, box):
    return ImageStat.Stat(im.crop(box).convert("L")).mean[0]


def gleich_alle(a, b, name, ausser=()):
    for d in BILDER:
        u = unterschied(bild(a, d), bild(b, d))
        if d in ausser:
            continue
        pruef(name, f"{d} pixelgleich {a} ↔ {b}", u is None, u)


def main():
    for b in (1920, 1024):
        box = fenster(b)
        for sp in ("de", "en"):
            k = f"{sp}-{b}"
            # t01: installiertes Thema, hell und dunkel eingestellt
            for art in ("hell", "dunkel"):
                gleich_alle(f"012-t01-{art}-alt-{k}", f"012-t01-{art}-neu-{k}", f"t01-{art}-{k}")
            # t02 hell: nur der Hinweis
            a, n = f"012-t02-hell-alt-{k}", f"012-t02-hell-neu-{k}"
            gleich_alle(a, n, f"t02-hell-{k}", ausser=("01-erscheinungsbild.png", "14-erscheinungsbild-unten.png"))
            for d in ("01-erscheinungsbild.png", "14-erscheinungsbild-unten.png"):
                u = unterschied(bild(a, d), bild(n, d))
                pruef(f"t02-hell-{k}", f"{d}: Abweichung nur im Fenster (Hinweis)",
                      u not in (None, "fehlt") and u[1] >= box[1],
                      f"Abweichung {u}")
            # t02 dunkel ab dem Start
            for v, soll in (("neu", "dunkel"), ("alt", "hell")):
                lauf = f"012-t02-dunkel-{v}-{k}"
                for d in BILDER:
                    im = bild(lauf, d)
                    h = helligkeit(im, box) if im else -1
                    ok = im is not None and ((h < 90) if soll == "dunkel" else (h > 180))
                    pruef(f"t02-dunkel-{v}-{k}", f"{d}: {soll} beim Start", ok, f"mittlere Helligkeit {h:.1f}")
            im = bild(f"012-t02-dunkel-neu-{k}", "09-bildschirm.png")
            # Anordnungsfläche: Mitte des oberen Drittels der Seite, wo keine Ausgabe liegt
            if im:
                farben = im.crop(box).getcolors(1 << 20)
                hat = any(c[:3] == (0x2b, 0x2a, 0x27) and n_ > 2000 for n_, c in farben)
                pruef(f"t02-dunkel-neu-{k}", "09-bildschirm.png: Anordnungsfläche #2b2a27", hat,
                      "größte Farben " + ", ".join(f"#{c[0]:02x}{c[1]:02x}{c[2]:02x}×{n_}" for n_, c in sorted(farben, reverse=True)[:4]))
            else:
                pruef(f"t02-dunkel-neu-{k}", "09-bildschirm.png vorhanden", False)
            # Wechsel im Lauf
            w = f"012-t02-wechsel-neu-{k}"
            teile = ["erscheinungsbild", "erscheinungsbild-unten", "bildschirm", "hintergrundbild"]
            for t in teile:
                h1, d2, h3 = (bild(w, f"{i}-{wort}-{t}.png") for i, wort in ((1, "hell"), (2, "dunkel"), (3, "hell")))
                pruef(w, f"{t}: hell nach dem Rückwechsel pixelgleich mit vorher", unterschied(h1, h3) is None, unterschied(h1, h3))
                hd = helligkeit(d2, box) if d2 else -1
                hh = helligkeit(h1, box) if h1 else -1
                pruef(w, f"{t}: dunkel nach dem Wechsel", d2 is not None and hd < 90 and hh > 180, f"hell {hh:.1f}, dunkel {hd:.1f}")
            start = {"erscheinungsbild": "01-erscheinungsbild.png", "erscheinungsbild-unten": "14-erscheinungsbild-unten.png",
                     "bildschirm": "09-bildschirm.png", "hintergrundbild": "02-hintergrundbild.png"}
            for t, d in start.items():
                u = unterschied(bild(w, f"2-dunkel-{t}.png"), bild(f"012-t02-dunkel-neu-{k}", d))
                pruef(w, f"{t}: Wechsel ins Dunkle = Start unter dunkel", u is None, u)
            # das Hell im Wechsellauf gleicht dem hellen Start
            for t, d in start.items():
                u = unterschied(bild(w, f"1-hell-{t}.png"), bild(f"012-t02-hell-neu-{k}", d))
                pruef(w, f"{t}: hell im Wechsellauf = Start unter hell", u is None, u)
    # 0.1.1 im Wechsel (Vergleich, nur 1920 de): folgt dem Wechsel im Lauf schon
    w = "012-t02-wechsel-alt-de-1920"
    d2, h1 = bild(w, "2-dunkel-erscheinungsbild.png"), bild(w, "1-hell-erscheinungsbild.png")
    hd = helligkeit(d2, fenster(1920)) if d2 else -1
    zeilen.append((w, "0.1.1 folgt dem Wechsel im Lauf (Information)", "INFO", f"hell {helligkeit(h1, fenster(1920)) if h1 else -1:.1f}, dunkel {hd:.1f}"))
    with open(sys.argv[1], "w", encoding="utf-8") as f:
        f.write("lauf\tfall\tergebnis\tdetail\n")
        for z in zeilen:
            f.write("\t".join(z) + "\n")
    p = sum(z[2] == "PASS" for z in zeilen)
    fl = sum(z[2] == "FAIL" for z in zeilen)
    print(f"ERGEBNIS {p} PASS, {fl} FAIL ({sys.argv[1]})")
    for z in zeilen:
        if z[2] == "FAIL":
            print("FAIL", *z[:2], z[3])
    return 1 if fl else 0


if __name__ == "__main__":
    sys.exit(main())
