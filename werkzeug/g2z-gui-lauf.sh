#!/bin/bash
# G2-Z: isolierte GUI-Prüfung des Bereichs „Standardanwendungen“ (z1–z7) mit dem
# LIVE-Desktop QE-04 und dem Panel der Werkzeugvorlage (nur gelesen). GIO schreibt
# nur in die mimeapps.list des Wegwerfprofils (XDG_CONFIG_HOME); die Starter liest
# es aus ~/.local/share/applications. Läufe nach nachweis/g2z-z*/; Auswertung
# nach nachweis/g2z/standardanwendungen-g2z.tsv.
#
#   QE_APP_BIN=<release 0.0.5> bash werkzeug/g2z-gui-lauf.sh
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
Z=szenarien/standardanwendungen
export QE_APP_BIN=${QE_APP_BIN:-$P/nachweis/g2z/release/querencia-einstellungen}
export QE_DESKTOP=$HOME/opt/querencia/desktop/2c78da3ad2b68a30/querencia-desktop QE_DESKTOP_SHA16=2c78da3ad2b68a30
ECHT=$Z/echt-20261019.list
mkdir -p nachweis/g2z
echo "App: $QE_APP_BIN $(sha256sum "$QE_APP_BIN" | cut -c1-16) ($(bash "$P/werkzeug/version-isoliert.sh" "$QE_APP_BIN"))"
echo "Desktop: $QE_DESKTOP ($QE_DESKTOP_SHA16)"
werkzeug/produktiv-stand.sh > nachweis/g2z/produktiv-vorher-g2z.txt
l() { local n=$1 a=$2 s=$3 sz=$4; QE_APP_ARGS=$a bash werkzeug/isoliert.sh "g2z-$n" "$s" "$Z/$sz" >/dev/null 2>&1; echo "Lauf $n: $(tail -1 nachweis/g2z-$n/logs/eingabe.log)"; }
# z1: ohne mimeapps.list – Vim, dann den Vorschlag Texteditor festlegen
l z1-vorschlag-1920 --standardanwendungen 1920x1080 z1-vorschlag-1920.txt
# z2–z6: Kopie der echten Datei vom 19.10.
QE_MIMEAPPS=$ECHT l z2-abbrechen-1920 --standardanwendungen 1920x1080 z2-abbrechen-1920.txt
QE_MIMEAPPS=$ECHT l z3-bilder-1920 --standardanwendungen 1920x1080 z3-bilder-1920.txt
QE_MIMEAPPS=$ECHT l z4-kz-1024 - 1024x768 z4-kz-1024.txt
QE_MIMEAPPS=$ECHT l z5-kz-1920 - 1920x1080 z5-kz-1920.txt
QE_MIMEAPPS=$ECHT l z6-direkt-1024 --standardanwendungen 1024x768 z6-direkt-1024.txt
# z7: x-querencia-mimeapps.list legt text/plain auf Vim fest – Warnung statt Erfolg
QE_MIMEAPPS=$ECHT QE_MIMEAPPS_VORRANG=$Z/vorrang-vim.list l z7-vorrang-1920 --standardanwendungen 1920x1080 z7-vorrang-1920.txt
werkzeug/produktiv-stand.sh > nachweis/g2z/produktiv-nachher-g2z.txt
if diff nachweis/g2z/produktiv-vorher-g2z.txt nachweis/g2z/produktiv-nachher-g2z.txt; then echo "PRODUKTIV UNVERÄNDERT"; else echo "PRODUKTIV VERÄNDERT"; fi
python3 werkzeug/auswerten-standardanwendungen.py nachweis/g2z/standardanwendungen-g2z.tsv; echo "auswertung exit=$?"
