#!/bin/bash
# G2-AA: isolierte GUI-Prüfung des Fenstersymbols (QREL-05 Nr. 2.9). Jeder Lauf
# startet einen eigenen headless labwc 0.9.8 mit Debug-Protokoll; daraus kommt,
# woher labwc das Titelleistensymbol nimmt („by app_id“ = Starter/Symbolthema,
# „client buffer“ = vom Programm geschickt). Dazu Bildschirmfotos und Ausschnitte
# der Titelleiste (werkzeug/titelleiste.py). Läufe nach nachweis/g2aa-*/,
# Auswertung nach nachweis/g2aa/fenstersymbol-g2aa.tsv.
#
# „frisch“: labwc liest den heutigen Starter (Icon=org.querencia.Einstellungen)
#           und ~/.local/share/icons (nur gelesen) – wie nach einer Neuanmeldung.
# „alt“:    labwc sieht nur szenarien/fenstersymbol/alt-datenhome (Starter mit
#           Icon=preferences-system, keine eigenen Symbole) und das Symbolthema
#           gnome-human wie live – wie der Live-labwc seit 09:27 (vor G2-V).
#
#   QE_APP_BIN=<release 0.0.6> bash werkzeug/g2aa-gui-lauf.sh
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
Z=szenarien/fenstersymbol
NEU=${QE_APP_BIN:-$P/nachweis/g2aa/release/querencia-einstellungen}
ALT=${ALT:-$HOME/opt/querencia/einstellungen/d6378490b5e53237/querencia-einstellungen}
ECHT=szenarien/standardanwendungen/echt-20261019.list
mkdir -p nachweis/g2aa
echo "Neu: $NEU $(sha256sum "$NEU" | cut -c1-16) ($(bash "$P/werkzeug/version-isoliert.sh" "$NEU"))"
echo "Alt: $ALT $(sha256sum "$ALT" | cut -c1-16) ($(bash "$P/werkzeug/version-isoliert.sh" "$ALT"))"
werkzeug/produktiv-stand.sh > nachweis/g2aa/produktiv-vorher-g2aa.txt
# l <name> <binary> <args> <auflösung> <szenario> <frisch|alt> [rc.xml]
l() {
    local n=$1 bin=$2 a=$3 s=$4 sz=$5 lage=$6 rc=${7:-}
    local env=(QE_APP_BIN="$bin" QE_APP_ARGS="$a" QE_LABWC_DEBUG=1 QE_APP_WAYLAND_DEBUG=1 QE_MIMEAPPS="$ECHT")
    [ -n "$rc" ] && env+=(QE_LABWC_RC="$rc")
    [ "$lage" = alt ] && env+=(QE_DATA_HOME="$Z/alt-datenhome")
    env "${env[@]}" bash werkzeug/isoliert.sh "g2aa-$n" "$s" "$Z/$sz" >/dev/null 2>&1
    echo "Lauf $n: $(tail -1 nachweis/g2aa-$n/logs/eingabe.log)"
}
# Hauptfall: 0.0.6, frisch, beide Auflösungen, mit Unterdialog (Standardanwendung wählen)
l f1-neu-frisch-1024 "$NEU" - 1024x768 f-kz-1024.txt frisch
l f2-neu-frisch-1920 "$NEU" - 1920x1080 f-kz-1920.txt frisch
# Vergleich 0.0.5
l f3-alt005-frisch-1024 "$ALT" - 1024x768 f-kz-1024.txt frisch
l f4-alt005-frisch-1920 "$ALT" - 1920x1080 f-kz-1920.txt frisch
# Symbolthema wie live (gnome-human), heutiger Starter: Stand nach labwc --reconfigure
l f5-alt005-reconf-1920 "$ALT" - 1920x1080 f-kz-1920.txt frisch "$Z/rc-wie-live.xml"
l f6-neu-reconf-1920 "$NEU" - 1920x1080 f-kz-1920.txt frisch "$Z/rc-wie-live.xml"
# Altstand des Live-labwc nachgestellt (Werkstatt 2.9)
l f7-alt005-altstand-1920 "$ALT" - 1920x1080 f-kz-1920.txt alt "$Z/rc-wie-live.xml"
l f8-neu-altstand-1920 "$NEU" - 1920x1080 f-kz-1920.txt alt "$Z/rc-wie-live.xml"
# Altstand mit Fensterregel iconPriority="client"
l f9-alt005-client-1920 "$ALT" - 1920x1080 f-kz-1920.txt alt "$Z/rc-wie-live-client.xml"
l f10-neu-client-1920 "$NEU" - 1920x1080 f-kz-1920.txt alt "$Z/rc-wie-live-client.xml"
l f11-neu-client-1024 "$NEU" - 1024x768 f-kz-1024.txt alt "$Z/rc-wie-live-client.xml"
# Einstieg vom Desktop-Menü (--hintergrund)
l f12-neu-hintergrund-1920 "$NEU" --hintergrund 1920x1080 f-hintergrund-1920.txt frisch
werkzeug/produktiv-stand.sh > nachweis/g2aa/produktiv-nachher-g2aa.txt
if diff nachweis/g2aa/produktiv-vorher-g2aa.txt nachweis/g2aa/produktiv-nachher-g2aa.txt; then echo "PRODUKTIV UNVERÄNDERT"; else echo "PRODUKTIV VERÄNDERT"; fi
python3 werkzeug/auswerten-fenstersymbol.py nachweis/g2aa/fenstersymbol-g2aa.tsv; echo "auswertung exit=$?"
