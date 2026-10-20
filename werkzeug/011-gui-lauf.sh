#!/bin/bash
# Einstellungen 0.1.1: isolierte GUI-Prüfung in 1920×1080 und 1024×768 unter
# den Bedingungen von G2-AJ/0.1.0 (werkzeug/g2aj-lauf.sh). Kern ist der Ablauf
# von Werkstatt QREL-06 A17.6 (r1): Ändern und Zurückstellen in Tastatur, Maus,
# Fensterdesign und Arbeitsflächen; danach muss die labwc-Benutzerkopie des
# Profils byte-gleich sein (labwc-vorher = labwc-nachher, keine Sicherungen).
# r1 läuft mit beiden festen Vorgaben (bb90803, 548483c), englisch und ohne
# Benutzerkopie; r2 stellt eigene Werte über „Standard wiederherstellen“
# zurück; eb1 zeigt den Hinweis zu gsettings/GNOME; a1 alle Seiten.
# Läufe nach nachweis/<präfix>-<lauf>-<b>/, Auswertung nach nachweis/011/gui-<präfix>.tsv.
#
#   QE_APP_BIN=<release> bash werkzeug/011-gui-lauf.sh [präfix]
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
Z=szenarien/011
V=${1:-011}
export QE_APP_BIN=${QE_APP_BIN:-$P/nachweis/011/release/querencia-einstellungen}
EN=$P/szenarien/010/daten-en
V548=$P/$Z/vorgabe-548483c
mkdir -p nachweis/011
echo "App: $QE_APP_BIN $(sha256sum "$QE_APP_BIN" | cut -c1-16)"
werkzeug/produktiv-stand.sh > nachweis/011/produktiv-vorher.txt
gsettings list-recursively org.gnome.desktop.interface > nachweis/011/gsettings-live-vorher.txt
for a in 1920 1024; do
    s=$([ $a = 1920 ] && echo 1920x1080 || echo 1024x768)
    bash werkzeug/g2aj-lauf.sh "$V-r1-bb90803-$a" $s $Z/r1-rueckstellung-$a.txt
    QE_LABWC_VORGABE_QUELLE=$V548 bash werkzeug/g2aj-lauf.sh "$V-r1-548483c-$a" $s $Z/r1-rueckstellung-$a.txt
    QE_LABWC_VORGABE_QUELLE=$V548 QE_LABWC_DATEIEN=$EN bash werkzeug/g2aj-lauf.sh "$V-r1-en-$a" $s $Z/r1-rueckstellung-$a.txt
    QE_LABWC_VORGABE_QUELLE=$V548 QE_LABWC=ohne bash werkzeug/g2aj-lauf.sh "$V-r1-ohne-$a" $s $Z/r1-rueckstellung-$a.txt
    QE_LABWC_VORGABE_QUELLE=$V548 bash werkzeug/g2aj-lauf.sh "$V-r2-$a" $s $Z/r2-standard-$a.txt
    bash werkzeug/g2aj-lauf.sh "$V-eb1-de-$a" $s $Z/eb1-erscheinungsbild-$a.txt
    QE_LABWC_DATEIEN=$EN bash werkzeug/g2aj-lauf.sh "$V-eb1-en-$a" $s $Z/eb1-erscheinungsbild-$a.txt
    bash werkzeug/g2aj-lauf.sh "$V-a1-de-$a" $s $Z/a1-alle-seiten-$a.txt
    QE_LABWC_DATEIEN=$EN bash werkzeug/g2aj-lauf.sh "$V-a1-en-$a" $s $Z/a1-alle-seiten-$a.txt
done
werkzeug/produktiv-stand.sh > nachweis/011/produktiv-nachher.txt
gsettings list-recursively org.gnome.desktop.interface > nachweis/011/gsettings-live-nachher.txt
if diff nachweis/011/produktiv-vorher.txt nachweis/011/produktiv-nachher.txt \
   && diff nachweis/011/gsettings-live-vorher.txt nachweis/011/gsettings-live-nachher.txt; then
    echo "PRODUKTIV UNVERÄNDERT"
else
    echo "PRODUKTIV VERÄNDERT"
fi
python3 -I werkzeug/auswerten-011.py "nachweis/011/gui-$V.tsv" "$V"; echo "auswertung exit=$?"
