#!/bin/bash
# G2-AG: isolierte GUI-Prüfung der Bereiche Tastatur, Maus, Fensterdesign,
# Arbeitsflächen (labwc-Benutzerkopie im Wegwerfprofil, headless labwc mit -C
# darauf) in 1920×1080 und 1024×768. Die Vorgabe kommt als Kopie aus dem
# Runtime-Repo (nur gelesen). Szenarien aus werkzeug/g2ag-szenarien.py; Läufe nach nachweis/g2ag-<lauf>/, Auswertung
# nach nachweis/g2ag/gui-g2ag.tsv.
#
#   QE_APP_BIN=<release 0.0.7> bash werkzeug/g2ag-gui-lauf.sh
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
Z=szenarien/labwc
export QE_APP_BIN=${QE_APP_BIN:-$P/nachweis/g2ag/release/querencia-einstellungen}
# Desktop: Standard QE-04a (live bis zur Neuanmeldung); G2-AD v2 mit
# QE_DESKTOP=<pfad> QE_DESKTOP_SHA16=<sha16> (Läufe heißen dann *-<sha16>)
export QE_DESKTOP=${QE_DESKTOP:-$HOME/opt/querencia/desktop/f48795676f867b82/querencia-desktop}
export QE_DESKTOP_SHA16=${QE_DESKTOP_SHA16:-f48795676f867b82}
# feste Vorgabe (Kopie main bb90803), damit Vorgabenänderungen die Läufe nicht verschieben
export QE_LABWC_VORGABE_QUELLE=${QE_LABWC_VORGABE_QUELLE:-$P/szenarien/labwc/vorgabe}
# Schrift wie live seit 20.10. 01:02 (Erscheinungsbild Variante A), fest für gleiche Klickpunkte
export QE_GTK_SCHRIFT=${QE_GTK_SCHRIFT:-DejaVu Sans 10}
mkdir -p nachweis/g2ag
echo "App: $QE_APP_BIN $(sha256sum "$QE_APP_BIN" | cut -c1-16) ($(bash "$P/werkzeug/version-isoliert.sh" "$QE_APP_BIN"))"
werkzeug/produktiv-stand.sh > nachweis/g2ag/produktiv-vorher.txt
l() {   # lauf auflösung modus [daten]
    local n=$1 s=$2 m=$3 d=${4:-}
    QE_LABWC=$m QE_LABWC_DATEIEN=$d QE_APP_ARGS=- bash werkzeug/isoliert.sh "g2ag-$n" "$s" "$Z/$n.txt" >/dev/null 2>&1
    echo "Lauf $n: $(tail -1 nachweis/g2ag-$n/logs/eingabe.log)"
}
for a in 1920 1024; do
    s=$([ $a = 1920 ] && echo 1920x1080 || echo 1024x768)
    l g1-tastatur-$a $s benutzer
    l g2-maus-$a $s benutzer
    l g2k-doppelklick-kontrolle-$a $s benutzer
    l g3-fensterdesign-$a $s benutzer
    l g4-arbeitsflaechen-$a $s benutzer
    l k1-standard-$a $s benutzer $Z/daten-k1
    l v1-vorgabe-$a $s benutzer $Z/daten-v1
    l o1-ohne-kopie-$a $s ohne
done
werkzeug/produktiv-stand.sh > nachweis/g2ag/produktiv-nachher.txt
if diff nachweis/g2ag/produktiv-vorher.txt nachweis/g2ag/produktiv-nachher.txt; then echo "PRODUKTIV UNVERÄNDERT"; else echo "PRODUKTIV VERÄNDERT"; fi
python3 -I werkzeug/auswerten-g2ag.py "nachweis/g2ag/gui-g2ag-${QE_DESKTOP_SHA16:0:8}.tsv"; echo "auswertung exit=$?"
