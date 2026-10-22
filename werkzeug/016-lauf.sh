#!/bin/bash
# Einstellungen 0.1.6: ein isolierter Lauf unter den Bedingungen der
# 0.1.5-Matrix (werkzeug/015-gui-lauf.sh): Thema-Ablage Querencia-Klassisch,
# Settings-Portal-Ersatz hell/dunkel, eingebaute Bilder 0.1 + 0.2, englische
# Benutzerkopie für „en“. Die QUERENCIA_*-Variablen der Live-Sitzung werden
# vorher entfernt (13 §5, Nachtrag 21.10. 20:38); isoliert.sh setzt
# QUERENCIA_STATE_DIR selbst ins Profil. Weitere QE_*-Variablen gehen durch
# (z. B. QE_OUTPUTS, QE_BILDSCHIRM_TOML, QE_BILDSCHIRM_INI,
# QE_BILDSCHIRM_OHNE_EDID).
#
#   bash werkzeug/016-lauf.sh <lauf> <binary> <hell|dunkel> <de|en> <BREITExHÖHE> <szenario> [app-args]
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
NAME=${1:?Lauf}; BIN=${2:?Binary}; ART=${3:?hell|dunkel}; SP=${4:?de|en}; AUFL=${5:?Auflösung}; SZ=${6:?Szenario}; ARGS=${7:--}
THEMA=${THEMA:-$HOME/opt/querencia-theme/22824d05ceb49cad/Querencia-Klassisch}
BILDER01=${BILDER01:-$HOME/projects/querencia-einstellungen-014/nachweis/bilder-01}
BILDER02=${BILDER02:-$HOME/projects/querencia-einstellungen-014/nachweis/bilder-02}
NW=$P/nachweis/016
mkdir -p "$NW"
GS=$P/szenarien/g2aj/gsettings-live.txt
case $ART in
    hell) { cat "$GS"; echo "color-scheme default"; } > "$NW/gsettings-hell.txt" ;;
    dunkel) { cat "$GS"; echo "color-scheme prefer-dark"; } > "$NW/gsettings-dunkel.txt" ;;
    *) echo "hell oder dunkel" >&2; exit 2 ;;
esac
EN=; [ "$SP" = en ] && EN=$P/szenarien/010/daten-en
for v in $(compgen -e); do case $v in QUERENCIA_*) unset "$v" ;; esac; done
QE_PROFIL_STABIL=$HOME/.cache/qe01/einstellungen-016-profil QE_APP_BIN=$BIN QE_PORTAL=settings QE_GSETTINGS=$NW/gsettings-$ART.txt \
    QE_THEMA_ORDNER=$THEMA QE_LABWC_DATEIEN=$EN QE_HINTERGRUENDE=$BILDER01:$BILDER02 QE_APP_ARGS=$ARGS \
    bash werkzeug/g2aj-lauf.sh "$NAME" "$AUFL" "$SZ"
rm -rf "nachweis/$NAME/profil/heim/.config/dconf"
