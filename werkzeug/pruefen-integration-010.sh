#!/bin/bash
# Probe für einstellungen-010-integrieren.sh (0.0.9 → 0.1.0) in Wegwerf-
# Umgebungen nachweis/profile/integration-010/<fall>. Jede Umgebung bildet den
# Live-Stand nach G2-AJ nach: Ablage de7e95a52a8eec92 (Kopie der echten), Link
# darauf, Menüeintrag = Kopie des ECHTEN Menüeintrags, protokoll.txt,
# bin-Links für Desktop/Panel, dazu die 0.0.8-Ablage (Rückfallziel des
# G2-AJ-Skripts, für die Rückbaukette und den Fall „Ausgangsstand 0.0.8 wird
# verweigert“). Zusätzlich: ~/.config/querencia/labwc (umgelenkt) bleibt
# byte-gleich, live auch dconf. Umgelenkt über QE_OPT, QE_APPS, QE_STATE (und
# XDG_CONFIG_HOME); Fehler über QE_PROBE=1 und QE_FEHLER; beschädigte Vorlage
# über QE_VORSCHLAG. Echte Dateien werden nur gelesen. Aufbau wie
# pruefen-integration-g2aj.sh.
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
S=${QE_SKRIPT:-$P/werkzeug/einstellungen-010-integrieren.sh}
G2AJ=~/projects/querencia-einstellungen-wt-g2aj/werkzeug/einstellungen-g2aj-integrieren.sh
SYM=~/projects/querencia-symbole/integration/symbole-integrieren.sh
V=$P/integration/010/vorschlag
BASIS=$P/nachweis/profile/integration-010
EO=~/opt/querencia
EA=~/.local/share/applications
EM=~/.config/mimeapps.list
ALT=de7e95a52a8eec92
G2AG16=452599512a8b268a
NEU=$(grep ' querencia-einstellungen$' "$V/SHA256SUMS" | cut -c1-16)
DESKNAME=org.querencia.Einstellungen.desktop
MO_DE=share/locale/de/LC_MESSAGES/querencia-einstellungen.mo
MO_EN=share/locale/en/LC_MESSAGES/querencia-einstellungen.mo
[ -n "$NEU" ] && [ ${#NEU} = 16 ] || { echo "SHA256SUMS der Vorlage fehlt" >&2; exit 2; }
[ -f "$EO/einstellungen/$ALT/querencia-einstellungen" ] || { echo "0.0.9-Ablage fehlt live: $EO/einstellungen/$ALT" >&2; exit 2; }

if [ -e "$BASIS" ]; then
    [ -f "$BASIS/.querencia-testprofil" ] || { echo "kein Testordner: $BASIS" >&2; exit 2; }
    chmod -R u+w -- "$BASIS"; rm -rf -- "$BASIS"
fi
mkdir -p -- "$BASIS"; : >"$BASIS/.querencia-testprofil"
# Momentaufnahmen (andere Sessions können während der Probe live ändern)
cp -p "$EA/$DESKNAME" "$BASIS/menueeintrag-echt.desktop"
cp -p "$EM" "$BASIS/mimeapps-echt.list" 2>/dev/null || true
ECHT_ICON=$(grep -m1 '^Icon=' "$BASIS/menueeintrag-echt.desktop")
echo "echter Menüeintrag: $(sha256sum <"$BASIS/menueeintrag-echt.desktop" | cut -c1-16), $ECHT_ICON"

echt() {
    sha256sum "$EO/protokoll.txt" "$EO/einstellungen/$ALT/querencia-einstellungen" "$EM"
    readlink "$EO/bin/querencia-desktop" "$EO/bin/querencia-panel" "$EO/bin/querencia-einstellungen"
    ls -lai "$EO/bin" "$EO/einstellungen"
    [ ! -e "$EO/einstellungen/$NEU" ] && echo "keine 0.1.0-Ablage live"
    (cd ~/.config/querencia 2>/dev/null && find labwc -type f 2>/dev/null | sort | xargs -r sha256sum)
    sha256sum ~/.config/dconf/user
    ls ~/.local/state/querencia/ | grep -c 'einstellungen-010' || true
}
echt_vorher=$(echt)

ok=0; bad=0
pruef() { if eval "$2"; then ok=$((ok+1)); echo "PASS   $1"; else bad=$((bad+1)); echo "FAIL   $1"; fi; }

umgebung() {
    U=$BASIS/$1
    mkdir -p "$U/opt/bin" "$U/opt/einstellungen/$ALT" "$U/opt/einstellungen/$G2AG16" "$U/apps" "$U/state" "$U/config"
    cp -P "$EO/bin/querencia-desktop" "$EO/bin/querencia-panel" "$U/opt/bin/"
    cp -p "$EO/einstellungen/$ALT/querencia-einstellungen" "$U/opt/einstellungen/$ALT/"
    cp -p "$EO/einstellungen/$G2AG16/querencia-einstellungen" "$U/opt/einstellungen/$G2AG16/"
    chmod 0555 "$U/opt/einstellungen/$ALT" "$U/opt/einstellungen/$G2AG16"
    echo "Quelle: Probe" >"$U/opt/einstellungen/$ALT.herkunft"
    ln -s "$U/opt/einstellungen/$ALT/querencia-einstellungen" "$U/opt/bin/querencia-einstellungen"
    cp -p "$EO/protokoll.txt" "$U/opt/protokoll.txt"
    cp -p "$BASIS/menueeintrag-echt.desktop" "$U/apps/$DESKNAME"
    cp -p "$U/apps/$DESKNAME" "$U/menueeintrag-ausgang.desktop"
    [ -f "$BASIS/mimeapps-echt.list" ] && cp -p "$BASIS/mimeapps-echt.list" "$U/config/mimeapps.list"
    export QE_OPT=$U/opt QE_APPS=$U/apps QE_STATE=$U/state QE_PROBE=1 XDG_CONFIG_HOME=$U/config
    unset QE_FEHLER QE_VORSCHLAG
}
stand() {
    readlink "$U/opt/bin/querencia-einstellungen" 2>/dev/null || echo "kein Link"
    sha256sum "$U/apps/$DESKNAME" "$U/opt/protokoll.txt" 2>/dev/null | cut -c1-64
    ls -A "$U/opt/einstellungen"
}
zustand() { bash "$S" pruefen 2>&1 | grep -oE '(Programm-Link|Menüeintrag) +[^ ]+: [a-z]+' | awk '{print $NF}' | tr '\n' ' '; }
zustand_g2aj() { bash "$G2AJ" pruefen 2>&1 | grep -oE '(Programm-Link|Menüeintrag) +[^ ]+: [a-z]+' | awk '{print $NF}' | tr '\n' ' '; }
# G2-V 0.2: was meldet symbole-integrieren.sh pruefen (umgelenkt) für den Starter?
sym_starter() {
    mkdir -p "$U/icons/hicolor"
    SYM_OPT=$U/opt SYM_APPS=$U/apps SYM_ICONS=$U/icons SYM_STATE=$U/state bash "$SYM" pruefen 2>&1 |
        grep -oE '^symbole-integrieren: Starter +[^ ]+: [a-z]+' | awk '{print $NF}'
}
keine_reste() { [ -z "$(find "$U" -name '*.010-tmp*' 2>/dev/null)" ]; }
lauf() { bash "$S" "$@" >"$U/ausgabe.txt" 2>&1; echo $?; }
mime_gleich() { cmp -s "$U/config/mimeapps.list" "$BASIS/mimeapps-echt.list"; }
A2='alt alt '
N2='neu neu '

echo "=== 1 Ausgangslage (wie live nach G2-AJ)"
umgebung t01
rc=$(lauf pruefen); cat "$U/ausgabe.txt"
pruef "pruefen: Exit 0, alt alt, Stand $ALT, nennt labwc-Benutzerkopie, letzte Zeile alt alt" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$A2' ] && grep -q 'Stand: alt – Querencia Einstellungen $ALT' $U/ausgabe.txt && grep -q 'labwc-Benutzerkopie' $U/ausgabe.txt && [ \"\$(tail -1 $U/ausgabe.txt)\" = 'alt alt' ]"
pruef "G2-AJ-Skript sieht denselben Stand als seinen Endstand (neu neu)" "[ \"\$(zustand_g2aj)\" = '$N2' ]"
pruef "Vorlage: Binary 0555, Kataloge und Menüeintrag 0444, Ordner 0555, SHA256SUMS stimmt" "[ \$(stat -c %a $V/querencia-einstellungen) = 555 ] && [ \$(stat -c %a $V/$MO_DE) = 444 ] && [ \$(stat -c %a $V/$MO_EN) = 444 ] && [ \$(stat -c %a $V/$DESKNAME) = 444 ] && [ -z \"\$(find $V -type d ! -perm 555)\" ] && (cd $V && sha256sum -c --quiet SHA256SUMS)"
pruef "Vorlage: Menüeintrag gültig, zweisprachig, Icon wie live" "desktop-file-validate $V/$DESKNAME && grep -qx 'Name\\[de\\]=Systemeinstellungen' $V/$DESKNAME && grep -qx 'Name=System Settings' $V/$DESKNAME && grep -qxF '$ECHT_ICON' $V/$DESKNAME"

echo "=== 2 Fehler bei aktivieren"
for stelle in vor-erstem ablage waehrend-programm nach-programm waehrend-menue nach-menue waehrend-programm-nach-mv waehrend-menue-nach-mv signal-vor-erstem signal-nach-programm signal-waehrend-menue-nach-mv; do
    echo "--- $stelle"
    umgebung "f-$stelle"; vorher=$(stand)
    export QE_FEHLER=$stelle
    rc=$(lauf aktivieren)
    unset QE_FEHLER
    if [ "$stelle" = ablage ] || [ "$stelle" = vor-erstem ] || [ "$stelle" = signal-vor-erstem ]; then
        pruef "$stelle: Exit 1, zurückgerollt, alles wie vorher (keine Ablage), keine Reste" "[ $rc = 1 ] && grep -q 'zurückgerollt' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ] && [ \"\$(zustand)\" = '$A2' ] && keine_reste"
    else
        # die vollständige Ablage bleibt liegen (wie G2-AJ), Link und Menüeintrag wie vorher
        pruef "$stelle: Exit 1, zurückgerollt, Link und Menüeintrag wie vorher, keine Reste" "[ $rc = 1 ] && grep -q 'zurückgerollt' $U/ausgabe.txt && [ \"\$(stand | head -3)\" = \"\$(echo \"$vorher\" | head -3)\" ] && [ \"\$(zustand)\" = '$A2' ] && keine_reste"
    fi
    rc=$(lauf aktivieren)
    pruef "$stelle: danach normale Aktivierung" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$N2' ]"
done

echo "=== 3 normale Aktivierung"
umgebung t03; n=$(wc -l <"$U/opt/protokoll.txt")
rc=$(lauf aktivieren); cat "$U/ausgabe.txt"
pruef "aktivieren: Exit 0, neu neu" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$N2' ]"
pruef "Ablage $NEU: genau Binary, de.mo, en.mo, SHA256SUMS; Ordner 0555, Binary 0555, Rest 0444; byte-gleich mit der Vorlage; Herkunft" "[ \"\$(cd $U/opt/einstellungen/$NEU && find . -type f | LC_ALL=C sort | tr '\n' ' ')\" = './SHA256SUMS ./querencia-einstellungen ./$MO_DE ./$MO_EN ' ] && [ -z \"\$(find $U/opt/einstellungen/$NEU -type d ! -perm 555)\" ] && [ \$(stat -c %a $U/opt/einstellungen/$NEU/querencia-einstellungen) = 555 ] && [ \$(stat -c %a $U/opt/einstellungen/$NEU/$MO_DE) = 444 ] && cmp -s $U/opt/einstellungen/$NEU/querencia-einstellungen $V/querencia-einstellungen && cmp -s $U/opt/einstellungen/$NEU/$MO_DE $V/$MO_DE && cmp -s $U/opt/einstellungen/$NEU/$MO_EN $V/$MO_EN && (cd $U/opt/einstellungen/$NEU && sha256sum -c --quiet SHA256SUMS) && grep -q '^Quelle: ' $U/opt/einstellungen/$NEU.herkunft"
pruef "Link → 0.1.0; --version meldet 0.1.0" "[ \"\$(readlink $U/opt/bin/querencia-einstellungen)\" = $U/opt/einstellungen/$NEU/querencia-einstellungen ] && [ \"\$($U/opt/bin/querencia-einstellungen --version)\" = 'querencia-einstellungen 0.1.0' ]"
pruef "Menüeintrag: byte-gleich mit der Vorlage, gültig, Exec und TryExec auf $NEU, Icon wie vorher" "cmp -s $U/apps/$DESKNAME $V/$DESKNAME && desktop-file-validate $U/apps/$DESKNAME && grep -qx 'Exec=~/opt/querencia/einstellungen/$NEU/querencia-einstellungen' $U/apps/$DESKNAME && grep -qx 'TryExec=~/opt/querencia/einstellungen/$NEU/querencia-einstellungen' $U/apps/$DESKNAME && grep -qxF '$ECHT_ICON' $U/apps/$DESKNAME"
pruef "Menüeintrag des Ausgangs byte-genau gemerkt" "cmp -s $U/state/einstellungen-010-menue-vorher.desktop $U/menueeintrag-ausgang.desktop"
pruef "G2-V symbole-integrieren.sh pruefen (umgelenkt) meldet nach aktivieren weiter „Starter neu“" "[ \"\$(sym_starter)\" = neu ] && [ \$(grep -c '^Icon=' $U/apps/$DESKNAME) = 1 ]"
pruef "0.0.9-Ablage bleibt, genau eine Protokollzeile" "cmp -s $U/opt/einstellungen/$ALT/querencia-einstellungen $EO/einstellungen/$ALT/querencia-einstellungen && [ \$(wc -l <$U/opt/protokoll.txt) = $((n+1)) ]"
pruef "Sicherung (alt alt, Linkziel 0.0.9, Ausgangsstand gemerkt, Schritte P D) und Journal" "s=\$(ls -d $U/state/sicherung-einstellungen-010-aktivieren-*) && [ \"\$(cat \$s/zustand-vorher.txt)\" = 'alt alt' ] && grep -q $ALT \$s/programm-link.txt && grep -q $ALT \$s/ausgangsstand.txt && grep -q $ALT $U/state/einstellungen-010-vorher.txt && cmp -s \$s/$DESKNAME $U/menueeintrag-ausgang.desktop && [ \"\$(tr '\n' ' ' <\$s/schritte.txt)\" = 'P D ' ] && grep -q 'einstellungen-010: aktiviert' $U/state/panel-rueckfall.txt"
pruef "mimeapps.list byte-gleich" "mime_gleich"
pruef "G2-AJ-Skript verweigert auf dem 0.1.0-Stand (zurueck), nichts geändert" "v=\$(stand); bash $G2AJ zurueck >/dev/null 2>&1; [ \$? = 2 ] && [ \"\$(stand)\" = \"\$v\" ]"

echo "=== 4 erneut, zurück, wiederholt"
nsich=$(ls -d $U/state/sicherung-einstellungen-010-* | wc -l); neu=$(stand)
rc=$(lauf aktivieren)
pruef "zweites aktivieren: bereits aktiv, keine neue Sicherung" "[ $rc = 0 ] && grep -q 'bereits aktiv' $U/ausgabe.txt && [ \"\$(stand)\" = \"$neu\" ] && [ \$(ls -d $U/state/sicherung-einstellungen-010-* | wc -l) = $nsich ]"
rc=$(lauf zurueck)
pruef "zurueck: Exit 0, Link 0.0.9, Menüeintrag byte-gleich mit dem Ausgang, Ablage 0.1.0 bleibt, Reihenfolge D P" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$A2' ] && [ \"\$(readlink $U/opt/bin/querencia-einstellungen)\" = $U/opt/einstellungen/$ALT/querencia-einstellungen ] && cmp -s $U/apps/$DESKNAME $U/menueeintrag-ausgang.desktop && [ -x $U/opt/einstellungen/$NEU/querencia-einstellungen ] && [ \"\$(tr '\n' ' ' <\$(ls -d $U/state/sicherung-einstellungen-010-zurueck-*)/schritte.txt)\" = 'D P ' ]"
pruef "nach zurueck sieht das G2-AJ-Skript seinen Endstand (neu neu)" "[ \"\$(zustand_g2aj)\" = '$N2' ]"
pruef "G2-V symbole-integrieren.sh pruefen meldet auch nach zurueck „Starter neu“" "[ \"\$(sym_starter)\" = neu ]"
rc=$(lauf zurueck)
pruef "zweites zurueck: nichts zu tun" "[ $rc = 0 ] && grep -q 'nichts zu tun' $U/ausgabe.txt"
rc=$(lauf aktivieren); rc2=$(lauf zurueck); rc3=$(lauf aktivieren)
pruef "aktivieren → zurück → aktivieren wiederholbar" "[ $rc$rc2$rc3 = 000 ] && [ \"\$(zustand)\" = '$N2' ] && cmp -s $U/apps/$DESKNAME $V/$DESKNAME"
pruef "Rückbau-Kette: 0.1.0 zurueck, dann G2-AJ zurueck → 0.0.8" "[ \$(lauf zurueck) = 0 ] && bash $G2AJ zurueck >/dev/null 2>&1 && [ \"\$(zustand_g2aj)\" = '$A2' ] && grep -qxF '$ECHT_ICON' $U/apps/$DESKNAME && [ \"\$(readlink $U/opt/bin/querencia-einstellungen)\" = $U/opt/einstellungen/$G2AG16/querencia-einstellungen ]"
pruef "… danach: 0.0.8 ist kein zulässiger Ausgangsstand, pruefen meldet fremd fremd, aktivieren verweigert" "[ \"\$(zustand)\" = 'fremd fremd ' ] && v=\$(stand) && [ \$(lauf aktivieren) = 2 ] && [ \"\$(stand)\" = \"\$v\" ]"
pruef "… G2-AJ aktivieren, dann 0.1.0 aktivieren: wieder 0.1.0" "bash $G2AJ aktivieren >/dev/null 2>&1 && [ \$(lauf aktivieren) = 0 ] && [ \"\$(zustand)\" = '$N2' ]"
pruef "mimeapps.list nach allen Läufen byte-gleich" "mime_gleich"

echo "=== 5 Fehler bei zurueck"
for stelle in zurueck-vor-erstem zurueck-waehrend-menue zurueck-nach-menue zurueck-waehrend-programm zurueck-waehrend-programm-nach-mv signal-zurueck-nach-menue; do
    echo "--- $stelle"
    umgebung "z-$stelle"; lauf aktivieren >/dev/null; vorher=$(stand)
    export QE_FEHLER=$stelle
    rc=$(lauf zurueck)
    unset QE_FEHLER
    pruef "$stelle: Exit 1, Zustand neu neu wie vor dem Rückfall" "[ $rc = 1 ] && [ \"\$(stand)\" = \"$vorher\" ] && [ \"\$(zustand)\" = '$N2' ] && keine_reste"
    rc=$(lauf zurueck)
    pruef "$stelle: danach normaler Rückfall" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$A2' ] && cmp -s $U/apps/$DESKNAME $U/menueeintrag-ausgang.desktop"
done

echo "=== 6 Teilstände und fremde Stände"
umgebung t06a; sed -i 's/^Icon=.*/Icon=preferences-system/' "$U/apps/$DESKNAME"; cp -p "$U/apps/$DESKNAME" "$U/menueeintrag-ausgang.desktop"
rc=$(lauf aktivieren); rc2=$(lauf zurueck)
pruef "anderes Icon= im Ausgang: zurueck stellt den Menüeintrag byte-gleich her" "[ $rc$rc2 = 00 ] && cmp -s $U/apps/$DESKNAME $U/menueeintrag-ausgang.desktop"
umgebung t06b; lauf aktivieren >/dev/null; rm "$U/state/einstellungen-010-menue-vorher.desktop"; vorher=$(stand)
rc=$(lauf zurueck)
pruef "gemerkter Menüeintrag fehlt: zurueck verweigert, nichts geändert" "[ $rc = 2 ] && grep -q 'gemerkter Menüeintrag fehlt' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t06c; lauf aktivieren >/dev/null; lauf zurueck >/dev/null
ln -sfn "$U/opt/einstellungen/$NEU/querencia-einstellungen" "$U/opt/bin/querencia-einstellungen"
rc=$(lauf aktivieren)
pruef "nur Link neu: aktivieren ergänzt nur D" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$N2' ] && [ \"\$(tr '\n' ' ' <\$(ls -td $U/state/sicherung-einstellungen-010-aktivieren-* | head -1)/schritte.txt)\" = 'D ' ]"
umgebung t06d; rm "$U/opt/bin/querencia-einstellungen"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "Link fehlt: verweigert, nichts geändert" "[ $rc = 2 ] && grep -q 'zuerst die Vorversionen' $U/ausgabe.txt && [ ! -e $U/opt/einstellungen/$NEU ] && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t06e; ln -sfn /nirgends "$U/opt/bin/querencia-einstellungen"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "fremder Link: verweigert, nichts geändert" "[ $rc = 2 ] && grep -q 'Programm ist fremd' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t06f; sed -i 's#^Exec=.*#Exec=/usr/bin/anders#' "$U/apps/$DESKNAME"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "fremdes Exec im Menüeintrag: verweigert" "[ $rc = 2 ] && grep -q 'Menüeintrag ist fremd' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t06g; echo 'Exec=/zweite' >>"$U/apps/$DESKNAME"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "doppelte Exec-Zeile: verweigert" "[ $rc = 2 ] && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t06h; printf '\n[Desktop Action neu]\nName=Neu\nExec=/x\n' >>"$U/apps/$DESKNAME"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "zusätzliche Gruppe im Menüeintrag: verweigert (nicht raten)" "[ $rc = 2 ] && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t06i; chmod u+w "$U/opt/einstellungen/$ALT"; rm "$U/opt/einstellungen/$ALT/querencia-einstellungen"
rc=$(lauf aktivieren)
pruef "Ablage des Ausgangsstands fehlt (Rückfallziel): verweigert" "[ $rc = 2 ] && grep -q 'Ablage des Ausgangsstands' $U/ausgabe.txt"
umgebung t06j; mkdir -p "$U/opt/einstellungen/$NEU"; cp "$V/querencia-einstellungen" "$U/opt/einstellungen/$NEU/"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "vorhandene 0.1.0-Ablage ohne Kataloge: verweigert (nie eine Binary ohne de.mo)" "[ $rc = 2 ] && grep -q '0.1.0-Ablage unvollständig' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t06k; lauf aktivieren >/dev/null; printf '\nComment=anders\n' >>"$U/apps/$DESKNAME"; vorher=$(stand)
rc=$(lauf zurueck)
pruef "Menüeintrag nach Aktivierung von Hand geändert: zurueck ändert nichts (fremd)" "[ $rc = 2 ] && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t06l; lauf aktivieren >/dev/null; chmod u+w "$U/opt/einstellungen/$NEU" "$U/opt/einstellungen/$NEU/share/locale/de/LC_MESSAGES"; rm "$U/opt/einstellungen/$NEU/$MO_DE"; vorher=$(stand)
rc=$(lauf pruefen); rc2=$(lauf zurueck)
pruef "de.mo aus der Ablage verschwunden: pruefen und zurueck verweigern (Ablage unvollständig), nichts geändert" "[ $rc = 2 ] && [ $rc2 = 2 ] && grep -q 'unvollständig' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"

echo "=== 6b Ausgangsstand 0.0.8 (G2-AJ nicht aktiv): verweigert, nichts geändert"
umgebung t06z
ln -sfn "$U/opt/einstellungen/$G2AG16/querencia-einstellungen" "$U/opt/bin/querencia-einstellungen"
sed -i "s#/einstellungen/$ALT/#/einstellungen/$G2AG16/#" "$U/apps/$DESKNAME"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "0.0.8 als Ausgang: verweigert (nur 0.0.9 erlaubt), keine Ablage angelegt" "[ $rc = 2 ] && grep -q 'Programm ist fremd' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ] && [ ! -e $U/opt/einstellungen/$NEU ]"

echo "=== 6c beschädigte Vorlage (QE_VORSCHLAG, nur in der Probe)"
umgebung t06v; K=$U/vorlage; cp -a "$V" "$K"; chmod -R u+w "$K"
rm "$K/$MO_DE"; (cd "$K" && grep -v " $MO_DE\$" SHA256SUMS >S && mv S SHA256SUMS); vorher=$(stand)
rc=$(QE_VORSCHLAG=$K lauf aktivieren)
pruef "Vorlage ohne de.mo: verweigert (sonst sähe der Benutzer Englisch), nichts geändert" "[ $rc = 2 ] && grep -q 'deutscher Katalog fehlt' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t06w; K=$U/vorlage; cp -a "$V" "$K"; chmod -R u+w "$K"; printf 'x' >>"$K/$MO_EN"; vorher=$(stand)
rc=$(QE_VORSCHLAG=$K lauf aktivieren)
pruef "Vorlage mit verändertem Katalog: verweigert (SHA256SUMS), nichts geändert" "[ $rc = 2 ] && grep -q 'stimmt nicht mit SHA256SUMS' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t06x; vorher=$(stand); unset QE_PROBE
rc=$(QE_VORSCHLAG=/nirgends lauf pruefen)
pruef "QE_VORSCHLAG ohne QE_PROBE wird nicht beachtet" "[ $rc = 0 ] && [ \"\$(stand)\" = \"$vorher\" ] && [ \"\$(tail -1 $U/ausgabe.txt)\" = 'alt alt' ]"
export QE_PROBE=1

echo "=== 7 Sperre (gemeinsam mit den anderen einstellungen-*-integrieren.sh)"
umgebung t07; vorher=$(stand)
( exec 7>"$U/state/einstellungen-integrieren.lock"; flock 7; sleep 5 ) & sleep 1
rc=$(lauf aktivieren); wait
pruef "zweiter Lauf während eines ersten: verweigert" "[ $rc = 2 ] && grep -q 'anderer Lauf' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"

echo "=== 8 mimeapps.list, desktop.ini und labwc-Benutzerkopie werden nie angefasst"
umgebung t08; mkdir -p "$U/config/querencia"
printf '[Background]\nFile=\nMode=zoom\nColor=#1c4e96\n\n[Volumes]\nNetwork=false\n' >"$U/config/querencia/desktop.ini"; d=$(sha256sum <"$U/config/querencia/desktop.ini")
mkdir -p "$U/config/querencia/labwc/environment.d"; printf '<labwc_config/>\n' >"$U/config/querencia/labwc/rc.xml"; printf 'XCURSOR_SIZE=32\nLANG=de_DE.UTF-8\n' >"$U/config/querencia/labwc/environment.d/zz-querencia-einstellungen.env"
l=$(cd "$U/config/querencia" && find labwc -type f | sort | xargs sha256sum)
lauf aktivieren >/dev/null; rc=$(lauf pruefen); cp "$U/ausgabe.txt" "$U/ausgabe-pruefen.txt"; lauf zurueck >/dev/null
pruef "mimeapps.list, desktop.ini und labwc-Benutzerkopie (mit Sprache) byte-gleich" "[ $rc = 0 ] && mime_gleich && [ \"\$(sha256sum <$U/config/querencia/desktop.ini)\" = \"$d\" ] && [ \"\$(cd $U/config/querencia && find labwc -type f | sort | xargs sha256sum)\" = \"$l\" ]"
unset XDG_CONFIG_HOME

pruef "echte Dateien unverändert (Inhalt, Inode, mimeapps.list, labwc-Benutzerkopie, dconf, kein Zustand von 0.1.0); live keine 0.1.0-Ablage" "[ \"\$echt_vorher\" = \"\$(echt)\" ] && echo \"\$echt_vorher\" | grep -q 'keine 0.1.0-Ablage live'"
# Hinweis: Ändert eine andere Session während der Probe live den Menüeintrag
# oder protokoll.txt, schlagen die beiden letzten Prüfungen an, ohne dass
# dieses Skript etwas geändert hat – dann Zeitstempel im Protokoll ansehen.
pruef "echter Menüeintrag unverändert seit Probebeginn" "cmp -s $EA/$DESKNAME $BASIS/menueeintrag-echt.desktop"
echo "ERGEBNIS integration-010: $ok PASS, $bad FAIL"
[ "$bad" = 0 ]
