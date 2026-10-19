#!/usr/bin/env python3
"""G2-X: Auswertung der isolierten Läufe g2x-a1 … g2x-a11 (Bereich „Arbeitsfläche“).

Prüft desktop.ini nachher byte-genau gegen die Erwartung der Spezifikation
(docs/desktop-ini-arbeitsflaeche.md), die Protokollzeilen der App und des
Desktops (QE-03) und das Vorhandensein der Belegbilder. Schreibt eine TSV
(Nr, Prüfung, Ergebnis, Beleg) und endet mit 0 nur, wenn alles PASS ist.
Ohne PIL: Bilder werden nur auf Vorhandensein geprüft, gesichtet werden sie von Hand.
"""
import os
import sys

P = "~/projects/querencia-einstellungen"
N = os.path.join(P, "nachweis")
Z = os.path.join(P, "szenarien", "arbeitsflaeche")
ziel = sys.argv[1] if len(sys.argv) > 1 else os.path.join(N, "g2x", "arbeitsflaeche-g2x.tsv")
zeilen = []


def lesen(lauf, datei):
    try:
        with open(os.path.join(N, "g2x-" + lauf, datei), encoding="utf-8", errors="replace") as f:
            return f.read()
    except OSError:
        return None


def vorlage(name):
    with open(os.path.join(Z, name), encoding="utf-8") as f:
        return f.read()


def bild(lauf, name):
    return os.path.exists(os.path.join(N, "g2x-" + lauf, name))


def pruefe(nr, text, ok, beleg=""):
    zeilen.append((nr, text, "PASS" if ok else "FAIL", str(beleg).replace("\t", " ").replace("\n", "\\n")[:300]))


def bloecke(c=True, h=True, t=True, s=48, rem=True, opt=True, net=True, intern=False):
    w = lambda b: "true" if b else "false"
    return (
        f"[Icons]\nComputer={w(c)}\nHome={w(h)}\nTrash={w(t)}\nSize={s}\n\n"
        f"[Volumes]\nRemovable={w(rem)}\nOptical={w(opt)}\nNetwork={w(net)}\nInternal={w(intern)}\n"
    )


def app_log(lauf):
    return lesen(lauf, "logs/app-direkt.log") or ""


def desktop_ohne_neue_probleme(lauf):
    d = lesen(lauf, "logs/desktop.log") or ""
    schlecht = [z for z in d.splitlines() if "desktop.ini" in z or "panicked" in z or "CRITICAL" in z]
    return not schlecht, schlecht[:3]


def szenario_ende(lauf):
    e = lesen(lauf, "logs/eingabe.log") or ""
    return "szenario exit=0" in e


HG = vorlage("hg-farbe.ini")
GEMISCHT = vorlage("gemischt.ini")
QE03_HINWEIS = "wertet diese Einstellungen noch nicht aus"

# ---------------------------------------------------------------- a1
L = "a1-aendern-1920"
ini = lesen(L, "desktop.ini.nachher")
pruefe("1.1", "a1: Szenario vollständig, Bilder 01–03", szenario_ende(L) and all(bild(L, b) for b in ["01-start.png", "02-geaendert.png", "03-uebernommen.png"]))
pruefe("1.2", "a1: ohne Datei geladen = Standard (Rechner/Persönlich/Papierkorb an, 48, Wechsel/Optisch/Netz an, Eingebaut aus)",
       "Arbeitsfläche geladen: Rechner an Persönlich an Papierkorb an Größe 48 | Wechsel an Optisch an Netz an Eingebaut aus" in app_log(L))
pruefe("1.3", "a1: desktop.ini = genau [Icons]/[Volumes] mit Papierkorb aus, 72, Netz aus, Eingebaut an",
       ini == bloecke(t=False, s=72, net=False, intern=True), repr(ini))
