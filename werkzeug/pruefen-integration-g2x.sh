#!/bin/bash
# Probe für einstellungen-g2x-integrieren.sh (0.0.2 → 0.0.3) in Wegwerf-
# Umgebungen nachweis/profile/integration-g2x/<fall>. Jede Umgebung bildet
# den Live-Stand nach G2-G nach: Ablage 03c6dec5e63966b9 (Kopie der echten),
# Link darauf, Menüeintrag = Kopie des ECHTEN Menüeintrags (mit dem Icon=,
# das gerade live gilt), protokoll.txt, bin-Links für Desktop/Panel.
# Umgelenkt über QE_OPT, QE_APPS, QE_STATE; Fehler über QE_PROBE=1 und
# QE_FEHLER. Echte Dateien werden nur gelesen.
set -uo pipefail
P=~/projects/querencia-einstellungen
S=${QE_SKRIPT:-$P/werkzeug/einstellungen-g2x-integrieren.sh}
G2G=$P/werkzeug/einstellungen-g2g-integrieren.sh
V=$P/integration/g2x/vorschlag
BASIS=$P/nachweis/profile/integration-g2x
EO=~/opt/querencia
EA=~/.local/share/applications
ALT=03c6dec5e63966b9
NEU=$(grep '^PROG_NEU=' "$S" | cut -d= -f2 | cut -c1-16)
DESKNAME=org.querencia.Einstellungen.desktop
[ -n "$NEU" ] && [ ${#NEU} = 16 ] || { echo "PROG_NEU im Skript nicht gesetzt" >&2; exit 2; }
[ -f "$EO/einstellungen/$ALT/querencia-einstellungen" ] || { echo "0.0.2-Ablage fehlt live: $EO/einstellungen/$ALT" >&2; exit 2; }

if [ -e "$BASIS" ]; then
    [ -f "$BASIS/.querencia-testprofil" ] || { echo "kein Testordner: $BASIS" >&2; exit 2; }
    chmod -R u+w -- "$BASIS"; rm -rf -- "$BASIS"
fi
mkdir -p -- "$BASIS"; : >"$BASIS/.querencia-testprofil"
# Momentaufnahme des echten Menüeintrags (G2-V kann ihn während der Probe ändern)
cp -p "$EA/$DESKNAME" "$BASIS/menueeintrag-echt.desktop"
ECHT_ICON=$(grep -m1 '^Icon=' "$BASIS/menueeintrag-echt.desktop")
echo "echter Menüeintrag: $(sha256sum <"$BASIS/menueeintrag-echt.desktop" | cut -c1-16), $ECHT_ICON"

echt() {
    sha256sum "$EO/protokoll.txt" "$EO/einstellungen/$ALT/querencia-einstellungen"
    readlink "$EO/bin/querencia-desktop" "$EO/bin/querencia-panel" "$EO/bin/querencia-einstellungen"
    ls -lai "$EO/bin" "$EO/einstellungen"
    [ ! -e "$EO/einstellungen/$NEU" ] && echo "keine 0.0.3-Ablage live"
}
echt_vorher=$(echt)

ok=0; bad=0
pruef() { if eval "$2"; then ok=$((ok+1)); echo "PASS   $1"; else bad=$((bad+1)); echo "FAIL   $1"; fi; }

umgebung() {
    U=$BASIS/$1
    mkdir -p "$U/opt/bin" "$U/opt/einstellungen/$ALT" "$U/apps" "$U/state"
    cp -P "$EO/bin/querencia-desktop" "$EO/bin/querencia-panel" "$U/opt/bin/"
    cp -p "$EO/einstellungen/$ALT/querencia-einstellungen" "$U/opt/einstellungen/$ALT/"
    chmod 0555 "$U/opt/einstellungen/$ALT"
    echo "Quelle: Probe" >"$U/opt/einstellungen/$ALT.herkunft"
    ln -s "$U/opt/einstellungen/$ALT/querencia-einstellungen" "$U/opt/bin/querencia-einstellungen"
    cp -p "$EO/protokoll.txt" "$U/opt/protokoll.txt"
    cp -p "$BASIS/menueeintrag-echt.desktop" "$U/apps/$DESKNAME"
    cp -p "$U/apps/$DESKNAME" "$U/menueeintrag-ausgang.desktop"
    export QE_OPT=$U/opt QE_APPS=$U/apps QE_STATE=$U/state QE_PROBE=1
    unset QE_FEHLER
}
stand() {
    readlink "$U/opt/bin/querencia-einstellungen" 2>/dev/null || echo "kein Link"
    sha256sum "$U/apps/$DESKNAME" "$U/opt/protokoll.txt" 2>/dev/null | cut -c1-64
}
zustand() { bash "$S" pruefen 2>&1 | grep -oE '(Programm-Link|Menüeintrag) +[^ ]+: [a-z]+' | awk '{print $NF}' | tr '\n' ' '; }
zustand_g2g() { bash "$G2G" pruefen 2>&1 | grep -oE '(Programm-Link|Menüeintrag) +[^ ]+: [a-z]+' | awk '{print $NF}' | tr '\n' ' '; }
keine_reste() { [ -z "$(find "$U" -name '*.g2x-tmp*' 2>/dev/null)" ]; }
lauf() { bash "$S" "$@" >"$U/ausgabe.txt" 2>&1; echo $?; }
# alles außer Exec/TryExec/Comment gleich wie im Ausgang (Icon= von G2-V!)
rest_gleich() { diff <(grep -vE '^(Exec|TryExec|Comment)=' "$U/menueeintrag-ausgang.desktop") <(grep -vE '^(Exec|TryExec|Comment)=' "$U/apps/$DESKNAME") >/dev/null; }
A2='alt alt '
N2='neu neu '

echo "=== 1 Ausgangslage (wie live nach G2-G)"
umgebung t01
rc=$(lauf pruefen); cat "$U/ausgabe.txt"
pruef "pruefen: Exit 0, alt alt, Stand 0.0.2, nennt Icon" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$A2' ] && grep -q 'Stand: alt' $U/ausgabe.txt && grep -qF '$ECHT_ICON' $U/ausgabe.txt"

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
pruef "Ablage $NEU: 0555, byte-gleich mit dem Release-Binary 0.0.3, Herkunft" "[ \$(stat -c %a $U/opt/einstellungen/$NEU) = 555 ] && [ \$(stat -c %a $U/opt/einstellungen/$NEU/querencia-einstellungen) = 555 ] && cmp -s $U/opt/einstellungen/$NEU/querencia-einstellungen $V/querencia-einstellungen && grep -q '^Quelle: ' $U/opt/einstellungen/$NEU.herkunft"
pruef "Link → 0.0.3; --version meldet 0.0.3" "[ \"\$(readlink $U/opt/bin/querencia-einstellungen)\" = $U/opt/einstellungen/$NEU/querencia-einstellungen ] && [ \"\$($U/opt/bin/querencia-einstellungen --version)\" = 'querencia-einstellungen 0.0.3' ]"
pruef "Menüeintrag: gültig, Exec und TryExec auf $NEU, Comment nennt Symbole" "desktop-file-validate $U/apps/$DESKNAME && grep -qx 'Exec=~/opt/querencia/einstellungen/$NEU/querencia-einstellungen' $U/apps/$DESKNAME && grep -qx 'TryExec=~/opt/querencia/einstellungen/$NEU/querencia-einstellungen' $U/apps/$DESKNAME && grep -q '^Comment=.*Symbole der Arbeitsfläche' $U/apps/$DESKNAME"
pruef "Menüeintrag: alle anderen Zeilen byte-gleich, Icon= unverändert ($ECHT_ICON)" "rest_gleich && grep -qxF '$ECHT_ICON' $U/apps/$DESKNAME && [ \$(wc -l <$U/apps/$DESKNAME) = \$(wc -l <$U/menueeintrag-ausgang.desktop) ]"
pruef "0.0.2-Ablage bleibt, genau eine Protokollzeile" "cmp -s $U/opt/einstellungen/$ALT/querencia-einstellungen $EO/einstellungen/$ALT/querencia-einstellungen && [ \$(wc -l <$U/opt/protokoll.txt) = $((n+1)) ]"
pruef "Sicherung (alt alt, Linkziel 0.0.2, Menüeintrag Ausgang, Schritte P D) und Journal" "s=\$(ls -d $U/state/sicherung-einstellungen-g2x-aktivieren-*) && [ \"\$(cat \$s/zustand-vorher.txt)\" = 'alt alt' ] && grep -q $ALT \$s/programm-link.txt && cmp -s \$s/$DESKNAME $U/menueeintrag-ausgang.desktop && [ \"\$(tr '\n' ' ' <\$s/schritte.txt)\" = 'P D ' ] && grep -q 'einstellungen-g2x: aktiviert' $U/state/panel-rueckfall.txt"
pruef "G2-G-Skript verweigert auf dem 0.0.3-Stand, nichts geändert" "v=\$(stand); bash $G2G zurueck >/dev/null 2>&1; [ \$? = 2 ] && [ \"\$(stand)\" = \"\$v\" ]"

echo "=== 4 erneut, zurück, wiederholt"
nsich=$(ls -d $U/state/sicherung-einstellungen-g2x-* | wc -l); neu=$(stand)
rc=$(lauf aktivieren)
pruef "zweites aktivieren: bereits aktiv, keine neue Sicherung" "[ $rc = 0 ] && grep -q 'bereits aktiv' $U/ausgabe.txt && [ \"\$(stand)\" = \"$neu\" ] && [ \$(ls -d $U/state/sicherung-einstellungen-g2x-* | wc -l) = $nsich ]"
rc=$(lauf zurueck)
pruef "zurueck: Exit 0, Link 0.0.2, Menüeintrag byte-gleich mit dem Ausgang, Ablage 0.0.3 bleibt, Reihenfolge D P" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$A2' ] && [ \"\$(readlink $U/opt/bin/querencia-einstellungen)\" = $U/opt/einstellungen/$ALT/querencia-einstellungen ] && cmp -s $U/apps/$DESKNAME $U/menueeintrag-ausgang.desktop && [ -x $U/opt/einstellungen/$NEU/querencia-einstellungen ] && [ \"\$(tr '\n' ' ' <\$(ls -d $U/state/sicherung-einstellungen-g2x-zurueck-*)/schritte.txt)\" = 'D P ' ]"
pruef "nach zurueck sieht das G2-G-Skript seinen Endstand (neu neu), sofern der echte Menüeintrag noch der G2-G-Vorlage gleicht" "if cmp -s $U/menueeintrag-ausgang.desktop $P/integration/g2g/vorschlag/$DESKNAME; then [ \"\$(zustand_g2g)\" = '$N2' ]; else echo '  (Menüeintrag live schon von G2-V geändert – G2-G-Skript sieht ihn als fremd, erwartet)'; true; fi"
rc=$(lauf zurueck)
pruef "zweites zurueck: nichts zu tun" "[ $rc = 0 ] && grep -q 'nichts zu tun' $U/ausgabe.txt"
rc=$(lauf aktivieren); rc2=$(lauf zurueck); rc3=$(lauf aktivieren)
pruef "aktivieren → zurück → aktivieren wiederholbar" "[ $rc$rc2$rc3 = 000 ] && [ \"\$(zustand)\" = '$N2' ] && rest_gleich"

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

echo "=== 6 Icon= und Teilstände"
umgebung t06a; sed -i 's/^Icon=.*/Icon=org.querencia.Einstellungen/' "$U/apps/$DESKNAME"; cp -p "$U/apps/$DESKNAME" "$U/menueeintrag-ausgang.desktop"
rc=$(lauf aktivieren); rc2=$(lauf zurueck)
pruef "eigenes Symbol (Icon=org.querencia.Einstellungen) übersteht aktivieren und zurueck" "[ $rc$rc2 = 00 ] && cmp -s $U/apps/$DESKNAME $U/menueeintrag-ausgang.desktop"
lauf aktivieren >/dev/null
pruef "… und steht nach erneutem aktivieren weiter da" "grep -qx 'Icon=org.querencia.Einstellungen' $U/apps/$DESKNAME && ! grep -q 'preferences-system' $U/apps/$DESKNAME"
umgebung t06b; lauf aktivieren >/dev/null; sed -i 's/^Icon=.*/Icon=org.querencia.Einstellungen/' "$U/apps/$DESKNAME"; cp -p "$U/apps/$DESKNAME" "$U/erwartet.desktop"
rc=$(lauf zurueck)
pruef "Icon= nach der Aktivierung geändert (G2-V danach): zurueck lässt es stehen" "[ $rc = 0 ] && grep -qx 'Icon=org.querencia.Einstellungen' $U/apps/$DESKNAME && [ \"\$(zustand)\" = '$A2' ]"
umgebung t06c; lauf aktivieren >/dev/null; lauf zurueck >/dev/null
ln -sfn "$U/opt/einstellungen/$NEU/querencia-einstellungen" "$U/opt/bin/querencia-einstellungen"
rc=$(lauf aktivieren)
pruef "nur Link neu: aktivieren ergänzt nur D" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$N2' ] && [ \"\$(tr '\n' ' ' <\$(ls -td $U/state/sicherung-einstellungen-g2x-aktivieren-* | head -1)/schritte.txt)\" = 'D ' ]"
umgebung t06d; rm "$U/opt/bin/querencia-einstellungen"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "Link fehlt (0.0.2 nicht installiert): verweigert, nichts geändert" "[ $rc = 2 ] && grep -q 'zuerst 0.0.2' $U/ausgabe.txt && [ ! -e $U/opt/einstellungen/$NEU ] && [ \"\$(stand)\" = \"$vorher\" ]"
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
pruef "0.0.2-Ablage fehlt (Rückfallziel): verweigert" "[ $rc = 2 ] && grep -q '0.0.2-Ablage' $U/ausgabe.txt"
umgebung t06j; mkdir -p "$U/opt/einstellungen/$NEU"; echo x >"$U/opt/einstellungen/$NEU/querencia-einstellungen"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "vorhandene 0.0.3-Ablage mit falschem Inhalt: verweigert" "[ $rc = 2 ] && grep -q '0.0.3-Ablage verändert' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t06k; lauf aktivieren >/dev/null; sed -i 's/^Comment=.*/Comment=anders/' "$U/apps/$DESKNAME"; vorher=$(stand)
rc=$(lauf zurueck)
pruef "fremder Comment nach Aktivierung: zurueck ändert nichts" "[ $rc = 2 ] && [ \"\$(stand)\" = \"$vorher\" ]"

echo "=== 7 Sperre (gemeinsam mit G2-C/G2-G-Skript)"
umgebung t07; vorher=$(stand)
( exec 7>"$U/state/einstellungen-integrieren.lock"; flock 7; sleep 5 ) & sleep 1
rc=$(lauf aktivieren); wait
pruef "zweiter Lauf während eines ersten: verweigert" "[ $rc = 2 ] && grep -q 'anderer Lauf' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"

echo "=== 8 desktop.ini wird nie angefasst"
umgebung t08; export XDG_CONFIG_HOME=$U/config; mkdir -p "$U/config/querencia"
printf '[Background]\nFile=\nMode=zoom\nColor=#1c4e96\n\n[Volumes]\nNetwork=false\n' >"$U/config/querencia/desktop.ini"; d=$(sha256sum <"$U/config/querencia/desktop.ini")
lauf aktivieren >/dev/null; rc=$(lauf pruefen); lauf zurueck >/dev/null
pruef "desktop.ini byte-gleich, pruefen nennt ihre Gruppen" "[ \"\$(sha256sum <$U/config/querencia/desktop.ini)\" = \"$d\" ] && grep -q 'Gruppen: \[Background\] \[Volumes\]' $U/ausgabe.txt"
unset XDG_CONFIG_HOME

pruef "echte Dateien unverändert (Inhalt, Inode); live keine 0.0.3-Ablage" "[ \"\$echt_vorher\" = \"\$(echt)\" ] && echo \"\$echt_vorher\" | grep -q 'keine 0.0.3-Ablage live'"
pruef "echter Menüeintrag unverändert seit Probebeginn" "cmp -s $EA/$DESKNAME $BASIS/menueeintrag-echt.desktop"
echo "ERGEBNIS integration-g2x: $ok PASS, $bad FAIL"
[ "$bad" = 0 ]
