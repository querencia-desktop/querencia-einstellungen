#!/usr/bin/python3
"""Einstellungen 0.1.4 (Hintergründe 0.2): Auswertung von werkzeug/014-gui-lauf.sh.

Verglichen wird im Fensterausschnitt wie auswerten-013.py:
  alt ↔ ohne  0.1.3 gegen 0.1.4 ohne die Bilder von Hintergründe 0.2: jedes
              Bild pixelgleich bis auf die benannten Zeilenbänder
              (Versionsnummer, Uhrzeit im Formatbeispiel), hell und dunkel
  alt ↔ mit   0.1.3 gegen 0.1.4 mit den 32 neuen Dateien: alle Bilder ohne
              Galerie pixelgleich (bis auf die benannten); 02-hintergrundbild,
              15/16 (Galerie gescrollt) nur im Seiteninhalt rechts der
              Seitenleiste über der Knopfleiste (Galerie; in 02 dazu Vorschau
              und Name des eingebauten Bilds, das jetzt Tiefblau ist)
  ohne ↔ mit  die Galerie-Bilder unterscheiden sich (die neuen Kacheln sind da)
Standard-Probe s1 (Dialog ab „Querencia Erde“, de 1024 hell, „Standard
wiederherstellen“ und „Übernehmen“): File= in desktop.ini nachher leer in
alt, ohne und mit; alt ↔ ohne pixelgleich (ganzer Bildschirm ohne
Panelzeile mit der Uhr); ohne ↔ mit nach dem Zurücksetzen verschieden
(Vorschau Tiefblau statt Blau).
Benannte Ausnahmen außerdem: Rollbalken-Spalte in 15/16 nur als
Farbübergang (Ein-/Ausblenden zu einem anderen Zeitpunkt fotografiert,
höchstens 8 Stufen je Kanal, Lage gleich); in s1 03-uebernommen die
Statuszeile (Uhrzeit und Name der Sicherung).
Erkennen e1: File= auf den Verweis …/querencia-glas-4x3.png bleibt, eine
Kachel ist gewählt (Auswahlverlauf im Bild).
Kontrast im Bild: Namen der Kacheln (dunkelstes bzw. hellstes Schriftpixel
gegen den Grund der Zeile) und Name der gewählten Kachel (gegen den
Auswahlverlauf an derselben Stelle), Formel wie kontrast.py, ≥ 4,5:1.
Bilder der Galerie alt | mit | Markierung unter nachweis/014/bilder/.

  . ~/projects/querencia-boot-splash/werkzeug/pil-umgebung.sh  # Pillow (PYTHONPATH)
  python3 werkzeug/auswerten-014.py <ziel.tsv>
"""
import os
import sys

from PIL import Image, ImageChops

sys.dont_write_bytecode = True
P = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
N = os.path.join(P, "nachweis")
BILDORDNER = os.path.join(N, "014", "bilder")
SEITEN = ["erscheinungsbild", "hintergrundbild", "symbole", "fensterdesign", "schriften", "panels",
          "tastatur", "maus", "bildschirm", "arbeitsflaechen", "standardanwendungen", "sprache",
          "informationen"]
BILDER = ([f"{n:02d}-{s}.png" for n, s in enumerate(SEITEN, 1)]
          + ["14-erscheinungsbild-unten.png", "15-galerie-mitte.png", "16-galerie-ende.png"])
ERLAUBT = {
    "13-informationen.png": ((150, 180), "nur die Versionsnummer"),
    "12-sprache.png": ((470, 506), "nur die Uhrzeit im Formatbeispiel (jetzt)"),
}
# Fensterausschnitt (wie auswerten-013.py): Seiteninhalt ab x 230, Knopfleiste ab y 602
INHALT_X, LEISTE_Y, KOPF_Y = 230, 602, 98
GALERIE = {
    "02-hintergrundbild.png": (INHALT_X, KOPF_Y, "Seiteninhalt (eingebautes Bild und Galerie)"),
    "15-galerie-mitte.png": (INHALT_X, KOPF_Y, "Seiteninhalt (Galerie gescrollt)"),
    "16-galerie-ende.png": (INHALT_X, KOPF_Y, "Seiteninhalt (Galerie gescrollt)"),
}
ROLLBALKEN = (850, 875)  # Spalte des Rollbalkens im Fensterausschnitt
STATUSZEILE = (615, 645)  # s1: Zeilenband der Statuszeile (Bildschirm ohne Panelzeile)
AUSWAHL = [(0xF6, 0xA1, 0x79), (0xF0, 0x77, 0x46)]  # Verlauf flowboxchild:selected (Thema)
zeilen = []


