#!/bin/bash
# G2-AJ: differenzielle Regression der Vorgänger-GUI-Prüfungen (G2-C, G2-G,
# G2-X, G2-Z, G2-AA, G2-AF, G2-AG) wie werkzeug/g2ag-regression.sh: jedes
# Skript läuft zweimal unter gleichen Bedingungen, mit dem Live-Stand 0.0.8
# (G2-AG, 452599512a8b268a) und mit 0.0.9; verglichen werden die
# Ergebnistabellen (Fall und Ergebnis). Gleiche Ergebnisse = keine Regression
# durch 0.0.9. Ergebnisse nach nachweis/g2aj/regression/<skript>-{008,009}/;
# danach werden die eingecheckten Nachweise der Vorgänger wiederhergestellt.
#
#   bash werkzeug/g2aj-regression.sh
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
export REG_ALT=${REG_ALT:-~/opt/querencia/einstellungen/452599512a8b268a/querencia-einstellungen}
export REG_NEU=${REG_NEU:-$P/nachweis/g2aj/release/querencia-einstellungen}
export REG_ALT_NAME=008 REG_NEU_NAME=009
export REG_SKRIPTE=${REG_SKRIPTE:-g2c g2g g2x g2z g2aa g2af g2ag}
export REG_TSV_ZUSATZ="nachweis/g2af/g2af-gui.tsv nachweis/g2ag/gui-g2ag-f4879567.tsv"
export REG_ZIEL=nachweis/g2aj/regression
bash werkzeug/g2ag-regression.sh
