#!/usr/bin/env python3
"""Einstellungen 0.1.4 (Hintergründe 0.2): WCAG-2.x-Kontrast der
Beschriftungen in der Galerie „Eingebaute Bilder“, hell und dunkel, mit der
Formel aus kontrast.py des Thema-Worktrees
(~/projects/querencia-thema-wt-02/werkzeug/kontrast.py: verhaeltnis).

Die Galerie hat keine eigenen Farben; die Namen der Kacheln zeichnet das
Thema (Querencia-Klassisch 0.2.1, Live-Ablage, nur gelesen):
  Kachelname        Themenschrift auf dem Fenstergrund (wie kontrast-013.py)
  gewählte Kachel   `flowbox > flowboxchild:selected`: Schrift auf dem
                    Verlauf, gegen beide Enden geprüft
Mindestwert AA 4,5:1. Farben der gewählten Kachel liest das Skript aus
gtk.css bzw. gtk-dark.css (letzte Regel mit `flowboxchild:selected` und
`color`/`linear-gradient`) und bricht ab, wenn eine fehlt.

  python3 -I werkzeug/kontrast-014.py [--markdown]
Exit 1, wenn ein Paar verfehlt oder eine Farbe fehlt.
"""
import importlib.util
import os
import re
import sys
from pathlib import Path

sys.dont_write_bytecode = True  # nichts in den Thema-Worktree schreiben
THEMA_KONTRAST = Path(os.environ.get("THEMA_WT", Path.home() / "projects/querencia-thema-wt-02")) / "werkzeug/kontrast.py"
THEMA_ORDNER = Path(os.environ.get("THEMA", Path.home() / "opt/querencia-theme/22824d05ceb49cad/Querencia-Klassisch"))
_spec = importlib.util.spec_from_file_location("kontrast_thema", THEMA_KONTRAST)
kt = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(kt)

# Themenfarben (Querencia-Klassisch 0.2/0.2.1, wie kontrast.py und kontrast-013.py)
THEMA = {
    "hell": {"schrift": "#3c3c3c", "fenster": "#f2f1f0", "css": "gtk.css"},
    "dunkel": {"schrift": "#dfdbd2", "fenster": "#3c3b37", "css": "gtk-dark.css"},
}


def auswahl(css_name):
    """Schrift und Verlaufsenden der letzten Regel für flowboxchild:selected."""
    css = (THEMA_ORDNER / "gtk-4.0" / css_name).read_text()
    treffer = None
    for sel, rumpf in re.findall(r"([^{}]+)\{([^{}]*)\}", css):
        if "flowboxchild:selected" in sel and "linear-gradient" in rumpf and re.search(r"(^|[;\s])color:", rumpf):
            treffer = rumpf
    if treffer is None:
        sys.exit(f"FAIL keine Regel flowboxchild:selected mit Verlauf in {css_name}")
    farbe = re.search(r"(?:^|[;\s])color:\s*(#[0-9a-fA-F]{6})", treffer).group(1).lower()
    enden = [e.lower() for e in re.findall(r"#[0-9a-fA-F]{6}", re.search(r"linear-gradient\(([^)]*)\)", treffer).group(1))]
    return farbe, enden


def main():
    md = "--markdown" in sys.argv
    zeilen = []
    for art, t in THEMA.items():
        zeilen.append((art, "Kachelname auf dem Fenstergrund", t["schrift"], t["fenster"]))
        farbe, enden = auswahl(t["css"])
        for e in enden:
            zeilen.append((art, "Name der gewählten Kachel auf dem Auswahlverlauf", farbe, e))
    if md:
        print(f"Formel: {THEMA_KONTRAST}; Thema: {THEMA_ORDNER}\n")
        print("| Fassung | Paar | Vordergrund | Grund | Kontrast | Mindestens | Ergebnis |\n|---|---|---|---|---|---|---|")
    schlecht = 0
    for art, name, v, g in zeilen:
        k = kt.verhaeltnis(v, g)
        ok = k >= 4.5
        schlecht += not ok
        k_txt = f"{k:.2f}:1".replace(".", ",")
        if md:
            print(f"| {art} | {name} | `{v}` | `{g}` | {k_txt} | 4.5:1 | {'PASS' if ok else 'FAIL'} |")
        else:
            print(f"{'PASS' if ok else 'FAIL'} {art:6} {k_txt:>8} (≥ 4.5) {name}: {v} auf {g}")
    return 1 if schlecht else 0


if __name__ == "__main__":
    sys.exit(main())
