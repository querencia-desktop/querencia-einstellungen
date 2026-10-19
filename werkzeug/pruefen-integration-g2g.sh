#!/bin/bash
# Probe für einstellungen-g2g-integrieren.sh (0.0.1 → 0.0.2) in Wegwerf-
# Umgebungen nachweis/profile/integration-g2g/<fall>. Jede Umgebung bildet
# den Live-Stand nach G2-C nach: Ablage 2f44261d01fd2948 (Kopie der echten),
# Link darauf, Menüeintrag 0.0.1, protokoll.txt, bin-Links für Desktop/Panel.
# Das Skript wird über QE_OPT, QE_APPS, QE_STATE umgelenkt; Fehler über
# QE_PROBE=1 und QE_FEHLER. Echte Dateien werden nur gelesen.
set -uo pipefail
P=~/projects/querencia-einstellungen
S=${QE_SKRIPT:-$P/werkzeug/einstellungen-g2g-integrieren.sh}
G2C=$P/werkzeug/einstellungen-integrieren.sh
V=$P/integration/g2g/vorschlag
A=$P/integration/g2g/ausgang
BASIS=$P/nachweis/profile/integration-g2g
EO=~/opt/querencia
EA=~/.local/share/applications
ALT=2f44261d01fd2948
NEU=$(grep '^PROG_NEU=' "$S" | cut -d= -f2 | cut -c1-16)
DESKNAME=org.querencia.Einstellungen.desktop
[ -n "$NEU" ] && [ ${#NEU} = 16 ] || { echo "PROG_NEU im Skript nicht gesetzt" >&2; exit 2; }
[ -f "$EO/einstellungen/$ALT/querencia-einstellungen" ] || { echo "0.0.1-Ablage fehlt live: $EO/einstellungen/$ALT" >&2; exit 2; }

if [ -e "$BASIS" ]; then
    [ -f "$BASIS/.querencia-testprofil" ] || { echo "kein Testordner: $BASIS" >&2; exit 2; }
    chmod -R u+w -- "$BASIS"; rm -rf -- "$BASIS"
fi
mkdir -p -- "$BASIS"; : >"$BASIS/.querencia-testprofil"

echt() {
    sha256sum "$EO/protokoll.txt" "$EA"/*.desktop "$EO/einstellungen/$ALT/querencia-einstellungen"
    readlink "$EO/bin/querencia-desktop" "$EO/bin/querencia-panel" "$EO/bin/querencia-einstellungen"
    ls -lai "$EO" "$EO/bin" "$EO/einstellungen" "$EA"
    [ ! -e "$EO/einstellungen/$NEU" ] && echo "keine 0.0.2-Ablage live"
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
    cp -p "$EA"/*.desktop "$U/apps/"
    cp -p "$A/$DESKNAME" "$U/apps/$DESKNAME"
    export QE_OPT=$U/opt QE_APPS=$U/apps QE_STATE=$U/state QE_PROBE=1
    unset QE_FEHLER
}
stand() {
    readlink "$U/opt/bin/querencia-einstellungen" 2>/dev/null || echo "kein Link"
    sha256sum "$U/apps/$DESKNAME" "$U/opt/protokoll.txt" "$U/apps"/*.desktop 2>/dev/null | cut -c1-64
}
zustand() { bash "$S" pruefen 2>&1 | grep -oE '(Programm-Link|Menüeintrag) +[^ ]+: [a-z]+' | awk '{print $NF}' | tr '\n' ' '; }
zustand_g2c() { bash "$G2C" pruefen 2>&1 | grep -oE '(Programm-Link|Menüeintrag) +[^ ]+: [a-z]+' | awk '{print $NF}' | tr '\n' ' '; }
keine_reste() { [ -z "$(find "$U" -name '*.g2g-tmp*' 2>/dev/null)" ]; }
lauf() { bash "$S" "$@" >"$U/ausgabe.txt" 2>&1; echo $?; }
A2='alt alt '
N2='neu neu '

echo "=== 1 Ausgangslage (wie live nach G2-C)"
umgebung t01
rc=$(lauf pruefen)
pruef "pruefen: Exit 0, alt alt, Stand 0.0.1" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$A2' ] && grep -q 'Stand: alt' $U/ausgabe.txt"
pruef "G2-C-Skript sieht dieselbe Umgebung als seinen Endstand (neu neu)" "[ \"\$(zustand_g2c)\" = '$N2' ]"

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
umgebung t03; alt=$(stand); n=$(wc -l <"$U/opt/protokoll.txt")
rc=$(lauf aktivieren); cat "$U/ausgabe.txt"
pruef "aktivieren: Exit 0, neu neu" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$N2' ]"
pruef "Ablage $NEU: 0555, byte-gleich mit dem Release-Binary 0.0.2, Herkunft" "[ \$(stat -c %a $U/opt/einstellungen/$NEU) = 555 ] && cmp -s $U/opt/einstellungen/$NEU/querencia-einstellungen $V/querencia-einstellungen && grep -q '^Quelle: ' $U/opt/einstellungen/$NEU.herkunft"
pruef "Link → 0.0.2; --version meldet 0.0.2" "[ \"\$(readlink $U/opt/bin/querencia-einstellungen)\" = $U/opt/einstellungen/$NEU/querencia-einstellungen ] && [ \"\$($U/opt/bin/querencia-einstellungen --version)\" = 'querencia-einstellungen 0.0.2' ]"
pruef "Menüeintrag: Vorlage 0.0.2, gültig, Exec auf $NEU, Name=Systemeinstellungen" "cmp -s $U/apps/$DESKNAME $V/$DESKNAME && desktop-file-validate $U/apps/$DESKNAME && grep -qx 'Exec=~/opt/querencia/einstellungen/$NEU/querencia-einstellungen' $U/apps/$DESKNAME && grep -qx 'Name=Systemeinstellungen' $U/apps/$DESKNAME"
pruef "0.0.1-Ablage bleibt, übrige Menüeinträge und Links unverändert, genau eine Protokollzeile" "cmp -s $U/opt/einstellungen/$ALT/querencia-einstellungen $EO/einstellungen/$ALT/querencia-einstellungen && for f in $EA/*.desktop; do [ \$(basename \$f) = $DESKNAME ] || cmp -s \$f $U/apps/\$(basename \$f) || exit 1; done && [ \$(wc -l <$U/opt/protokoll.txt) = $((n+1)) ]"
pruef "Sicherung (alt alt, Linkziel 0.0.1, Menüeintrag 0.0.1, Schritte P D) und Journal" "s=\$(ls -d $U/state/sicherung-einstellungen-g2g-aktivieren-*) && [ \"\$(cat \$s/zustand-vorher.txt)\" = 'alt alt' ] && grep -q $ALT \$s/programm-link.txt && cmp -s \$s/$DESKNAME $A/$DESKNAME && [ \"\$(tr '\n' ' ' <\$s/schritte.txt)\" = 'P D ' ] && grep -q 'einstellungen-g2g: aktiviert' $U/state/panel-rueckfall.txt"
pruef "G2-C-Skript verweigert auf dem 0.0.2-Stand (fremd), nichts geändert" "bash $G2C zurueck >/dev/null 2>&1; [ \$? = 2 ] && [ \"\$(zustand)\" = '$N2' ]"

echo "=== 4 erneut, zurück, wiederholt"
nsich=$(ls -d $U/state/sicherung-einstellungen-g2g-* | wc -l); neu=$(stand)
rc=$(lauf aktivieren)
pruef "zweites aktivieren: bereits aktiv, keine neue Sicherung" "[ $rc = 0 ] && grep -q 'bereits aktiv' $U/ausgabe.txt && [ \"\$(stand)\" = \"$neu\" ] && [ \$(ls -d $U/state/sicherung-einstellungen-g2g-* | wc -l) = $nsich ]"
rc=$(lauf zurueck)
pruef "zurueck: Exit 0, byte-gleich 0.0.1 (Link, Menüeintrag), Ablage 0.0.2 bleibt, Reihenfolge D P" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$A2' ] && [ \"\$(readlink $U/opt/bin/querencia-einstellungen)\" = $U/opt/einstellungen/$ALT/querencia-einstellungen ] && cmp -s $U/apps/$DESKNAME $A/$DESKNAME && [ -x $U/opt/einstellungen/$NEU/querencia-einstellungen ] && [ \"\$(tr '\n' ' ' <\$(ls -d $U/state/sicherung-einstellungen-g2g-zurueck-*)/schritte.txt)\" = 'D P ' ]"
rc=$(lauf zurueck)
pruef "zweites zurueck: nichts zu tun" "[ $rc = 0 ] && grep -q 'nichts zu tun' $U/ausgabe.txt"
rc=$(lauf aktivieren); rc2=$(lauf zurueck); rc3=$(lauf aktivieren)
pruef "aktivieren → zurück → aktivieren wiederholbar" "[ $rc$rc2$rc3 = 000 ] && [ \"\$(zustand)\" = '$N2' ]"
rc=$(lauf zurueck)
pruef "Kette: nach g2g zurueck führt einstellungen-integrieren.sh zurueck ganz zurück (kein Link, kein Menüeintrag)" "[ $rc = 0 ] && bash $G2C zurueck >/dev/null 2>&1 && [ ! -e $U/opt/bin/querencia-einstellungen ] && [ ! -e $U/apps/$DESKNAME ]"

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

echo "=== 6 Teilstände und fremde Stände"
umgebung t06a; lauf aktivieren >/dev/null; lauf zurueck >/dev/null
ln -sfn "$U/opt/einstellungen/$NEU/querencia-einstellungen" "$U/opt/bin/querencia-einstellungen"
rc=$(lauf aktivieren)
pruef "nur Link neu: aktivieren ergänzt nur D" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$N2' ] && [ \"\$(tr '\n' ' ' <\$(ls -td $U/state/sicherung-einstellungen-g2g-aktivieren-* | head -1)/schritte.txt)\" = 'D ' ]"
umgebung t06b; rm "$U/opt/bin/querencia-einstellungen"; rm "$U/apps/$DESKNAME"
rc=$(lauf aktivieren)
pruef "0.0.1 nicht installiert (fehlt): verweigert mit Hinweis auf einstellungen-integrieren.sh" "[ $rc = 2 ] && grep -q 'zuerst einstellungen-integrieren.sh' $U/ausgabe.txt && [ ! -e $U/opt/einstellungen/$NEU ]"
umgebung t06c; ln -sfn /nirgends "$U/opt/bin/querencia-einstellungen"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "fremder Link: verweigert, nichts geändert" "[ $rc = 2 ] && grep -q 'Programm ist fremd' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t06d; sed -i 's/^Name=.*/Name=Anders/' "$U/apps/$DESKNAME"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "fremder Menüeintrag: verweigert" "[ $rc = 2 ] && grep -q 'Menüeintrag ist fremd' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t06e; chmod u+w "$U/opt/einstellungen/$ALT"; rm "$U/opt/einstellungen/$ALT/querencia-einstellungen"
rc=$(lauf aktivieren)
pruef "0.0.1-Ablage fehlt (Rückfallziel): verweigert" "[ $rc = 2 ] && grep -q '0.0.1-Ablage' $U/ausgabe.txt"
umgebung t06f; mkdir -p "$U/opt/einstellungen/$NEU"; echo x >"$U/opt/einstellungen/$NEU/querencia-einstellungen"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "vorhandene 0.0.2-Ablage mit falschem Inhalt: verweigert" "[ $rc = 2 ] && grep -q '0.0.2-Ablage verändert' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t06g; lauf aktivieren >/dev/null; sed -i 's/^Name=.*/Name=Anders/' "$U/apps/$DESKNAME"; vorher=$(stand)
rc=$(lauf zurueck)
pruef "fremder Menüeintrag nach Aktivierung: zurueck ändert nichts" "[ $rc = 2 ] && [ \"\$(stand)\" = \"$vorher\" ]"

echo "=== 7 Sperre (gemeinsam mit G2-C-Skript)"
umgebung t07; vorher=$(stand)
( exec 7>"$U/state/einstellungen-integrieren.lock"; flock 7; sleep 5 ) & sleep 1
rc=$(lauf aktivieren); wait
pruef "zweiter Lauf während eines ersten: verweigert" "[ $rc = 2 ] && grep -q 'anderer Lauf' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"

pruef "echte Dateien unverändert (Inhalt, Inode); live keine 0.0.2-Ablage" "[ \"\$echt_vorher\" = \"\$(echt)\" ] && echo \"\$echt_vorher\" | grep -q 'keine 0.0.2-Ablage live'"
echo "ERGEBNIS integration-g2g: $ok PASS, $bad FAIL"
[ "$bad" = 0 ]
