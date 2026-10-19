#!/usr/bin/env python3
"""G2-G: wertet die isolierten Läufe b0–b11 (Bereich Bildschirm, zwei
headless-Ausgaben HEADLESS-2 1280×720 bei 0,0 und HEADLESS-1 1920×1080 bei
1280,0) aus: Ausgabenzustand am Ende (logs/ausgaben-ende.txt), Größen der
Bildschirmaufnahmen (= Layout), bildschirm.ini, App-Log.
Ergebnis: nachweis/bildschirm-g2g.tsv (bzw. argv[1])."""
import os
import re
import sys

from PIL import Image

N = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "nachweis")
erg = []


def pruefe(nr, test, ok, beleg):
    erg.append((nr, test, "PASS" if ok else "FAIL", beleg))
    print(f"{'PASS' if ok else 'FAIL'}  {nr:5s} {test}: {beleg}")


def lesen(lauf, name):
    p = os.path.join(N, lauf, name)
    return open(p, encoding="utf-8", errors="replace").read() if os.path.exists(p) else ""


def groesse(lauf, name):
    p = os.path.join(N, lauf, name)
    return Image.open(p).size if os.path.exists(p) else None


def ausgaben(lauf, datei="logs/ausgaben-ende.txt"):
    """{name: {an, modus, position, drehung, skalierung, logisch}}"""
    d = {}
    for z in lesen(lauf, datei).splitlines():
        t = z.split()
        if len(t) < 7:
            continue
        w = dict(kv.split("=", 1) for kv in t[2:] if "=" in kv)
        d[t[0]] = {"an": t[1] == "an", **w}
    return d


def log(lauf):
    return lesen(lauf, "logs/app-direkt.log")


START = {"HEADLESS-1": ("1920x1080@0", "1280,0"), "HEADLESS-2": ("1280x720@0", "0,0")}


def wie_start(a):
    return all(n in a and a[n]["modus"] == m and a[n]["position"] == p and a[n]["an"] for n, (m, p) in START.items())


# b0 Kontrollzentrum
L = "b0-kontrollzentrum-bildschirm"
pruefe("0.1", "Kontrollzentrum zeigt die Seite „Bildschirm“ (Lauf vorhanden, kein Absturz)",
       groesse(L, "01-kontrollzentrum-bildschirm.png") == (1920, 1080) and "panicked" not in log(L), "Bild 01")

# b1 Auflösung, Countdown läuft ab
L = "b1-aufloesung-rueckfall"
a = ausgaben(L)
pruefe("1.1", "Anwenden setzt HEADLESS-1 auf 1600×900 (Layout 2880×900 im Dialogbild)", groesse(L, "03-dialog.png") == (2880, 900), f"{groesse(L, '03-dialog.png')}")
pruefe("1.2", "App-Log: angewendet, dann nach Ablauf zurückgesetzt", "Bildschirm angewendet: HEADLESS-1 1600x900" in log(L) and "Bildschirm zurückgesetzt auf HEADLESS-1 1920x1080" in log(L), "Log")
pruefe("1.3", "Nach dem Countdown wieder Ausgangszustand (Layout 3200×1080, Ausgaben wie am Start)", groesse(L, "04-nach-countdown.png") == (3200, 1080) and wie_start(a), f"{a.get('HEADLESS-1', {}).get('modus')}")
pruefe("1.4", "Nichts gespeichert", not os.path.exists(os.path.join(N, L, "bildschirm.ini.nachher")) and "geschrieben" not in log(L), "keine bildschirm.ini")

# b2 Auflösung behalten
L = "b2-aufloesung-behalten"
a = ausgaben(L)
ini = lesen(L, "bildschirm.ini.nachher")
pruefe("2.1", "Dialog erscheint auf dem neuen Layout 2880×900", groesse(L, "01-dialog.png") == (2880, 900), f"{groesse(L, '01-dialog.png')}")
pruefe("2.2", "„Behalten“: HEADLESS-1 bleibt 1600×900 bei 1280,0", a.get("HEADLESS-1", {}).get("modus") == "1600x900@0" and a["HEADLESS-1"]["position"] == "1280,0", f"{a.get('HEADLESS-1')}")
pruefe("2.3", "bildschirm.ini geschrieben: HEADLESS-1 1600x900, HEADLESS-2 Hauptmonitor", "[Ausgabe HEADLESS-1]" in ini and "Modus=1600x900@0" in ini and re.search(r"\[Ausgabe HEADLESS-2\][^\[]*Haupt=ja", ini) is not None and "geschrieben" in log(L), repr(ini[-160:]))

