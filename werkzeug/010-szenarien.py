#!/usr/bin/env python3
"""Einstellungen 0.1.0: erzeugt die Eingabeszenarien unter szenarien/010/.

Aufbau wie werkzeug/g2aj-szenarien.py: Koordinaten für 1920×1080, das
Kontrollzentrum (860×600) liegt mittig, in 1024×768 um (−448, −156)
verschoben. Die Seitenleiste hat 28 px pro Zeile, die erste Zeile liegt bei
y = 316; neu ist „Sprache und Region“ vor „Informationen über Querencia“.
"""
import pathlib

HIER = pathlib.Path(__file__).resolve().parent.parent / "szenarien" / "010"

SEITEN = [
    "erscheinungsbild",
    "hintergrundbild",
    "symbole",
    "fensterdesign",
    "schriften",
    "panels",
    "tastatur",
    "maus",
    "bildschirm",
    "arbeitsflaechen",
    "standardanwendungen",
    "sprache",
    "informationen",
]


def seite(name):
    i = SEITEN.index(name)
    return (600, 316 + 28 * i)


class Szenario:
    def __init__(self, breite):
        self.breite = breite
        self.dx, self.dy = (0, 0) if breite == 1920 else (-448, -156)
        hoehe = 1080 if breite == 1920 else 768
        self.zeilen = [f"extent {breite} {hoehe}", "wait 3500"]

    def klick(self, x, y, warten=300):
        self.zeilen += [f"move {x + self.dx} {y + self.dy}", "click", f"wait {warten}"]

    def seite(self, name, warten=1200):
        self.klick(*seite(name), warten=warten)

    def bild(self, datei):
        self.zeilen.append(f"shot {datei}")

    def marke(self, text):
        self.zeilen.append(f"mark {text}")

    def roh(self, *zeilen):
        self.zeilen += list(zeilen)

    def schreiben(self, name):
        HIER.mkdir(parents=True, exist_ok=True)
        pfad = HIER / f"{name}-{self.breite}.txt"
        pfad.write_text("\n".join(self.zeilen) + "\n", encoding="utf-8")




# Knöpfe der Fußleiste (labwc-Seiten)
UEBERNEHMEN = (1319, 825)
STANDARD = (877, 825)
ENV = '"$XDG_CONFIG_HOME/querencia/labwc/environment.d/zz-querencia-einstellungen.env"'
# Welche Kataloge die App eingeblendet hat (nur Prozesse dieser Sitzung)
MAPS = (
    'run e=$(readlink -f "$QE_APP"); for p in /proc/[0-9]*; do p=${p#/proc/}; '
    '[ "$(readlink /proc/$p/exe 2>/dev/null)" = "$e" ] && grep -qz "^XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR$" '
    '/proc/$p/environ && grep -o "share/locale/.*mo" /proc/$p/maps; done | sort -u > maps.txt'
)

# Sprache und Formate, mit denen die App dieser Sitzung läuft
UMGEBUNG = (
    'run e=$(readlink -f "$QE_APP"); for p in /proc/[0-9]*; do p=${p#/proc/}; '
    '[ "$(readlink /proc/$p/exe 2>/dev/null)" = "$e" ] && grep -qz "^XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR$" '
    '/proc/$p/environ && tr "\\0" "\\n" < /proc/$p/environ | grep -E "^(LANG|LANGUAGE|LC_[A-Z]+)="; '
    "done | sort -u > app-umgebung.txt"
)


def festhalten(s, n):
    """Zwischenstand der eigenen Umgebungsdatei und der rc.xml im Laufordner."""
    s.roh(
        f"run cat {ENV} > env-{n}.txt",
        f'run cat "$XDG_CONFIG_HOME/querencia/labwc/rc.xml" > rc-{n}.xml',
        "wait 700",
    )


def alle_seiten(breite, name="a1-alle-seiten"):
    """Jede Seite einmal anklicken und fotografieren; Kataloge festhalten."""
    s = Szenario(breite)
    for n, seite_ in enumerate(SEITEN, 1):
        s.seite(seite_)
        s.marke(f"seite {seite_}")
        s.bild(f"{n:02d}-{seite_}.png")
    s.roh(MAPS, "wait 500")
    s.schreiben(name)


def sprache_de(breite):
    """Deutsche Oberfläche: Formate Deutschland übernehmen, dann Englisch mit
    Formaten Schweiz, zuletzt Standard (Vorgabe ohne eigene Angaben)."""
    s = Szenario(breite)
    s.seite("sprache")
    s.bild("01-sprache.png")
    s.klick(1070, 589, 800)  # Formate: Deutsch (Deutschland)
    s.bild("02-formate-deutschland.png")
    s.marke("uebernehmen 1")
    s.klick(*UEBERNEHMEN, 1500)
    s.bild("03-uebernommen.png")
    festhalten(s, 1)
    s.klick(797, 464, 400)  # Anzeigesprache English
    s.klick(1070, 641, 800)  # Formate: Deutsch (Schweiz)
    s.bild("04-englisch-schweiz.png")
    s.marke("uebernehmen 2")
    s.klick(*UEBERNEHMEN, 1500)
    s.bild("05-uebernommen.png")
    festhalten(s, 2)
    s.klick(*STANDARD, 600)
    s.bild("06-standard.png")
    s.marke("uebernehmen 3")
    s.klick(*UEBERNEHMEN, 1500)
    s.bild("07-standard-uebernommen.png")
    festhalten(s, 3)
    s.schreiben("sp1-sprache")