def pruef(lauf, fall, ok, detail=""):
    zeilen.append((lauf, fall, "PASS" if ok else "FAIL", str(detail).replace("\t", " ")[:400]))


def info(lauf, fall, detail):
    zeilen.append((lauf, fall, "INFO", str(detail).replace("\t", " ")[:400]))


def pixel(im):
    """Alle Pixel (Pillow ab 12: get_flattened_data, vorher getdata)."""
    return getattr(im, "get_flattened_data", im.getdata)()


def roh(lauf, datei):
    p = os.path.join(N, lauf, datei)
    return Image.open(p).convert("RGB") if os.path.exists(p) else None


def bild(lauf, datei):
    """Nur das Fenster samt Titelleiste (Panel-Uhr und Desktop außen vor)."""
    im = roh(lauf, datei)
    if im is None:
        return None
    b = int(lauf.rsplit("-", 1)[1])
    dx, dy = (0, 0) if b == 1920 else (-448, -156)
    return im.crop((520 + dx, 200 + dy, 1400 + dx, 850 + dy))


def lum(c):
    def k(v):
        v /= 255
        return v / 12.92 if v <= 0.03928 else ((v + 0.055) / 1.055) ** 2.4
    return 0.2126 * k(c[0]) + 0.7152 * k(c[1]) + 0.0722 * k(c[2])


def kontrast(a, b):
    x, y = sorted((lum(a), lum(b)), reverse=True)
    return (x + 0.05) / (y + 0.05)


def hexf(c):
    return f"#{c[0]:02x}{c[1]:02x}{c[2]:02x}"


def markierung(a, b, box, name):
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


def vergleichen(lauf_a, lauf_b, datei, gruppe, galerie_erlaubt, bild_name=None):
    a, b = bild(lauf_a, datei), bild(lauf_b, datei)
    fall = f"{datei} {lauf_a} ↔ {lauf_b}"
    if a is None or b is None or a.size != b.size:
        pruef(gruppe, fall, False, "Bild fehlt")
        return None
    box = ImageChops.difference(a, b).getbbox()
    if box is None:
        pruef(gruppe, f"{fall}: pixelgleich", True)
        return None
    if datei in ERLAUBT:
        (oben, unten), grund = ERLAUBT[datei]
        pruef(gruppe, f"{fall}: gleich bis auf {grund}", oben <= box[1] and box[3] <= unten, box)
        return box
    if datei in ("15-galerie-mitte.png", "16-galerie-ende.png") and ROLLBALKEN[0] <= box[0] and box[2] <= ROLLBALKEN[1]:
        stufen = max(ImageChops.difference(a, b).getextrema(), key=lambda e: e[1])[1]
        pruef(gruppe, f"{fall}: nur Farbübergang des Rollbalkens", stufen <= 8, f"Abweichung {box}, höchstens {stufen} Stufen")
        return box
    if galerie_erlaubt and datei in GALERIE:
        x0, y0, grund = GALERIE[datei]
        ok = box[0] >= x0 and box[1] >= y0 and box[3] <= LEISTE_Y
        pruef(gruppe, f"{fall}: nur {grund}", ok, f"Abweichung {box}, erlaubt x ≥ {x0}, {y0} ≤ y < {LEISTE_Y}")
        if bild_name:
            markierung(a, b, box, bild_name)
        return box
    pruef(gruppe, f"{fall}: pixelgleich", False, f"Abweichung {box}")
    return box


