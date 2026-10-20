#!/usr/bin/env python3
"""Einstellungen 0.1.1: erzeugt die Eingabeszenarien unter szenarien/011/.

Koordinaten wie werkzeug/010-szenarien.py (1920×1080, Kontrollzentrum
860×600 mittig, 1024×768 um (−448, −156) verschoben; gemessen an den
Bildern von 0.1.0, Schrift DejaVu Sans 10).
"""
import importlib.util
import pathlib

W = pathlib.Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location("sz010", W / "010-szenarien.py")
sz010 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(sz010)
sz010.HIER = W.parent / "szenarien" / "011"
Szenario, UEBERNEHMEN, STANDARD = sz010.Szenario, sz010.UEBERNEHMEN, sz010.STANDARD

VERZ_PLUS, VERZ_MINUS = (1221, 542), (1196, 542)  # Tastatur: Verzögerung ± 10 ms
RATE_PLUS = (1147, 578)  # Tastatur: Geschwindigkeit + 1
BESCHLEUNIGUNG = (797, 515)  # Maus
RADIUS_PLUS, RADIUS_MINUS = (1046, 498), (1021, 498)  # Fensterdesign: Eckenradius ± 1
SCHATTEN = (797, 531)  # Fensterdesign
EINBLENDEN = (797, 681)  # Arbeitsflächen: Namen kurz einblenden
LABWC = '"$XDG_CONFIG_HOME/querencia/labwc"'


def festhalten(s, n):
    """rc.xml und alle Dateien der Benutzerkopie (Name und SHA256) nach Schritt n."""
    s.roh(
        f"run cat {LABWC}/rc.xml > rc-{n}.xml",
        f"run (cd {LABWC} && find . -type f | LC_ALL=C sort | xargs -d '\\n' sha256sum) > dateien-{n}.txt",
        "wait 500",
    )


def uebernehmen(s, n, bild):
    s.marke(f"uebernehmen {n}")
    s.klick(*UEBERNEHMEN, 2000)
    s.bild(bild)
    festhalten(s, n)


def rueckstellung(breite):
    """Werkstatt QREL-06 A17.6 in einem Programmlauf: Tastatur 600→610→600, Maus
    Beschleunigung aus/an, Fensterdesign Eckenradius 8→9→8, Arbeitsflächen
    Namen einblenden an/aus, jeweils mit „Übernehmen“."""
    s = Szenario(breite)
    festhalten(s, 0)
    s.seite("tastatur")
    s.bild("01-tastatur.png")
    s.klick(*VERZ_PLUS, 500)
    s.bild("02-verzoegerung-610.png")
    uebernehmen(s, 1, "03-uebernommen.png")
    s.klick(*VERZ_MINUS, 500)
    uebernehmen(s, 2, "04-zurueck-600.png")
    s.seite("maus")
    s.klick(*BESCHLEUNIGUNG, 500)
    s.bild("05-beschleunigung-aus.png")
    uebernehmen(s, 3, "06-uebernommen.png")
    s.klick(*BESCHLEUNIGUNG, 500)
    uebernehmen(s, 4, "07-beschleunigung-an.png")
    s.seite("fensterdesign")
    s.klick(*RADIUS_PLUS, 500)
    s.bild("08-radius-9.png")
    uebernehmen(s, 5, "09-uebernommen.png")
    s.klick(*RADIUS_MINUS, 500)
    uebernehmen(s, 6, "10-radius-8.png")
    s.seite("arbeitsflaechen")
    s.klick(*EINBLENDEN, 500)
    s.bild("11-einblenden-an.png")
    uebernehmen(s, 7, "12-uebernommen.png")
    s.klick(*EINBLENDEN, 500)
    uebernehmen(s, 8, "13-einblenden-aus.png")
    s.schreiben("r1-rueckstellung")


def standard(breite):
    """Eigene Werte (Verzögerung 630, Geschwindigkeit 27, Radius 7, Schatten),
    dann je Seite „Standard wiederherstellen“ und „Übernehmen“."""
    s = Szenario(breite)
    festhalten(s, 0)
    s.seite("tastatur")
    for _ in range(3):
        s.klick(*VERZ_PLUS, 200)
    s.klick(*RATE_PLUS, 200)
    s.klick(*RATE_PLUS, 400)
    s.bild("01-tastatur-eigen.png")
    uebernehmen(s, 1, "02-uebernommen.png")
    s.seite("fensterdesign")
    s.klick(*RADIUS_MINUS, 200)
    s.klick(*SCHATTEN, 400)
    s.bild("03-fensterdesign-eigen.png")
    uebernehmen(s, 2, "04-uebernommen.png")
    s.seite("tastatur")
    s.klick(*STANDARD, 600)
    s.bild("05-tastatur-standard.png")
    uebernehmen(s, 3, "06-uebernommen.png")
    s.seite("fensterdesign")
    s.klick(*STANDARD, 600)
    s.bild("07-fensterdesign-standard.png")
    uebernehmen(s, 4, "08-uebernommen.png")
    s.schreiben("r2-standard")


def erscheinungsbild(breite):
    """Hinweis zu gsettings/GNOME (KOEX-1 K4) unten auf der Seite."""
    s = Szenario(breite)
    s.seite("erscheinungsbild")
    s.bild("01-oben.png")
    s.roh(f"move {1100 + s.dx} {560 + s.dy}", "scroll 15", "wait 800")
    s.bild("02-unten-hinweis.png")
    s.schreiben("eb1-erscheinungsbild")


if __name__ == "__main__":
    for b in (1920, 1024):
        rueckstellung(b)
        standard(b)
        erscheinungsbild(b)
        sz010.alle_seiten(b)
    print("geschrieben:", sorted(p.name for p in sz010.HIER.glob("*.txt")))
