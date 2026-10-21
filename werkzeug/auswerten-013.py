#!/usr/bin/python3
"""Einstellungen 0.1.3 (QD-089): Auswertung von werkzeug/013-gui-lauf.sh.

Verglichen wird jedes Bild 0.1.2 (alt) ↔ 0.1.3 (neu) im Fensterausschnitt
(wie auswerten-012.py). Jedes abweichende Pixel muss zu einer der gewollten
Stellen gehören, sonst FAIL:
  Kürzel   nur in 07-tastatur.png, nur hell: reiner Deckkraft-Tausch des
           Kürzels, je Kanal neu − Grund = 1,25 · (alt − Grund) (60 % → 75 %,
           gleiche Glyphendeckung), Grund aus den Themenfarben, Toleranz 3
  Auswahl  nur in *bildschirm.png, nur hell: innerhalb des Rechtecks der
           gewählten Ausgabe (alle Pixel der alten Auswahlfarbe, +3 px Rand
           für Kantenglättung, oben +7 px für Panel-Andeutung und Rand); gezählt, wie viele sich als Mischung der
           Auswahlfarbe mit Weiß, Rand, Leiste oder Fläche bei gleicher
           Deckung erklären (Information)
  benannt  13-informationen.png (Versionsnummer), 12-sprache.png (Uhrzeit im
           Formatbeispiel) – Zeilenbänder wie in auswerten-012.py
Dunkel muss jedes Bild pixelgleich sein (bis auf die benannten). Dazu der
Wechsel im Lauf (de 1024): alt ↔ neu je Bild wie oben; neu: Rückwechsel
hell = vorher (Rollbalken-Spalte ausgenommen), Dunkel im Wechsel = Start
unter dunkel, Hell im Wechsel = Start unter hell.
Kontrast im Bild: dunkelstes Kürzelpixel gegen seinen Grund, weiße Schrift
gegen die Auswahlfläche (Formel wie kontrast.py).
Mit Bildern der gewollten Abweichungen unter nachweis/013/bilder/ (alt | neu
| Markierung: rot Kürzel, orange Auswahl, blau unerklärt).

  . ~/projects/querencia-boot-splash/werkzeug/pil-umgebung.sh  # Pillow (PYTHONPATH)
  python3 werkzeug/auswerten-013.py <ziel.tsv>
"""
import os
import sys

from PIL import Image, ImageChops

sys.dont_write_bytecode = True
P = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
N = os.path.join(P, "nachweis")
BILDORDNER = os.path.join(N, "013", "bilder")
SEITEN = ["erscheinungsbild", "hintergrundbild", "symbole", "fensterdesign", "schriften", "panels",
          "tastatur", "maus", "bildschirm", "arbeitsflaechen", "standardanwendungen", "sprache",
          "informationen"]
BILDER = [f"{n:02d}-{s}.png" for n, s in enumerate(SEITEN, 1)] + ["14-erscheinungsbild-unten.png"]
ERLAUBT = {
    "13-informationen.png": ((150, 180), "nur die Versionsnummer"),
    "12-sprache.png": ((470, 506), "nur die Uhrzeit im Formatbeispiel (jetzt)"),
}
# Gründe des Kürzels: Fenstergrund und Ansicht (Querencia-Klassisch hell)
GRUENDE = [(0xF2, 0xF1, 0xF0), (0xFF, 0xFF, 0xFF), (0xF5, 0xF4, 0xF2)]
# 0.1.2: (0.21, 0.52, 0.89) – cairo zeichnet daraus #3585e3
AUSWAHL_ALT = (0x35, 0x85, 0xE3)
AUSWAHL_NEU = (0x2A, 0x6C, 0xC4)
PARTNER = [(0xFF, 0xFF, 0xFF), (0x40, 0x40, 0x40), (0x26, 0x26, 0x26), (0xF7, 0xF7, 0xF7)]
ROLLBALKEN_X = 850
zeilen = []


def pruef(lauf, fall, ok, detail=""):
    zeilen.append((lauf, fall, "PASS" if ok else "FAIL", str(detail).replace("\t", " ")[:400]))


def info(lauf, fall, detail):
    zeilen.append((lauf, fall, "INFO", str(detail).replace("\t", " ")[:400]))


def bild(lauf, datei):
    """Nur das Fenster samt Titelleiste (Panel-Uhr und Desktop außen vor)."""
    p = os.path.join(N, lauf, datei)
    if not os.path.exists(p):
        return None
    b = int(lauf.rsplit("-", 1)[1])
    dx, dy = (0, 0) if b == 1920 else (-448, -156)
    return Image.open(p).convert("RGB").crop((520 + dx, 200 + dy, 1400 + dx, 850 + dy))


def lum(c):
    def k(v):
        v /= 255
        return v / 12.92 if v <= 0.03928 else ((v + 0.055) / 1.055) ** 2.4
    return 0.2126 * k(c[0]) + 0.7152 * k(c[1]) + 0.0722 * k(c[2])


