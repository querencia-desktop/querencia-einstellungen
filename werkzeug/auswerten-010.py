#!/usr/bin/python3 -I
"""Einstellungen 0.1.0: Auswertung der GUI-Läufe nachweis/<präfix>-<lauf>-<1920|1024>.

Prüft die geschriebene eigene Umgebungsdatei der labwc-Benutzerkopie
(Sprache und Region, Tastaturbelegungen), das Protokoll der App, welche
Kataloge die App eingeblendet hat (/proc/<pid>/maps) und mit welcher Sprache
und welchen Formaten sie nach einer Neuanmeldung läuft. Schreibt eine TSV;
Exit 0 nur ohne FAIL.

  python3 -I werkzeug/auswerten-010.py <ziel.tsv> [<präfix>]
"""
import os
import re
import sys

P = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
N = os.path.join(P, "nachweis")
ZIEL = sys.argv[1]
PRAEFIX = sys.argv[2] if len(sys.argv) > 2 else "010"
zeilen = []
DE_MO = "share/locale/de/LC_MESSAGES/querencia-einstellungen.mo"
EN_MO = "share/locale/en/LC_MESSAGES/querencia-einstellungen.mo"
SPRACHE = ["LANG", "LANGUAGE", "LC_TIME", "LC_NUMERIC", "LC_MONETARY", "LC_PAPER", "LC_MEASUREMENT"]
FORMATE = SPRACHE[2:]
XKB = ["XKB_DEFAULT_LAYOUT", "XKB_DEFAULT_VARIANT", "XKB_DEFAULT_OPTIONS"]


def pruef(lauf, fall, ok, detail=""):
    zeilen.append((lauf, fall, "PASS" if ok else "FAIL", str(detail).replace("\t", " ").replace("\n", " ")[:300]))


def lesen(*teile):
    try:
        with open(os.path.join(*teile), encoding="utf-8") as f:
            return f.read()
    except OSError:
        return None


def env(text, schluessel):
    m = re.search(rf"^{re.escape(schluessel)}=(.*)$", text or "", re.M)
    return m.group(1).strip() if m else None


def nur(text, erwartet, schluessel):
    """Genau diese Werte der Schlüsselliste stehen in der Datei, die übrigen fehlen."""
    ist = {k: env(text, k) for k in schluessel}
    soll = {k: erwartet.get(k) for k in schluessel}
    return ist == soll, " ".join(f"{k}={v}" for k, v in ist.items() if v is not None) or "(keine)"


def protokoll(lauf):
    return lesen(N, lauf, "logs", "app-direkt.log") or ""


def lauf_ok(lauf):
    e = lesen(N, lauf, "logs", "eingabe.log") or ""
    ok = "szenario exit=0" in e
    pruef(lauf, "Szenario vollständig", ok, e.strip().splitlines()[-1] if e.strip() else "fehlt")
    if ok:
        p = protokoll(lauf)
        pruef(lauf, "kein Absturz der App", "panicked" not in p and "CRITICAL" not in p,
              next((z for z in p.splitlines() if "panicked" in z or "CRITICAL" in z), ""))
    return ok


def bilder(lauf):
    try:
        return sorted(b for b in os.listdir(os.path.join(N, lauf)) if b.endswith(".png"))
    except OSError:
        return []


def uebernommen(lauf):
    """Protokollzeilen „… übernommen: …“ der labwc-Seiten."""
    return [z for z in protokoll(lauf).splitlines() if " übernommen: " in z]


