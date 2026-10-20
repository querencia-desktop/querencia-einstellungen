#!/bin/bash
# Einstellungen 0.1.0: isolierte GUI-Prüfung in 1920×1080 und 1024×768 unter
# den Bedingungen von G2-AJ (werkzeug/g2aj-lauf.sh: Querencia-Klassisch,
# Symbolthema Querencia, DejaVu Sans 10, gsettings wie live, labwc-Benutzer-
# kopie aus der festen Vorgabe). Deutsch wie live (LANG=en_US.UTF-8,
# LANGUAGE=de); Englisch wie nach „Sprache und Region“ über die eigene
# Umgebungsdatei der Benutzerkopie (szenarien/010/daten-en). Die Neuanmeldung
# startet eine neue Sitzung mit der Datei, die sp1 in Schritt 2 geschrieben hat.
# Szenarien aus werkzeug/010-szenarien.py, Läufe nach nachweis/<präfix>-<lauf>-<b>/,
# Auswertung nach nachweis/010/gui-<präfix>.tsv.
#
#   QE_APP_BIN=<release> bash werkzeug/010-gui-lauf.sh [präfix]
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
Z=szenarien/010
V=${1:-010}
export QE_APP_BIN=${QE_APP_BIN:-$P/nachweis/010/release/querencia-einstellungen}
EN=$P/$Z/daten-en
mkdir -p nachweis/010
echo "App: $QE_APP_BIN $(sha256sum "$QE_APP_BIN" | cut -c1-16) ($("$QE_APP_BIN" --version))"
werkzeug/produktiv-stand.sh > nachweis/010/produktiv-vorher.txt
gsettings list-recursively org.gnome.desktop.interface > nachweis/010/gsettings-live-vorher.txt
for a in 1920 1024; do
    s=$([ $a = 1920 ] && echo 1920x1080 || echo 1024x768)
    bash werkzeug/g2aj-lauf.sh "$V-a1-de-$a" $s $Z/a1-alle-seiten-$a.txt
    QE_LABWC_DATEIEN=$EN bash werkzeug/g2aj-lauf.sh "$V-a1-en-$a" $s $Z/a1-alle-seiten-$a.txt
    bash werkzeug/g2aj-lauf.sh "$V-e0-de-$a" $s $Z/e0-erkunden-$a.txt
    QE_LABWC_DATEIEN=$EN bash werkzeug/g2aj-lauf.sh "$V-e0-en-$a" $s $Z/e0-erkunden-$a.txt
    bash werkzeug/g2aj-lauf.sh "$V-sp1-$a" $s $Z/sp1-sprache-$a.txt
    QE_LABWC=ohne bash werkzeug/g2aj-lauf.sh "$V-o1-$a" $s $Z/sp1-sprache-$a.txt
    QE_LABWC_DATEIEN=$EN bash werkzeug/g2aj-lauf.sh "$V-sp2-$a" $s $Z/sp2-language-$a.txt
    # Neuanmeldung mit der Datei aus sp1, Schritt 2 (English, Formate Schweiz)
    d=$P/nachweis/$V-n1-daten-$a
    rm -rf "$d"; mkdir -p "$d/environment.d"
    cp "nachweis/$V-sp1-$a/env-2.txt" "$d/environment.d/zz-querencia-einstellungen.env" 2>/dev/null
    QE_LABWC_DATEIEN=$d bash werkzeug/g2aj-lauf.sh "$V-n1-$a" $s $Z/n1-neuanmeldung-$a.txt
    bash werkzeug/g2aj-lauf.sh "$V-tb1-$a" $s $Z/tb1-tastatur-$a.txt
done
werkzeug/produktiv-stand.sh > nachweis/010/produktiv-nachher.txt
gsettings list-recursively org.gnome.desktop.interface > nachweis/010/gsettings-live-nachher.txt
if diff nachweis/010/produktiv-vorher.txt nachweis/010/produktiv-nachher.txt \
   && diff nachweis/010/gsettings-live-vorher.txt nachweis/010/gsettings-live-nachher.txt; then
    echo "PRODUKTIV UNVERÄNDERT"
else
    echo "PRODUKTIV VERÄNDERT"
fi
python3 -I werkzeug/auswerten-010.py "nachweis/010/gui-$V.tsv" "$V"; echo "auswertung exit=$?"
