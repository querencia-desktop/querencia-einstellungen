#!/bin/bash
# Einstellungen 0.1.2: differenzielle Regression der Vorgänger-GUI-Prüfungen
# (G2-C, G2-G, G2-X, G2-Z, G2-AA, G2-AF, G2-AG, G2-AJ, 0.1.0, 0.1.1) wie
# werkzeug/011-regression.sh: jedes Skript läuft zweimal unter gleichen
# Bedingungen (seit 0.1.2 mit eigenem HOME in isoliert.sh), mit dem
# Live-Stand 0.1.1 (557957f3f09b7939, Ablage mit share/locale) und mit 0.1.2
# (Release mit share/locale daneben); verglichen werden die Ergebnistabellen
# (Fall und Ergebnis). Ergebnisse nach nachweis/012/regression/<skript>-{011,012}/;
# danach werden die eingecheckten Nachweise der Vorgänger wiederhergestellt.
#
#   bash werkzeug/012-regression.sh
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
export REG_ALT=${REG_ALT:-$HOME/opt/querencia/einstellungen/557957f3f09b7939/querencia-einstellungen}
export REG_NEU=${REG_NEU:-$P/nachweis/012/release/querencia-einstellungen}
export REG_ALT_NAME=011 REG_NEU_NAME=012
export REG_SKRIPTE=${REG_SKRIPTE:-g2c g2g g2x g2z g2aa g2af g2ag g2aj 010 011}
export REG_TSV_ZUSATZ="nachweis/g2af/g2af-gui.tsv nachweis/g2ag/gui-g2ag-f4879567.tsv nachweis/g2aj/gui-g2aj.tsv nachweis/010/gui-010.tsv nachweis/011/gui-011.tsv"
export REG_ZIEL=nachweis/012/regression
bash werkzeug/g2ag-regression.sh
