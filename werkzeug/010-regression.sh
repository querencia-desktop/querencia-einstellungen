#!/bin/bash
# Einstellungen 0.1.0: differenzielle Regression der Vorgänger-GUI-Prüfungen
# (G2-C, G2-G, G2-X, G2-Z, G2-AA, G2-AF, G2-AG, G2-AJ) wie
# werkzeug/g2aj-regression.sh: jedes Skript läuft zweimal unter gleichen
# Bedingungen, mit dem Live-Stand 0.0.9 (G2-AJ, de7e95a52a8eec92) und mit
# 0.1.0 (Release mit share/locale daneben, Sitzung wie live LANGUAGE=de);
# verglichen werden die Ergebnistabellen (Fall und Ergebnis). Gleiche
# Ergebnisse = keine Regression durch 0.1.0. Ergebnisse nach
# nachweis/010/regression/<skript>-{009,010}/; danach werden die
# eingecheckten Nachweise der Vorgänger wiederhergestellt.
#
#   bash werkzeug/010-regression.sh
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
export REG_ALT=${REG_ALT:-$HOME/opt/querencia/einstellungen/de7e95a52a8eec92/querencia-einstellungen}
export REG_NEU=${REG_NEU:-$P/nachweis/010/release/querencia-einstellungen}
export REG_ALT_NAME=009 REG_NEU_NAME=010
export REG_SKRIPTE=${REG_SKRIPTE:-g2c g2g g2x g2z g2aa g2af g2ag g2aj}
export REG_TSV_ZUSATZ="nachweis/g2af/g2af-gui.tsv nachweis/g2ag/gui-g2ag-f4879567.tsv nachweis/g2aj/gui-g2aj.tsv"
export REG_ZIEL=nachweis/010/regression
bash werkzeug/g2ag-regression.sh