pruefe("1.4", "a1: keine Sicherung (Datei war neu)", "desktop.ini.vorher" not in (lesen(L, "konfigordner-nachher.txt") or ""), lesen(L, "konfigordner-nachher.txt"))
pruefe("1.5", "a1: Hinweis am QE-03-Desktop (Fähigkeiten fehlen)", QE03_HINWEIS in app_log(L) and "laufwerkstypen=false" in (lesen(L, "logs/faehigkeiten.log") or ""))
ok, z = desktop_ohne_neue_probleme(L)
pruefe("1.6", "a1: QE-03-Desktop meldet keine Probleme mit der neuen Datei (F6)", ok, z)

# ---------------------------------------------------------------- a2
L = "a2-hintergrund-bleibt"
ini = lesen(L, "desktop.ini.nachher") or ""
pruefe("2.1", "a2: [Background] (Type=color) byte-gleich am Anfang, Gruppen angehängt (F3)", ini == HG + "\n" + bloecke(opt=False), repr(ini))
pruefe("2.2", "a2: Sicherung desktop.ini.vorher-<Zeit>", "desktop.ini.vorher-" in (lesen(L, "konfigordner-nachher.txt") or ""))
d = lesen(L, "logs/desktop.log") or ""
pruefe("2.3", "a2: Desktop lädt nach dem Schreiben neu und zeigt weiter nur Farbe #242424", d.count("Hintergrund nur Farbe #242424") >= 2, d.count("Hintergrund nur Farbe #242424"))
ok, z = desktop_ohne_neue_probleme(L)
pruefe("2.4", "a2: Desktop ohne Problemzeilen", ok, z)
pruefe("2.5", "a2: Bild 01", bild(L, "01-uebernommen.png") and szenario_ende(L))

# ---------------------------------------------------------------- a3
L = "a3-standard"
ini = lesen(L, "desktop.ini.nachher") or ""
erwartet = GEMISCHT.split("[Icons]")[0] + bloecke() + "\n[Fremd]\nX=1\n"
pruefe("3.1", "a3: gemischte Datei korrekt gelesen", "Arbeitsfläche geladen: Rechner aus Persönlich an Papierkorb aus Größe 96 | Wechsel aus Optisch an Netz aus Eingebaut an" in app_log(L))
pruefe("3.2", "a3: Standard + Übernehmen: Kommentar, [Background], [Fremd] byte-gleich, Gruppen = Standard", ini == erwartet, repr(ini))
pruefe("3.3", "a3: Bilder 01–03", all(bild(L, b) for b in ["01-geladen.png", "02-standard.png", "03-uebernommen.png"]) and szenario_ende(L))

# ---------------------------------------------------------------- a4
L = "a4-konflikt-volumes"
extern = GEMISCHT.replace("Internal=true", "Internal=false")
kon = lesen(L, "profil/config/querencia/nach-konflikt.ini")
ini = lesen(L, "desktop.ini.nachher") or ""
pruefe("4.1", "a4: fremd geänderte [Volumes] werden nicht überschrieben", kon == extern and "Konflikt, nicht geschrieben" in app_log(L), repr(kon))
pruefe("4.2", "a4: nach „Neu laden“ zeigt die Seite den fremden Stand; danach Übernehmen", ini == extern.replace("Computer=false", "Computer=true"), repr(ini))
pruefe("4.3", "a4: Bilder 01–03", all(bild(L, b) for b in ["01-konflikt.png", "02-neu-geladen.png", "03-uebernommen.png"]) and szenario_ende(L))

# ---------------------------------------------------------------- a5
L = "a5-fremder-hintergrund"
ini = lesen(L, "desktop.ini.nachher") or ""
pruefe("5.1", "a5: fremd geänderter Hintergrund blockiert nicht und bleibt erhalten", ini == GEMISCHT.replace("Color=#242424", "Color=#8f1b1b").replace("Internal=true", "Internal=false"), repr(ini))
pruefe("5.2", "a5: kein Konflikt gemeldet", "Konflikt" not in app_log(L) and "Arbeitsfläche übernommen" in app_log(L))

