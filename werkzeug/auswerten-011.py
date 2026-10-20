#!/usr/bin/python3 -I
"""Einstellungen 0.1.1: Auswertung der GUI-Läufe nachweis/<präfix>-<lauf>-<1920|1024>.

r1 (Werkstatt QREL-06 A17.6): jede der acht Übernahmen steht im Protokoll, die
Änderung ist in rc.xml angekommen (rc-1/3/5/7), jede Rückstellung gibt rc.xml
byte-gleich zurück (rc-2/4/6/8 = rc-0), und am Ende ist die ganze
Benutzerkopie byte-gleich mit dem Stand vor dem Lauf (labwc-vorher =
labwc-nachher, also auch keine Sicherungen). Ohne Benutzerkopie: die
angelegte Kopie gleicht der Vorgabe, nur rc.xml und .vorgabe/rc.xml.
r2: eigene Werte, dann „Standard wiederherstellen“ – wieder byte-gleich.
eb1/a1: Bilder vorhanden, Kataloge eingeblendet. Schreibt eine TSV; Exit 0
nur ohne FAIL.

  python3 -I werkzeug/auswerten-011.py <ziel.tsv> [<präfix>]
"""
import os
import re
import sys

P = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
N = os.path.join(P, "nachweis")
ZIEL = sys.argv[1]
PRAEFIX = sys.argv[2] if len(sys.argv) > 2 else "011"
zeilen = []


def pruef(lauf, fall, ok, detail=""):
    zeilen.append((lauf, fall, "PASS" if ok else "FAIL", str(detail).replace("\t", " ").replace("\n", " ")[:300]))


def lesen(*teile, binaer=False):
    try:
        with open(os.path.join(*teile), "rb" if binaer else "r", **({} if binaer else {"encoding": "utf-8"})) as f:
            return f.read()
    except OSError:
        return None


def baum(ordner):
    """Alle Dateien unter ordner: relativer Name → Inhalt (None, wenn es den Ordner nicht gibt)."""
    if not os.path.isdir(ordner):
        return None
    aus = {}
    for wurzel, _, dateien in os.walk(ordner):
        for d in dateien:
            p = os.path.join(wurzel, d)
            aus[os.path.relpath(p, ordner)] = lesen(p, binaer=True)
    return aus


def lauf_ok(lauf):
    e = lesen(N, lauf, "logs", "eingabe.log") or ""
    ok = "szenario exit=0" in e
    pruef(lauf, "Szenario vollständig", ok, e.strip().splitlines()[-1] if e.strip() else "fehlt")
    app = lesen(N, lauf, "logs", "app-direkt.log") or ""
    schlimm = [z for z in app.splitlines() if re.search(r"CRITICAL|panicked|Segmentation", z)]
    pruef(lauf, "keine CRITICAL/panicked im Programmprotokoll", not schlimm, schlimm[:2])
    return ok, app


def wert(xml, element):
    m = re.findall(rf"<{element}>([^<]*)</{element}>", xml or "")
    return m[-1] if m else None


AENDERUNG = {  # Schritt → (Prüfung auf rc-<n>.xml, Text)
    1: (lambda t: wert(t, "repeatDelay") == "610", "Verzögerung 610 in rc.xml"),
    3: (lambda t: wert(t, "accelProfile") == "flat", "Beschleunigung aus (accelProfile flat)"),
    5: (lambda t: wert(t, "cornerRadius") == "9", "Eckenradius 9"),
    7: (lambda t: 'popupTime="1000"' in (t or ""), "Namen einblenden (popupTime 1000)"),
}


