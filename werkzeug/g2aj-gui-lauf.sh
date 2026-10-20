#!/bin/bash
# G2-AJ: isolierte GUI-Prüfung von Einstellungen 0.0.9 in 1920×1080 und
# 1024×768 (GTK-Thema Querencia-Klassisch, Symbolthema Querencia, DejaVu
# Sans 10, gsettings im Profil wie live; Bedingungen in g2aj-lauf.sh).
# Szenarien aus werkzeug/g2aj-szenarien.py, Läufe nach nachweis/g2aj-<lauf>-<b>/,
# Auswertung nach nachweis/g2aj/gui-g2aj.tsv.
#
#   QE_APP_BIN=<release> bash werkzeug/g2aj-gui-lauf.sh [präfix]
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
Z=szenarien/g2aj
V=${1:-g2aj}
export QE_APP_BIN=${QE_APP_BIN:-$P/nachweis/g2aj/release/querencia-einstellungen}
mkdir -p nachweis/g2aj
echo "App: $QE_APP_BIN $(sha256sum "$QE_APP_BIN" | cut -c1-16) ($(bash "$P/werkzeug/version-isoliert.sh" "$QE_APP_BIN"))"
werkzeug/produktiv-stand.sh > nachweis/g2aj/produktiv-vorher.txt
gsettings list-recursively org.gnome.desktop.interface > nachweis/g2aj/gsettings-live-vorher.txt
for a in 1920 1024; do
    s=$([ $a = 1920 ] && echo 1920x1080 || echo 1024x768)
    for n in s1-alle-seiten e1-erscheinungsbild k1-konflikt s2-schriften h1-hintergrund p1-panels; do
        bash werkzeug/g2aj-lauf.sh "$V-$n-$a" $s $Z/$n-$a.txt
    done
    QE_LABWC=ohne bash werkzeug/g2aj-lauf.sh "$V-o1-ohne-kopie-$a" $s $Z/o1-ohne-kopie-$a.txt
    for n in w1-wirkung w2-schrift; do
        QE_PORTAL=settings bash werkzeug/g2aj-lauf.sh "$V-$n-$a" $s $Z/$n-$a.txt
    done
    QE_WLR_RENDERER=gles2 bash werkzeug/g2aj-lauf.sh "$V-z1-gles2-$a" $s $Z/z1-wechsel-$a.txt
    bash werkzeug/g2aj-lauf.sh "$V-z1-pixman-$a" $s $Z/z1-wechsel-$a.txt
    QE_WLR_RENDERER=gles2 bash werkzeug/g2aj-lauf.sh "$V-z2-symbole-$a" $s $Z/z2-symbole-$a.txt
done
werkzeug/produktiv-stand.sh > nachweis/g2aj/produktiv-nachher.txt
gsettings list-recursively org.gnome.desktop.interface > nachweis/g2aj/gsettings-live-nachher.txt
if diff nachweis/g2aj/produktiv-vorher.txt nachweis/g2aj/produktiv-nachher.txt \
   && diff nachweis/g2aj/gsettings-live-vorher.txt nachweis/g2aj/gsettings-live-nachher.txt; then
    echo "PRODUKTIV UNVERÄNDERT"
else
    echo "PRODUKTIV VERÄNDERT"
fi
python3 -I werkzeug/auswerten-g2aj.py "nachweis/g2aj/gui-$V.tsv" "$V"; echo "auswertung exit=$?"
