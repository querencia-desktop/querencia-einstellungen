#!/usr/bin/env python3
"""Wertet die isolierten QE-01-Läufe aus (nachweis/r1…r8): Pixel in Vorschau und
auf dem echten Desktop HMP-10a, desktop.ini vorher/nachher, Desktop-Log,
Symbolpositionen. Ergebnis: nachweis/pruefungen.tsv und Ausgabe PASS/FAIL."""
import os
import sys
from PIL import Image

N = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "nachweis")
erg = []


def pruefe(nr, test, ok, beleg):
    erg.append((nr, test, "PASS" if ok else "FAIL", beleg))
    print(f"{'PASS' if ok else 'FAIL'}  {nr:4s} {test}: {beleg}")


def bild(lauf, name):
    return Image.open(os.path.join(N, lauf, name)).convert("RGB")


def lesen(lauf, name):
    p = os.path.join(N, lauf, name)
    return open(p, encoding="utf-8").read() if os.path.exists(p) else None


def hexrgb(h):
    h = h.lstrip("#")
    return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))


def nah(p, q, tol=6):
    return all(abs(a - b) <= tol for a, b in zip(p, q))


ROT, GRUEN, BLAU, GELB = (220, 40, 40), (40, 160, 60), (40, 80, 220), (240, 200, 40)


def vorschau_rechteck(im, x0=560, x1=1360, y0=140, y1=560):
    """Findet die Vorschau über ihren 1-px-Rahmen (64,64,64)."""
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
    # zwei Pixel nach innen: der Rahmen ist an Bruchteil-Kanten kantengeglättet
    return left + 2, top + 2, right - 2, bottom - 2


def vorschau_punkte(im):
    r = vorschau_rechteck(im)
    if not r:
        return None, None
    l, t, rr, b = r
    w, h = rr - l, b - t
    punkte = {(fx, fy): im.getpixel((int(l + fx * w), int(t + fy * h)))
              for fx in (0.03, 0.25, 0.5, 0.75, 0.97) for fy in (0.04, 0.25, 0.5, 0.75, 0.96)}
    return r, punkte


def einfarbig(punkte, farbe):
    return punkte is not None and all(p == farbe for p in punkte.values())


def uebernehmen_aktiv(im, x=1244, y=906):
    # GTK-Akzentblau des suggested-action-Knopfs
    px = im.load()
    blau = sum(1 for dx in range(-40, 41, 4) for dy in range(-8, 9, 4)
               if (lambda p: p[2] > 180 and p[0] < 90)(px[x + dx, y + dy]))
    return blau > 20


