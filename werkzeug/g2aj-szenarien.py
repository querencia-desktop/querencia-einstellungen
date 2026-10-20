#!/usr/bin/env python3
"""G2-AJ: erzeugt die Eingabeszenarien unter szenarien/g2aj/.

Koordinaten für 1920×1080; das Kontrollzentrum (860×600) liegt mittig, in
1024×768 um (−448, −156) verschoben (wie G2-AG). Die Seitenleiste hat 28 px
pro Zeile, die erste Zeile liegt bei y = 316.
"""
import pathlib

HIER = pathlib.Path(__file__).resolve().parent.parent / "szenarien" / "g2aj"

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
        pfad = HIER / f"{name}-{self.breite}.txt"
        pfad.write_text("\n".join(self.zeilen) + "\n", encoding="utf-8")


def alle_seiten(breite):
    """Jede Seite einmal anklicken und fotografieren (vorher/nachher)."""
    s = Szenario(breite)
    for n, name in enumerate(SEITEN, 1):
        s.seite(name)
        s.marke(f"seite {name}")
        s.bild(f"{n:02d}-{name}.png")
    s.schreiben("s1-alle-seiten")


WECHSEL_Z1 = [
    ("erscheinungsbild", "hintergrundbild"),
    ("schriften", "panels"),
    ("standardanwendungen", "hintergrundbild"),
    ("tastatur", "panels"),
]
# fünftes Bild des Besitzers: Reste unter dem Laufwerke-Rahmen von „Symbole der Arbeitsfläche“
WECHSEL_Z2 = [
    ("erscheinungsbild", "symbole"),
    ("standardanwendungen", "symbole"),
    ("tastatur", "symbole"),
    ("schriften", "symbole"),
]


def wechsel(breite, folge=WECHSEL_Z1, name="z1-wechsel"):
    """Zeichenfehler: zwischen langen und kurzen Seiten hin und her, mit
    Zeigerbewegung über der Seitenleiste (Hover erzeugt kleine Teilbilder)."""
    s = Szenario(breite)
    n = 0
    for lang, kurz in folge:
        s.seite(lang, warten=800)
        for y in range(0, 8):
            s.roh(f"move {640 + s.dx} {330 + 28 * y + s.dy}", "wait 40")
        s.seite(kurz, warten=800)
        for y in range(8, 0, -1):
            s.roh(f"move {640 + s.dx} {330 + 28 * y + s.dy}", "wait 40")
        s.roh(f"move {1200 + s.dx} {900 + s.dy}", "wait 600")
        n += 1
        s.marke(f"wechsel {lang} {kurz}")
        s.bild(f"w{n:02d}-{lang}-{kurz}.png")
        # Referenz im selben Lauf: Doppelklick auf die Titelleiste maximiert,
        # ein zweiter stellt wieder her; die neue Puffergröße erzwingt ein
        # vollständiges Neuzeichnen. Ohne Zeichenfehler sind beide Bilder gleich.
        # maximiert liegt die Titelleiste direkt unter dem oberen Panel (y = 44)
        s.roh(f"move {760 + s.dx} {239 + s.dy}", "dclick", "wait 900")
        s.roh(f"move {760 + s.dx} 44", "dclick", "wait 900")
        s.roh(f"move {1200 + s.dx} {900 + s.dy}", "wait 600")
        s.bild(f"w{n:02d}-{lang}-{kurz}-ref.png")
    s.schreiben(name)


# Knöpfe der Fußleiste (labwc-Seiten und Hintergrundbild)
UEBERNEHMEN = (1319, 825)
STANDARD = (877, 825)
GS = "gsettings list-recursively org.gnome.desktop.interface"
ENV = '"$XDG_CONFIG_HOME/querencia/labwc/environment.d/zz-querencia-einstellungen.env"'


def festhalten(s, n):
    """Zwischenstand von gsettings, Umgebung, rc.xml und desktop.ini im Laufordner."""
    s.roh(
        f"run {GS} > gs-{n}.txt",
        f"run cat {ENV} > env-{n}.txt",
        f'run cat "$XDG_CONFIG_HOME/querencia/labwc/rc.xml" > rc-{n}.xml',
        f'run cat "$XDG_CONFIG_HOME/querencia/desktop.ini" > ini-{n}.txt',
        "wait 700",
    )