# b3 Anordnung ziehen
L = "b3-anordnung"
a = ausgaben(L)
pruefe("3.1", "Ziehen von HEADLESS-1 nach links: angewendet HEADLESS-1 bei 0,0, HEADLESS-2 bei 1920,0", a.get("HEADLESS-1", {}).get("position") == "0,0" and a.get("HEADLESS-2", {}).get("position") == "1920,0", f"{ {k: v['position'] for k, v in a.items()} }")
pruefe("3.2", "Layout bleibt 3200×1080, Moduswerte unverändert", groesse(L, "03-behalten.png") == (3200, 1080) and a.get("HEADLESS-1", {}).get("modus") == "1920x1080@0", f"{groesse(L, '03-behalten.png')}")
pruefe("3.3", "bildschirm.ini: Position=1920,0 für HEADLESS-2", re.search(r"\[Ausgabe HEADLESS-2\][^\[]*Position=1920,0", lesen(L, "bildschirm.ini.nachher")) is not None, "ini")

# b4 Skalierung 2
L = "b4-skalierung"
a = ausgaben(L)
# grim rendert das ganze Layout mit dem größten Skalierungsfaktor: 2240×720 logisch → 4480×1440 Pixel
pruefe("4.1", "Skalierung 2 auf HEADLESS-1: logisch 960×540, Layout 2240×720 (grim: 4480×1440 bei Faktor 2)", a.get("HEADLESS-1", {}).get("skalierung") == "2" and a["HEADLESS-1"]["logisch"] == "960x540" and groesse(L, "03-behalten.png") == (4480, 1440), f"{a.get('HEADLESS-1')} {groesse(L, '03-behalten.png')}")
pruefe("4.2", "bildschirm.ini: Skalierung=2", re.search(r"\[Ausgabe HEADLESS-1\][^\[]*Skalierung=2\n", lesen(L, "bildschirm.ini.nachher")) is not None, "ini")

# b5 aus / an
L = "b5-aus-an"
a = ausgaben(L)
pruefe("5.1", "HEADLESS-1 ausschalten: Layout 1280×720, nur HEADLESS-2", groesse(L, "03-aus.png") == (1280, 720), f"{groesse(L, '03-aus.png')}")
pruefe("5.2", "wieder einschalten: Layout 3200×1080, beide an, HEADLESS-1 1920×1080 bei 1280,0", groesse(L, "05-an.png") == (3200, 1080) and wie_start(a), f"{a}")
pruefe("5.3", "zwei Anwendungen im Log, bildschirm.ini zuletzt Aktiv=ja", log(L).count("Bildschirm angewendet") == 2 and re.search(r"\[Ausgabe HEADLESS-1\][^\[]*Aktiv=ja", lesen(L, "bildschirm.ini.nachher")) is not None, "Log/ini")

# b6 Drehung
L = "b6-drehung"
a = ausgaben(L)
pruefe("6.1", "Drehung 90° auf HEADLESS-1: logisch 1080×1920, Layout 2360×1920", a.get("HEADLESS-1", {}).get("drehung") == "90" and a["HEADLESS-1"]["logisch"] == "1080x1920" and groesse(L, "03-behalten.png") == (2360, 1920), f"{a.get('HEADLESS-1')} {groesse(L, '03-behalten.png')}")
pruefe("6.2", "bildschirm.ini: Drehung=90", re.search(r"\[Ausgabe HEADLESS-1\][^\[]*Drehung=90", lesen(L, "bildschirm.ini.nachher")) is not None, "ini")

