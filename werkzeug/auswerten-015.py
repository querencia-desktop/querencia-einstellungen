#!/usr/bin/python3
"""Einstellungen 0.1.5: Auswertung von werkzeug/015-gui-lauf.sh.

Verglichen wird im Fensterausschnitt wie auswerten-014.py (Fenster samt
Titelleiste; Panel-Uhr und Desktop außen vor), Seiten nach Namen gepaart
(0.1.5 hat die Seite „Netzwerk“ zusätzlich, Sprache und Informationen
rücken eine Zeile nach unten).
  d1 alt ↔ neu  je Seite: Abweichungen nur
                - in der Seitenleiste ab der Zeile „Netzwerk“ (neue Zeile,
                  die beiden letzten rücken nach unten), und
                - im Seiteninhalt ab der Oberkante der Fußzeile „Quelle: …“
                  von 0.1.4 (die Fußzeile ist jetzt kleiner und hat Abstand
                  nach oben; alles darüber bleibt pixelgleich).
                Ausnahmen: Uhrzeit im Formatbeispiel („Sprache“), die
                Spalte des Overlay-Rollbalkens, „Informationen“ (neu
                aufgebaut; nur Markierung). „Erscheinungsbild unten“: die
                Seite ist kürzer, ans Ende gescrollt steht der Inhalt um dy
                verschoben; darüber gleich bis auf höchstens 0,1 % der
                Fläche mit höchstens 16 Stufen (Zeiger-Hover, Animation).
  Kontrast  die neue Fußzeile (klein, grau) ≥ 4,5:1, hell und dunkel
  i1 ohne ↔ mit  Informationen: Abweichung nur in der Titelzeile
                 („Querencia Desktop“ ↔ „Querencia Desktop Alpha 2.1“)
  i1 mit ↔ nurversion  pixelgleich (Version aus der Datei version)
  n1 ↔ d1 12-netzwerk  --netzwerk öffnet die Seite „Netzwerk“: Seiteninhalt
                 gleich (Seitenleiste ohne Zeigerhervorhebung verschieden)
Bilder alt | neu | Markierung unter nachweis/015/bilder/.

  . ~/projects/querencia-boot-splash/werkzeug/pil-umgebung.sh  # Pillow (PYTHONPATH)
  python3 werkzeug/auswerten-015.py <ziel.tsv>
"""
import os
import sys

from PIL import Image, ImageChops

sys.dont_write_bytecode = True
P = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
N = os.path.join(P, "nachweis")
BILDORDNER = os.path.join(N, "015", "bilder")
ALT = ["erscheinungsbild", "hintergrundbild", "symbole", "fensterdesign", "schriften", "panels",
       "tastatur", "maus", "bildschirm", "arbeitsflaechen", "standardanwendungen", "sprache",
       "informationen"]
NEU = ALT[:11] + ["netzwerk"] + ALT[11:]
UNTEN = ("14-erscheinungsbild-unten.png", "15-erscheinungsbild-unten.png")
INHALT_X, LEISTE_Y, KOPF_Y = 244, 602, 98  # Seitenleiste mit Trennlinie bis x 243
SEITENLEISTE_AB_Y = 405  # Oberkante der Zeile „Netzwerk“ im Ausschnitt (Zeile 12, 28 px)
UHRZEIT = (470, 506)     # Zeilenband der Uhrzeit im Formatbeispiel („Sprache“)
ROLLBALKEN = (850, 875)
TITELZEILE = (KOPF_Y, 160)
zeilen = []


def pruef(gruppe, fall, ok, detail=""):
    zeilen.append((gruppe, fall, "PASS" if ok else "FAIL", str(detail).replace("\t", " ")[:400]))


def info(gruppe, fall, detail):
    zeilen.append((gruppe, fall, "INFO", str(detail).replace("\t", " ")[:400]))