def r1(lauf, ohne=False):
    ok, app = lauf_ok(lauf)
    if not ok:
        return
    uebernahmen = [z for z in app.splitlines() if " übernommen: " in z]
    pruef(lauf, "acht Übernahmen protokolliert", len(uebernahmen) == 8, len(uebernahmen))
    if ohne:
        wirkt = all(("nächsten Anmeldung" in z or "next login" in z) for z in uebernahmen)
        pruef(lauf, "ohne Benutzerkopie: wirksam ab der nächsten Anmeldung", wirkt and len(uebernahmen) == 8)
    else:
        neu = [z for z in uebernahmen if "labwc hat die Konfiguration neu geladen" in z or "labwc has reloaded its configuration" in z]
        pruef(lauf, "labwc --reconfigure nach jeder Übernahme", len(neu) == 8, len(neu))
    rc0 = lesen(N, lauf, "rc-0.xml", binaer=True)
    for n in range(1, 9):
        t = lesen(N, lauf, f"rc-{n}.xml", binaer=True)
        if n in AENDERUNG:
            test, text = AENDERUNG[n]
            pruef(lauf, f"Schritt {n}: {text}", t is not None and test(t.decode()), "")
        elif ohne:
            v = lesen(N, lauf, "profil", "vorgabe", "rc.xml", binaer=True)
            pruef(lauf, f"Schritt {n}: zurückgestellt, rc.xml = Vorgabe (byte-gleich)", t is not None and t == v)
        else:
            pruef(lauf, f"Schritt {n}: zurückgestellt, rc.xml byte-gleich", t is not None and t == rc0)
    vorher, nachher = baum(os.path.join(N, lauf, "labwc-vorher")), baum(os.path.join(N, lauf, "labwc-nachher"))
    sicherungen = sorted(k for k in (nachher or {}) if ".einstellungen-vorher-" in k or ".vorher-" in k)
    if ohne:
        v = lesen(N, lauf, "profil", "vorgabe", "rc.xml", binaer=True)
        soll = {"rc.xml": v, os.path.join(".vorgabe", "rc.xml"): v}
        pruef(lauf, "Benutzerkopie danach: nur rc.xml und .vorgabe/rc.xml, beide = Vorgabe", nachher == soll, sorted(nachher or {}))
    else:
        anders = sorted(k for k in set(vorher or {}) | set(nachher or {}) if (vorher or {}).get(k) != (nachher or {}).get(k))
        pruef(lauf, "Benutzerkopie danach byte-gleich (alle Dateien)", vorher is not None and not anders, anders or f"{len(vorher or {})} Dateien")
    pruef(lauf, "keine Sicherungsdateien übrig", not sicherungen, sicherungen)


def r2(lauf):
    ok, app = lauf_ok(lauf)
    if not ok:
        return
    t1 = (lesen(N, lauf, "rc-1.xml") or "")
    t2 = (lesen(N, lauf, "rc-2.xml") or "")
    pruef(lauf, "eigene Tastaturwerte geschrieben (630 ms, 27/s)", wert(t1, "repeatDelay") == "630" and wert(t1, "repeatRate") == "27")
    pruef(lauf, "eigenes Fensterdesign geschrieben (Radius 7, Schatten)", wert(t2, "cornerRadius") == "7" and wert(t2, "dropShadows") == "yes")
    rc0 = lesen(N, lauf, "rc-0.xml", binaer=True)
    pruef(lauf, "nach Standard + Übernehmen: rc.xml byte-gleich", lesen(N, lauf, "rc-4.xml", binaer=True) == rc0)
    vorher, nachher = baum(os.path.join(N, lauf, "labwc-vorher")), baum(os.path.join(N, lauf, "labwc-nachher"))
    pruef(lauf, "Benutzerkopie danach byte-gleich (auch keine Sicherungen)", vorher is not None and vorher == nachher,
          sorted(k for k in set(vorher or {}) | set(nachher or {}) if (vorher or {}).get(k) != (nachher or {}).get(k)))


def bilder(lauf, namen, sprache=None):
    ok, _ = lauf_ok(lauf)
    if not ok:
        return
    fehlen = [b for b in namen if not os.path.isfile(os.path.join(N, lauf, b))]
    pruef(lauf, f"{len(namen)} Bilder", not fehlen, fehlen)
    if sprache:
        maps = lesen(N, lauf, "maps.txt") or ""
        pruef(lauf, f"Katalog {sprache} eingeblendet", f"share/locale/{sprache}/LC_MESSAGES/querencia-einstellungen.mo" in maps, maps.strip())


SEITEN = ["erscheinungsbild", "hintergrundbild", "symbole", "fensterdesign", "schriften", "panels", "tastatur",
          "maus", "bildschirm", "arbeitsflaechen", "standardanwendungen", "sprache", "informationen"]
for a in ("1920", "1024"):
    for v in ("bb90803", "548483c", "en"):
        r1(f"{PRAEFIX}-r1-{v}-{a}")
    r1(f"{PRAEFIX}-r1-ohne-{a}", ohne=True)
    r2(f"{PRAEFIX}-r2-{a}")
    for s in ("de", "en"):
        bilder(f"{PRAEFIX}-eb1-{s}-{a}", ["01-oben.png", "02-unten-hinweis.png"])
        bilder(f"{PRAEFIX}-a1-{s}-{a}", [f"{i:02d}-{n}.png" for i, n in enumerate(SEITEN, 1)], s)

with open(ZIEL, "w", encoding="utf-8") as f:
    f.write("lauf\tfall\tergebnis\tdetail\n")
    for z in zeilen:
        f.write("\t".join(z) + "\n")
fail = [z for z in zeilen if z[2] != "PASS"]
print(f"{len(zeilen) - len(fail)}/{len(zeilen)} PASS, {len(fail)} FAIL → {ZIEL}")
for z in fail:
    print("FAIL", *z[:2], z[3])
sys.exit(1 if fail else 0)
