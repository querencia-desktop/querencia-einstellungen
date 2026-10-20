#!/usr/bin/env python3
"""Einstellungen 0.1.2 (CSS-Welle dunkel): erzeugt die Eingabeszenarien unter
szenarien/012/. Koordinaten wie werkzeug/010-szenarien.py (Kontrollzentrum
860×600 mittig, 1024×768 um (−448, −156) verschoben).

  d1  alle Seiten nacheinander, dazu das Ende der Seite Erscheinungsbild
      (Hinweis am Schalter „Dunkle Darstellung bevorzugen“); läuft mit
      color-scheme default oder prefer-dark ab dem Start (012-gui-lauf.sh)
  w1  Wechsel im Lauf: hell → dunkel → hell, je Erscheinungsbild (oben und
      unten), Bildschirm und Hintergrundbild
"""
import importlib.util
import pathlib

W = pathlib.Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location("sz010", W / "010-szenarien.py")
sz010 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(sz010)
sz010.HIER = W.parent / "szenarien" / "012"
Szenario, SEITEN = sz010.Szenario, sz010.SEITEN

FARBSCHEMA = "run gsettings set org.gnome.desktop.interface color-scheme {}"


def ende(s, datei):
    """Seite ans Ende rollen (wie szenarien/011/eb1), fotografieren und
    wieder nach oben rollen (der Stapel behält die Rollposition je Seite)."""
    s.roh(f"move {1100 + s.dx} {560 + s.dy}", "scroll 15", "wait 800")
    s.bild(datei)
    # Zeiger aus dem Fenster, bis der Rollbalken wieder schmal ist (sonst
    # bleibt er im nächsten Bild der Seite breit stehen)
    s.roh("scroll -15", "wait 500", f"move {200 + s.dx} {700 + s.dy}", "wait 2000")


def alle_seiten(breite):
    s = Szenario(breite)
    for n, seite in enumerate(SEITEN, 1):
        s.seite(seite)
        s.marke(f"seite {seite}")
        s.bild(f"{n:02d}-{seite}.png")
    s.seite("erscheinungsbild")
    ende(s, "14-erscheinungsbild-unten.png")
    s.schreiben("d1-alle-seiten")


def wechsel(breite):
    s = Szenario(breite)
    teile = [("erscheinungsbild", True), ("bildschirm", False), ("hintergrundbild", False)]

    def runde(nr, wort):
        for seite, unten in teile:
            s.seite(seite)
            s.bild(f"{nr}-{wort}-{seite}.png")
            if unten:
                ende(s, f"{nr}-{wort}-{seite}-unten.png")

    runde("1", "hell")
    s.marke("wechsel dunkel")
    s.roh(FARBSCHEMA.format("prefer-dark"), "wait 2000")
    runde("2", "dunkel")
    s.marke("wechsel hell")
    s.roh(FARBSCHEMA.format("default"), "wait 2000")
    runde("3", "hell")
    s.schreiben("w1-wechsel")


for b in (1920, 1024):
    alle_seiten(b)
    wechsel(b)
