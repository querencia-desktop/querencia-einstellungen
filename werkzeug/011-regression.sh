#!/bin/bash
# Einstellungen 0.1.1: differenzielle Regression der Vorgänger-GUI-Prüfungen
# (G2-C, G2-G, G2-X, G2-Z, G2-AA, G2-AF, G2-AG, G2-AJ, 0.1.0) wie
# werkzeug/010-regression.sh: jedes Skript läuft zweimal unter gleichen
# Bedingungen, mit dem Live-Stand 0.1.0 (df0f666b499ed551, Ablage mit
# share/locale) und mit 0.1.1 (Release mit share/locale daneben); verglichen
# werden die Ergebnistabellen (Fall und Ergebnis). Gleiche Ergebnisse = keine
# Regression durch 0.1.1. Ergebnisse nach nachweis/011/regression/<skript>-{010,011}/;
# danach werden die eingecheckten Nachweise der Vorgänger wiederhergestellt.
#
#   bash werkzeug/011-regression.sh
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
export REG_ALT=${REG_ALT:-~/opt/querencia/einstellungen/df0f666b499ed551/querencia-einstellungen}
export REG_NEU=${REG_NEU:-$P/nachweis/011/release/querencia-einstellungen}
export REG_ALT_NAME=010 REG_NEU_NAME=011
export REG_SKRIPTE=${REG_SKRIPTE:-g2c g2g g2x g2z g2aa g2af g2ag g2aj 010}
export REG_TSV_ZUSATZ="nachweis/g2af/g2af-gui.tsv nachweis/g2ag/gui-g2ag-f4879567.tsv nachweis/g2aj/gui-g2aj.tsv nachweis/010/gui-010.tsv"
export REG_ZIEL=nachweis/011/regression
bash werkzeug/g2ag-regression.sh