# ---------------------------------------------------------------- a6
L = "a6-kaputt"
ini = lesen(L, "desktop.ini.nachher") or ""
a = app_log(L)
probleme = [z for z in a.splitlines() if z.startswith("querencia-einstellungen: desktop.ini: [")]
pruefe("6.1", "a6: ungültige Einträge gemeldet (Size=50, Computer=vielleicht, Foo, Zeile ohne =), kein Abbruch", len(probleme) == 4 and "Arbeitsfläche geladen" in a, probleme)
pruefe("6.2", "a6: Standard + Übernehmen ersetzt nur die beiden Gruppen",
       ini == "[Background]\nFile=\nMode=zoom\nColor=#1c4e96\n\n" + bloecke().replace("\n\n[Volumes]", "\n\n[Volumes]"), repr(ini))
pruefe("6.3", "a6: Bilder", bild(L, "01-hinweis.png") and bild(L, "02-uebernommen.png") and szenario_ende(L))

# ---------------------------------------------------------------- a7–a9
L = "a7-1024"
pruefe("7.1", "a7 (1024×768, eigenes Fenster): Rechner aus, Sehr groß (96) übernommen", lesen(L, "desktop.ini.nachher") == bloecke(c=False, s=96), repr(lesen(L, "desktop.ini.nachher")))
pruefe("7.2", "a7: Bilder 01–03 (Liste klappt nach unten auf)", all(bild(L, b) for b in ["01-start.png", "02-liste.png", "03-uebernommen.png"]) and szenario_ende(L))
L = "a8-kz-1024"
pruefe("8.1", "a8 (1024×768, Kontrollzentrum): Netzlaufwerke aus übernommen", lesen(L, "desktop.ini.nachher") == bloecke(net=False), repr(lesen(L, "desktop.ini.nachher")))
L = "a9-kz-1920"
pruefe("9.1", "a9 (1920×1080, Kontrollzentrum): Eingebaute Laufwerke an übernommen", lesen(L, "desktop.ini.nachher") == bloecke(intern=True), repr(lesen(L, "desktop.ini.nachher")))

# ---------------------------------------------------------------- a10/a11
L = "a10-hintergrund-gemischt"
ini = lesen(L, "desktop.ini.nachher") or ""
std = "[Background]\nFile=\nMode=zoom\nColor=#1c4e96\n"
alt_bg = "[Background]\nType=color\nFile=\nMode=zoom\nColor=#242424\n"
pruefe("10.1", "a10: Hintergrund-Dialog 0.0.3 ersetzt nur [Background] (F4), Kommentar/[Icons]/[Volumes]/[Fremd] byte-gleich",
       ini == GEMISCHT.replace(alt_bg, std), repr(ini))
pruefe("10.2", "a10: Sicherung angelegt", "desktop.ini.vorher-" in (lesen(L, "konfigordner-nachher.txt") or ""))
L = "a11-gegenprobe-002"
ini = lesen(L, "desktop.ini.nachher") or ""
pruefe("11.1", "a11 Gegenprobe: 0.0.2 (live) schreibt die ganze Datei und verliert [Icons]/[Volumes] – die Prüfung 10.1 trennt", ini == std, repr(ini))

with open(ziel, "w", encoding="utf-8") as f:
    f.write("Nr\tPrüfung\tErgebnis\tBeleg\n")
    for z in zeilen:
        f.write("\t".join(z) + "\n")
for z in zeilen:
    print(f"{z[2]:4}  {z[0]:5} {z[1]}" + ("" if z[2] == "PASS" else f"   [{z[3]}]"))
n_ok = sum(1 for z in zeilen if z[2] == "PASS")
print(f"ERGEBNIS arbeitsflaeche: {n_ok}/{len(zeilen)} PASS")
sys.exit(0 if n_ok == len(zeilen) else 1)