# b7 Hauptmonitor
L = "b7-hauptmonitor"
a = ausgaben(L)
ini = lesen(L, "bildschirm.ini.nachher")
pruefe("7.1", "HEADLESS-1 als Hauptmonitor: bei 0,0, HEADLESS-2 rechts bei 1920,0", a.get("HEADLESS-1", {}).get("position") == "0,0" and a.get("HEADLESS-2", {}).get("position") == "1920,0", f"{ {k: v['position'] for k, v in a.items()} }")
pruefe("7.2", "bildschirm.ini: Haupt=ja bei HEADLESS-1, Haupt=nein bei HEADLESS-2", re.search(r"\[Ausgabe HEADLESS-1\][^\[]*Haupt=ja", ini) is not None and re.search(r"\[Ausgabe HEADLESS-2\][^\[]*Haupt=nein", ini) is not None, "ini")

# b8 Zurücksetzen
L = "b8-zuruecksetzen"
a = ausgaben(L)
pruefe("8.1", "Zurücksetzen ohne Anwenden: nichts angewendet, Ausgaben wie am Start", "Bildschirm angewendet" not in log(L) and wie_start(a), "Log/Ausgaben")

# b9 Wiederherstellen-Knopf
L = "b9-wiederherstellen-knopf"
a = ausgaben(L)
pruefe("9.1", "Dialog auf 2880×900; Knopf „Vorherige wiederherstellen“ setzt sofort zurück (3200×1080, Ausgangszustand)", groesse(L, "01-dialog.png") == (2880, 900) and groesse(L, "02-wiederhergestellt.png") == (3200, 1080) and wie_start(a), f"{groesse(L, '02-wiederhergestellt.png')}")
pruefe("9.2", "Log: zurückgesetzt, nichts geschrieben", "Bildschirm zurückgesetzt" in log(L) and "geschrieben" not in log(L), "Log")

# b10 Persistenz
L = "b10-persistenz"
a = ausgaben(L)
pruefe("10.1", "--bildschirm-anwenden setzt die Datei: HEADLESS-2 1024×768 bei 0,0, HEADLESS-1 1600×900 bei 1024,0", a.get("HEADLESS-2", {}).get("modus") == "1024x768@0" and a["HEADLESS-2"]["position"] == "0,0" and a.get("HEADLESS-1", {}).get("modus") == "1600x900@0" and a["HEADLESS-1"]["position"] == "1024,0", f"{a}")
pruefe("10.2", "Log: Bildschirm gesetzt aus …bildschirm.ini (2 von 2 Ausgaben); Layout 2624×900", "Bildschirm gesetzt aus" in log(L) and "(2 von 2 Ausgaben)" in log(L) and groesse(L, "01-ende.png") == (2624, 900), f"{groesse(L, '01-ende.png')}")

# b11 ohne Datei
L = "b11-anwenden-ohne-datei"
a = ausgaben(L)
pruefe("11.1", "--bildschirm-anwenden ohne Datei: „keine Datei, nichts zu tun“, Ausgaben unverändert", "keine Datei, nichts zu tun" in log(L) and wie_start(a), "Log")

# Symbole und Panel in allen Läufen
for L in ["b1-aufloesung-rueckfall", "b2-aufloesung-behalten", "b3-anordnung", "b5-aus-an", "b6-drehung"]:
    s1, s2 = lesen(L, "symbole-start"), lesen(L, "symbole-ende")
    pruefe("S." + L[1:2], f"{L}: Symbolpositionen unverändert, kein Absturz von App/Desktop", s1 == s2 and "panicked" not in log(L) and "panicked" not in lesen(L, "logs/desktop.log"), f"{len(s1.splitlines())} Zeilen")

ziel = sys.argv[1] if len(sys.argv) > 1 else "bildschirm-g2g.tsv"
with open(os.path.join(N, ziel), "w", encoding="utf-8") as f:
    f.write("Nr\tPrüfung\tErgebnis\tBeleg\n")
    for z in erg:
        f.write("\t".join(z) + "\n")
n_ok = sum(1 for z in erg if z[2] == "PASS")
print(f"ERGEBNIS bildschirm: {n_ok}/{len(erg)} PASS")
sys.exit(0 if n_ok == len(erg) else 1)
