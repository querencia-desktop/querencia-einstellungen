#!/usr/bin/env python3
"""Einstellungen 0.1.3 (QD-089): WCAG-2.x-Kontrast der eigenen Farben,
hell und dunkel, mit der Formel aus kontrast.py des Thema-Worktrees
(~/projects/querencia-thema-wt-02/werkzeug/kontrast.py: verhaeltnis,
mischen; Themenfarben wie dort).

  Anordnungsfläche (bildschirm.rs HELL/DUNKEL): Text AA 4,5:1, Rand gegen
  die Fläche 3:1 (WCAG 1.4.11)
  Kürzel der Tastaturbelegung (tastatur.rs, Pango alpha) auf Fenstergrund
  und auf Ansichtsgrund, hell und dunkel: AA 4,5:1

Die Farben liest das Skript aus bildschirm.rs bzw. tastatur.rs und bricht
ab, wenn eine fehlt. Mit --alt die Werte bis 0.1.2 (nur zur Information).

  python3 -I werkzeug/kontrast-013.py [--markdown] [--alt]
Exit 1, wenn ein Pflichtpaar verfehlt oder eine Farbe fehlt.
"""
import importlib.util
import os
import re
import sys
from pathlib import Path

sys.dont_write_bytecode = True  # nichts in den Thema-Worktree schreiben
SRC = Path(__file__).resolve().parent.parent / "crates/querencia-einstellungen/src"
THEMA_KONTRAST = Path(os.environ.get("THEMA_WT", Path.home() / "projects/querencia-thema-wt-02")) / "werkzeug/kontrast.py"
_spec = importlib.util.spec_from_file_location("kontrast_thema", THEMA_KONTRAST)
kt = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(kt)

# Themenfarben (Querencia-Klassisch 0.2/0.2.1, wie kontrast.py)
THEMA = {
    "hell": {"schrift": "#3c3c3c", "fenster": "#f2f1f0", "ansicht_schrift": "#1a1a1a", "ansicht": "#ffffff"},
    "dunkel": {"schrift": "#dfdbd2", "fenster": "#3c3b37", "ansicht_schrift": "#f2f1f0", "ansicht": "#2b2a27"},
}
# (Element, Vordergrund, Grund, Mindestwert)
PAARE = [
    ("„Keine Ausgabe eingeschaltet“ auf der Fläche", "leer", "flaeche", 4.5),
    ("Name/Größe der gewählten Ausgabe", "text_gewaehlt", "gewaehlt", 4.5),
    ("Name/Größe einer anderen Ausgabe", "text", "ausgabe", 4.5),
    ("Rand der Ausgabe gegen die Fläche", "rand", "flaeche", 3.0),
]


def farben(name):
    """Farben des Blocks `const <name>: Farben` aus bildschirm.rs als #rrggbb."""
    q = (SRC / "bildschirm.rs").read_text()
    m = re.search(rf"const {name}: Farben = Farben \{{(.*?)\n\}};", q, re.S)
    if not m:
        sys.exit(f"FAIL Block {name} nicht in bildschirm.rs")
    f = {}
    for k, v in re.findall(r"(\w+): (.+?),\n", m.group(1) + "\n"):
        if v.startswith("rgb("):
            r, g, b = (int(x, 16) for x in re.findall(r"0x([0-9a-f]{2})", v))
        else:
            r, g, b = (round(float(x) * 255) for x in v.strip("()").split(","))
        f[k] = f"#{r:02x}{g:02x}{b:02x}"
    return f


def kuerzel_alpha():
    """Deckkraft des Kürzels aus tastatur.rs (genau ein alpha-Wert)."""
    werte = set(re.findall(r'alpha=\\"(\d+)%\\"', (SRC / "tastatur.rs").read_text()))
    if len(werte) != 1:
        sys.exit(f"FAIL Kürzel-Deckkraft in tastatur.rs nicht eindeutig: {sorted(werte)}")
    return int(werte.pop()) / 100


def main():
    md = "--markdown" in sys.argv
    alt = "--alt" in sys.argv
    zeilen, schlecht = [], 0
    for art, block in (("hell", "HELL"), ("dunkel", "DUNKEL")):
        f = farben(block)
        if alt and art == "hell":
            f["gewaehlt"] = "#3685e3"
        for name, v, g, mind in PAARE:
            k = kt.verhaeltnis(f[v], f[g])
            zeilen.append((art, name, f[v], f[g], k, mind))
        a = 0.60 if alt and art == "hell" else kuerzel_alpha()
        t = THEMA[art]
        for grund_name, fg, bg in (("Fenstergrund", t["schrift"], t["fenster"]),
                                   ("Ansichtsgrund", t["ansicht_schrift"], t["ansicht"])):
            gemischt = kt.mischen(f"{fg}@{a}", bg)
            k = kt.verhaeltnis(gemischt, bg)
            zeilen.append((art, f"Kürzel der Tastaturbelegung ({fg} mit {round(a * 100)} %) auf {grund_name}",
                           gemischt, bg, k, 4.5))
    if md:
        print(f"Formel: {THEMA_KONTRAST}{' – Werte bis 0.1.2' if alt else ''}\n")
        print("| Fassung | Paar | Vordergrund | Grund | Kontrast | Mindestens | Ergebnis |\n|---|---|---|---|---|---|---|")
    for art, name, v, g, k, mind in zeilen:
        ok = k >= mind
        schlecht += not ok
        k_txt = f"{k:.2f}:1".replace(".", ",")
        if md:
            print(f"| {art} | {name} | `{v}` | `{g}` | {k_txt} | {mind}:1 | {'PASS' if ok else 'FAIL'} |")
        else:
            print(f"{'PASS' if ok else 'FAIL'} {art:6} {k_txt:>8} (≥ {mind}) {name}: {v} auf {g}")
    if alt:
        return 0
    return 1 if schlecht else 0


if __name__ == "__main__":
    sys.exit(main())
