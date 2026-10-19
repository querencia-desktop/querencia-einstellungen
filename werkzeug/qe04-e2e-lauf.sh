#!/bin/bash
# G2-X: Ende-zu-Ende Einstellungen 0.0.4 → desktop.ini → Desktop QE-04 im
# isolierten labwc (isoliert.sh); Gegenprobe q1 mit dem Live-Desktop QE-03.
#   QE04_BIN=<desktop qe-04> bash werkzeug/qe04-e2e-lauf.sh
set -uo pipefail
# G2-Z: Ordner des Skripts (läuft auch im Worktree)
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
Z=szenarien/arbeitsflaeche
export QE_APP_BIN=${QE_APP_BIN:-$P/nachweis/g2x/release/querencia-einstellungen}
B=${QE04_BIN:?QE04_BIN: Desktop QE-04}
QE03=~/opt/querencia/desktop/9f4a9dca39014f0f/querencia-desktop
echo "App $(sha256sum "$QE_APP_BIN" | cut -c1-16), Desktop QE-04 $(sha256sum "$B" | cut -c1-16)"
werkzeug/produktiv-stand.sh > nachweis/g2x/produktiv-vorher-qe04.txt
l() { local n=$1 d=$2 s=$3 sz=$4 ini=${5:-}; QE_DESKTOP=$d QE_DESKTOP_SHA16=$(sha256sum "$d" | cut -c1-16) QE_APP_ARGS=--arbeitsflaeche bash werkzeug/isoliert.sh "g2x-$n" "$s" "$Z/$sz" $ini >/dev/null 2>&1; echo "Lauf $n: $(tail -1 nachweis/g2x-$n/logs/eingabe.log)"; }
l q1-e2e-1920 "$B" 1920x1080 q1-ende-zu-ende-1920.txt
l q2-e2e-1024 "$B" 1024x768 q2-ende-zu-ende-1024.txt
l q3-standard "$B" 1920x1080 q3-zurueck-zum-standard.txt $Z/ohne-rechner-klein.ini
l q4-gegenprobe-qe03 "$QE03" 1920x1080 q1-ende-zu-ende-1920.txt
werkzeug/produktiv-stand.sh > nachweis/g2x/produktiv-nachher-qe04.txt
if diff nachweis/g2x/produktiv-vorher-qe04.txt nachweis/g2x/produktiv-nachher-qe04.txt; then echo "PRODUKTIV UNVERÄNDERT"; else echo "PRODUKTIV VERÄNDERT"; fi
python3 werkzeug/auswerten-qe04.py nachweis/g2x/qe04-e2e.tsv; echo "auswertung exit=$?"
