#!/bin/bash
# G2-G: isolierte Prüfung des Bereichs „Bildschirm“ mit zwei headless-Ausgaben
# (HEADLESS-1 1920×1080 über qausgabe, HEADLESS-2 1280×720), Läufe b0–b11,
# danach Auswertung. QE_APP_BIN wählt das Binary (Standard: Debug).
set -uo pipefail
P=~/projects/querencia-einstellungen
cd "$P"
export QE_DESKTOP=~/projects/querencia-werkstatt-w1-desktop/qe01/nachweis/bin/querencia-desktop-qe01 QE_DESKTOP_SHA16=9a599a8b521a5a4d
echo "App: ${QE_APP_BIN:-$P/target/debug/querencia-einstellungen} $(sha256sum "${QE_APP_BIN:-$P/target/debug/querencia-einstellungen}" | cut -c1-16)"
werkzeug/produktiv-stand.sh > nachweis/produktiv-vorher-g2g.txt
QE_APP_ARGS=- bash werkzeug/isoliert.sh b0-kontrollzentrum-bildschirm 1920x1080 szenarien/b0-kontrollzentrum-bildschirm.txt
for b in b1-aufloesung-rueckfall b2-aufloesung-behalten b3-anordnung b4-skalierung b5-aus-an b6-drehung b7-hauptmonitor b8-zuruecksetzen b9-wiederherstellen-knopf; do
    QE_OUTPUTS=2 QE_APP_ARGS=--bildschirm bash werkzeug/isoliert.sh "$b" 1920x1080 "szenarien/$b.txt"
done
QE_OUTPUTS=2 QE_APP_ARGS=--bildschirm-anwenden QE_BILDSCHIRM_INI=szenarien/bildschirm/b10-bildschirm.ini bash werkzeug/isoliert.sh b10-persistenz 1920x1080 szenarien/b10-persistenz.txt
QE_OUTPUTS=2 QE_APP_ARGS=--bildschirm-anwenden bash werkzeug/isoliert.sh b11-anwenden-ohne-datei 1920x1080 szenarien/b11-anwenden-ohne-datei.txt
werkzeug/produktiv-stand.sh > nachweis/produktiv-nachher-g2g.txt
if diff nachweis/produktiv-vorher-g2g.txt nachweis/produktiv-nachher-g2g.txt; then echo "PRODUKTIV UNVERÄNDERT"; else echo "PRODUKTIV VERÄNDERT"; fi
. ~/projects/querencia-boot-splash/werkzeug/pil-umgebung.sh
python3 werkzeug/auswerten-bildschirm.py bildschirm-g2g.tsv; echo "bildschirm exit=$?"