for a in ("1920", "1024"):
    # --- a1: alle 13 Seiten, deutsch und englisch; Kataloge in /proc/<pid>/maps
    for spr, mo, fremd in (("de", DE_MO, EN_MO), ("en", EN_MO, DE_MO)):
        lauf = f"{PRAEFIX}-a1-{spr}-{a}"
        if lauf_ok(lauf):
            b = [x for x in bilder(lauf) if re.match(r"\d\d-.*\.png$", x)]
            pruef(lauf, "13 Seiten fotografiert", len(b) == 13, str(len(b)))
            m = lesen(N, lauf, "maps.txt") or ""
            pruef(lauf, f"Katalog {spr} eingeblendet", mo in m, m.strip())
            pruef(lauf, f"anderer Katalog nicht eingeblendet", fremd not in m, m.strip())

    # --- sp1: Sprache und Region (deutsche Oberfläche, Benutzerkopie, labwc liest sie)
    for name in ("sp1", "o1"):
        lauf = f"{PRAEFIX}-{name}-{a}"
        if not lauf_ok(lauf):
            continue
        e1, e2, e3 = (lesen(N, lauf, f"env-{i}.txt") for i in (1, 2, 3))
        ok, d = nur(e1, {"LANG": "de_DE.UTF-8"}, SPRACHE)
        pruef(lauf, "Formate Deutschland: nur LANG=de_DE.UTF-8", ok, d)
        ok, d = nur(e2, {"LANGUAGE": "en", **{k: "de_CH.UTF-8" for k in FORMATE}}, SPRACHE)
        pruef(lauf, "English + Formate Schweiz: LANGUAGE=en, LC_*=de_CH.UTF-8, kein LANG", ok, d)
        ok, d = nur(e3, {}, SPRACHE)
        pruef(lauf, "Standard: keine eigenen Angaben mehr", ok, d)
        u = [z for z in uebernommen(lauf) if "Sprache und Region übernommen" in z]
        pruef(lauf, "dreimal übernommen", len(u) == 3, str(len(u)))
        pruef(lauf, "Hinweis „wirksam ab der nächsten Anmeldung“, kein labwc-Neuladen",
              all("wirksam ab der nächsten Anmeldung" in z and "neu geladen" not in z for z in u) and u,
              u[0][-120:] if u else "")
        vorher = lesen(N, lauf, "labwc-vorher", "rc.xml") if name == "sp1" else None
        if vorher is not None:
            pruef(lauf, "rc.xml unverändert", all(lesen(N, lauf, f"rc-{i}.xml") == vorher for i in (1, 2, 3)))
        if name == "o1":
            pruef(lauf, "ohne Benutzerkopie: Datei angelegt", e1 is not None and "LANG=de_DE.UTF-8" in e1, (e1 or "fehlt")[:120])

    # --- sp2: englische Oberfläche, zurück auf Deutsch + Formate Deutschland
    lauf = f"{PRAEFIX}-sp2-{a}"
    if lauf_ok(lauf):
        e1 = lesen(N, lauf, "env-1.txt")
        ok, d = nur(e1, {"LANG": "de_DE.UTF-8"}, SPRACHE)
        pruef(lauf, "Deutsch + Formate Deutschland: LANGUAGE=en entfernt, LANG=de_DE.UTF-8", ok, d)
        m = lesen(N, lauf, "maps.txt") or ""
        pruef(lauf, "englischer Katalog eingeblendet", EN_MO in m and DE_MO not in m, m.strip())

    # --- n1: neue Sitzung mit der Datei aus sp1 Schritt 2
    lauf = f"{PRAEFIX}-n1-{a}"
    if lauf_ok(lauf):
        u = lesen(N, lauf, "app-umgebung.txt") or ""
        pruef(lauf, "App läuft mit LANGUAGE=en", env(u, "LANGUAGE") == "en", u.replace("\n", " "))
        pruef(lauf, "App läuft mit Formaten de_CH.UTF-8",
              all(env(u, k) == "de_CH.UTF-8" for k in FORMATE), u.replace("\n", " "))
        m = lesen(N, lauf, "maps.txt") or ""
        pruef(lauf, "englischer Katalog eingeblendet", EN_MO in m and DE_MO not in m, m.strip())

    # --- tb1: zweite Belegung, Reihenfolge, Umschalttaste, entfernen, Standard
    lauf = f"{PRAEFIX}-tb1-{a}"
    if lauf_ok(lauf):
        e = [lesen(N, lauf, f"env-{i}.txt") for i in (1, 2, 3, 4)]
        ok, d = nur(e[0], {"XKB_DEFAULT_LAYOUT": "de,ch", "XKB_DEFAULT_OPTIONS": "grp:alt_shift_toggle"}, XKB)
        pruef(lauf, "hinzugefügt: de,ch mit Alt+Umschalt", ok, d)
        ok, d = nur(e[1], {"XKB_DEFAULT_LAYOUT": "ch,de", "XKB_DEFAULT_OPTIONS": "grp:ctrl_shift_toggle"}, XKB)
        pruef(lauf, "nach oben + Strg+Umschalt: ch,de", ok, d)
        ok, d = nur(e[2], {"XKB_DEFAULT_LAYOUT": "ch"}, XKB)
        pruef(lauf, "entfernt: nur ch, Umschalttaste weg", ok, d)
        ok, d = nur(e[3], {}, XKB)
        pruef(lauf, "Standard: keine eigenen XKB-Angaben (Vorgabe de)", ok, d)
        u = [z for z in uebernommen(lauf) if "Tastatur übernommen" in z]
        pruef(lauf, "viermal übernommen, labwc jedes Mal neu geladen",
              len(u) == 4 and all("labwc hat die Konfiguration neu geladen" in z for z in u), str(len(u)))
        vorher = lesen(N, lauf, "labwc-vorher", "rc.xml")
        pruef(lauf, "rc.xml unverändert (nur Umgebung geändert)",
              all(lesen(N, lauf, f"rc-{i}.xml") == vorher for i in (1, 2, 3, 4)))

    # --- d1: eigenständige Fenster deutsch und englisch (Katalog in /proc/<pid>/maps)
    for f in ("hintergrund", "bildschirm", "arbeitsflaeche", "standardanwendungen"):
        for spr, mo, fremd in (("de", DE_MO, EN_MO), ("en", EN_MO, DE_MO)):
            lauf = f"{PRAEFIX}-d1-{f}-{spr}-{a}"
            if lauf_ok(lauf):
                m = lesen(N, lauf, "maps.txt") or ""
                pruef(lauf, f"Fenster --{f} offen, Katalog {spr}", "01-fenster.png" in bilder(lauf) and mo in m and fremd not in m, m.strip())

with open(ZIEL, "w", encoding="utf-8") as f:
    f.write("lauf\tfall\tergebnis\tdetail\n")
    for z in zeilen:
        f.write("\t".join(z) + "\n")
fails = [z for z in zeilen if z[2] == "FAIL"]
print(f"{len(zeilen) - len(fails)}/{len(zeilen)} PASS")
for z in fails:
    print("FAIL", *z)
sys.exit(1 if fails else 0)
