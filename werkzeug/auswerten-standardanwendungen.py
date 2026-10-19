#!/usr/bin/env python3
"""G2-Z: Wertet die isolierten Läufe nachweis/g2z-z*/ aus (Bereich
„Standardanwendungen“): mimeapps.list vorher/nachher, Meldungen der App,
Bilder. Ergebnis: TSV (argv[1]) und Ausgabe PASS/FAIL je Prüfung."""
import os
import sys

P = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
N = os.path.join(P, "nachweis")
ziel = sys.argv[1] if len(sys.argv) > 1 else os.path.join(N, "g2z", "standardanwendungen-g2z.tsv")

EDITOR = "org.querencia.Editor.desktop"
BILDER = "org.querencia.Bilder.desktop"
TEXT = ["text/plain", "text/markdown", "text/x-markdown"]
BILDTYPEN = ["image/png", "image/jpeg", "image/gif", "image/webp", "image/bmp", "image/svg+xml", "image/tiff"]


def lesen(lauf, name):
    try:
        with open(os.path.join(N, "g2z-" + lauf, name), encoding="utf-8", errors="replace") as f:
            return f.read()
    except FileNotFoundError:
        return None


def gruppe(text, name="Default Applications"):
    aus, drin = {}, False
    for z in (text or "").splitlines():
        z = z.strip()
        if not z or z.startswith("#"):
            continue
        if z.startswith("[") and z.endswith("]"):
            drin = z[1:-1] == name
            continue
        if drin and "=" in z:
            k, v = z.split("=", 1)
            aus[k.strip()] = [i for i in (s.strip() for s in v.split(";")) if i]
    return aus


def meldungen(lauf):
    return [z for z in (lesen(lauf, "logs/app-direkt.log") or "").splitlines() if z.startswith("standardanwendungen: ")]


def bild(lauf, name):
    return os.path.isfile(os.path.join(N, "g2z-" + lauf, name))


def szenario_ok(lauf):
    return "szenario exit=0" in (lesen(lauf, "logs/eingabe.log") or "")


def ohne_absturz(lauf):
    t = lesen(lauf, "logs/app-direkt.log") or ""
    return not any(w in t for w in ("panicked", "CRITICAL", "Segmentation"))


zeilen = []


def pruefe(nr, text, ok, beleg=""):
    zeilen.append((nr, "PASS" if ok else "FAIL", text, beleg))


# z1: ohne Datei → Vim (nur text/plain), dann Vorschlag Texteditor (alle drei Texttypen)
n = lesen("z1-vorschlag-1920", "mimeapps.list.nachher")
d = gruppe(n)
m = meldungen("z1-vorschlag-1920")
pruefe("Z1-1", "Szenario lief, kein Absturz", szenario_ok("z1-vorschlag-1920") and ohne_absturz("z1-vorschlag-1920"))
pruefe("Z1-2", "mimeapps.list im Profil angelegt", n is not None)
pruefe("Z1-3", "text/plain, text/markdown, text/x-markdown → Texteditor", all(d.get(t) == [EDITOR] for t in TEXT), str({t: d.get(t) for t in TEXT}))
pruefe("Z1-4", "nur Texttypen als Standard geschrieben", set(d) == set(TEXT), ",".join(sorted(d)))
pruefe("Z1-5", "Meldung Vim nur für text/plain", any("„Vim“ ist jetzt Standard für text/plain." in z for z in m), " | ".join(m))
pruefe("Z1-6", "Meldung Texteditor für alle drei Texttypen", any("„Texteditor“ ist jetzt Standard für text/plain, text/markdown, text/x-markdown." in z for z in m))
pruefe("Z1-7", "im Profil keine weitere *mimeapps.list", (lesen("z1-vorschlag-1920", "konfig-wurzel-nachher.txt") or "").split().count("mimeapps.list") == 1
       and not any(x.endswith("-mimeapps.list") for x in (lesen("z1-vorschlag-1920", "konfig-wurzel-nachher.txt") or "").split()))
pruefe("Z1-8", "Belegbilder Start, Dialog, Ende", all(bild("z1-vorschlag-1920", b) for b in ("01-start.png", "02-dialog.png", "04-dialog-vim-aktuell.png", "05-editor.png")))

