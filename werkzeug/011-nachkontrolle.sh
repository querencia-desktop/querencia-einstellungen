#!/bin/bash
# Einstellungen 0.1.1: Nachkontrolle nach der Aktivierung (21.10.2026) mit der
# INSTALLIERTEN Binary (Ziel des Live-Links ~/opt/querencia/bin/querencia-einstellungen,
# nur über readlink bestimmt, nie im äußeren Shell gestartet). Jeder Start läuft in
# werkzeug/isoliert.sh (eigenes HOME-Profil/XDG_*, eigener Bus, headless labwc).
# Läufe nach nachweis/nk-<lauf>-<b>/, Auswertung nach nachweis/011/nachkontrolle/.
#
#   bash werkzeug/011-nachkontrolle.sh
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
Z=szenarien/011
K=nachweis/011/nachkontrolle
V548=$P/$Z/vorgabe-548483c
EN=$P/szenarien/010/daten-en
export QE_APP_BIN=$(readlink -f "$HOME/opt/querencia/bin/querencia-einstellungen")
mkdir -p "$K"
echo "App (installiert): $QE_APP_BIN $(sha256sum "$QE_APP_BIN" | cut -d' ' -f1)"
werkzeug/produktiv-stand.sh > "$K/produktiv-vorher.txt"
gsettings list-recursively org.gnome.desktop.interface > "$K/gsettings-live-vorher.txt"
QE_LABWC_VORGABE_QUELLE=$V548 bash werkzeug/g2aj-lauf.sh nk-r1-548483c-1920 1920x1080 $Z/r1-rueckstellung-1920.txt
QE_LABWC_VORGABE_QUELLE=$V548 bash werkzeug/g2aj-lauf.sh nk-r1-548483c-1024 1024x768 $Z/r1-rueckstellung-1024.txt
QE_LABWC_VORGABE_QUELLE=$V548 QE_LABWC=ohne bash werkzeug/g2aj-lauf.sh nk-r1-ohne-1920 1920x1080 $Z/r1-rueckstellung-ohne-1920.txt
QE_LABWC_VORGABE_QUELLE=$V548 bash werkzeug/g2aj-lauf.sh nk-r2-1920 1920x1080 $Z/r2-standard-1920.txt
QE_LABWC_VORGABE_QUELLE=$V548 QE_LABWC_DATEIEN=$P/$Z/nachkontrolle/daten-rotation bash werkzeug/g2aj-lauf.sh nk-rot-1920 1920x1080 $Z/nachkontrolle/rot-1920.txt
bash werkzeug/g2aj-lauf.sh nk-eb1-de-1920 1920x1080 $Z/eb1-erscheinungsbild-1920.txt
QE_LABWC_DATEIEN=$EN bash werkzeug/g2aj-lauf.sh nk-eb1-en-1920 1920x1080 $Z/eb1-erscheinungsbild-1920.txt
bash werkzeug/g2aj-lauf.sh nk-a1-de-1920 1920x1080 $Z/a1-alle-seiten-1920.txt
QE_LABWC_DATEIEN=$EN bash werkzeug/g2aj-lauf.sh nk-a1-en-1920 1920x1080 $Z/a1-alle-seiten-1920.txt
werkzeug/produktiv-stand.sh > "$K/produktiv-nachher.txt"
gsettings list-recursively org.gnome.desktop.interface > "$K/gsettings-live-nachher.txt"
if diff "$K/produktiv-vorher.txt" "$K/produktiv-nachher.txt" && diff "$K/gsettings-live-vorher.txt" "$K/gsettings-live-nachher.txt"; then
    echo "PRODUKTIV UNVERÄNDERT"
else
    echo "PRODUKTIV VERÄNDERT"
fi
python3 -I werkzeug/auswerten-011-nachkontrolle.py "$K/nachkontrolle.tsv"; echo "auswertung exit=$?"