def erscheinungsbild(breite):
    """Symbolthema, GTK-Thema, dunkel, Zeiger ändern, übernehmen; dann Standard."""
    s = Szenario(breite)
    s.seite("erscheinungsbild")
    s.bild("01-erscheinungsbild.png")
    s.klick(1180, 454, 600)  # Symbole: GNOME-Human
    s.bild("02-gnome-human.png")
    s.klick(796, 658)  # GTK-Standard (eingebaut)
    s.klick(796, 688)  # Dunkle Darstellung bevorzugen
    s.roh(f"move {1070 + s.dx} {600 + s.dy}", "scroll 5", "wait 600")
    s.klick(950, 621)  # Zeiger DMZ (Schwarz)
    s.klick(970, 652)  # Größe Groß (32)
    s.bild("03-geaendert.png")
    s.marke("uebernehmen 1")
    s.klick(*UEBERNEHMEN, 1500)
    s.bild("04-uebernommen.png")
    festhalten(s, 1)
    s.klick(*STANDARD, 600)
    s.bild("05-standard.png")
    s.marke("uebernehmen 2")
    s.klick(*UEBERNEHMEN, 1500)
    s.bild("06-standard-uebernommen.png")
    festhalten(s, 2)
    s.schreiben("e1-erscheinungsbild")


def konflikt(breite):
    """Während einer offenen Änderung setzt jemand anderes das Symbolthema."""
    s = Szenario(breite)
    s.seite("erscheinungsbild")
    s.klick(1180, 454, 400)  # GNOME-Human
    s.roh("run gsettings set org.gnome.desktop.interface icon-theme Humanity", "wait 800")
    s.marke("uebernehmen konflikt")
    s.klick(*UEBERNEHMEN, 1200)
    s.bild("01-konflikt.png")
    festhalten(s, 1)
    s.schreiben("k1-konflikt")


def schriften(breite):
    """Dokumente → DejaVu Serif 12, Fenstertitel → DejaVu Sans (normal) 12; dann Standard."""
    s = Szenario(breite)
    s.seite("schriften")
    s.bild("01-schriften.png")
    s.klick(982, 477, 1500)  # Dokumente: Schriftwähler
    s.roh("type Serif", "wait 900")
    s.klick(851, 443, 400)  # DejaVu Serif Book
    s.klick(1137, 776, 200)  # Größe +
    s.klick(1137, 776, 400)  # Größe +
    s.bild("02-dialog-dokumente.png")
    s.klick(1105, 321, 1000)  # Auswählen
    s.klick(875, 644, 1500)  # Fenstertitel: Schriftwähler
    s.roh("type DejaVu Sans Book", "wait 900")
    s.klick(851, 403, 400)  # erste Zeile: DejaVu Sans Book
    s.klick(1137, 776, 200)
    s.klick(1137, 776, 400)
    s.bild("03-dialog-titel.png")
    s.klick(1105, 321, 1000)
    s.bild("04-geaendert.png")
    s.marke("uebernehmen 1")
    s.klick(*UEBERNEHMEN, 1500)
    s.bild("05-uebernommen.png")
    festhalten(s, 1)
    s.klick(*STANDARD, 600)
    s.marke("uebernehmen 2")
    s.klick(*UEBERNEHMEN, 1500)
    s.bild("06-standard-uebernommen.png")
    festhalten(s, 2)
    s.schreiben("s2-schriften")


def hintergrund(breite):
    """Galerie auf der Seite: Abend wählen, übernehmen, zurück auf Blau; Dialog öffnen."""
    s = Szenario(breite)
    s.seite("hintergrundbild")
    s.bild("01-hintergrund.png")
    s.klick(1003, 560, 900)  # Kachel Abend
    s.bild("02-abend-gewaehlt.png")
    s.marke("uebernehmen 1")
    s.klick(*UEBERNEHMEN, 2500)
    s.bild("03-abend-uebernommen.png")
    festhalten(s, 1)
    s.klick(855, 560, 900)  # Kachel Blau
    s.marke("uebernehmen 2")
    s.klick(*UEBERNEHMEN, 2500)
    s.bild("04-blau-uebernommen.png")
    festhalten(s, 2)
    s.klick(1107, 425, 2500)  # Hintergrund …
    s.bild("05-dialog.png")
    s.schreiben("h1-hintergrund")


