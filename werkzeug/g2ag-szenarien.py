#!/usr/bin/python3 -I
"""Erzeugt die GUI-Szenarien von G2-AG (szenarien/labwc/*.txt) aus einer
Koordinatentabelle für 1920×1080 (Schrift DejaVu Sans 10, Kontrollzentrum
860×600 mittig). 1024×768 = 1920er-Punkt − (448, 156) (gleiche Fensterlage,
gemessen 20.10. 01:16)."""
import os

Z = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "szenarien", "labwc")
P = {  # 1920×1080
    "fensterdesign": (600, 434), "tastatur": (600, 548), "maus": (600, 586), "arbeitsflaechen": (600, 662),
    "uebernehmen": (1315, 823), "standard": (891, 823),
    "aendern": (1316, 428), "verz_minus": (1205, 548), "rate_plus": (1166, 588), "numlock": (797, 674),
    "dlg_erste_zeile": (900, 367), "dlg_auswaehlen": (1152, 798),
    "natuerlich": (806, 693), "linkshaendig": (806, 663), "doppel_minus": (1113, 590), "regler": (1290, 454),
    "titel": (900, 239),
    "radius_minus": (1030, 500), "schatten": (806, 535), "nur_schliessen": (806, 691), "menue_links": (806, 635),
    "anzahl_plus": (1064, 397), "name1": (1090, 498), "name3": (1090, 578), "einblenden": (807, 717),
    "inhalt": (1080, 560), "gross32": (999, 647), "vorgabe_knopf": (888, 498),
}

def pkt(name, aufl, dy=0):
    x, y = P[name]
    if aufl == "1024":
        x, y = x - 448, y - 156
    return f"move {x} {y + dy}"

def schreiben(name, aufl, schritte):
    zeilen = ["extent 1920 1080" if aufl == "1920" else "extent 1024 768", "wait 3500"]
    for s in schritte:
        if isinstance(s, tuple):
            n = s[0]
            zeilen.append(pkt(n, aufl, s[1] if len(s) > 1 else 0))
        else:
            zeilen.append(s)
    with open(os.path.join(Z, f"{name}-{aufl}.txt"), "w", encoding="utf-8") as f:
        f.write("\n".join(zeilen) + "\n")

def klicks(n):
    out = []
    for _ in range(n):
        out += ["click", "wait 120"]
    return out

for a in ("1920", "1024"):
    schreiben("g1-tastatur", a, [("tastatur",), "click", "wait 1200", "shot 01-tastatur.png",
        ("rate_plus",), *klicks(5), ("verz_minus",), *klicks(10), ("numlock",), "click", "wait 500", "shot 02-geaendert.png",
        ("aendern",), "click", "wait 1500", "type schweiz", "wait 1000", "shot 03-suche.png",
        ("dlg_erste_zeile",), "click", "wait 400", ("dlg_auswaehlen",), "click", "wait 1000", "shot 04-gewaehlt.png",
        ("uebernehmen",), "click", "wait 2000", "shot 05-uebernommen.png"])
    schreiben("g2-maus", a, [("maus",), "click", "wait 1200", "shot 01-maus.png",
        ("natuerlich",), "click", "wait 200", ("linkshaendig",), "click", "wait 200", ("doppel_minus",), *klicks(4),
        ("regler",), "click", "wait 500", "shot 02-geaendert.png",
        ("uebernehmen",), "click", "wait 2000", "shot 03-uebernommen.png",
        ("titel",), "click", "wait 330", "click", "wait 1200", "shot 04-titel-zwei-klicks-gross-abstand.png",
        "click", "wait 60", "click", "wait 1200", "shot 05-titel-zwei-klicks-klein-abstand.png"])
    schreiben("g2k-doppelklick-kontrolle", a, ["shot 01-start.png",
        ("titel",), "click", "wait 330", "click", "wait 1200", "shot 02-titel-zwei-klicks-gross-abstand.png"])
    schreiben("g3-fensterdesign", a, [("fensterdesign",), "click", "wait 1200", "shot 01-fensterdesign.png",
        ("radius_minus",), *klicks(8), ("schatten",), "click", "wait 200", ("nur_schliessen",), "click", "wait 500",
        "shot 02-geaendert.png", ("uebernehmen",), "click", "wait 2500", "shot 03-uebernommen.png"])
    schreiben("g4-arbeitsflaechen", a, [("arbeitsflaechen",), "click", "wait 1200", "shot 01-arbeitsflaechen.png",
        ("einblenden",), "click", "wait 200",
        ("anzahl_plus",), *klicks(2), ("name1",), "click", "type Post", ("name3",), "click", "type Web", "wait 500",
        "shot 02-geaendert.png", ("uebernehmen",), "click", "wait 2500", "shot 03-uebernommen.png"])
    # Standard wiederherstellen (Daten k1: Rate 40, us/intl, Zeiger 48), dann Zeigergröße 32
    schreiben("k1-standard", a, [("tastatur",), "click", "wait 1200", "shot 01-tastatur-geaendert.png",
        ("standard",), "click", "wait 600", "shot 02-standard.png", ("uebernehmen",), "click", "wait 2000", "shot 03-uebernommen.png",
        ("maus",), "click", "wait 1000", ("inhalt",), "scroll 5", "wait 600", "shot 04-maus-gerollt.png",
        ("gross32",), "click", "wait 400", "shot 05-maus-gross.png", ("uebernehmen",), "click", "wait 2000", "shot 06-maus-uebernommen.png"])
    # Neue Vorgabe (Daten v1)
    schreiben("v1-vorgabe", a, [("tastatur",), "click", "wait 1200", "shot 01-vorgabe-kasten.png",
        ("vorgabe_knopf",), "click", "wait 2000", "shot 02-uebernommen.png"])
    # Ohne Benutzerkopie: Hinweis eine Zeile länger (+16)
    schreiben("o1-ohne-kopie", a, [("arbeitsflaechen",), "click", "wait 1200", "shot 01-arbeitsflaechen.png",
        ("einblenden", 16), "click", "wait 300", ("anzahl_plus", 16), "click", "wait 400", "shot 02-geaendert.png",
        ("uebernehmen",), "click", "wait 2000", "shot 03-uebernommen.png"])
print("geschrieben:", sorted(f for f in os.listdir(Z) if f.endswith(".txt")))