def galerie_kontrast(lauf, datei, gruppe):
    """Kontrast der Kachelnamen im Bild. Vorschaubilder sind Zeilen mit
    mindestens einem Lauf ≥ 100 px ohne Grundfarbe; darunter bis zum nächsten
    Vorschaubild die Namenszeilen. Gewählte Kachel: Pixel des Auswahlverlaufs."""
    im = bild(lauf, datei)
    if im is None:
        pruef(gruppe, f"{datei}: Kontrast im Bild", False, "Bild fehlt")
        return
    w, h = im.size
    px = im.load()
    x0, x1 = INHALT_X + 8, w - 20  # ohne Rahmen und Rollbalken
    grund = max(set(px[x, KOPF_Y + 4] for x in range(x0, x1)), key=lambda c: sum(px[x, KOPF_Y + 4] == c for x in range(x0, x1)))

    def ist_auswahl(c):
        return all(min(AUSWAHL[0][i], AUSWAHL[1][i]) - 3 <= c[i] <= max(AUSWAHL[0][i], AUSWAHL[1][i]) + 3 for i in range(3))

    def vorschauzeile(y):
        laeufe, lauf_l = 0, 0
        for x in range(x0, x1):
            c = px[x, y]
            if c != grund and not ist_auswahl(c):
                lauf_l += 1
                if lauf_l == 100:
                    laeufe += 1
            else:
                lauf_l = 0
        return laeufe >= 1

    vz = [vorschauzeile(y) for y in range(KOPF_Y, LEISTE_Y)]
    namen_y = [KOPF_Y + i for i in range(len(vz)) if not vz[i] and any(vz[max(0, i - 45):i])]
    normal, gewaehlt = [], []
    for y in namen_y:
        for x in range(x0, x1):
            c = px[x, y]
            if ist_auswahl(c):
                continue
            # gewählte Kachel: Schrift zwischen Auswahlpixeln derselben Zeile
            links = any(ist_auswahl(px[xx, y]) for xx in range(max(x0, x - 70), x))
            rechts = any(ist_auswahl(px[xx, y]) for xx in range(x + 1, min(x1, x + 70)))
            if links and rechts:
                gewaehlt.append((x, y))
            elif c != grund:
                normal.append((x, y))
    dunkel = lum(grund) < 0.2
    if normal:
        ext = (max if dunkel else min)((px[x, y] for x, y in normal), key=lum)
        k = kontrast(ext, grund)
        info(gruppe, f"{datei}: Kachelnamen im Bild", f"{len(normal)} Schriftpixel, {'hellstes' if dunkel else 'dunkelstes'} {hexf(ext)} auf {hexf(grund)} {k:.2f}:1")
        pruef(gruppe, f"{datei}: Kachelnamen ≥ 4,5:1", k >= 4.5, f"{k:.2f}:1")
    else:
        pruef(gruppe, f"{datei}: Kachelnamen gefunden", False, "keine Namenszeilen")
    if gewaehlt:
        x, y = min(gewaehlt, key=lambda p: lum(px[p[0], p[1]]))
        tx = px[x, y]
        nachbarn = [px[xx, y] for xx in range(max(x0, x - 70), min(x1, x + 70)) if ist_auswahl(px[xx, y])]
        # ungünstigster Verlaufston derselben Zeile (für dunkle Schrift der dunkelste)
        g = max(nachbarn, key=lum) if lum(tx) > 0.5 else min(nachbarn, key=lum)
        k = kontrast(tx, g)
        info(gruppe, f"{datei}: gewählte Kachel im Bild", f"{len(gewaehlt)} Schriftpixel, dunkelstes {hexf(tx)} auf {hexf(g)} {k:.2f}:1")
        pruef(gruppe, f"{datei}: Name der gewählten Kachel ≥ 4,5:1", k >= 4.5, f"{k:.2f}:1")


def desktop_ini_file(lauf):
    p = os.path.join(N, lauf, "desktop.ini.nachher")
    if not os.path.exists(p):
        return None
    for z in open(p, encoding="utf-8"):
        if z.startswith("File="):
            return z[5:].rstrip("\n")
    return None


def bildschirm_gleich(lauf_a, lauf_b, datei, gruppe, band=None):
    a, b = roh(lauf_a, datei), roh(lauf_b, datei)
    if a is None or b is None:
        pruef(gruppe, f"{datei}: Bild fehlt", False)
        return
    a, b = a.crop((0, 32, a.width, a.height)), b.crop((0, 32, b.width, b.height))
    u = ImageChops.difference(a, b).getbbox()
    if band and u is not None:
        pruef(gruppe, f"{datei} {lauf_a} ↔ {lauf_b}: gleich bis auf die Statuszeile (Uhrzeit, Sicherung)", band[0] <= u[1] and u[3] <= band[1], u)
        return
    pruef(gruppe, f"{datei} {lauf_a} ↔ {lauf_b}: pixelgleich (ohne Panelzeile)", u is None, u)


