#!/usr/bin/python3 -I
"""G2-AJ: Auswertung der GUI-Läufe nachweis/g2aj-<lauf>-<1920|1024>.

Prüft die geschriebenen Werte (gsettings im Profil-dconf, Umgebung und
rc.xml der labwc-Benutzerkopie, desktop.ini), das Protokoll der App und die
Zeichenfehler-Messung. Schreibt eine TSV; Exit 0 nur ohne FAIL.

  python3 -I werkzeug/auswerten-g2aj.py <ziel.tsv> [<präfix>]

<präfix> (Standard „g2aj“) wählt die Läufe, z. B. „g2aj-r“ für Wiederholungen.
"""
import os
import re
import subprocess
import sys

import gi

gi.require_version("GdkPixbuf", "2.0")
from gi.repository import GdkPixbuf  # noqa: E402

P = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
N = os.path.join(P, "nachweis")
ZIEL = sys.argv[1]
PRAEFIX = sys.argv[2] if len(sys.argv) > 2 else "g2aj"
zeilen = []


def pruef(lauf, fall, ok, detail=""):
    zeilen.append((lauf, fall, "PASS" if ok else "FAIL", detail.replace("\t", " ").replace("\n", " ")[:300]))


def lesen(*teile):
    try:
        with open(os.path.join(*teile), encoding="utf-8") as f:
            return f.read()
    except OSError:
        return None


def gs(text, schluessel):
    """Wert aus `gsettings list-recursively` (ohne Anführungszeichen)."""
    m = re.search(rf"^org\.gnome\.desktop\.interface {re.escape(schluessel)} (.*)$", text or "", re.M)
    return m.group(1).strip().strip("'") if m else None


def env(text, schluessel):
    m = re.search(rf"^{re.escape(schluessel)}=(.*)$", text or "", re.M)
    return m.group(1).strip() if m else None


def font(text, ort, schluessel):
    m = re.search(rf'<font place="{ort}">(.*?)</font>', text or "", re.S)
    if not m:
        return None
    w = re.search(rf"<{schluessel}>(.*?)</{schluessel}>", m.group(1))
    return w.group(1) if w else None


def bild(lauf, datei):
    try:
        return GdkPixbuf.Pixbuf.new_from_file(os.path.join(N, lauf, datei))
    except Exception:  # noqa: BLE001 – fehlendes oder kaputtes Bild zählt als FAIL
        return None


def pixel(pb, x, y):
    n, r = pb.get_n_channels(), pb.get_rowstride()
    d = pb.get_pixels()
    return tuple(d[y * r + x * n : y * r + x * n + 3])


def unterschied(a, b, x0, y0, x1, y1):
    """Anteil abweichender Bildpunkte im Rechteck (0..1)."""
    if a is None or b is None:
        return 0.0
    gesamt = abweichend = 0
    for y in range(y0, y1, 2):
        for x in range(x0, x1, 2):
            gesamt += 1
            if pixel(a, x, y) != pixel(b, x, y):
                abweichend += 1
    return abweichend / max(gesamt, 1)


def orange(p):
    r, g, b = p
    return r > 200 and 90 < g < 170 and b < 120


def protokoll(lauf):
    return lesen(N, lauf, "logs", "app-direkt.log") or ""


def lauf_ok(lauf):
    log = lesen(N, lauf, "logs", "eingabe.log")
    if log is None:
        pruef(lauf, "Lauf vorhanden", False, "kein Laufordner")
        return False
    ok = "szenario exit=0" in log
    pruef(lauf, "Szenario vollständig", ok, log.strip().splitlines()[-1] if log.strip() else "")
    app = protokoll(lauf)
    pruef(lauf, "kein Absturz", "panicked" not in app and "Speicherzugriffsfehler" not in app)
    return ok


