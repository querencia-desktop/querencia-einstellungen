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

# Klickpunkte je Oberflächensprache (1920; Englisch: schmalere Seitenleiste,
# einzeiliger Hinweis – gemessen an den a1-en-Bildern von 0.1.0)
PUNKTE = {
    "de": {
        "verz_plus": (1221, 542), "verz_minus": (1196, 542), "rate_plus": (1147, 578),
        "beschleunigung": (797, 515), "radius_plus": (1046, 498), "radius_minus": (1021, 498),
        "schatten": (797, 531), "einblenden": (797, 681), "uebernehmen": UEBERNEHMEN,
    },
    "en": {
        "verz_plus": (1126, 526), "verz_minus": (1101, 526),
        "beschleunigung": (786, 499), "radius_plus": (1039, 482), "radius_minus": (1014, 482),
        "einblenden": (786, 665), "uebernehmen": (1343, 825),
    },
}
# Ohne Benutzerkopie ist der Hinweis oben eine Zeile länger (16 px), bis die
# Seite nach ihrem ersten „Übernehmen“ neu lädt (dann wie mit Kopie).
OHNE_DY = 16
LABWC = '"$XDG_CONFIG_HOME/querencia/labwc"'


def festhalten(s, n):
    """rc.xml und alle Dateien der Benutzerkopie (Name und SHA256) nach Schritt n."""
    s.roh(
        f"run cat {LABWC}/rc.xml > rc-{n}.xml",
        f"run (cd {LABWC} && find . -type f | LC_ALL=C sort | xargs -d '\\n' sha256sum) > dateien-{n}.txt",
        "wait 500",
    )


def uebernehmen(s, n, bild, punkt=UEBERNEHMEN):
    s.marke(f"uebernehmen {n}")
    s.klick(*punkt, 2000)
    s.bild(bild)
    festhalten(s, n)


def rueckstellung(breite, sprache="de", ohne=False):
    """Werkstatt QREL-06 A17.6 in einem Programmlauf: Tastatur 600→610→600, Maus
    Beschleunigung aus/an, Fensterdesign Eckenradius 8→9→8, Arbeitsflächen
    Namen einblenden an/aus, jeweils mit „Übernehmen“."""
    k = PUNKTE[sprache]
    dy = OHNE_DY if ohne else 0

    def p(name, erster):
        x, y = k[name]
        return x, y + (dy if erster else 0)

    def ue(n, bild):
        uebernehmen(s, n, bild, k["uebernehmen"])

    s = Szenario(breite)
    festhalten(s, 0)
    s.seite("tastatur")
    s.bild("01-tastatur.png")
    s.klick(*p("verz_plus", True), 500)
    s.bild("02-verzoegerung-610.png")
    ue(1, "03-uebernommen.png")
    s.klick(*p("verz_minus", False), 500)
    ue(2, "04-zurueck-600.png")
    s.seite("maus")
    s.klick(*p("beschleunigung", True), 500)
    s.bild("05-beschleunigung-aus.png")
    ue(3, "06-uebernommen.png")
    s.klick(*p("beschleunigung", False), 500)
    ue(4, "07-beschleunigung-an.png")
    s.seite("fensterdesign")
    s.klick(*p("radius_plus", True), 500)
    s.bild("08-radius-9.png")
    ue(5, "09-uebernommen.png")
    s.klick(*p("radius_minus", False), 500)
    ue(6, "10-radius-8.png")
    s.seite("arbeitsflaechen")
    s.klick(*p("einblenden", True), 500)
    s.bild("11-einblenden-an.png")
    ue(7, "12-uebernommen.png")
    s.klick(*p("einblenden", False), 500)
    ue(8, "13-einblenden-aus.png")
    name = "r1-rueckstellung" + ("-en" if sprache == "en" else "") + ("-ohne" if ohne else "")
    s.schreiben(name)


def standard(breite):
    """Eigene Werte (Verzögerung 630, Geschwindigkeit 27, Radius 7, Schatten),
    dann je Seite „Standard wiederherstellen“ und „Übernehmen“."""
    s = Szenario(breite)
    festhalten(s, 0)
    s.seite("tastatur")
    k = PUNKTE["de"]
    for _ in range(3):
        s.klick(*k["verz_plus"], 200)
    s.klick(*k["rate_plus"], 200)
    s.klick(*k["rate_plus"], 400)
    s.bild("01-tastatur-eigen.png")
    uebernehmen(s, 1, "02-uebernommen.png")
    s.seite("fensterdesign")
    s.klick(*k["radius_minus"], 200)
    s.klick(*k["schatten"], 400)
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
        rueckstellung(b, "en")
        rueckstellung(b, ohne=True)
        standard(b)
        erscheinungsbild(b)
        sz010.alle_seiten(b)
    print("geschrieben:", sorted(p.name for p in sz010.HIER.glob("*.txt")))
