#!/bin/bash
# Einstellungen 0.1.0: erzeugt die Integrationsvorlage integration/010/vorschlag
# aus dem Release-Ordner (Binary + share/locale aus werkzeug/i18n.py mo):
#   querencia-einstellungen                                   0555
#   share/locale/{de,en}/LC_MESSAGES/querencia-einstellungen.mo 0444
#   org.querencia.Einstellungen.desktop  (aus data/…desktop, Exec/TryExec → Ablage) 0444
#   alt-erlaubt.txt  (nur 0.0.9 G2-AJ)                       0444
#   SHA256SUMS       (alle Dateien, relativ)                  0444
# Ordner 0555. Eine vorhandene Vorlage wird nur mit VORLAGE_NEU=ja ersetzt
# (nach der Freigabe nie: das Integrationsskript prüft gegen sie).
#
#   VORLAGE_NEU=ja bash werkzeug/vorlage-010.sh [release-ordner]
set -euo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
R=${1:-nachweis/010/release}
V=integration/010/vorschlag
PROG=querencia-einstellungen
DESKNAME=org.querencia.Einstellungen.desktop
ALT=de7e95a52a8eec9214bb5cc2e3c88ce96af5c36b362fcc58d1ab27f716d587f7
for f in "$R/$PROG" "$R/share/locale/de/LC_MESSAGES/$PROG.mo" "$R/share/locale/en/LC_MESSAGES/$PROG.mo"; do
    [ -f "$f" ] || { echo "fehlt: $f" >&2; exit 2; }
done
[ "$("$R/$PROG" --version)" = "$PROG 0.1.0" ] || { echo "Release meldet nicht 0.1.0" >&2; exit 2; }
if [ -e "$V" ]; then
    [ "${VORLAGE_NEU:-}" = ja ] || { echo "Vorlage $V existiert – nur mit VORLAGE_NEU=ja ersetzen" >&2; exit 2; }
    chmod -R u+w "$V"; rm -rf "$V"
fi
NEU=$(sha256sum "$R/$PROG" | cut -c1-16)
mkdir -p "$V/share/locale/de/LC_MESSAGES" "$V/share/locale/en/LC_MESSAGES"
cp "$R/$PROG" "$V/$PROG"
cp "$R/share/locale/de/LC_MESSAGES/$PROG.mo" "$V/share/locale/de/LC_MESSAGES/"
cp "$R/share/locale/en/LC_MESSAGES/$PROG.mo" "$V/share/locale/en/LC_MESSAGES/"
EXEC=~/opt/querencia/einstellungen/$NEU/$PROG
awk -v e="$EXEC" -v n="$NEU" '
    /^\[Desktop Entry\]$/ {
        print
        print "# Querencia Einstellungen 0.1.0 (Kontrollzentrum im Stil von GNOME 2; deutsch und"
        print "# englisch über gettext, Sprache und Region, mehrere Tastaturbelegungen)."
        print "# Desktop-ID = GApplication-ID org.querencia.Einstellungen. Name, Comment und"
        print "# Keywords aus po/ (werkzeug/i18n.py desktop). Ablage mit Katalogen unter"
        print "# ~/opt/querencia/einstellungen/" n "/ (Querencia-Werkstatt, Einstellungen 0.1.0, 2026-10-20)."
        print "# Rückfall: bash ~/projects/querencia-einstellungen-wt-010/werkzeug/einstellungen-010-integrieren.sh zurueck"
        next
    }
    /^Exec=/ { print "Exec=" e; next }
    /^TryExec=/ { print "TryExec=" e; next }
    { print }' "crates/querencia-einstellungen/data/$DESKNAME" >"$V/$DESKNAME"
desktop-file-validate "$V/$DESKNAME"
{
    echo "# Zulässige Ausgangsstände für einstellungen-010-integrieren.sh (volle SHA256, eine je Zeile)"
    echo "$ALT 0.0.9 G2-AJ (live seit 20.10. 14:03)"
} >"$V/alt-erlaubt.txt"
(cd "$V" && sha256sum -- "$PROG" "share/locale/de/LC_MESSAGES/$PROG.mo" "share/locale/en/LC_MESSAGES/$PROG.mo" "$DESKNAME" alt-erlaubt.txt >SHA256SUMS)
chmod 0555 "$V/$PROG"
find "$V" -type f ! -name "$PROG" -exec chmod 0444 {} +
find "$V" -type d -exec chmod 0555 {} +
(cd "$V" && sha256sum -c SHA256SUMS)
echo "Vorlage $V: $NEU ($(sha256sum "$V/SHA256SUMS" | cut -c1-16) SHA256SUMS)"
