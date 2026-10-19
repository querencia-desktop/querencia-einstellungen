#!/bin/bash
# Probe für einstellungen-g2z-integrieren.sh (0.0.4 → 0.0.5) in Wegwerf-
# Umgebungen nachweis/profile/integration-g2z/<fall>. Jede Umgebung bildet
# den Live-Stand nach G2-X nach: Ablage e5acdbead9555b6f (Kopie der echten),
# Link darauf, Menüeintrag = Kopie des ECHTEN Menüeintrags (mit Icon= von
# G2-V und Comment= von G2-X), protokoll.txt, bin-Links für Desktop/Panel,
# dazu die 0.0.2-Ablage (Rückfallziel des G2-X-Skripts, für die Gegenprobe).
# Umgelenkt über QE_OPT, QE_APPS, QE_STATE (und XDG_CONFIG_HOME für die
# mimeapps.list); Fehler über QE_PROBE=1 und QE_FEHLER. Echte Dateien werden
# nur gelesen.
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
S=${QE_SKRIPT:-$P/werkzeug/einstellungen-g2z-integrieren.sh}
G2X=$P/werkzeug/einstellungen-g2x-integrieren.sh
SYM=~/projects/querencia-symbole/integration/symbole-integrieren.sh
V=$P/integration/g2z/vorschlag
BASIS=$P/nachweis/profile/integration-g2z
EO=~/opt/querencia
EA=~/.local/share/applications
EM=~/.config/mimeapps.list
ALT=e5acdbead9555b6f
G2G16=03c6dec5e63966b9
NEU=$(grep '^PROG_NEU=' "$S" | cut -d= -f2 | cut -c1-16)
DESKNAME=org.querencia.Einstellungen.desktop
[ -n "$NEU" ] && [ ${#NEU} = 16 ] || { echo "PROG_NEU im Skript nicht gesetzt" >&2; exit 2; }
[ -f "$EO/einstellungen/$ALT/querencia-einstellungen" ] || { echo "0.0.4-Ablage fehlt live: $EO/einstellungen/$ALT" >&2; exit 2; }

if [ -e "$BASIS" ]; then
    [ -f "$BASIS/.querencia-testprofil" ] || { echo "kein Testordner: $BASIS" >&2; exit 2; }
    chmod -R u+w -- "$BASIS"; rm -rf -- "$BASIS"
fi
mkdir -p -- "$BASIS"; : >"$BASIS/.querencia-testprofil"
# Momentaufnahmen (andere Sessions können während der Probe live ändern)
cp -p "$EA/$DESKNAME" "$BASIS/menueeintrag-echt.desktop"
cp -p "$EM" "$BASIS/mimeapps-echt.list" 2>/dev/null || true
ECHT_ICON=$(grep -m1 '^Icon=' "$BASIS/menueeintrag-echt.desktop")
ECHT_COMMENT=$(grep -m1 '^Comment=' "$BASIS/menueeintrag-echt.desktop")
echo "echter Menüeintrag: $(sha256sum <"$BASIS/menueeintrag-echt.desktop" | cut -c1-16), $ECHT_ICON"

echt() {
    sha256sum "$EO/protokoll.txt" "$EO/einstellungen/$ALT/querencia-einstellungen" "$EM"
    readlink "$EO/bin/querencia-desktop" "$EO/bin/querencia-panel" "$EO/bin/querencia-einstellungen"
    ls -lai "$EO/bin" "$EO/einstellungen"
    [ ! -e "$EO/einstellungen/$NEU" ] && echo "keine 0.0.5-Ablage live"
}
echt_vorher=$(echt)

ok=0; bad=0
pruef() { if eval "$2"; then ok=$((ok+1)); echo "PASS   $1"; else bad=$((bad+1)); echo "FAIL   $1"; fi; }

umgebung() {
    U=$BASIS/$1
    mkdir -p "$U/opt/bin" "$U/opt/einstellungen/$ALT" "$U/opt/einstellungen/$G2G16" "$U/apps" "$U/state" "$U/config"
    cp -P "$EO/bin/querencia-desktop" "$EO/bin/querencia-panel" "$U/opt/bin/"
    cp -p "$EO/einstellungen/$ALT/querencia-einstellungen" "$U/opt/einstellungen/$ALT/"
    cp -p "$EO/einstellungen/$G2G16/querencia-einstellungen" "$U/opt/einstellungen/$G2G16/"
    chmod 0555 "$U/opt/einstellungen/$ALT" "$U/opt/einstellungen/$G2G16"
    echo "Quelle: Probe" >"$U/opt/einstellungen/$ALT.herkunft"
    ln -s "$U/opt/einstellungen/$ALT/querencia-einstellungen" "$U/opt/bin/querencia-einstellungen"
    cp -p "$EO/protokoll.txt" "$U/opt/protokoll.txt"
    cp -p "$BASIS/menueeintrag-echt.desktop" "$U/apps/$DESKNAME"
    cp -p "$U/apps/$DESKNAME" "$U/menueeintrag-ausgang.desktop"
    [ -f "$BASIS/mimeapps-echt.list" ] && cp -p "$BASIS/mimeapps-echt.list" "$U/config/mimeapps.list"
    export QE_OPT=$U/opt QE_APPS=$U/apps QE_STATE=$U/state QE_PROBE=1 XDG_CONFIG_HOME=$U/config
    unset QE_FEHLER
}
stand() {
    readlink "$U/opt/bin/querencia-einstellungen" 2>/dev/null || echo "kein Link"
    sha256sum "$U/apps/$DESKNAME" "$U/opt/protokoll.txt" 2>/dev/null | cut -c1-64
}
zustand() { bash "$S" pruefen 2>&1 | grep -oE '(Programm-Link|Menüeintrag) +[^ ]+: [a-z]+' | awk '{print $NF}' | tr '\n' ' '; }
zustand_g2x() { bash "$G2X" pruefen 2>&1 | grep -oE '(Programm-Link|Menüeintrag) +[^ ]+: [a-z]+' | awk '{print $NF}' | tr '\n' ' '; }
# G2-V 0.2: was meldet symbole-integrieren.sh pruefen (umgelenkt) für den Starter?
sym_starter() {
    mkdir -p "$U/icons/hicolor"
    SYM_OPT=$U/opt SYM_APPS=$U/apps SYM_ICONS=$U/icons SYM_STATE=$U/state bash "$SYM" pruefen 2>&1 |
        grep -oE '^symbole-integrieren: Starter +[^ ]+: [a-z]+' | awk '{print $NF}'
}
keine_reste() { [ -z "$(find "$U" -name '*.g2z-tmp*' 2>/dev/null)" ]; }
lauf() { bash "$S" "$@" >"$U/ausgabe.txt" 2>&1; echo $?; }
# alles außer Exec/TryExec gleich wie im Ausgang (Icon= von G2-V, Comment= von G2-X)
rest_gleich() { diff <(grep -vE '^(Exec|TryExec)=' "$U/menueeintrag-ausgang.desktop") <(grep -vE '^(Exec|TryExec)=' "$U/apps/$DESKNAME") >/dev/null; }
mime_gleich() { cmp -s "$U/config/mimeapps.list" "$BASIS/mimeapps-echt.list"; }
A2='alt alt '
N2='neu neu '

echo "=== 1 Ausgangslage (wie live nach G2-X)"
umgebung t01
rc=$(lauf pruefen); cat "$U/ausgabe.txt"
pruef "pruefen: Exit 0, alt alt, Stand 0.0.4, nennt Icon und mimeapps.list" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$A2' ] && grep -q 'Stand: alt' $U/ausgabe.txt && grep -qF '$ECHT_ICON' $U/ausgabe.txt && grep -q 'mimeapps.list vorhanden' $U/ausgabe.txt"
pruef "G2-X-Skript sieht denselben Stand als seinen Endstand (neu neu)" "[ \"\$(zustand_g2x)\" = '$N2' ]"

echo "=== 2 Fehler bei aktivieren"
for stelle in vor-erstem waehrend-programm nach-programm waehrend-menue nach-menue waehrend-programm-nach-mv waehrend-menue-nach-mv signal-vor-erstem signal-nach-programm signal-waehrend-menue-nach-mv; do
    echo "--- $stelle"
    umgebung "f-$stelle"; vorher=$(stand)
    export QE_FEHLER=$stelle
    rc=$(lauf aktivieren)
    unset QE_FEHLER
    pruef "$stelle: Exit 1, zurückgerollt, alles wie vorher, keine Reste" "[ $rc = 1 ] && grep -q 'zurückgerollt' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ] && [ \"\$(zustand)\" = '$A2' ] && keine_reste"
    rc=$(lauf aktivieren)
    pruef "$stelle: danach normale Aktivierung" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$N2' ]"
done

echo "=== 3 normale Aktivierung"
umgebung t03; n=$(wc -l <"$U/opt/protokoll.txt")
rc=$(lauf aktivieren); cat "$U/ausgabe.txt"
pruef "aktivieren: Exit 0, neu neu" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$N2' ]"
pruef "Ablage $NEU: 0555, byte-gleich mit dem Release-Binary 0.0.5, Herkunft" "[ \$(stat -c %a $U/opt/einstellungen/$NEU) = 555 ] && [ \$(stat -c %a $U/opt/einstellungen/$NEU/querencia-einstellungen) = 555 ] && cmp -s $U/opt/einstellungen/$NEU/querencia-einstellungen $V/querencia-einstellungen && grep -q '^Quelle: ' $U/opt/einstellungen/$NEU.herkunft"
pruef "Link → 0.0.5; --version meldet 0.0.5" "[ \"\$(readlink $U/opt/bin/querencia-einstellungen)\" = $U/opt/einstellungen/$NEU/querencia-einstellungen ] && [ \"\$($U/opt/bin/querencia-einstellungen --version)\" = 'querencia-einstellungen 0.0.5' ]"
pruef "Menüeintrag: gültig, Exec und TryExec auf $NEU" "desktop-file-validate $U/apps/$DESKNAME && grep -qx 'Exec=~/opt/querencia/einstellungen/$NEU/querencia-einstellungen' $U/apps/$DESKNAME && grep -qx 'TryExec=~/opt/querencia/einstellungen/$NEU/querencia-einstellungen' $U/apps/$DESKNAME"
pruef "Menüeintrag: alle anderen Zeilen byte-gleich, Icon= und Comment= unverändert" "rest_gleich && grep -qxF '$ECHT_ICON' $U/apps/$DESKNAME && grep -qxF '$ECHT_COMMENT' $U/apps/$DESKNAME && [ \$(wc -l <$U/apps/$DESKNAME) = \$(wc -l <$U/menueeintrag-ausgang.desktop) ]"
pruef "G2-V symbole-integrieren.sh pruefen (umgelenkt) meldet nach aktivieren weiter „Starter neu“" "[ \"\$(sym_starter)\" = neu ] && [ \$(grep -c '^Icon=' $U/apps/$DESKNAME) = 1 ]"
pruef "0.0.4-Ablage bleibt, genau eine Protokollzeile" "cmp -s $U/opt/einstellungen/$ALT/querencia-einstellungen $EO/einstellungen/$ALT/querencia-einstellungen && [ \$(wc -l <$U/opt/protokoll.txt) = $((n+1)) ]"
pruef "Sicherung (alt alt, Linkziel 0.0.4, Menüeintrag Ausgang, Schritte P D) und Journal" "s=\$(ls -d $U/state/sicherung-einstellungen-g2z-aktivieren-*) && [ \"\$(cat \$s/zustand-vorher.txt)\" = 'alt alt' ] && grep -q $ALT \$s/programm-link.txt && cmp -s \$s/$DESKNAME $U/menueeintrag-ausgang.desktop && [ \"\$(tr '\n' ' ' <\$s/schritte.txt)\" = 'P D ' ] && grep -q 'einstellungen-g2z: aktiviert' $U/state/panel-rueckfall.txt"
pruef "mimeapps.list byte-gleich" "mime_gleich"
pruef "G2-X-Skript verweigert auf dem 0.0.5-Stand (zurueck), nichts geändert" "v=\$(stand); bash $G2X zurueck >/dev/null 2>&1; [ \$? = 2 ] && [ \"\$(stand)\" = \"\$v\" ]"

echo "=== 4 erneut, zurück, wiederholt"
nsich=$(ls -d $U/state/sicherung-einstellungen-g2z-* | wc -l); neu=$(stand)
rc=$(lauf aktivieren)
pruef "zweites aktivieren: bereits aktiv, keine neue Sicherung" "[ $rc = 0 ] && grep -q 'bereits aktiv' $U/ausgabe.txt && [ \"\$(stand)\" = \"$neu\" ] && [ \$(ls -d $U/state/sicherung-einstellungen-g2z-* | wc -l) = $nsich ]"
rc=$(lauf zurueck)
pruef "zurueck: Exit 0, Link 0.0.4, Menüeintrag byte-gleich mit dem Ausgang, Ablage 0.0.5 bleibt, Reihenfolge D P" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$A2' ] && [ \"\$(readlink $U/opt/bin/querencia-einstellungen)\" = $U/opt/einstellungen/$ALT/querencia-einstellungen ] && cmp -s $U/apps/$DESKNAME $U/menueeintrag-ausgang.desktop && [ -x $U/opt/einstellungen/$NEU/querencia-einstellungen ] && [ \"\$(tr '\n' ' ' <\$(ls -d $U/state/sicherung-einstellungen-g2z-zurueck-*)/schritte.txt)\" = 'D P ' ]"
pruef "nach zurueck sieht das G2-X-Skript seinen Endstand (neu neu)" "[ \"\$(zustand_g2x)\" = '$N2' ]"
pruef "G2-V symbole-integrieren.sh pruefen meldet auch nach zurueck „Starter neu“" "[ \"\$(sym_starter)\" = neu ]"
rc=$(lauf zurueck)
pruef "zweites zurueck: nichts zu tun" "[ $rc = 0 ] && grep -q 'nichts zu tun' $U/ausgabe.txt"
rc=$(lauf aktivieren); rc2=$(lauf zurueck); rc3=$(lauf aktivieren)
pruef "aktivieren → zurück → aktivieren wiederholbar" "[ $rc$rc2$rc3 = 000 ] && [ \"\$(zustand)\" = '$N2' ] && rest_gleich"
pruef "Rückbau-Kette: G2-Z zurueck, dann G2-X zurueck → 0.0.2, Icon= bleibt" "[ \$(lauf zurueck) = 0 ] && bash $G2X zurueck >/dev/null 2>&1 && [ \"\$(zustand_g2x)\" = '$A2' ] && grep -qxF '$ECHT_ICON' $U/apps/$DESKNAME && [ \"\$(readlink $U/opt/bin/querencia-einstellungen)\" = $U/opt/einstellungen/$G2G16/querencia-einstellungen ]"
pruef "… danach verweigert G2-Z (Menüeintrag 0.0.2 ist für G2-Z fremd), nichts geändert" "v=\$(stand); [ \$(lauf aktivieren) = 2 ] && [ \"\$(stand)\" = \"\$v\" ]"
pruef "… G2-X aktivieren, dann G2-Z aktivieren: wieder 0.0.5" "bash $G2X aktivieren >/dev/null 2>&1 && [ \$(lauf aktivieren) = 0 ] && [ \"\$(zustand)\" = '$N2' ] && rest_gleich"
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
    pruef "$stelle: danach normaler Rückfall" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$A2' ]"
done

echo "=== 6 Icon=, Comment= und Teilstände"
umgebung t06a; sed -i 's/^Icon=.*/Icon=preferences-system/' "$U/apps/$DESKNAME"; cp -p "$U/apps/$DESKNAME" "$U/menueeintrag-ausgang.desktop"
rc=$(lauf aktivieren); rc2=$(lauf zurueck)
pruef "anderes Icon= (Stand vor G2-V) übersteht aktivieren und zurueck byte-gleich" "[ $rc$rc2 = 00 ] && cmp -s $U/apps/$DESKNAME $U/menueeintrag-ausgang.desktop"
umgebung t06b; lauf aktivieren >/dev/null; sed -i 's/^Comment=.*/Comment=anders/' "$U/apps/$DESKNAME"; cp -p "$U/apps/$DESKNAME" "$U/erwartet.desktop"
rc=$(lauf zurueck)
pruef "Comment= nach der Aktivierung geändert: zurueck lässt ihn stehen" "[ $rc = 0 ] && grep -qx 'Comment=anders' $U/apps/$DESKNAME && [ \"\$(zustand)\" = '$A2' ]"
umgebung t06c; lauf aktivieren >/dev/null; lauf zurueck >/dev/null
ln -sfn "$U/opt/einstellungen/$NEU/querencia-einstellungen" "$U/opt/bin/querencia-einstellungen"
rc=$(lauf aktivieren)
pruef "nur Link neu: aktivieren ergänzt nur D" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$N2' ] && [ \"\$(tr '\n' ' ' <\$(ls -td $U/state/sicherung-einstellungen-g2z-aktivieren-* | head -1)/schritte.txt)\" = 'D ' ]"
umgebung t06d; rm "$U/opt/bin/querencia-einstellungen"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "Link fehlt: verweigert, nichts geändert" "[ $rc = 2 ] && grep -q 'zuerst 0.0.4' $U/ausgabe.txt && [ ! -e $U/opt/einstellungen/$NEU ] && [ \"\$(stand)\" = \"$vorher\" ]"
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
pruef "0.0.4-Ablage fehlt (Rückfallziel): verweigert" "[ $rc = 2 ] && grep -q '0.0.4-Ablage' $U/ausgabe.txt"
umgebung t06j; mkdir -p "$U/opt/einstellungen/$NEU"; echo x >"$U/opt/einstellungen/$NEU/querencia-einstellungen"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "vorhandene 0.0.5-Ablage mit falschem Inhalt: verweigert" "[ $rc = 2 ] && grep -q '0.0.5-Ablage verändert' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t06k; lauf aktivieren >/dev/null; sed -i 's#^TryExec=.*#TryExec=/usr/bin/anders#' "$U/apps/$DESKNAME"; vorher=$(stand)
rc=$(lauf zurueck)
pruef "fremdes TryExec nach Aktivierung: zurueck ändert nichts" "[ $rc = 2 ] && [ \"\$(stand)\" = \"$vorher\" ]"

echo "=== 7 Sperre (gemeinsam mit G2-C/G2-G/G2-X-Skript)"
umgebung t07; vorher=$(stand)
( exec 7>"$U/state/einstellungen-integrieren.lock"; flock 7; sleep 5 ) & sleep 1
rc=$(lauf aktivieren); wait
pruef "zweiter Lauf während eines ersten: verweigert" "[ $rc = 2 ] && grep -q 'anderer Lauf' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"

echo "=== 8 mimeapps.list und desktop.ini werden nie angefasst"
umgebung t08; mkdir -p "$U/config/querencia"
printf '[Background]\nFile=\nMode=zoom\nColor=#1c4e96\n\n[Volumes]\nNetwork=false\n' >"$U/config/querencia/desktop.ini"; d=$(sha256sum <"$U/config/querencia/desktop.ini")
lauf aktivieren >/dev/null; rc=$(lauf pruefen); cp "$U/ausgabe.txt" "$U/ausgabe-pruefen.txt"; lauf zurueck >/dev/null
pruef "mimeapps.list und desktop.ini byte-gleich" "[ $rc = 0 ] && mime_gleich && [ \"\$(sha256sum <$U/config/querencia/desktop.ini)\" = \"$d\" ]"
umgebung t08b; rm -f "$U/config/mimeapps.list"
lauf aktivieren >/dev/null; rc=$(lauf pruefen); cp "$U/ausgabe.txt" "$U/ausgabe-pruefen.txt"; lauf zurueck >/dev/null
pruef "ohne mimeapps.list: pruefen meldet das, es entsteht keine" "[ $rc = 0 ] && grep -q 'keine mimeapps.list' $U/ausgabe-pruefen.txt && [ ! -e $U/config/mimeapps.list ]"
unset XDG_CONFIG_HOME

pruef "echte Dateien unverändert (Inhalt, Inode, mimeapps.list); live keine 0.0.5-Ablage" "[ \"\$echt_vorher\" = \"\$(echt)\" ] && echo \"\$echt_vorher\" | grep -q 'keine 0.0.5-Ablage live'"
# Hinweis: Ändert eine andere Session während der Probe live den Menüeintrag
# oder protokoll.txt, schlagen die beiden letzten Prüfungen an, ohne dass
# dieses Skript etwas geändert hat – dann Zeitstempel im Protokoll ansehen.
pruef "echter Menüeintrag unverändert seit Probebeginn" "cmp -s $EA/$DESKNAME $BASIS/menueeintrag-echt.desktop"
echo "ERGEBNIS integration-g2z: $ok PASS, $bad FAIL"
[ "$bad" = 0 ]
