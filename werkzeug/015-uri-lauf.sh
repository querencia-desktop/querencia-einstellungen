#!/bin/bash
# Einstellungen 0.1.5: „Online-Hilfe“, „Fehler melden“ und der Link der
# Webseite auf „Informationen über Querencia“ als Klick (isoliert wie
# werkzeug/015-gui-lauf.sh). Damit kein Browser startet, bekommt das
# Profil-HOME einen Starter für http/https, der die Adresse nur nach
# $XDG_STATE_HOME/uri-protokoll.txt schreibt (szenarien/015/uri-heim,
# mimeapps.list szenarien/015/uri-mimeapps.list). GtkUriLauncher nimmt ohne
# Portal (GTK_USE_PORTAL=0) die Standardanwendung über GIO.
#   u1 ohne  ohne Versionsdatei: Rückfalladressen (Hilfe in der Sprache)
#   u1 mit   mit release (Alpha 2.1): Adressen aus der Datei
# Erwartung je Lauf: genau drei Zeilen in dieser Reihenfolge.
#
#   bash werkzeug/015-uri-lauf.sh
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
NEU=${NEU_BIN:-$P/nachweis/015/release/querencia-einstellungen}
THEMA=${THEMA:-$HOME/opt/querencia-theme/22824d05ceb49cad/Querencia-Klassisch}
Z=szenarien/015
EN=$P/szenarien/010/daten-en
NW=nachweis/015
[ ! -e /usr/share/querencia ] || { echo "/usr/share/querencia existiert auf diesem Rechner" >&2; exit 2; }
AB=$(date +%s)
werkzeug/produktiv-stand.sh > "$NW/produktiv-vorher-uri.txt"
ok=0; bad=0
lauf() {  # lauf <name> <sprache> <aufl> <szenario> <ohne|ordner> <erwartet …>
    local name=$1 sp=$2 aufl=$3 sz=$4 datei=$5; shift 5
    local en=; [ "$sp" = en ] && en=$EN
    local wrap=()
    if [ "$datei" != ohne ]; then
        wrap=(bwrap --dev-bind / / --overlay-src /usr/share --tmp-overlay /usr/share
              --ro-bind "$P/$Z/$datei" /usr/share/querencia --tmpfs /tmp/.X11-unix)
    fi
    QE_APP_BIN=$NEU QE_PORTAL=settings QE_GSETTINGS=$NW/gsettings-hell.txt QE_THEMA_ORDNER=$THEMA \
        QE_LABWC_DATEIEN=$en QE_APP_ARGS="--seite informationen" \
        QE_HEIM_DATEIEN=$P/$Z/uri-heim QE_MIMEAPPS=$P/$Z/uri-mimeapps.list \
        "${wrap[@]}" bash werkzeug/g2aj-lauf.sh "$name" "$aufl" "$sz"
    rm -rf "nachweis/$name/profil/heim/.config/dconf"
    local log=nachweis/$name/profil/state/uri-protokoll.txt
    cp "$log" "$NW/$name-uri-protokoll.txt" 2>/dev/null || : > "$NW/$name-uri-protokoll.txt"
    local soll ist
    soll=$(printf '%s\n' "$@")
    ist=$(cat "$NW/$name-uri-protokoll.txt")
    if [ "$soll" = "$ist" ]; then ok=$((ok+1)); echo "PASS   $name: $(echo "$ist" | tr '\n' ' ')"
    else bad=$((bad+1)); echo "FAIL   $name: erwartet $(echo "$soll" | tr '\n' ' ') – ist $(echo "$ist" | tr '\n' ' ')"; fi
}
lauf 015-u1-ohne-de-1024 de 1024x768 $Z/u1-adressen-1024.txt ohne \
    https://querencialinux.org/de/docs/ https://github.com/querencia-desktop/querencia/issues/new/choose https://querencialinux.org
lauf 015-u1-mit-de-1024 de 1024x768 $Z/u1-adressen-1024.txt release-alpha21 \
    https://querencialinux.org/de/docs/ https://github.com/querencia-desktop/querencia/issues https://querencialinux.org
lauf 015-u1-ohne-en-1920 en 1920x1080 $Z/u1-adressen-1920.txt ohne \
    https://querencialinux.org/en/docs/ https://github.com/querencia-desktop/querencia/issues/new/choose https://querencialinux.org
werkzeug/produktiv-stand.sh > "$NW/produktiv-nachher-uri.txt"
if diff "$NW/produktiv-vorher-uri.txt" "$NW/produktiv-nachher-uri.txt" && [ ! -e /usr/share/querencia ]; then echo "PRODUKTIV UNVERÄNDERT"; else echo "PRODUKTIV VERÄNDERT"; bad=$((bad+1)); fi
bash werkzeug/wsdd-reste.sh "$AB"
echo "ERGEBNIS uri: $ok PASS, $bad FAIL"
[ "$bad" = 0 ]
