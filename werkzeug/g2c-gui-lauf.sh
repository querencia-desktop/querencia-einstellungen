#!/bin/bash
# G2-C: komplette isolierte GUI-Prüfung mit dem Release-Build und dem
# QE-01-Desktop: r9–r12 (16), r1–r8 (40), r13/r14 (Direktstart, 5).
# Produktiver Stand vorher/nachher; Auswertung mit PIL aus dem Boot-Splash-Projekt.
set -uo pipefail
P=~/projects/querencia-werkstatt-w6-einstellungen
cd "$P"
# Standard: Release 0.0.1 (G2-C); mit QE_APP_BIN=<pfad> ein anderes Binary (G2-G: 0.0.2)
export QE_APP_BIN=${QE_APP_BIN:-$P/nachweis/release/querencia-einstellungen}
export QE_DESKTOP=~/projects/querencia-werkstatt-w1-desktop/qe01/nachweis/bin/querencia-desktop-qe01 QE_DESKTOP_SHA16=9a599a8b521a5a4d
echo "App: $QE_APP_BIN $(sha256sum "$QE_APP_BIN" | cut -c1-16)"
werkzeug/produktiv-stand.sh > nachweis/produktiv-vorher-g2c.txt
for r in r9-farbe-uebernehmen r10-zentrieren r11-kacheln r12-menue; do bash werkzeug/isoliert.sh $r 1920x1080 szenarien/$r.txt; done
bash werkzeug/isoliert.sh r1-bild 1920x1080 szenarien/r1-bild.txt
bash werkzeug/isoliert.sh r2-farbe 1920x1080 szenarien/r2-farbe.txt nachweis/r1-bild/desktop.ini.nachher
bash werkzeug/isoliert.sh r3-standard 1920x1080 szenarien/r3-standard.txt nachweis/r1-bild/desktop.ini.nachher
bash werkzeug/isoliert.sh r4-fehlend 1920x1080 szenarien/r4-fehler.txt szenarien/r4-fehlend.ini
bash werkzeug/isoliert.sh r5-kaputt 1920x1080 szenarien/r4-fehler.txt szenarien/r5-kaputt.ini
bash werkzeug/isoliert.sh r6-1024 1024x768 szenarien/r6-1024.txt nachweis/r1-bild/desktop.ini.nachher
bash werkzeug/isoliert.sh r7-kacheln-webp 1920x1080 szenarien/r7-kacheln-webp.txt
bash werkzeug/isoliert.sh r8-konflikt 1920x1080 szenarien/r8-konflikt.txt nachweis/r1-bild/desktop.ini.nachher
QE_APP_ARGS=--hintergrund bash werkzeug/isoliert.sh r13-hintergrund-direkt 1920x1080 szenarien/r13-hintergrund-direkt.txt
QE_APP_ARGS=- bash werkzeug/isoliert.sh r14-kontrollzentrum 1920x1080 szenarien/r14-kontrollzentrum.txt
werkzeug/produktiv-stand.sh > nachweis/produktiv-nachher-g2c.txt
if diff nachweis/produktiv-vorher-g2c.txt nachweis/produktiv-nachher-g2c.txt; then echo "PRODUKTIV UNVERÄNDERT"; else echo "PRODUKTIV VERÄNDERT"; fi
. ~/projects/querencia-boot-splash/werkzeug/pil-umgebung.sh
python3 werkzeug/auswerten-nachpruefung.py nachpruefung-release-g2c.tsv; echo "nachpruefung exit=$?"
python3 werkzeug/auswerten.py; echo "regression exit=$?"; cp nachweis/pruefungen.tsv nachweis/regression-release-g2c.tsv
python3 werkzeug/auswerten-g2c.py g2c-direktstart.tsv; echo "direktstart exit=$?"
echo "PASS-Zähler: nachpruefung $(grep -c PASS nachweis/nachpruefung-release-g2c.tsv)/$(($(wc -l < nachweis/nachpruefung-release-g2c.tsv)-1)), regression $(grep -c PASS nachweis/regression-release-g2c.tsv)/$(($(wc -l < nachweis/regression-release-g2c.tsv)-1)), direktstart $(grep -c PASS nachweis/g2c-direktstart.tsv)/$(($(wc -l < nachweis/g2c-direktstart.tsv)-1))"