def panels(breite, ini):
    """Seite Panels; panel.ini ändert sich von außen, die Seite folgt."""
    s = Szenario(breite)
    s.seite("panels")
    s.bild("01-panels.png")
    s.roh(f'run cp {ini} "$XDG_CONFIG_HOME/querencia/panel.ini.neu" && mv "$XDG_CONFIG_HOME/querencia/panel.ini.neu" "$XDG_CONFIG_HOME/querencia/panel.ini"', "wait 1500")
    s.marke("panel.ini geaendert")
    s.bild("02-panels-neu.png")
    s.schreiben("p1-panels")


def ohne_kopie(breite):
    """Ohne Benutzerkopie (labwc liest die Vorgabe): Zeiger ändern legt sie an."""
    s = Szenario(breite)
    s.seite("erscheinungsbild")
    s.roh(f"move {1070 + s.dx} {600 + s.dy}", "scroll 5", "wait 600")
    s.klick(950, 621)  # Zeiger DMZ (Schwarz)
    s.bild("01-geaendert.png")
    s.marke("uebernehmen ohne kopie")
    s.klick(*UEBERNEHMEN, 1500)
    s.bild("02-uebernommen.png")
    festhalten(s, 1)
    s.schreiben("o1-ohne-kopie")


def wirkung(breite):
    """Mit Settings-Portal (QE_PORTAL=settings): Symbole MATE, GTK-Standard,
    dunkel übernehmen; App, Panel und Desktop wechseln ohne Neustart."""
    s = Szenario(breite)
    s.seite("erscheinungsbild")
    s.roh(f"move {1200 + s.dx} {980 + s.dy}", "wait 300")
    s.bild("01-vorher.png")
    s.klick(1180, 480, 400)  # Symbole: MATE (3. Spalte, 2. Zeile)
    s.klick(796, 658)  # GTK-Standard (eingebaut)
    s.klick(796, 688)  # Dunkle Darstellung bevorzugen
    s.marke("uebernehmen wirkung")
    s.klick(*UEBERNEHMEN, 2500)
    s.roh(f"move {1200 + s.dx} {980 + s.dy}", "wait 500")
    s.bild("02-nachher.png")
    festhalten(s, 1)
    s.schreiben("w1-wirkung")


def wirkung_schrift(breite):
    """Mit Settings-Portal: Schrift der Anwendungen DejaVu Serif 12 übernehmen."""
    s = Szenario(breite)
    s.seite("schriften")
    s.bild("01-vorher.png")
    s.klick(982, 441, 1500)  # Anwendungen: Schriftwähler
    s.roh("type Serif", "wait 900")
    s.klick(851, 443, 400)  # DejaVu Serif Book
    s.klick(1137, 776, 200)
    s.klick(1137, 776, 400)
    s.klick(1105, 321, 1000)  # Auswählen
    s.marke("uebernehmen schrift")
    s.klick(*UEBERNEHMEN, 2500)
    s.roh(f"move {1200 + s.dx} {980 + s.dy}", "wait 500")
    s.bild("02-nachher.png")
    festhalten(s, 1)
    s.schreiben("w2-schrift")


if __name__ == "__main__":
    HIER.mkdir(parents=True, exist_ok=True)
    # Pfad relativ zum Laufordner nachweis/<lauf>/ (dort läuft qe-eingabe)
    ini = "../../szenarien/g2aj/panel-geaendert.ini"
    for b in (1920, 1024):
        alle_seiten(b)
        wechsel(b)
        wechsel(b, WECHSEL_Z2, "z2-symbole")
        erscheinungsbild(b)
        konflikt(b)
        schriften(b)
        hintergrund(b)
        panels(b, ini)
        ohne_kopie(b)
        wirkung(b)
        wirkung_schrift(b)