def bild(lauf, datei):
    p = os.path.join(N, lauf, datei)
    if not os.path.exists(p):
        return None
    im = Image.open(p).convert("RGB")
    b = int(lauf.rsplit("-", 1)[1])
    dx, dy = (0, 0) if b == 1920 else (-448, -156)
    return im.crop((520 + dx, 200 + dy, 1400 + dx, 850 + dy))


def markierung(a, b, name):
    os.makedirs(BILDORDNER, exist_ok=True)
    m = b.point(lambda v: v // 3 + 170)
    d = ImageChops.difference(a, b).convert("L").point(lambda v: 255 if v else 0)
    m.paste((255, 140, 0), mask=d)
    w, h = a.size
    ges = Image.new("RGB", (3 * w + 20, h), (255, 255, 255))
    ges.paste(a, (0, 0))
    ges.paste(b, (w + 10, 0))
    ges.paste(m, (2 * w + 20, 0))
    ges.save(os.path.join(BILDORDNER, name))


def box_in(im, x0, y0, x1, y1):
    return im.crop((x0, y0, x1, y1)).getbbox()


def lum(c):
    def k(v):
        v /= 255
        return v / 12.92 if v <= 0.03928 else ((v + 0.055) / 1.055) ** 2.4
    return 0.2126 * k(c[0]) + 0.7152 * k(c[1]) + 0.0722 * k(c[2])


def kontrast(a, b):
    x, y = sorted((lum(a), lum(b)), reverse=True)
    return (x + 0.05) / (y + 0.05)


def bloecke(im):
    """Textblöcke im Seiteninhalt (Zeilen mit Pixeln außer der Grundfarbe,
    Lücken bis 4 px), je (oben, unten, stärkster Kontrast zum Grund)."""
    w, h = im.size
    px = im.load()
    x0, x1 = INHALT_X + 10, w - 30
    grund = px[x1 - 5, KOPF_Y + 2]
    zeilen_k = []
    for y in range(KOPF_Y + 1, h - 2):
        k = max((kontrast(px[x, y], grund) for x in range(x0, x1) if px[x, y] != grund), default=0)
        zeilen_k.append((y, k))
    out, cur, luecke = [], None, 0
    for y, k in zeilen_k:
        if k > 0:
            if cur is None:
                cur = [y, y, k]
            else:
                cur[1], cur[2] = y, max(cur[2], k)
            luecke = 0
        elif cur is not None:
            luecke += 1
            if luecke > 4:
                out.append(tuple(cur))
                cur = None
    if cur is not None:
        out.append(tuple(cur))
    return out


def fusszeile_oben(im):
    """Oberkante der Fußzeile von 0.1.4: der unterste graue Textblock
    (dim-label, Kontrast unter 7:1) im Seiteninhalt; darunter stehen nur
    Statuszeile und Knöpfe (volle Schriftfarbe)."""
    for oben, unten, k in reversed(bloecke(im)):
        if 2.5 < k < 7:
            return oben, unten
    return None, None


def verschoben_gleich(a, b, oben_bis):
    """Inhalt oberhalb der Fußzeile gleich bis auf eine senkrechte
    Verschiebung (kürzere Seite, ans Ende gescrollt); gesucht dy in ±40 mit
    den wenigsten abweichenden Pixeln. Liefert (dy, Pixel, höchste Stufe)."""
    w, h = a.size
    x0, x1 = INHALT_X, ROLLBALKEN[0]
    best = None
    for dy in range(-40, 41):
        ya0, ya1 = KOPF_Y + 41, oben_bis
        if ya1 - ya0 < 40 or ya1 + dy > h or ya0 + dy < KOPF_Y:
            continue
        d = ImageChops.difference(a.crop((x0, ya0, x1, ya1)), b.crop((x0, ya0 + dy, x1, ya1 + dy))).convert("L")
        n = sum(1 for v in getattr(d, "get_flattened_data", d.getdata)() if v)
        if best is None or n < best[1]:
            best = (dy, n, d.getextrema()[1], (x1 - x0) * (ya1 - ya0))
    return best


def d1(sp_b, art):
    alt, neu = f"015-d1-{art}-alt-{sp_b}", f"015-d1-{art}-neu-{sp_b}"
    gruppe = f"d1 {art} {sp_b}"
    paare = [(f"{i:02d}-{s}.png", f"{NEU.index(s) + 1:02d}-{s}.png", s) for i, s in enumerate(ALT, 1)]
    paare.append((UNTEN[0], UNTEN[1], "erscheinungsbild-unten"))
    for da, dn, s in paare:
        a, b = bild(alt, da), bild(neu, dn)
        fall = f"{s}: {alt}/{da} ↔ {neu}/{dn}"
        if a is None or b is None:
            pruef(gruppe, fall, False, "Bild fehlt")
            continue
        markierung(a, b, f"{art}-{sp_b}-{s}.png")
        w, h = a.size
        # Seitenleiste
        sb = box_in(ImageChops.difference(a, b), 0, 0, INHALT_X, h)
        pruef(gruppe, f"{fall}: Seitenleiste nur ab der Zeile „Netzwerk“", sb is None or sb[1] >= SEITENLEISTE_AB_Y, sb)
        if s == "informationen":
            ib = box_in(ImageChops.difference(a, b), INHALT_X, 0, w, h)
            info(gruppe, f"{fall}: Seite neu aufgebaut (Markierung)", ib)
            continue
        d = ImageChops.difference(a, b)
        # benannte Ausnahmen ausblenden
        if s == "sprache":
            d.paste((0, 0, 0), (INHALT_X, UHRZEIT[0], w, UHRZEIT[1]))
        d.paste((0, 0, 0), (INHALT_X + ROLLBALKEN[0] - INHALT_X, 0, INHALT_X + ROLLBALKEN[1] - INHALT_X, h))
        ib = box_in(d, INHALT_X, 0, w, h)
        if ib is None:
            pruef(gruppe, f"{fall}: Seiteninhalt gleich (Fußzeile nicht im Bild)", True)
            continue
        oben, unten = fusszeile_oben(a)
        y0 = ib[1]
        if oben is not None and y0 >= oben - 1:
            pruef(gruppe, f"{fall}: Seiteninhalt nur ab der Fußzeile", True,
                  f"Abweichung {ib} (Ausschnitt ab x {INHALT_X}), Fußzeile 0.1.4 y {oben}–{unten}")
            continue
        v = verschoben_gleich(a, b, oben) if oben is not None else None
        # benannte Toleranz: höchstens 0,1 % der Fläche und 16 Stufen
        # (Zeiger-Hover und Ausblend-Animation an Optionsknöpfen unter dem
        # verschobenen Inhalt)
        ok = v is not None and v[1] <= v[3] // 1000 and v[2] <= 16
        pruef(gruppe, f"{fall}: Seiteninhalt über der Fußzeile gleich, um dy verschoben (Seite kürzer, ans Ende gescrollt)",
              ok, f"dy {v[0] if v else None}, danach {v[1] if v else '-'} Pixel abweichend (höchstens {v[2] if v else '-'} Stufen, Fläche {v[3] if v else '-'}), Fußzeile 0.1.4 y {oben}–{unten}")


def fuss_kontrast():
    """Kontrast der neuen Fußzeile (klein, dim-label): schwächster Block
    ganz unten im Inhalt von 15-erscheinungsbild-unten und 03-symbole."""
    for k in ("de-1024", "en-1920"):
        for art in ("hell", "dunkel"):
            for datei in ("03-symbole.png", "15-erscheinungsbild-unten.png"):
                im = bild(f"015-d1-{art}-neu-{k}", datei)
                if im is None:
                    pruef("kontrast", f"{art} {k} {datei}", False, "Bild fehlt")
                    continue
                f = fusszeile_oben(im)
                bl = [b for b in bloecke(im) if f[0] is not None and b[0] == f[0]]
                kk = bl[0][2] if bl else 0
                pruef("kontrast", f"{art} {k} {datei}: Fußzeile ≥ 4,5:1", kk >= 4.5, f"{kk:.2f}:1, Block y {f[0]}–{f[1]}")


def i1():
    for k in ("de-1024", "en-1920"):
        for art in ("hell", "dunkel"):
            o, m = f"015-i1-{art}-ohne-{k}", f"015-i1-{art}-mit-{k}"
            a, b = bild(o, "01-informationen.png"), bild(m, "01-informationen.png")
            gruppe = f"i1 {art} {k}"
            if a is None or b is None:
                pruef(gruppe, "Bilder vorhanden", False, "Bild fehlt")
                continue
            markierung(a, b, f"info-{art}-{k}-ohne-mit.png")
            box = ImageChops.difference(a, b).getbbox()
            pruef(gruppe, f"{o} ↔ {m}: nur die Titelzeile (Produktversion)",
                  box is not None and TITELZEILE[0] <= box[1] and box[3] <= TITELZEILE[1], box)
    a = bild("015-i1-hell-mit-de-1024", "01-informationen.png")
    b = bild("015-i1-hell-nurversion-de-1024", "01-informationen.png")
    if a is None or b is None:
        pruef("i1 nurversion", "Bilder vorhanden", False, "Bild fehlt")
    else:
        box = ImageChops.difference(a, b).getbbox()
        pruef("i1 nurversion", "release ↔ nur version: pixelgleich", box is None, box)


def n1():
    for lauf, d1lauf in (("015-n1-hell-neu-de-1024", "015-d1-hell-neu-de-1024"),
                         ("015-n1-dunkel-neu-en-1920", "015-d1-dunkel-neu-en-1920")):
        a, b = bild(lauf, "01-netzwerk.png"), bild(d1lauf, "12-netzwerk.png")
        if a is None or b is None:
            pruef("n1", lauf, False, "Bild fehlt")
            continue
        w, h = a.size
        box = box_in(ImageChops.difference(a, b), INHALT_X, 0, w, h)
        pruef("n1", f"{lauf} (--netzwerk) ↔ {d1lauf}/12-netzwerk: Seiteninhalt gleich", box is None, box)
        # gewählte Zeile in der Seitenleiste = „Netzwerk“ (Zeile 12)
        px = a.load()
        zeile = [px[40, y] for y in range(SEITENLEISTE_AB_Y + 10, SEITENLEISTE_AB_Y + 30)]
        leiste = px[40, SEITENLEISTE_AB_Y + 60]
        pruef("n1", f"{lauf}: Zeile „Netzwerk“ gewählt", all(c != leiste for c in zeile),
              f"Farben {sorted(set(zeile))[:3]} gegen Grund der Seitenleiste {leiste}")


def main():
    for k in ("de-1024", "en-1920"):
        for art in ("hell", "dunkel"):
            d1(k, art)
    fuss_kontrast()
    i1()
    n1()
    ziel = sys.argv[1] if len(sys.argv) > 1 else os.path.join(N, "015", "gui-015.tsv")
    with open(ziel, "w") as f:
        f.write("gruppe\tfall\tergebnis\tdetail\n")
        for z in zeilen:
            f.write("\t".join(z) + "\n")
    p = sum(z[2] == "PASS" for z in zeilen)
    fl = sum(z[2] == "FAIL" for z in zeilen)
    for z in zeilen:
        if z[2] == "FAIL":
            print("FAIL", z[0], z[1], z[3])
    print(f"{ziel}: PASS {p}, FAIL {fl}, INFO {sum(z[2] == 'INFO' for z in zeilen)}")
    return 1 if fl else 0


if __name__ == "__main__":
    sys.exit(main())
