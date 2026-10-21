#!/bin/bash
# G2-AJ: ein isolierter Lauf mit den festen Bedingungen dieser Prüfung:
# GTK-Thema Querencia-Klassisch, Symbolthema Querencia, DejaVu Sans 10,
# gsettings im Profil wie live (szenarien/g2aj/gsettings-live.txt),
# labwc-Benutzerkopie aus der festen Vorgabe (szenarien/labwc/vorgabe),
# Desktop G2-AD 0.2 (7f6744b5). Alles andere über die QE_*-Variablen von
# isoliert.sh (z. B. QE_WLR_RENDERER, QE_GSK_RENDERER, QE_LABWC_DATEIEN).
#
#   QE_APP_BIN=<binary> bash werkzeug/g2aj-lauf.sh <lauf> <BREITExHÖHE> <szenario> [desktop.ini-Vorlage]
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
export QE_APP_BIN=${QE_APP_BIN:-$P/nachweis/g2aj/release/querencia-einstellungen}
export QE_DESKTOP=${QE_DESKTOP:-$HOME/opt/querencia/desktop/7f6744b5c6cc3b1d/querencia-desktop}
export QE_DESKTOP_SHA16=${QE_DESKTOP_SHA16:-7f6744b5c6cc3b1d}
export QE_LABWC_VORGABE_QUELLE=${QE_LABWC_VORGABE_QUELLE:-$P/szenarien/labwc/vorgabe}
export QE_GTK_SCHRIFT=${QE_GTK_SCHRIFT:-DejaVu Sans 10}
export QE_GTK_THEMA=${QE_GTK_THEMA:-Querencia-Klassisch:Querencia}
export QE_GSETTINGS=${QE_GSETTINGS:-$P/szenarien/g2aj/gsettings-live.txt}
export QE_LABWC=${QE_LABWC:-benutzer}
export QE_APP_ARGS=${QE_APP_ARGS:--}
export QE_ZEIT=${QE_ZEIT:-120}
bash werkzeug/isoliert.sh "$1" "$2" "$3" ${4:+"$4"} >/dev/null 2>&1
echo "Lauf $1: $(tail -1 "nachweis/$1/logs/eingabe.log")"