def kontrast(a, b):
    x, y = sorted((lum(a), lum(b)), reverse=True)
    return (x + 0.05) / (y + 0.05)


def kuerzel_tausch(o, n):
    for g in GRUENDE:
        if all(abs((g[c] + 1.25 * (o[c] - g[c])) - n[c]) <= 3 for c in range(3)) and any(o[c] != g[c] for c in range(3)):
            return True
    return False


def auswahl_mischung(o, n):
    """Mischung der Auswahlfarbe mit einem Partner bei gleicher Deckung je
    Kanal (Kantenglättung der Schrift ist je Kanal verschieden)."""
    for p in PARTNER:
        ok = True
        for c in range(3):
            d = p[c] - AUSWAHL_ALT[c]
            a = (o[c] - AUSWAHL_ALT[c]) / d
            if not -0.03 <= a <= 1.03 or abs(AUSWAHL_NEU[c] + a * (p[c] - AUSWAHL_NEU[c]) - n[c]) > 3:
                ok = False
                break
        if ok:
            return True
    return False


def auswahl_rechteck(im):
    """Begrenzung aller Pixel der alten Auswahlfarbe, +3 px, oben +7 px
    (Panel-Andeutung 5 px und Rand liegen über der Fläche)."""
    m = Image.new("1", im.size)
    px, mp = im.load(), m.load()
    for y in range(im.height):
        for x in range(im.width):
            if px[x, y] == AUSWAHL_ALT:
                mp[x, y] = 1
    b = m.getbbox()
    return None if b is None else (b[0] - 3, b[1] - 7, b[2] + 3, b[3] + 3)


