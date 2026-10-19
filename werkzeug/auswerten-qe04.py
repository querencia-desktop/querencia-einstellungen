#!/usr/bin/env python3
"""G2-X: Auswertung der Ende-zu-Ende-Läufe g2x-q1 … q4 (Einstellungen 0.0.4 + Desktop QE-04).

Belege: logs/desktop.log (Startzeile „Symbole: …“, „Einstellungen geändert: …“,
letzte Zeile „N Symbole: …“), logs/app-direkt.log, logs/faehigkeiten.log,
symbole-start/-ende (Plätze bleiben), Bilder zum Sichten.
"""
import os
import re
import sys

N = "~/projects/querencia-einstellungen/nachweis"
ziel = sys.argv[1] if len(sys.argv) > 1 else os.path.join(N, "g2x", "qe04-e2e.tsv")
zeilen = []


def lesen(lauf, datei):
    try:
        with open(os.path.join(N, "g2x-" + lauf, datei), encoding="utf-8", errors="replace") as f:
            return f.read()
    except OSError:
        return ""


def pruefe(nr, text, ok, beleg=""):
    zeilen.append((nr, text, "PASS" if ok else "FAIL", str(beleg).replace("\t", " ").replace("\n", "\\n")[:300]))


def symbole(lauf):
    """Alle Zeilen „N Symbole: a, b“ des Desktop-Logs, als Listen."""
    out = []
    for z in lesen(lauf, "logs/desktop.log").splitlines():
        m = re.search(r": \d+ Symbole: (.*)$", z)
        if m:
            out.append([s.strip() for s in m.group(1).split(",")])
    return out


def bild(lauf, name):
    return os.path.exists(os.path.join(N, "g2x-" + lauf, name))


def schlecht(lauf):
    return [z for z in lesen(lauf, "logs/desktop.log").splitlines() if "panicked" in z or "CRITICAL" in z or "Kachel größer" in z]


STANDARD = "Symbole: Rechner an Persönlicher Ordner an Papierkorb an Größe 48 | Laufwerke: Wechsel an Optisch an Netz an Eingebaut aus"

# q1
L = "q1-e2e-1920"
d = lesen(L, "logs/desktop.log")
s = symbole(L)
pruefe("1.1", "q1: Desktop QE-04 meldet die neuen Fähigkeiten", "laufwerkstypen=true symbole=true symbolgroesse=true" in lesen(L, "logs/faehigkeiten.log"), lesen(L, "logs/faehigkeiten.log"))
pruefe("1.2", "q1: Einstellungen zeigen keinen Hinweis (Desktop wertet alles aus)", "Desktop wertet alles aus" in lesen(L, "logs/app-direkt.log") and "noch nicht aus" not in lesen(L, "logs/app-direkt.log"))
pruefe("1.3", "q1: Desktop startet ohne desktop.ini mit den Standardwerten", "querencia-desktop: " + STANDARD in d)
pruefe("1.4", "q1: vor dem Übernehmen Rechner, Persönlicher Ordner, Papierkorb und cidata da", bool(s) and all(x in s[0] for x in ["Rechner", "Persönlicher Ordner", "Papierkorb", "cidata"]), s[:1])
pruefe("1.5", "q1: Übernehmen wirkt sofort: Papierkorb aus, Größe 72, Optisch aus",
       "Einstellungen geändert: Symbole: Rechner an Persönlicher Ordner an Papierkorb aus Größe 72 | Laufwerke: Wechsel an Optisch aus Netz an Eingebaut aus" in d)
pruefe("1.6", "q1: danach ohne Papierkorb und ohne cidata, Rechner/Persönlicher Ordner bleiben", bool(s) and "Papierkorb" not in s[-1] and "cidata" not in s[-1] and "Rechner" in s[-1] and "Persönlicher Ordner" in s[-1], s[-1:])
pruefe("1.7", "q1: Plätze in desktop-symbole unverändert (ausgeblendete Symbole vergessen nichts)", lesen(L, "symbole-start") == lesen(L, "symbole-ende") and lesen(L, "symbole-start") not in ("", "fehlt\n"), "")
pruefe("1.8", "q1: kein Absturz, keine CRITICAL, keine zu große Kachel", not schlecht(L), schlecht(L)[:3])
pruefe("1.9", "q1: Bilder vorher/nachher", bild(L, "01-vorher.png") and bild(L, "02-nachher.png"))

# q2
L = "q2-e2e-1024"
d = lesen(L, "logs/desktop.log")
s = symbole(L)
pruefe("2.1", "q2 (1024×768): Rechner aus, Größe 96 sofort", "Einstellungen geändert: Symbole: Rechner aus Persönlicher Ordner an Papierkorb an Größe 96 | Laufwerke: Wechsel an Optisch an Netz an Eingebaut aus" in d)
pruefe("2.2", "q2: danach ohne Rechner, mit cidata", bool(s) and "Rechner" not in s[-1] and "cidata" in s[-1], s[-1:])
pruefe("2.3", "q2: kein Absturz, keine zu große Kachel", not schlecht(L), schlecht(L)[:3])
pruefe("2.4", "q2: Bilder", bild(L, "01-vorher.png") and bild(L, "02-nachher.png"))

# q3
L = "q3-standard"
d = lesen(L, "logs/desktop.log")
s = symbole(L)
pruefe("3.1", "q3: Start mit Rechner aus, 32 px, Optisch aus", "querencia-desktop: Symbole: Rechner aus Persönlicher Ordner an Papierkorb an Größe 32 | Laufwerke: Wechsel an Optisch aus Netz an Eingebaut aus" in d)
pruefe("3.2", "q3: beim Start ohne Rechner und cidata", bool(s) and "Rechner" not in s[0] and "cidata" not in s[0], s[:1])
pruefe("3.3", "q3: Standard + Übernehmen: alles wieder da, 48 px", "Einstellungen geändert: " + STANDARD in d and bool(s) and all(x in s[-1] for x in ["Rechner", "Persönlicher Ordner", "Papierkorb", "cidata"]), s[-1:])
pruefe("3.4", "q3: kein Absturz, keine zu große Kachel", not schlecht(L), schlecht(L)[:3])

# q4 Gegenprobe
L = "q4-gegenprobe-qe03"
d = lesen(L, "logs/desktop.log")
s = symbole(L)
pruefe("4.1", "q4 Gegenprobe QE-03: Einstellungen zeigen den Hinweis", "noch nicht aus" in lesen(L, "logs/app-direkt.log"))
pruefe("4.2", "q4 Gegenprobe QE-03: Symbole bleiben (Papierkorb und cidata weiter da) – die Prüfungen 1.5/1.6 trennen", bool(s) and "Papierkorb" in s[-1] and "cidata" in s[-1] and "Einstellungen geändert" not in d, s[-1:])

with open(ziel, "w", encoding="utf-8") as f:
    f.write("Nr\tPrüfung\tErgebnis\tBeleg\n")
    for z in zeilen:
        f.write("\t".join(z) + "\n")
for z in zeilen:
    print(f"{z[2]:4}  {z[0]:5} {z[1]}" + ("" if z[2] == "PASS" else f"   [{z[3]}]"))
n = sum(1 for z in zeilen if z[2] == "PASS")
print(f"ERGEBNIS qe04-e2e: {n}/{len(zeilen)} PASS")
sys.exit(0 if n == len(zeilen) else 1)
