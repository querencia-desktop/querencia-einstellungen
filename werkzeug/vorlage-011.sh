#!/bin/bash
# Einstellungen 0.1.1: erzeugt die Integrationsvorlage integration/011/vorschlag
# aus dem Release-Ordner (Binary + share/locale aus werkzeug/i18n.py mo):
#   querencia-einstellungen                                     0555
#   share/locale/{de,en}/LC_MESSAGES/querencia-einstellungen.mo 0444
#   org.querencia.Einstellungen.desktop  (aus data/…desktop, Exec/TryExec → Ablage) 0444
#   SHA256SUMS       (alle Dateien, relativ)                    0444
# Ordner 0555. Den Ausgangsstand (0.1.0 df0f666b) und alle Prüfsummen trägt
# einstellungen-011-integrieren.sh fest in sich, die Vorlage hat deshalb keine
# alt-erlaubt.txt. Eine vorhandene Vorlage wird nur mit VORLAGE_NEU=ja ersetzt
# (nach der Freigabe nie: das Integrationsskript prüft gegen sie).
#
#   VORLAGE_NEU=ja bash werkzeug/vorlage-011.sh [release-ordner]
set -euo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
R=${1:-nachweis/011/release}
V=integration/011/vorschlag
PROG=querencia-einstellungen
DESKNAME=org.querencia.Einstellungen.desktop
for f in "$R/$PROG" "$R/share/locale/de/LC_MESSAGES/$PROG.mo" "$R/share/locale/en/LC_MESSAGES/$PROG.mo"; do
    [ -f "$f" ] || { echo "fehlt: $f" >&2; exit 2; }
done
T=$(mktemp -d "$P/nachweis/vorlage-011-XXXXXX"); chmod 0700 "$T"
VERSION=$(env -u WAYLAND_DISPLAY -u DISPLAY -u DBUS_SESSION_BUS_ADDRESS HOME="$T" XDG_RUNTIME_DIR="$T" "$R/$PROG" --version)
rm -rf "$T"
[ "$VERSION" = "$PROG 0.1.1" ] || { echo "Release meldet nicht 0.1.1: $VERSION" >&2; exit 2; }
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
        print "# Querencia Einstellungen 0.1.1 (Kontrollzentrum im Stil von GNOME 2; deutsch und"
        print "# englisch über gettext; Rückstellung auf den Vorgabewert byte-gleich, eigene"
        print "# Sicherungen). Desktop-ID = GApplication-ID org.querencia.Einstellungen. Name,"
        print "# Comment und Keywords aus po/ (werkzeug/i18n.py desktop). Ablage mit Katalogen"
        print "# unter ~/opt/querencia/einstellungen/" n "/ (Querencia-Werkstatt, Einstellungen 0.1.1, 2026-10-20)."
        print "# Rückfall: bash ~/projects/querencia-einstellungen-wt-011/werkzeug/einstellungen-011-integrieren.sh zurueck"
        next
    }
    /^Exec=/ { print "Exec=" e; next }
    /^TryExec=/ { print "TryExec=" e; next }
    { print }' "crates/querencia-einstellungen/data/$DESKNAME" >"$V/$DESKNAME"
desktop-file-validate "$V/$DESKNAME"
(cd "$V" && sha256sum -- "$PROG" "share/locale/de/LC_MESSAGES/$PROG.mo" "share/locale/en/LC_MESSAGES/$PROG.mo" "$DESKNAME" >SHA256SUMS)
chmod 0555 "$V/$PROG"
find "$V" -type f ! -name "$PROG" -exec chmod 0444 {} +
find "$V" -type d -exec chmod 0555 {} +
(cd "$V" && sha256sum -c SHA256SUMS)
echo "Vorlage $V: $NEU (SHA256SUMS $(sha256sum "$V/SHA256SUMS" | cut -d' ' -f1))"
cat "$V/SHA256SUMS"
