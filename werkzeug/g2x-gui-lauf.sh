#!/bin/bash
# G2-X: isolierte GUI-Prüfung des Bereichs „Arbeitsfläche“ (a1–a9) und der
# Hintergrund-Regression mit gemischter desktop.ini (a10, Gegenprobe 0.0.2 a11),
# mit dem LIVE-Desktop QE-03 aus der Ablage (nur gelesen) und dem Panel QREL-04c.
# Läufe nach nachweis/g2x-a*/; Auswertung nach nachweis/g2x/arbeitsflaeche-g2x.tsv.
#
#   QE_APP_BIN=<release 0.0.3> bash werkzeug/g2x-gui-lauf.sh
set -uo pipefail
P=~/projects/querencia-einstellungen
cd "$P"
Z=szenarien/arbeitsflaeche
export QE_APP_BIN=${QE_APP_BIN:-$P/nachweis/g2x/release/querencia-einstellungen}
export QE_DESKTOP=~/opt/querencia/desktop/9f4a9dca39014f0f/querencia-desktop QE_DESKTOP_SHA16=9f4a9dca39014f0f
ALT_APP=~/opt/querencia/einstellungen/03c6dec5e63966b9/querencia-einstellungen
echo "App: $QE_APP_BIN $(sha256sum "$QE_APP_BIN" | cut -c1-16) ($("$QE_APP_BIN" --version))"
echo "Desktop: $QE_DESKTOP ($QE_DESKTOP_SHA16)"
werkzeug/produktiv-stand.sh > nachweis/g2x/produktiv-vorher-g2x.txt
l() { local n=$1 a=$2 s=$3 sz=$4 ini=${5:-}; QE_APP_ARGS=$a bash werkzeug/isoliert.sh "g2x-$n" "$s" "$Z/$sz" $ini >/dev/null 2>&1; echo "Lauf $n: $(tail -1 nachweis/g2x-$n/logs/eingabe.log)"; }
l a1-aendern-1920 --arbeitsflaeche 1920x1080 a1-aendern-1920.txt
l a2-hintergrund-bleibt --arbeitsflaeche 1920x1080 a2-hintergrund-bleibt.txt $Z/hg-farbe.ini
l a3-standard --arbeitsflaeche 1920x1080 a3-standard.txt $Z/gemischt.ini
l a4-konflikt-volumes --arbeitsflaeche 1920x1080 a4-konflikt-volumes.txt $Z/gemischt.ini
l a5-fremder-hintergrund --arbeitsflaeche 1920x1080 a5-fremder-hintergrund.txt $Z/gemischt.ini
l a6-kaputt --arbeitsflaeche 1920x1080 a6-kaputt.txt $Z/kaputt.ini
l a7-1024 --arbeitsflaeche 1024x768 a7-1024.txt
l a8-kz-1024 - 1024x768 a8-kz-1024.txt
l a9-kz-1920 - 1920x1080 a9-kz-1920.txt
# a10: Hintergrund-Dialog (Szenario r3: Standard wiederherstellen + Übernehmen) auf gemischter Datei
l a10-hintergrund-gemischt "" 1920x1080 ../r3-standard.txt $Z/gemischt.ini
# a11: Gegenprobe mit dem Live-Stand 0.0.2 (schreibt die ganze Datei)
QE_APP_BIN=$ALT_APP l a11-gegenprobe-002 "" 1920x1080 ../r3-standard.txt $Z/gemischt.ini
werkzeug/produktiv-stand.sh > nachweis/g2x/produktiv-nachher-g2x.txt
if diff nachweis/g2x/produktiv-vorher-g2x.txt nachweis/g2x/produktiv-nachher-g2x.txt; then echo "PRODUKTIV UNVERÄNDERT"; else echo "PRODUKTIV VERÄNDERT"; fi
python3 werkzeug/auswerten-arbeitsflaeche.py nachweis/g2x/arbeitsflaeche-g2x.tsv; echo "auswertung exit=$?"
