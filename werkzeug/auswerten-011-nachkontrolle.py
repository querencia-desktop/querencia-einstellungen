#!/usr/bin/python3 -I
"""Einstellungen 0.1.1: Auswertung der Nachkontrolle (werkzeug/011-nachkontrolle.sh).

Benutzt die Prüfungen aus auswerten-011.py (r1, r2, Bilder) für die Läufe
nachweis/nk-*, dazu die Rotation der Sicherungen (nk-rot), die Version auf der
Seite „Informationen“ (Bild) und die Kataloge aus der Ablage und GTK-Meldungen in allen Programmprotokollen.

  python3 -I werkzeug/auswerten-011-nachkontrolle.py <ziel.tsv>
"""
import os
import re
import sys

HIER = os.path.dirname(os.path.abspath(__file__))
quelle = open(os.path.join(HIER, "auswerten-011.py"), encoding="utf-8").read()
ziel = sys.argv[1]
sys.argv = [sys.argv[0], ziel, "nk"]
exec(compile(quelle[: quelle.index("SEITEN = [")], "auswerten-011.py", "exec"))

SEITEN = ["erscheinungsbild", "hintergrundbild", "symbole", "fensterdesign", "schriften", "panels", "tastatur",
          "maus", "bildschirm", "arbeitsflaechen", "standardanwendungen", "sprache", "informationen"]
r1("nk-r1-548483c-1920")
r1("nk-r1-548483c-1024")
r1("nk-r1-ohne-1920", ohne=True)
r2("nk-r2-1920")
for s in ("de", "en"):
    bilder(f"nk-eb1-{s}-1920", ["01-oben.png", "02-unten-hinweis.png"])
    bilder(f"nk-a1-{s}-1920", [f"{i:02d}-{n}.png" for i, n in enumerate(SEITEN, 1)], s)

# Rotation: 6 alte eigene + 1 neue → die 5 jüngsten; nach dem Zurückstellen geht die neue
# (gleich der Datei); fremde *.vorher-* byte-gleich
L = "nk-rot-1920"
ok, app = lauf_ok(L)
if ok:
    vorher = baum(os.path.join(N, L, "labwc-vorher"))
    d1 = lesen(N, L, "dateien-1.txt") or ""
    d2 = lesen(N, L, "dateien-2.txt") or ""
    eigene1 = sorted(re.findall(r"rc\.xml\.einstellungen-vorher-\S+", d1))
    eigene2 = sorted(re.findall(r"rc\.xml\.einstellungen-vorher-\S+", d2))
    alt = [f"rc.xml.einstellungen-vorher-20261019-00000{i}" for i in range(1, 7)]
    pruef(L, "nach Übernehmen 1: genau 5 eigene, die zwei ältesten weg, neue dabei",
          len(eigene1) == 5 and alt[0] not in eigene1 and alt[1] not in eigene1 and set(alt[2:]) <= set(eigene1), eigene1)
    pruef(L, "nach dem Zurückstellen: die neue (= Datei) weg, 4 alte bleiben", eigene2 == alt[2:], eigene2)
    nachher = baum(os.path.join(N, L, "labwc-nachher"))
    fremd = [k for k in (vorher or {}) if ".vorher-" in k and ".einstellungen-vorher-" not in k]
    pruef(L, f"{len(fremd)} fremde *.vorher-* byte-gleich", len(fremd) == 5 and all((nachher or {}).get(k) == vorher[k] for k in fremd), sorted(fremd))
    pruef(L, "rc.xml byte-gleich mit dem Anfang", (nachher or {}).get("rc.xml") == (vorher or {}).get("rc.xml"))

# Version: die Seite „Informationen“ (13-informationen.png) wird am Bild geprüft. Die
# Kataloge prüft bilder() (Sprache in /proc/<pid>/maps); die Binary ist das Ziel des
# Live-Links (lauf.log, Zeile „App (installiert)“).

# GTK-Meldungen. Bekannt aus der Prüfumgebung: Gdk-WARNING VK_ERROR_SURFACE_LOST_KHR beim
# Start im headless labwc (Vulkan-Renderer von GSK, pixman-Ausgabe) – genauso mit 0.1.0
# (Lauf nk-vergleich-010-a1-de-1920) und nicht mit GSK_RENDERER=gl (nk-a1-de-gl-1920).
MUSTER = r"(Gtk|Gdk|GLib|GLib-GObject|Gsk|Pango)-(WARNING|CRITICAL|ERROR|Message)|CRITICAL|WARNING \*\*"
for L in sorted(d for d in os.listdir(N) if d.startswith("nk-")):
    app = lesen(N, L, "logs", "app-direkt.log") or ""
    alle = [z for z in app.splitlines() if re.search(MUSTER, z)]
    vk = [z for z in alle if "VK_ERROR_SURFACE_LOST_KHR" in z]
    andere = [z for z in alle if z not in vk]
    pruef(L, "0 GTK-Meldungen außer der bekannten Vulkan-Startmeldung", not andere, andere[:2])
    if L in ("nk-a1-de-gl-1920",):
        pruef(L, "mit GSK_RENDERER=gl: 0 GTK-Meldungen überhaupt", not alle, alle[:2])
    zeilen.append((L, "Vulkan-Startmeldung (Prüfumgebung, nur gezählt)", "INFO", str(len(vk))))

with open(ziel, "w", encoding="utf-8") as f:
    f.write("lauf\tfall\tergebnis\tdetail\n")
    for z in zeilen:
        f.write("\t".join(z) + "\n")
fail = [z for z in zeilen if z[2] == "FAIL"]
pz = [z for z in zeilen if z[2] != "INFO"]
print(f"{len(pz) - len(fail)}/{len(pz)} PASS, {len(fail)} FAIL → {ziel}")
for z in fail:
    print("FAIL", *z[:2], z[3])
sys.exit(1 if fail else 0)
