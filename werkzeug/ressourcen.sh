#!/bin/bash
# Erzeugt die eingebettete GResource (Fenstersymbol, G2-AA) neu. Das Ergebnis
# ist eingecheckt; der Build braucht glib-compile-resources nicht.
set -euo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
R="$P/crates/querencia-einstellungen/data/resource"
glib-compile-resources --sourcedir="$R" --target="$R/einstellungen.gresource" "$R/einstellungen.gresource.xml"
sha256sum "$R/einstellungen.gresource"