def panels_sichtbar(im):
    w, h = im.size
    return min(im.getpixel((w // 2, 15))) > 200 and min(im.getpixel((w // 2, h - 17))) > 200


def kann(lauf, was):
    """Fähigkeit des Desktops in diesem Lauf laut Dialog (logs/faehigkeiten.log)."""
    return f"{was}=true" in (lesen(lauf, "logs/faehigkeiten.log") or "")


def freigabe_erwartet(lauf, was, im):
    """„Übernehmen“ muss genau dann aktiv sein, wenn der Desktop es kann."""
    erwartet = kann(lauf, was)
    aktiv = uebernehmen_aktiv(im)
    return aktiv == erwartet, f"Desktop kann {was}: {erwartet}; Knopf aktiv: {aktiv}"


def symbole_gleich(lauf):
    a, b = lesen(lauf, "symbole-start"), lesen(lauf, "symbole-ende")
    return a is not None and a == b and "fehlt" not in a, f"{len((a or '').splitlines())} Zeilen, start==ende: {a == b}"


STANDARD_TEXT = "[Background]\nFile=\nMode=zoom\nColor=#1c4e96\n"
MUSTER = "~/projects/querencia-werkstatt-w6-einstellungen/nachweis/testbilder/muster-4zu3"

def main():
    # ---------------------------------------------------------------- R1 Bild
    L = "r1-bild"
    r, p = vorschau_punkte(bild(L, "02-bild-ausfuellen.png"))
    pruefe("1.1", "Bild auswählen (PNG über Dateidialog), Vorschau „Bildschirm ausfüllen“",
           p and nah(p[(0.03, 0.25)], ROT) and nah(p[(0.97, 0.75)], GELB) and nah(p[(0.03, 0.75)], BLAU),
           f"Vorschau {r}, links oben rot, rechts unten gelb, keine Ränder")
    r, p = vorschau_punkte(bild(L, "03-einpassen.png"))
    pruefe("1.2", "Einpassen: ganzes Bild, Ränder in der Farbe, Proportion 4:3",
           p and p[(0.03, 0.5)] == hexrgb("#1c4e96") and p[(0.97, 0.5)] == hexrgb("#1c4e96") and nah(p[(0.25, 0.25)], ROT),
           f"Rand {p[(0.03, 0.5)] if p else None}, Bild links oben {p[(0.25, 0.25)] if p else None}")
    im = bild(L, "04-zentrieren.png")
    r, p = vorschau_punkte(im)
    pruefe("1.3", "Zentrieren: Originalgröße in der Mitte (Vorschau)",
           p and p[(0.03, 0.04)] == hexrgb("#1c4e96") and p[(0.97, 0.96)] == hexrgb("#1c4e96") and nah(p[(0.5, 0.5)], (255, 255, 255)),
           "Ecken in der Farbe, Mitte weiß")
    ok, beleg = freigabe_erwartet(L, "zentrieren", im)
    pruefe("1.4", "Zentrieren: „Übernehmen“ genau dann aktiv, wenn der Desktop es kann", ok, beleg)
    r, p = vorschau_punkte(bild(L, "05-strecken.png"))
    pruefe("1.5", "Strecken: füllt ohne Ränder (Vorschau)",
           p and nah(p[(0.03, 0.04)], ROT) and nah(p[(0.97, 0.96)], GELB), "Ecken rot/gelb")
    im = bild(L, "06-uebernommen.png")
    ini = lesen(L, "desktop.ini.nachher")
    pruefe("1.6", "Übernehmen schreibt desktop.ini (Format des Desktops)",
           ini == f"[Background]\nFile={MUSTER}.png\nMode=scaled\nColor=#1c4e96\n", repr(ini))
    pruefe("1.7", "Echter Desktop HMP-10a zeigt das Bild sofort eingepasst",
           im.getpixel((150, 900)) == hexrgb("#1c4e96") and nah(im.getpixel((400, 300)), ROT) and nah(im.getpixel((1500, 900)), GELB),
           f"Rand {im.getpixel((150, 900))}, (400,300) {im.getpixel((400, 300))}, (1500,900) {im.getpixel((1500, 900))}")
    log = lesen(L, "logs/desktop.log") or ""
    pruefe("1.8", "Desktop-Log: neue Einstellung geladen", f"{MUSTER}.png, Modus scaled" in log, "„Hintergrund …muster-4zu3.png, Modus scaled“")
    ok, beleg = symbole_gleich(L)
    pruefe("1.9", "Desktop-Symbolpositionen unverändert", ok, beleg)
    pruefe("1.10", "Panels sichtbar", panels_sichtbar(im), "oben und unten hell")
    pruefe("1.11", "Keine fremde desktop.ini-Sicherung (Datei war neu)", "desktop.ini.vorher" not in (lesen(L, "konfigordner-nachher.txt") or ""),
           (lesen(L, "konfigordner-nachher.txt") or "").replace("\n", " "))

    # ---------------------------------------------------------------- R2 Farbe, HEX, Abbrechen, Persistenz
    L = "r2-farbe"
    im = bild(L, "01-desktop-nach-neustart.png")
    pruefe("2.1", "Persistenz: neue Sitzung zeigt das übernommene Bild",
           im.getpixel((150, 900)) == hexrgb("#1c4e96") and nah(im.getpixel((400, 300)), ROT), "wie 1.7")
    r, p = vorschau_punkte(bild(L, "02-dialog-geladen.png"))
    pruefe("2.2", "Dialog lädt die gespeicherte Einstellung (Bild, Einpassen)",
           p and p[(0.03, 0.5)] == hexrgb("#1c4e96") and nah(p[(0.25, 0.25)], ROT), "Vorschau wie 1.2")
    for nr, datei, farbe, name in [("2.3", "03-farbe-standard.png", "#1c4e96", "Wechsel Bild → Farbe"),
                                    ("2.4", "04-tango.png", "#204a87", "Palette #204A87"),
                                    ("2.5", "05-anthrazit.png", "#242424", "Palette #242424"),
                                    ("2.6", "06-orange.png", "#f07746", "Palette #F07746")]:
        r, p = vorschau_punkte(bild(L, datei))
        pruefe(nr, f"{name}: Vorschau vollständig einfarbig, keine Bildreste", einfarbig(p, hexrgb(farbe)),
               f"25 Messpunkte = {farbe}" if einfarbig(p, hexrgb(farbe)) else str(sorted(set(p.values())) if p else None)[:120])
    im = bild(L, "06-orange.png")
    ok, beleg = freigabe_erwartet(L, "farbe", im)
    pruefe("2.7", "Farbmodus: „Übernehmen“ genau dann aktiv, wenn der Desktop es kann", ok, beleg)
    r, p = vorschau_punkte(bild(L, "07-hex-ungueltig.png"))
    pruefe("2.8", "Ungültige HEX-Eingabe „#12zz“ ändert nichts", einfarbig(p, hexrgb("#f07746")), "Vorschau bleibt #f07746")
    r, p = vorschau_punkte(bild(L, "08-hex-gueltig.png"))
    pruefe("2.9", "HEX-Eingabe „#204A87“ (Großbuchstaben)", einfarbig(p, hexrgb("#204a87")), "Vorschau #204a87")
    im = bild(L, "09-zurueck-zum-bild.png")
    r, p = vorschau_punkte(im)
    pruefe("2.10", "Wechsel Farbe → Bild: Bild wieder da, Ränder in der neuen Farbe",
           p and p[(0.03, 0.5)] == hexrgb("#204a87") and nah(p[(0.25, 0.25)], ROT) and uebernehmen_aktiv(im), "Rand #204a87, „Übernehmen“ aktiv")
    im = bild(L, "10-abgebrochen.png")
    pruefe("2.11", "Abbrechen: desktop.ini unverändert", lesen(L, "desktop.ini.nachher") == lesen(L, "desktop.ini.vorher"), "byte-gleich")
    pruefe("2.12", "Abbrechen: Fenster zu, Desktop unverändert",
           nah(im.getpixel((700, 300)), ROT) and im.getpixel((150, 900)) == hexrgb("#1c4e96"), "Desktopbild sichtbar wie vorher")
    ok, beleg = symbole_gleich(L)
    pruefe("2.13", "Desktop-Symbolpositionen unverändert", ok, beleg)

    # ---------------------------------------------------------------- R3 Standard
    L = "r3-standard"
    r, p = vorschau_punkte(bild(L, "01-standard-vorschau.png"))
    pruefe("3.1", "Standard wiederherstellen: Vorschau zeigt „Querencia Blau“",
           p and not any(nah(v, ROT, 30) or nah(v, GELB, 30) for v in p.values()) and all(v[2] > v[0] for v in p.values()), "kein Muster, blau")
    pruefe("3.2", "Standard erst nach „Übernehmen“: desktop.ini = Standardtext", lesen(L, "desktop.ini.nachher") == STANDARD_TEXT, repr(lesen(L, "desktop.ini.nachher")))
    im = bild(L, "02-standard-uebernommen.png")
    pruefe("3.3", "Echter Desktop zeigt wieder das eingebaute Bild", not nah(im.getpixel((400, 300)), ROT, 40) and im.getpixel((400, 900))[2] > 120, f"(400,300) {im.getpixel((400, 300))}")
    k = lesen(L, "konfigordner-nachher.txt") or ""
    pruefe("3.4", "Vorherige Einstellung als desktop.ini.vorher-<Zeit> gesichert", "desktop.ini.vorher-" in k, k.replace("\n", " "))

    # ---------------------------------------------------------------- R4/R5 fehlende und ungültige Bilddatei
    for L, nr, name, muster in [("r4-fehlend", "4", "Fehlende Bilddatei", "berg.png nicht lesbar"),
                                ("r5-kaputt", "5", "Ungültige Bilddatei", "kaputt.png nicht lesbar")]:
        log = lesen(L, "logs/desktop.log") or ""
        pruefe(f"{nr}.1", f"{name}: Desktop meldet und fällt auf das eingebaute Bild zurück", muster in log and "Hintergrund" in log, muster)
        pruefe(f"{nr}.2", f"{name}: Desktop läuft weiter, Symbole unverändert", symbole_gleich(L)[0], symbole_gleich(L)[1])
        pruefe(f"{nr}.3", f"{name}: Dialog öffnet, desktop.ini unverändert", os.path.exists(os.path.join(N, L, "02-dialog.png")) and
               lesen(L, "desktop.ini.nachher") == lesen(L, "desktop.ini.vorher"), "Hinweis im Dialog (Bild 02-dialog.png)")

    # ---------------------------------------------------------------- R6 1024x768
    L = "r6-1024"
    im = bild(L, "01-dialog-1024.png")
    px = im.load()
    blau = [(x, y) for y in range(400, 768) for x in range(0, 1024, 2) if px[x, y][2] > 200 and px[x, y][0] < 80 and px[x, y][1] < 160]
    pruefe("6.1", "1024×768: Dialog passt zwischen die Panels, „Übernehmen“ sichtbar",
           bool(blau) and max(y for _, y in blau) < 768 - 32 and panels_sichtbar(im), f"Knopf bis y={max(y for _, y in blau) if blau else None}")
    pruefe("6.2", "1024×768: Kontrollzentrum öffnet", os.path.exists(os.path.join(N, L, "02-kontrollzentrum-1024.png")), "Bild 02")

    # ---------------------------------------------------------------- R7 Kacheln, WebP
    L = "r7-kacheln-webp"
    r, p = vorschau_punkte(bild(L, "01-kacheln.png"))
    werte = set(p.values()) if p else set()
    pruefe("7.1", "Kacheln: Vorschau wiederholt das Bild", any(nah(v, (240, 119, 70), 10) for v in werte) and any(v == (30, 30, 30) for v in werte),
           f"{len(werte)} Farben in 25 Messpunkten")
    ok, beleg = freigabe_erwartet(L, "kacheln", bild(L, "01-kacheln.png"))
    pruefe("7.2", "Kacheln: „Übernehmen“ genau dann aktiv, wenn der Desktop es kann", ok, beleg)
    im = bild(L, "03-webp-uebernommen.png")
    ini = lesen(L, "desktop.ini.nachher") or ""
    pruefe("7.3", "WebP: übernommen und vom echten Desktop gezeichnet",
           f"File={MUSTER}.webp" in ini and "Mode=zoom" in ini and nah(im.getpixel((400, 300)), ROT, 12) and nah(im.getpixel((1500, 900)), GELB, 12),
           f"(400,300) {im.getpixel((400, 300))}")

    # ---------------------------------------------------------------- R8 Konflikt
    L = "r8-konflikt"
    ini = lesen(L, "desktop.ini.nachher")
    pruefe("8.1", "Fremd geänderte desktop.ini wird nicht unbemerkt überschrieben",
           ini == "[Background]\nFile=\nMode=stretched\nColor=#000000\n", repr(ini))

    with open(os.path.join(N, "pruefungen.tsv"), "w", encoding="utf-8") as f:
        f.write("Nr\tPrüfung\tErgebnis\tBeleg\n")
        for e in erg:
            f.write("\t".join(e) + "\n")
    fails = [e for e in erg if e[2] != "PASS"]
    print(f"\n{len(erg) - len(fails)}/{len(erg)} PASS")
    sys.exit(1 if fails else 0)


if __name__ == "__main__":
    main()
