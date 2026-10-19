#!/usr/bin/env python3
"""Nachprüfung QE-01 mit dem erweiterten Desktop (W1, A1–A3): Läufe r9–r12.
Prüft Farbe, Zentrieren und Kacheln auf dem ECHTEN Desktop sowie den
Menüpunkt. Ergebnis: nachweis/nachpruefung.tsv (bzw. Dateiname aus argv[1])."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import auswerten as a  # noqa: E402  (nutzt dessen Hilfsfunktionen; dessen Hauptteil läuft dabei mit)

erg = []


def pruefe(nr, test, ok, beleg):
    erg.append((nr, test, "PASS" if ok else "FAIL", beleg))
    print(f"{'PASS' if ok else 'FAIL'}  {nr:5s} {test}: {beleg}")


ORANGE = a.hexrgb("#f07746")
STANDARD = a.hexrgb("#1c4e96")
DUNKEL = (30, 30, 30)
# Messpunkte auf dem Desktop: weg von Symbolspalte, Panels; Dialog ist dann geschlossen
RASTER = [(x, y) for x in (200, 500, 900, 1300, 1700) for y in (100, 300, 500, 700, 900, 1000)]


def desktop_einfarbig(im, farbe):
    abw = [(p, im.getpixel(p)) for p in RASTER if im.getpixel(p) != farbe]
    return not abw, abw[:3]


# ---------------------------------------------------------------- r9 Farbe
L = "r9-farbe-uebernehmen"
ini = a.lesen(L, "desktop.ini.nachher") or ""
f = (a.lesen(L, "logs/faehigkeiten.log") or "").strip()
pruefe("9.0", "Dialog erkennt die Fähigkeiten des neuen Desktops", "farbe=true" in f, f.replace("\n", " ")[-60:])
im = a.bild(L, "02-dialog-uebernommen.png")
pruefe("9.1", "Farbmodus: „Übernehmen“ freigegeben", a.uebernehmen_aktiv(a.bild(L, "01-dialog-farbe.png")), "Knopf mit Akzentfarbe")
im = a.bild(L, "03-desktop-farbe.png")
ok, abw = desktop_einfarbig(im, ORANGE)
pruefe("9.2", "Echter Desktop vollständig einfarbig #F07746, keine Bildreste", ok, f"30 Messpunkte; Abweichungen {abw}")
pruefe("9.3", "Panels und Symbole bleiben sichtbar", a.panels_sichtbar(im) and im.getpixel((54, 65)) != ORANGE, "Panels hell, Rechner-Symbol gezeichnet")
r, p = a.vorschau_punkte(a.bild(L, "04-dialog-neu-geladen.png"))
pruefe("9.4", "Dialog lädt „Einfarbig“ wieder (Persistenz der Datei)", a.einfarbig(p, ORANGE), "Vorschau einfarbig orange")
im = a.bild(L, "05-desktop-wieder-bild.png")
pruefe("9.5", "Wechsel Farbe → Bild wirkt auf dem Desktop", not desktop_einfarbig(im, ORANGE)[0] and im.getpixel((900, 500))[2] > 100,
       f"(900,500) {im.getpixel((900, 500))}")
pruefe("9.6", "Datei am Ende: Bild ohne Type (byte-kompatibel)", "Type=" not in ini and "Color=#f07746" in ini, repr(ini))
ok, beleg = a.symbole_gleich(L)
pruefe("9.7", "Symbolpositionen unverändert", ok, beleg)

# ---------------------------------------------------------------- r10 Zentrieren
L = "r10-zentrieren"
ini = a.lesen(L, "desktop.ini.nachher") or ""
pruefe("10.1", "„Übernehmen“ freigegeben, desktop.ini Mode=centered", "Mode=centered" in ini, repr(ini))
im = a.bild(L, "03-desktop.png")
pruefe("10.2", "Echter Desktop: 800×600 unskaliert in der Mitte, Rest in der Farbe",
       im.getpixel((300, 900)) == STANDARD and im.getpixel((1700, 200)) == STANDARD and a.nah(im.getpixel((580, 260)), a.ROT)
       and a.nah(im.getpixel((1340, 820)), a.GELB) and a.nah(im.getpixel((960, 540)), (255, 255, 255)),
       f"(300,900) {im.getpixel((300, 900))}, (580,260) {im.getpixel((580, 260))}, (1340,820) {im.getpixel((1340, 820))}")
pruefe("10.3", "Bildkante genau bei x=560 (keine Skalierung)", im.getpixel((555, 400)) == STANDARD and a.nah(im.getpixel((565, 400)), a.ROT),
       f"(555,400) {im.getpixel((555, 400))}, (565,400) {im.getpixel((565, 400))}")
ok, beleg = a.symbole_gleich(L)
pruefe("10.4", "Symbolpositionen unverändert", ok, beleg)

# ---------------------------------------------------------------- r11 Kacheln
L = "r11-kacheln"
ini = a.lesen(L, "desktop.ini.nachher") or ""
pruefe("11.1", "„Übernehmen“ freigegeben, desktop.ini Mode=tiled", "Mode=tiled" in ini, repr(ini))
im = a.bild(L, "03-desktop.png")
soll = {(1640, 510): ORANGE, (1720, 510): DUNKEL, (1640, 570): DUNKEL, (1720, 570): ORANGE,
        (840, 990): ORANGE, (920, 990): DUNKEL}
ist = {pt: im.getpixel(pt) for pt in soll}
pruefe("11.2", "Echter Desktop: 160×120-Kachel ab links oben wiederholt",
       all(a.nah(ist[pt], c, 8) for pt, c in soll.items()), str(ist))
ok, beleg = a.symbole_gleich(L)
pruefe("11.3", "Symbolpositionen unverändert", ok, beleg)

# ---------------------------------------------------------------- r12 Menüpunkt
L = "r12-menue"
im = a.bild(L, "01-nach-menuepunkt.png")
pruefe("12.1", "Desktop-Menüpunkt öffnet „Querencia – Hintergrund“", a.vorschau_rechteck(im) is not None,
       f"Vorschau-Rahmen {a.vorschau_rechteck(im)}")

ziel = os.path.join(a.N, sys.argv[1] if len(sys.argv) > 1 else "nachpruefung.tsv")
with open(ziel, "w", encoding="utf-8") as fh:
    fh.write("Nr\tPrüfung\tErgebnis\tBeleg\n")
    for e in erg:
        fh.write("\t".join(e) + "\n")
fails = [e for e in erg if e[2] != "PASS"]
print(f"\n{len(erg) - len(fails)}/{len(erg)} PASS → {ziel}")
sys.exit(1 if fails else 0)
