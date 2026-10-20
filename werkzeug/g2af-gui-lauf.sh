#!/bin/bash
# G2-AF: isolierte GUI-Prüfung der Galerie eingebauter Hintergründe (f1–f9)
# und der Desktop-Darstellung jedes Motivs (l-*: Lesbarkeit am linken Rand,
# Logo-Lage) mit dem LIVE-Desktop aus der Ablage (nur gelesen) und dem
# Panel QREL-04c. Die 17 Bilder kommen aus dem Runtime-Worktree über
# QE_DATA_HOME (Symlinks unter ~/.cache/g2af/share), Schrift fest
# „Adwaita Sans 11“ (Klickpunkte). Läufe nach nachweis/g2af-*/, Auswertung
# nach nachweis/g2af/g2af-gui.tsv.
#
#   QE_APP_BIN=<release> bash werkzeug/g2af-gui-lauf.sh [nur-f|nur-l]
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
Z=szenarien/hintergruende
BILDER=${G2AF_BILDER:-$HOME/projects/querencia-runtime-wt-hintergruende/data/hintergruende}
DATEN=$HOME/.cache/g2af/share
export QE_APP_BIN=${QE_APP_BIN:-$P/target/debug/querencia-einstellungen}
# Desktop: Standard QE-04a (live bis zur nächsten Anmeldung); G2AF_DESKTOP=<sha16>
# nimmt eine andere Ablage, z. B. 9d622162efd66f93 (G2-AD, ab nächster Anmeldung).
D16=${G2AF_DESKTOP:-f48795676f867b82}
export QE_DESKTOP=$HOME/opt/querencia/desktop/$D16/querencia-desktop QE_DESKTOP_SHA16=$D16
export QE_DATA_HOME=$DATEN QE_GTK_SCHRIFT=${QE_GTK_SCHRIFT:-Adwaita Sans 11}
mkdir -p "$DATEN/querencia/hintergruende" nachweis/g2af
: >"$HOME/.cache/g2af/.querencia-testprofil"
(cd "$BILDER" && sha256sum -c --quiet SHA256SUMS) || { echo "Bilder stimmen nicht mit SHA256SUMS: $BILDER" >&2; exit 2; }
for f in "$BILDER"/querencia-*.png; do ln -sfn "$f" "$DATEN/querencia/hintergruende/"; done
echo "App: $QE_APP_BIN $(sha256sum "$QE_APP_BIN" | cut -c1-16) ($(bash "$P/werkzeug/version-isoliert.sh" "$QE_APP_BIN"))"
echo "Desktop: $QE_DESKTOP ($QE_DESKTOP_SHA16), Bilder: $BILDER"
werkzeug/produktiv-stand.sh > nachweis/g2af/produktiv-vorher.txt
l() { local n=$1 a=$2 s=$3 sz=$4 ini=${5:-}; QE_APP_ARGS=$a bash werkzeug/isoliert.sh "g2af-$n" "$s" "$sz" $ini >/dev/null 2>&1; echo "Lauf $n: $(tail -1 nachweis/g2af-$n/logs/eingabe.log)"; }
if [ "${1:-}" != nur-l ]; then
    l f1-abend-1920 --hintergrund 1920x1080 $Z/f1-abend-1920.txt
    l f2-erde-logo-1024 --hintergrund 1024x768 $Z/f2-erde-logo-1024.txt
    l f3-nacht-logo-2560 --hintergrund 2560x1440 $Z/f3-nacht-logo-2560.txt
    l f4-zentrieren-1920 --hintergrund 1920x1080 $Z/f4-zentrieren-1920.txt
    l f5-kacheln-1024 --hintergrund 1024x768 $Z/f5-kacheln-1024.txt
    l f6-vorschau-1920 --hintergrund 1920x1080 $Z/f6-vorschau-1920.txt
    l f7-erkennen-1920 --hintergrund 1920x1080 $Z/f7-erkennen-1920.txt $Z/ini-erde-4x3.ini
    l f8-fremd-1920 --hintergrund 1920x1080 $Z/f8-fremd-1920.txt $Z/ini-fremd.ini
    l f9-tastatur-1920 --hintergrund 1920x1080 $Z/f9-tastatur-1920.txt
fi
if [ "${1:-}" != nur-f ]; then
    T=$(mktemp -d)
    lauf_l() {   # Datei Auflösung
        printf '[Background]\nFile=%s/querencia/hintergruende/%s\nMode=zoom\nColor=#1c4e96\n' "$DATEN" "$1" >"$T/$1.ini"
        l "l-${1%.png}-$2" "" "$2" "$Z/l-desktop.txt" "$T/$1.ini"
    }
    for b in blau abend abend-logo blau2 blau2-logo erde erde-logo nacht nacht-logo; do lauf_l "querencia-$b.png" 1920x1080; done
    for b in abend-logo blau2-logo erde-logo nacht-logo; do lauf_l "querencia-$b-4x3.png" 1024x768; lauf_l "querencia-$b.png" 2560x1440; done
    rm -rf "$T"
fi
werkzeug/produktiv-stand.sh > nachweis/g2af/produktiv-nachher.txt
if diff nachweis/g2af/produktiv-vorher.txt nachweis/g2af/produktiv-nachher.txt; then echo "PRODUKTIV UNVERÄNDERT"; else echo "PRODUKTIV VERÄNDERT"; fi
python3 -I werkzeug/auswerten-g2af.py nachweis/g2af/g2af-gui${G2AF_DESKTOP:+-$G2AF_DESKTOP}.tsv; echo "auswertung exit=$?"