for a in ("1920", "1024"):
    # --- e1: Erscheinungsbild ändern, übernehmen, Standard
    lauf = f"{PRAEFIX}-e1-erscheinungsbild-{a}"
    if lauf_ok(lauf):
        g1, e1 = lesen(N, lauf, "gs-1.txt"), lesen(N, lauf, "env-1.txt")
        soll = {"icon-theme": "gnome-human", "gtk-theme": "Adwaita", "color-scheme": "prefer-dark", "cursor-theme": "DMZ-Black", "cursor-size": "32"}
        for k, v in soll.items():
            pruef(lauf, f"gsettings {k} = {v}", gs(g1, k) == v, str(gs(g1, k)))
        pruef(lauf, "Umgebung XCURSOR_THEME=DMZ-Black", env(e1, "XCURSOR_THEME") == "DMZ-Black", str(env(e1, "XCURSOR_THEME")))
        pruef(lauf, "Umgebung XCURSOR_SIZE=32", env(e1, "XCURSOR_SIZE") == "32", str(env(e1, "XCURSOR_SIZE")))
        g2, e2 = lesen(N, lauf, "gs-2.txt"), lesen(N, lauf, "env-2.txt")
        soll2 = {"icon-theme": "Querencia", "gtk-theme": "Querencia-Klassisch", "color-scheme": "default", "cursor-theme": "DMZ-White", "cursor-size": "24"}
        for k, v in soll2.items():
            pruef(lauf, f"Standard: gsettings {k} = {v}", gs(g2, k) == v, str(gs(g2, k)))
        pruef(lauf, "Standard: keine eigene Zeigerangabe mehr (Vorgabe gilt)", env(e2, "XCURSOR_THEME") is None and env(e2, "XCURSOR_SIZE") is None, (e2 or "").strip())
        vorher, nachher = lesen(N, lauf, "labwc-vorher", "rc.xml"), lesen(N, lauf, "labwc-nachher", "rc.xml")
        pruef(lauf, "rc.xml unverändert", vorher is not None and vorher == nachher)
        app = protokoll(lauf)
        n = len(re.findall(r"Erscheinungsbild übernommen", app))
        pruef(lauf, "zweimal übernommen", n == 2, f"{n}×")
        pruef(lauf, "labwc neu geladen (Zeiger)", app.count("labwc hat die Konfiguration neu geladen") >= 2)
        pruef(lauf, "Statuszeile: sofort in Programmen", "GTK-Programme, Panel und Desktop übernehmen es sofort" in app)

    # --- k1: jemand anderes ändert gsettings, während die Seite offen ist
    lauf = f"{PRAEFIX}-k1-konflikt-{a}"
    if lauf_ok(lauf):
        g1 = lesen(N, lauf, "gs-1.txt")
        pruef(lauf, "fremder Wert bleibt (Humanity)", gs(g1, "icon-theme") == "Humanity", str(gs(g1, "icon-theme")))
        pruef(lauf, "Konflikt gemeldet", "Erscheinungsbild: Konflikt, nicht geschrieben" in protokoll(lauf))

    # --- s2: Programmschriften über gsettings, Fenstertitel über rc.xml
    lauf = f"{PRAEFIX}-s2-schriften-{a}"
    if lauf_ok(lauf):
        g1, r1 = lesen(N, lauf, "gs-1.txt"), lesen(N, lauf, "rc-1.xml")
        pruef(lauf, "Dokumente = DejaVu Serif 12", gs(g1, "document-font-name") == "DejaVu Serif 12", str(gs(g1, "document-font-name")))
        pruef(lauf, "Anwendungen unverändert", gs(g1, "font-name") == "DejaVu Sans 10", str(gs(g1, "font-name")))
        pruef(lauf, "Dicktengleich unverändert", gs(g1, "monospace-font-name") == "DejaVu Sans Mono 10", str(gs(g1, "monospace-font-name")))
        for ort in ("ActiveWindow", "InactiveWindow"):
            werte = (font(r1, ort, "name"), font(r1, ort, "size"), font(r1, ort, "weight"))
            pruef(lauf, f"rc.xml {ort} = DejaVu Sans 12 normal", werte == ("DejaVu Sans", "12", "normal"), str(werte))
        pruef(lauf, "Menüschrift unberührt", font(r1, "MenuItem", "size") == "10" and font(r1, "MenuHeader", "weight") == "bold")
        g2 = lesen(N, lauf, "gs-2.txt")
        pruef(lauf, "Standard: Dokumente = DejaVu Sans 10", gs(g2, "document-font-name") == "DejaVu Sans 10", str(gs(g2, "document-font-name")))
        vorher, r2 = lesen(N, lauf, "labwc-vorher", "rc.xml"), lesen(N, lauf, "rc-2.xml")
        pruef(lauf, "Standard: rc.xml wieder byte-gleich", vorher is not None and vorher == r2)
        app = protokoll(lauf)
        pruef(lauf, "zweimal übernommen", len(re.findall(r"Schriften übernommen", app)) == 2)
        pruef(lauf, "labwc neu geladen (Fenstertitel)", app.count("labwc hat die Konfiguration neu geladen") >= 2)

    # --- h1: Galerie auf der Seite
    lauf = f"{PRAEFIX}-h1-hintergrund-{a}"
    if lauf_ok(lauf):
        datei = "querencia-abend.png" if a == "1920" else "querencia-abend-4x3.png"
        i1, i2 = lesen(N, lauf, "ini-1.txt") or "", lesen(N, lauf, "ini-2.txt") or ""
        pruef(lauf, f"desktop.ini: Abend ({datei})", re.search(rf"^File=.*/{re.escape(datei)}$", i1, re.M) is not None, env(i1, "File") or "")
        pruef(lauf, "Typ Bild", env(i1, "Type") in (None, "image"), str(env(i1, "Type")))
        pruef(lauf, "zurück auf Querencia Blau (File= leer)", env(i2, "File") == "", str(env(i2, "File")))
        app = protokoll(lauf)
        pruef(lauf, "zweimal übernommen", app.count("Hintergrundbild übernommen") == 2)
        pruef(lauf, "Dialog „Hintergrund …“ geöffnet", os.path.exists(os.path.join(N, lauf, "05-dialog.png")))

    # --- p1: Panels folgt panel.ini
    lauf = f"{PRAEFIX}-p1-panels-{a}"
    if lauf_ok(lauf):
        zeilen_p = [z for z in protokoll(lauf).splitlines() if "Panels gelesen:" in z]
        pruef(lauf, "Aufbau beim Start gelesen", len(zeilen_p) >= 1, zeilen_p[0] if zeilen_p else "")
        pruef(lauf, "Änderung von außen angezeigt (Lautstärke neu)", any("Lautstärke" in z for z in zeilen_p[1:]), zeilen_p[-1] if zeilen_p else "")

    # --- s1: alle zwölf Seiten fotografiert (Vorher/Nachher-Bilder)
    lauf = f"{PRAEFIX}-s1-alle-seiten-{a}"
    if lauf_ok(lauf):
        bilder = [b for b in os.listdir(os.path.join(N, lauf)) if re.match(r"\d\d-.*\.png$", b)]
        pruef(lauf, "12 Seiten fotografiert", len(bilder) == 12, str(len(bilder)))

    # --- z1: Zeichenfehler (mit Behebung, gles2 und pixman)
    for r in ("gles2", "pixman"):
        lauf = f"{PRAEFIX}-z1-{r}-{a}"
        if lauf_ok(lauf):
            e = subprocess.run([sys.executable, "-I", os.path.join(P, "werkzeug", "g2aj-reste.py"), os.path.join(N, lauf), a], capture_output=True, text=True)
            pruef(lauf, f"keine Textreste nach Seitenwechsel ({r})", e.returncode == 0, e.stdout.strip().splitlines()[-1] if e.stdout.strip() else e.stderr)

    lauf = f"{PRAEFIX}-z2-symbole-{a}"
    if lauf_ok(lauf):
        e = subprocess.run([sys.executable, "-I", os.path.join(P, "werkzeug", "g2aj-reste.py"), os.path.join(N, lauf), a], capture_output=True, text=True)
        pruef(lauf, "keine Textreste auf „Symbole der Arbeitsfläche“ (gles2)", e.returncode == 0, e.stdout.strip().splitlines()[-1] if e.stdout.strip() else e.stderr)

    # --- w1/w2: Wirkung mit Settings-Portal (QE_PORTAL=settings)
    dx, dy = (0, 0) if a == "1920" else (-448, -156)
    lauf = f"{PRAEFIX}-w1-wirkung-{a}"
    if lauf_ok(lauf):
        v, n = bild(lauf, "01-vorher.png"), bild(lauf, "02-nachher.png")
        g1 = lesen(N, lauf, "gs-1.txt")
        pruef(lauf, "gsettings icon-theme = mate", gs(g1, "icon-theme") == "mate", str(gs(g1, "icon-theme")))
        anteil = unterschied(v, n, 20, 40, 96, 110)
        pruef(lauf, "Desktop-Symbol „Rechner“ wechselt sofort", anteil > 0.2, f"{anteil:.0%} der Bildpunkte anders")
        vor = pixel(v, 600 + dx, 316 + dy) if v else (0, 0, 0)
        nach = pixel(n, 600 + dx, 316 + dy) if n else (0, 0, 0)
        pruef(lauf, "App wechselt sofort aufs GTK-Standardthema (Auswahl nicht mehr orange)", orange(vor) and not orange(nach), f"vorher {vor}, nachher {nach}")
    lauf = f"{PRAEFIX}-w2-schrift-{a}"
    if lauf_ok(lauf):
        v, n = bild(lauf, "01-vorher.png"), bild(lauf, "02-nachher.png")
        g1 = lesen(N, lauf, "gs-1.txt")
        pruef(lauf, "gsettings font-name = DejaVu Serif 12", gs(g1, "font-name") == "DejaVu Serif 12", str(gs(g1, "font-name")))
        anteil = unterschied(v, n, 540 + dx, 305 + dy, 755 + dx, 640 + dy)
        pruef(lauf, "Seitenleiste in neuer Schrift (sofort)", anteil > 0.05, f"{anteil:.0%} der Bildpunkte anders")

    # --- o1: ohne Benutzerkopie: Hinweis und Schreiben
    lauf = f"{PRAEFIX}-o1-ohne-kopie-{a}"
    if lauf_ok(lauf):
        e1 = lesen(N, lauf, "env-1.txt")
        pruef(lauf, "Benutzerkopie angelegt, XCURSOR_THEME=DMZ-Black", env(e1, "XCURSOR_THEME") == "DMZ-Black", str(env(e1, "XCURSOR_THEME")))
        pruef(lauf, "Hinweis: ab der nächsten Anmeldung", "wirksam ab der nächsten Anmeldung" in protokoll(lauf))

# Kein Platzhaltertext mehr im geprüften Programm (Release, falls vorhanden)
REL = os.path.join(N, "g2aj", "release", "querencia-einstellungen")
if os.path.exists(REL):
    with open(REL, "rb") as f:
        inhalt = f.read()
    pruef("release", "kein „Noch nicht verfügbar“ im Programm", "Noch nicht verfügbar".encode() not in inhalt)

os.makedirs(os.path.dirname(os.path.abspath(ZIEL)), exist_ok=True)
with open(ZIEL, "w", encoding="utf-8") as f:
    f.write("lauf\tfall\tergebnis\tdetail\n")
    for z in zeilen:
        f.write("\t".join(z) + "\n")
p = sum(1 for z in zeilen if z[2] == "PASS")
print(f"{p}/{len(zeilen)} PASS")
for z in zeilen:
    if z[2] != "PASS":
        print("FAIL", z[0], z[1], z[3])
sys.exit(0 if p == len(zeilen) else 1)