def sprache_en(breite):
    """Englische Oberfläche (eigene Datei LANGUAGE=en): zurück auf Deutsch mit
    Formaten Deutschland."""
    s = Szenario(breite)
    s.seite("sprache")
    s.bild("01-language.png")
    s.klick(786, 438, 400)  # Deutsch – German
    s.klick(1086, 600, 800)  # German (Germany)
    s.bild("02-deutsch-germany.png")
    s.marke("uebernehmen 1")
    s.klick(*UEBERNEHMEN, 1500)
    s.bild("03-applied.png")
    festhalten(s, 1)
    s.roh(MAPS, "wait 500")
    s.schreiben("sp2-language")


def neuanmeldung(breite):
    """Neue Sitzung mit der Datei aus sp1 Schritt 2 (English, Formate Schweiz)."""
    s = Szenario(breite)
    s.seite("sprache")
    s.bild("01-sprache-neu.png")
    s.seite("tastatur")
    s.bild("02-tastatur-neu.png")
    s.roh(MAPS, UMGEBUNG, "wait 500")
    s.schreiben("n1-neuanmeldung")


def tastatur(breite):
    """Zweite Belegung (Schweiz) hinzufügen, übernehmen; nach oben schieben und
    Umschalttaste Strg+Umschalt; entfernen; Standard."""
    s = Szenario(breite)
    s.seite("tastatur")
    s.roh(f"move {1100 + s.dx} {560 + s.dy}", "scroll 10", "wait 800")
    s.bild("01-tastatur-unten.png")
    s.klick(877, 434, 1500)  # Belegung hinzufügen …
    s.roh("type schweiz", "wait 1000")
    s.bild("02-dialog.png")
    s.klick(900, 361, 400)  # erste Zeile: Deutsch (Schweiz)
    s.klick(1152, 798, 1200)  # Auswählen
    s.bild("03-hinzugefuegt.png")
    s.marke("uebernehmen 1")
    s.klick(*UEBERNEHMEN, 2000)
    s.bild("04-uebernommen.png")
    festhalten(s, 1)
    s.klick(1110, 412, 600)  # Nach oben
    s.klick(797, 535, 600)  # Strg+Umschalt
    s.bild("05-getauscht.png")
    s.marke("uebernehmen 2")
    s.klick(*UEBERNEHMEN, 2000)
    s.bild("06-uebernommen.png")
    festhalten(s, 2)
    s.klick(1320, 412, 800)  # Entfernen (jetzt Deutsch)
    s.bild("07-entfernt.png")
    s.marke("uebernehmen 3")
    s.klick(*UEBERNEHMEN, 2000)
    s.bild("08-uebernommen.png")
    festhalten(s, 3)
    s.klick(*STANDARD, 600)
    s.marke("uebernehmen 4")
    s.klick(*UEBERNEHMEN, 2000)
    s.bild("09-standard-uebernommen.png")
    festhalten(s, 4)
    s.schreiben("tb1-tastatur")


def erkunden(breite):
    """Kalibrierung: Tastatur und Sprache, jeweils nach unten gerollt."""
    s = Szenario(breite)
    for name in ("tastatur", "sprache"):
        s.seite(name)
        s.bild(f"{name}-oben.png")
        s.roh(f"move {1100 + s.dx} {560 + s.dy}", "scroll 10", "wait 800")
        s.bild(f"{name}-unten.png")
    s.schreiben("e0-erkunden")



def kalibrieren(breite):
    """Kalibrierung Tastatur: hinzufügen (Schweiz), Bilder nach jedem Schritt."""
    s = Szenario(breite)
    s.seite("tastatur")
    s.roh(f"move {1100 + s.dx} {560 + s.dy}", "scroll 10", "wait 800")
    s.bild("k01-unten.png")
    s.klick(877, 434, 1500)
    s.bild("k02-dialog.png")
    s.roh("type schweiz", "wait 1000")
    s.bild("k03-suche.png")
    s.klick(900, 367, 400)
    s.klick(1152, 798, 1200)
    s.bild("k04-hinzugefuegt.png")
    s.schreiben("e1-kalibrieren")


def fenster(breite):
    """Eigenständige Fenster (QE_APP_ARGS=--hintergrund usw.): nur fotografieren."""
    s = Szenario(breite)
    s.roh("wait 1500")
    s.bild("01-fenster.png")
    s.roh(MAPS, "wait 500")
    s.schreiben("d1-fenster")


if __name__ == "__main__":
    for b in (1920, 1024):
        alle_seiten(b)
        erkunden(b)
        kalibrieren(b)
        sprache_de(b)
        sprache_en(b)
        neuanmeldung(b)
        tastatur(b)
        fenster(b)