def markierung(a, b, kuerz, ausw, schlecht, name):
    os.makedirs(BILDORDNER, exist_ok=True)
    m = b.point(lambda v: v // 3 + 170)
    pm = m.load()
    for farbe, pkt in (((220, 0, 0), kuerz), ((255, 140, 0), ausw), ((0, 0, 230), schlecht)):
        for x, y in pkt:
            pm[x, y] = farbe
    w, h = a.size
    ges = Image.new("RGB", (3 * w + 20, h), (255, 255, 255))
    ges.paste(a, (0, 0))
    ges.paste(b, (w + 10, 0))
    ges.paste(m, (2 * w + 20, 0))
    ges.save(os.path.join(BILDORDNER, name))


def vergleichen(lauf_a, lauf_b, datei, art, gruppe, bild_name=None):
    a, b = bild(lauf_a, datei), bild(lauf_b, datei)
    fall = f"{datei} {lauf_a} ↔ {lauf_b}"
    if a is None or b is None or a.size != b.size:
        pruef(gruppe, fall, False, "Bild fehlt")
        return
    box = ImageChops.difference(a, b).getbbox()
    if box is None:
        pruef(gruppe, f"{fall}: pixelgleich", True)
        return
    if datei in ERLAUBT:
        (oben, unten), grund = ERLAUBT[datei]
        pruef(gruppe, f"{fall}: gleich bis auf {grund}", oben <= box[1] and box[3] <= unten, box)
        return
    pa, pb = a.load(), b.load()
    rechteck = auswahl_rechteck(a) if art == "hell" and datei.endswith("bildschirm.png") else None
    kuerz, ausw, ausw_misch, schlecht = [], [], 0, []
    for y in range(box[1], box[3]):
        for x in range(box[0], box[2]):
            o, n = pa[x, y], pb[x, y]
            if o == n:
                continue
            if art == "hell" and datei == "07-tastatur.png" and kuerzel_tausch(o, n):
                kuerz.append((x, y))
            elif rechteck and rechteck[0] <= x < rechteck[2] and rechteck[1] <= y < rechteck[3]:
                ausw.append((x, y))
                ausw_misch += o == AUSWAHL_ALT and n == AUSWAHL_NEU or auswahl_mischung(o, n)
            else:
                schlecht.append((x, y))
    detail = f"Abweichung {box}: {len(kuerz)} Kürzel (60 → 75 %), {len(ausw)} Auswahl"
    if ausw:
        detail += f" (davon {ausw_misch} als Mischung erklärt, Rechteck {rechteck})"
    detail += f", {len(schlecht)} unerklärt"
    if schlecht:
        xs = [p[0] for p in schlecht]
        ys = [p[1] for p in schlecht]
        detail += f" in {min(xs)}–{max(xs)} × {min(ys)}–{max(ys)}"
    gewollt = "Kürzel" if kuerz else "Auswahl" if ausw else "?"
    pruef(gruppe, f"{fall}: nur gewollte Abweichung ({gewollt})", not schlecht and (kuerz or ausw), detail)
    if bild_name and (kuerz or ausw or schlecht):
        markierung(a, b, kuerz, ausw, schlecht, bild_name)
    if kuerz:
        # Kontrast im Bild: dunkelstes Kürzelpixel neu/alt gegen seinen Grund
        dn = min((pb[x, y] for x, y in kuerz), key=sum)
        da = min((pa[x, y] for x, y in kuerz), key=sum)
        grund = max((pb[x, kuerz[0][1] - 4] for x in range(box[0], box[2])), key=sum)
        info(gruppe, f"{datei}: Kontrast Kürzel im Bild", f"neu #{dn[0]:02x}{dn[1]:02x}{dn[2]:02x} auf "
             f"#{grund[0]:02x}{grund[1]:02x}{grund[2]:02x} {kontrast(dn, grund):.2f}:1, alt "
             f"#{da[0]:02x}{da[1]:02x}{da[2]:02x} {kontrast(da, grund):.2f}:1")
        pruef(gruppe, f"{datei}: Kürzel im Bild ≥ 4,5:1", kontrast(dn, grund) >= 4.5, f"{kontrast(dn, grund):.2f}:1")
    if ausw:
        flaeche = [pb[x, y] for x, y in ausw if pb[x, y] == AUSWAHL_NEU]
        weiss = [pb[x, y] for x, y in ausw if min(pb[x, y]) >= 250]
        k = kontrast((255, 255, 255), AUSWAHL_NEU)
        info(gruppe, f"{datei}: Kontrast Auswahl im Bild",
             f"Fläche #2a6cc4 {len(flaeche)} px, weiße Schrift (je Kanal ≥ 250) {len(weiss)} px, {k:.2f}:1 (alt {kontrast((255, 255, 255), AUSWAHL_ALT):.2f}:1)")
        pruef(gruppe, f"{datei}: neue Auswahlfarbe und weiße Schrift im Bild, ≥ 4,5:1", bool(flaeche) and bool(weiss) and k >= 4.5,
              f"{len(flaeche)} / {len(weiss)} px")


def gleich(a, b, gruppe, fall, rollbalken=False):
    x, y = bild(*a), bild(*b)
    if x is None or y is None:
        pruef(gruppe, fall, False, "Bild fehlt")
        return
    if rollbalken:
        x, y = x.crop((0, 0, ROLLBALKEN_X, x.height)), y.crop((0, 0, ROLLBALKEN_X, y.height))
    u = ImageChops.difference(x, y).getbbox()
    pruef(gruppe, fall, u is None, u)


def main():
    for k in ("de-1024", "en-1920"):
        for art in ("hell", "dunkel"):
            a, n = f"013-{art}-alt-{k}", f"013-{art}-neu-{k}"
            for d in BILDER:
                name = f"{art}-{k}-{d}" if art == "hell" and d in ("07-tastatur.png", "09-bildschirm.png") else None
                vergleichen(a, n, d, art, f"d1-{art}-{k}", name)
    # Wechsel im Lauf nur in de 1024 (gekürzte Matrix); die Bildschirm-Seite zeigt die Auswahl
    wa, wn = "013-wechsel-alt-de-1024", "013-wechsel-neu-de-1024"
    teile = ["erscheinungsbild", "erscheinungsbild-unten", "bildschirm", "hintergrundbild"]
    for i, wort in ((1, "hell"), (2, "dunkel"), (3, "hell")):
        for t in teile:
            d = f"{i}-{wort}-{t}.png"
            vergleichen(wa, wn, d, wort, "w1-de-1024", f"wechsel-{d}" if t == "bildschirm" and wort == "hell" else None)
    for t in teile:
        gleich((wn, f"1-hell-{t}.png"), (wn, f"3-hell-{t}.png"), "w1-neu",
               f"{t}: hell nach dem Rückwechsel = vorher (Rollbalken-Spalte ausgenommen)", rollbalken=True)
    start = {"erscheinungsbild": "01-erscheinungsbild.png", "erscheinungsbild-unten": "14-erscheinungsbild-unten.png",
             "bildschirm": "09-bildschirm.png", "hintergrundbild": "02-hintergrundbild.png"}
    for t, d in start.items():
        if not t.startswith("erscheinungsbild"):
            gleich((wn, f"2-dunkel-{t}.png"), ("013-dunkel-neu-de-1024", d), "w1-neu", f"{t}: Dunkel im Wechsel = Start unter dunkel")
            gleich((wn, f"1-hell-{t}.png"), ("013-hell-neu-de-1024", d), "w1-neu", f"{t}: Hell im Wechsel = Start unter hell")
    with open(sys.argv[1], "w", encoding="utf-8") as f:
        f.write("lauf\tfall\tergebnis\tdetail\n")
        for z in zeilen:
            f.write("\t".join(z) + "\n")
    p = sum(z[2] == "PASS" for z in zeilen)
    fl = sum(z[2] == "FAIL" for z in zeilen)
    print(f"ERGEBNIS {p} PASS, {fl} FAIL ({sys.argv[1]})")
    for z in zeilen:
        if z[2] in ("FAIL", "INFO") or "gewollte" in z[1]:
            print(*z)
    return 1 if fl else 0


if __name__ == "__main__":
    sys.exit(main())