# z2: Abbrechen, Escape, Übernehmen ohne Änderung → Datei byte-gleich, keine Meldung
v, n = lesen("z2-abbrechen-1920", "mimeapps.list.vorher"), lesen("z2-abbrechen-1920", "mimeapps.list.nachher")
pruefe("Z2-1", "Szenario lief, kein Absturz", szenario_ok("z2-abbrechen-1920") and ohne_absturz("z2-abbrechen-1920"))
pruefe("Z2-2", "Abbrechen/Escape/gleiche Wahl: mimeapps.list byte-gleich", v is not None and v == n)
pruefe("Z2-3", "keine Schreibmeldung", meldungen("z2-abbrechen-1920") == [], " | ".join(meldungen("z2-abbrechen-1920")))
pruefe("Z2-4", "Belegbilder der drei Dialoge", all(bild("z2-abbrechen-1920", b) for b in ("02-bilder-dialog.png", "03-dateimanager-dialog.png", "04-email-dialog.png")))

# z3: Bilder → Firefox → zurück; fremde Einträge bleiben
v, n = gruppe(lesen("z3-bilder-1920", "mimeapps.list.vorher")), gruppe(lesen("z3-bilder-1920", "mimeapps.list.nachher"))
m = meldungen("z3-bilder-1920")
pruefe("Z3-1", "Szenario lief, kein Absturz", szenario_ok("z3-bilder-1920") and ohne_absturz("z3-bilder-1920"))
pruefe("Z3-2", "zwei Schreibmeldungen (Firefox, Bildbetrachter)", len(m) == 2 and "„Firefox“" in m[0] and "„Bildbetrachter“" in m[1], " | ".join(m))
pruefe("Z3-3", "alle Einträge von vorher gleich", all(n.get(k) == w for k, w in v.items()), str({k: (w, n.get(k)) for k, w in v.items() if n.get(k) != w}))
pruefe("Z3-4", "Bildtypen wieder beim Bildbetrachter", all(n.get(t) == [BILDER] for t in BILDTYPEN[:6]))
pruefe("Z3-5", "Texteditor-Einträge unberührt", all(n.get(t) == [EDITOR] for t in TEXT))

# z4–z6: Kontrollzentrum und Direktstart, nur ansehen
for lauf, bilder in (("z4-kz-1024", ("01-kz-1024.png", "02-dialog-1024.png")), ("z5-kz-1920", ("01-kz-1920.png", "02-dialog-1920.png")), ("z6-direkt-1024", ("01-direkt-1024.png",))):
    v, n = lesen(lauf, "mimeapps.list.vorher"), lesen(lauf, "mimeapps.list.nachher")
    nr = lauf.split("-")[0].upper()
    pruefe(nr + "-1", f"{lauf}: Szenario lief, kein Absturz", szenario_ok(lauf) and ohne_absturz(lauf))
    pruefe(nr + "-2", f"{lauf}: mimeapps.list byte-gleich", v is not None and v == n)
    pruefe(nr + "-3", f"{lauf}: Belegbilder", all(bild(lauf, b) for b in bilder), ",".join(bilder))

# z7: vorrangige x-querencia-mimeapps.list → Warnung, Benutzerdatei trotzdem geschrieben
m = meldungen("z7-vorrang-1920")
n = gruppe(lesen("z7-vorrang-1920", "mimeapps.list.nachher"))
pruefe("Z7-1", "Szenario lief, kein Absturz", szenario_ok("z7-vorrang-1920") and ohne_absturz("z7-vorrang-1920"))
pruefe("Z7-2", "Warnung „vorrangige Liste“ statt Erfolg", len(m) == 1 and "vorrangige Liste" in m[0] and "vim.desktop" in m[0], " | ".join(m))
pruefe("Z7-3", "Benutzerdatei: text/plain → Texteditor", n.get("text/plain") == [EDITOR])
pruefe("Z7-4", "vorrangige Datei unverändert", lesen("z7-vorrang-1920", "profil/config/x-querencia-mimeapps.list") == "[Default Applications]\ntext/plain=vim.desktop\n")

# Produktivstand
a, b = (open(os.path.join(N, "g2z", f), encoding="utf-8").read() if os.path.exists(os.path.join(N, "g2z", f)) else None for f in ("produktiv-vorher-g2z.txt", "produktiv-nachher-g2z.txt"))
pruefe("P-1", "produktive Sitzung, ~/.config/mimeapps.list und Starter unverändert", a is not None and a == b)

os.makedirs(os.path.dirname(ziel), exist_ok=True)
with open(ziel, "w", encoding="utf-8") as f:
    f.write("nr\tergebnis\tpruefung\tbeleg\n")
    for z in zeilen:
        f.write("\t".join(z) + "\n")
for z in zeilen:
    print(f"{z[1]}  {z[0]}  {z[2]}" + (f"  [{z[3]}]" if z[1] == "FAIL" and z[3] else ""))
fehler = sum(1 for z in zeilen if z[1] != "PASS")
print(f"{len(zeilen) - fehler}/{len(zeilen)} PASS")
sys.exit(1 if fehler else 0)