def main():
    for k in ("de-1024", "en-1920"):
        for art in ("hell", "dunkel"):
            a, o, m = f"014-{art}-alt-{k}", f"014-{art}-ohne-{k}", f"014-{art}-mit-{k}"
            for d in BILDER:
                vergleichen(a, o, d, f"d1-{art}-{k}-ohne", False)
                vergleichen(a, m, d, f"d1-{art}-{k}-mit", True, f"{art}-{k}-{d}" if d in GALERIE else None)
            for d in GALERIE:
                x, y = bild(o, d), bild(m, d)
                pruef(f"d1-{art}-{k}-mit", f"{d}: mit ≠ ohne (neue Kacheln sichtbar)",
                      x is not None and y is not None and ImageChops.difference(x, y).getbbox() is not None)
            for d in ("02-hintergrundbild.png", "16-galerie-ende.png"):
                galerie_kontrast(m, d, f"kontrast-{art}-{k}")
                galerie_kontrast(a, d, f"kontrast-{art}-{k}-alt")
    # Standard-Probe
    for lauf in ("014-standard-alt-de-1024", "014-standard-ohne-de-1024", "014-standard-mit-de-1024"):
        v = open(os.path.join(N, lauf, "desktop.ini.vorher"), encoding="utf-8").read() if os.path.exists(os.path.join(N, lauf, "desktop.ini.vorher")) else ""
        pruef("s1", f"{lauf}: vorher File= Erde", "querencia-erde.png" in v)
        f = desktop_ini_file(lauf)
        pruef("s1", f"{lauf}: nachher File= leer (eingebautes Bild des Desktops)", f == "", repr(f))
    for d in ("01-dialog.png", "02-standard.png", "03-uebernommen.png"):
        bildschirm_gleich("014-standard-alt-de-1024", "014-standard-ohne-de-1024", d, "s1",
                          STATUSZEILE if d == "03-uebernommen.png" else None)
    for d in ("01-dialog.png", "02-standard.png"):
        a, b = roh("014-standard-ohne-de-1024", d), roh("014-standard-mit-de-1024", d)
        pruef("s1", f"{d} ohne ≠ mit (Galerie, nach dem Zurücksetzen Vorschau Tiefblau)", a is not None and b is not None and ImageChops.difference(a, b).getbbox() is not None)
    f = desktop_ini_file("014-erkennen-mit-de-1024")
    pruef("e1", "File= bleibt der Verweis auf Glas 4:3", f is not None and f.endswith("/querencia/hintergruende/querencia-glas-4x3.png"), repr(f))
    for d in ("01-hintergrundbild.png", "02-galerie-glas.png"):
        im = bild("014-erkennen-mit-de-1024", d)
        n = 0 if im is None else sum(1 for c in pixel(im) if all(min(AUSWAHL[0][i], AUSWAHL[1][i]) <= c[i] <= max(AUSWAHL[0][i], AUSWAHL[1][i]) for i in range(3)))
        info("e1", f"{d}: Pixel des Auswahlverlaufs", n)
    im = bild("014-erkennen-mit-de-1024", "02-galerie-glas.png")
    pruef("e1", "02-galerie-glas.png: eine Kachel gewählt", im is not None and any(all(min(AUSWAHL[0][i], AUSWAHL[1][i]) <= c[i] <= max(AUSWAHL[0][i], AUSWAHL[1][i]) for i in range(3)) for c in pixel(im.crop((INHALT_X, KOPF_Y, im.width, LEISTE_Y)))))
    for lauf in ("014-dialog-dunkel-mit-de-1024", "014-dialog-hell-mit-en-1920", "014-dialog-dunkel-mit-en-1920"):
        pruef("s2", f"{lauf}: Bildschirmfoto da", roh(lauf, "01-dialog.png") is not None)
    with open(sys.argv[1], "w", encoding="utf-8") as f:
        f.write("lauf\tfall\tergebnis\tdetail\n")
        for z in zeilen:
            f.write("\t".join(z) + "\n")
    p = sum(z[2] == "PASS" for z in zeilen)
    fl = sum(z[2] == "FAIL" for z in zeilen)
    print(f"ERGEBNIS {p} PASS, {fl} FAIL ({sys.argv[1]})")
    for z in zeilen:
        if z[2] in ("FAIL", "INFO") or "nur " in z[1] or z[0].startswith("s"):
            print(*z)
    return 1 if fl else 0


if __name__ == "__main__":
    sys.exit(main())
