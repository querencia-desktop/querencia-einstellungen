#!/bin/bash
# Probe für einstellungen-integrieren.sh in Wegwerf-Umgebungen
# (nachweis/profile/integration/<fall>). Jede Umgebung bildet den Live-Stand
# nach: ~/opt/querencia mit bin-Links (Desktop, Panel; als Links kopiert),
# protokoll.txt, ~/.local/share/applications mit den echten Menüeinträgen,
# leerer state. Das Skript wird über QE_OPT, QE_APPS, QE_STATE umgelenkt,
# künstliche Fehler über QE_PROBE=1 und QE_FEHLER. Die echten Dateien werden
# nur gelesen (Prüfsumme und Inode vorher/nachher).
set -uo pipefail
P=~/projects/querencia-werkstatt-w6-einstellungen
S=${QE_SKRIPT:-$P/werkzeug/einstellungen-integrieren.sh}
V=$P/integration/vorschlag
BASIS=$P/nachweis/profile/integration
EO=~/opt/querencia
EA=~/.local/share/applications
NEU=2f44261d01fd2948
DESKNAME=org.querencia.Einstellungen.desktop

# Löschschutz: nur Ordner unter $BASIS mit Marke .querencia-testprofil
if [ -e "$BASIS" ]; then
    [ -f "$BASIS/.querencia-testprofil" ] || { echo "kein Testordner: $BASIS" >&2; exit 2; }
    chmod -R u+w -- "$BASIS"; rm -rf -- "$BASIS"
fi
mkdir -p -- "$BASIS"; : >"$BASIS/.querencia-testprofil"

echt() {
    sha256sum "$EO/protokoll.txt" "$EA"/*.desktop
    readlink "$EO/bin/querencia-desktop" "$EO/bin/querencia-panel"
    ls -lai "$EO" "$EO/bin" "$EA"
    [ ! -e "$EO/einstellungen" ] && echo "keine Ablage einstellungen"
    [ ! -e "$EO/bin/querencia-einstellungen" ] && echo "kein Link"
    [ ! -e "$EA/$DESKNAME" ] && echo "kein Menüeintrag"
}
echt_vorher=$(echt)

ok=0; bad=0
pruef() { if eval "$2"; then ok=$((ok+1)); echo "PASS   $1"; else bad=$((bad+1)); echo "FAIL   $1"; fi; }

umgebung() {
    U=$BASIS/$1
    mkdir -p "$U/opt/bin" "$U/apps" "$U/state"
    cp -P "$EO/bin/querencia-desktop" "$EO/bin/querencia-panel" "$U/opt/bin/"
    cp -p "$EO/protokoll.txt" "$U/opt/protokoll.txt"
    cp -p "$EA"/*.desktop "$U/apps/"
    export QE_OPT=$U/opt QE_APPS=$U/apps QE_STATE=$U/state QE_PROBE=1
    unset QE_FEHLER
}
# Fingerabdruck: Link, Menüeintrag, Protokoll, übrige Menüeinträge
stand() {
    readlink "$U/opt/bin/querencia-einstellungen" 2>/dev/null || echo "kein Link"
    sha256sum "$U/apps/$DESKNAME" 2>/dev/null || echo "kein Menüeintrag"
    sha256sum "$U/opt/protokoll.txt" "$U/apps"/*.desktop "$U/opt/bin/querencia-desktop" 2>/dev/null | cut -c1-64
}
zustand() { bash "$S" pruefen 2>&1 | grep -oE '(Programm-Link|Menüeintrag) +[^ ]+: [a-z]+' | awk '{print $NF}' | tr '\n' ' '; }
keine_reste() { [ -z "$(find "$U" -name '*.g2c-tmp*' 2>/dev/null)" ]; }
lauf() { bash "$S" "$@" >"$U/ausgabe.txt" 2>&1; echo $?; }
protokoll_zeilen() { wc -l <"$U/opt/protokoll.txt"; }
A2='alt alt '
N2='neu neu '

echo "=== 1 Ausgangslage (wie live)"
umgebung t01
rc=$(lauf pruefen)
pruef "pruefen: Exit 0, alt alt, nennt „nicht installiert“" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$A2' ] && grep -q 'nicht installiert' $U/ausgabe.txt"
pruef "pruefen: Ablage fehlt, Desktop-Link kennt --faehigkeiten (QE-01 live)" "grep -q 'Ablage .*fehlt' $U/ausgabe.txt && grep -q 'kennt --faehigkeiten (QE-01)' $U/ausgabe.txt"

echo "=== 2 Fehler bei aktivieren (jede Stelle, auch zwischen Austausch und Kontrolle)"
for stelle in vor-erstem waehrend-programm nach-programm waehrend-menue nach-menue \
    waehrend-programm-nach-mv waehrend-menue-nach-mv \
    signal-vor-erstem signal-nach-programm signal-waehrend-menue signal-waehrend-programm-nach-mv signal-waehrend-menue-nach-mv; do
    echo "--- $stelle"
    umgebung "f-$stelle"; vorher=$(stand); n=$(protokoll_zeilen)
    export QE_FEHLER=$stelle
    rc=$(lauf aktivieren)
    unset QE_FEHLER
    pruef "$stelle: Exit 1 (zurückgerollt)" "[ $rc = 1 ] && grep -q 'zurückgerollt: Zustand wie vor dem Aufruf' $U/ausgabe.txt"
    pruef "$stelle: Link und Menüeintrag wie vorher, keine Zwischendateien, Protokoll ohne neue Zeile" "[ \"\$(stand)\" = \"$vorher\" ] && [ \"\$(zustand)\" = '$A2' ] && keine_reste && [ \$(protokoll_zeilen) = $n ]"
    pruef "$stelle: Journal vermerkt den Abbruch" "grep -q 'einstellungen: aktivieren abgebrochen' $U/state/panel-rueckfall.txt"
    rc=$(lauf aktivieren)
    pruef "$stelle: danach normale Aktivierung möglich" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$N2' ]"
done

echo "=== 2b Ablage nicht anlegbar"
umgebung t02b; vorher=$(stand); chmod 0555 "$U/opt"
rc=$(lauf aktivieren); chmod 0755 "$U/opt"
pruef "Ablage nicht anlegbar: Exit 1, nichts geändert, keine Ablage" "[ $rc = 1 ] && grep -q 'Ablage nicht anlegbar' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ] && [ ! -e $U/opt/einstellungen/$NEU ]"

echo "=== 3 normale Aktivierung"
umgebung t03; alt=$(stand); n=$(protokoll_zeilen)
rc=$(lauf aktivieren); cat "$U/ausgabe.txt"
pruef "aktivieren: Exit 0, neu neu" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$N2' ]"
pruef "Ablage $NEU: Ordner und Datei 0555, byte-gleich mit dem Release-Binary, ausführbar" "[ \$(stat -c %a $U/opt/einstellungen/$NEU) = 555 ] && [ \$(stat -c %a $U/opt/einstellungen/$NEU/querencia-einstellungen) = 555 ] && cmp -s $U/opt/einstellungen/$NEU/querencia-einstellungen $V/querencia-einstellungen && [ -x $U/opt/einstellungen/$NEU/querencia-einstellungen ]"
pruef "Herkunftsdatei $NEU.herkunft nennt die Vorlage" "grep -qx 'Quelle: $V/querencia-einstellungen' $U/opt/einstellungen/$NEU.herkunft"
pruef "Programm-Link → Ablage; läuft (--version)" "[ \"\$(readlink $U/opt/bin/querencia-einstellungen)\" = $U/opt/einstellungen/$NEU/querencia-einstellungen ] && [ \"\$($U/opt/bin/querencia-einstellungen --version)\" = 'querencia-einstellungen 0.0.1' ]"
pruef "Menüeintrag: byte-gleich mit der Vorlage, 0644, gültig, Name=Systemeinstellungen, Categories=Settings;" "cmp -s $U/apps/$DESKNAME $V/$DESKNAME && [ \$(stat -c %a $U/apps/$DESKNAME) = 644 ] && desktop-file-validate $U/apps/$DESKNAME && grep -qx 'Name=Systemeinstellungen' $U/apps/$DESKNAME && grep -qx 'Categories=Settings;' $U/apps/$DESKNAME"
pruef "Menüeintrag: Exec/TryExec auf die Live-Ablage $NEU, ohne --hintergrund (Kontrollzentrum)" "grep -qx 'Exec=~/opt/querencia/einstellungen/$NEU/querencia-einstellungen' $U/apps/$DESKNAME && grep -qx 'TryExec=~/opt/querencia/einstellungen/$NEU/querencia-einstellungen' $U/apps/$DESKNAME"
pruef "Protokoll: genau eine neue Zeile mit Link und Ablage" "[ \$(protokoll_zeilen) = $((n+1)) ] && tail -1 $U/opt/protokoll.txt | grep -q 'querencia-einstellungen -> $U/opt/einstellungen/$NEU/querencia-einstellungen (aus $V/querencia-einstellungen'"
pruef "übrige Menüeinträge, Desktop- und Panel-Link unverändert" "for f in $EA/*.desktop; do cmp -s \$f $U/apps/\$(basename \$f) || exit 1; done && [ \"\$(readlink $U/opt/bin/querencia-desktop)\" = \"\$(readlink $EO/bin/querencia-desktop)\" ] && [ \"\$(readlink $U/opt/bin/querencia-panel)\" = \"\$(readlink $EO/bin/querencia-panel)\" ]"
pruef "Sicherung mit Ausgangsstand alt alt, leerem Linkziel, Protokollkopie, Schrittliste P D" "s=\$(ls -d $U/state/sicherung-einstellungen-aktivieren-*) && [ \"\$(cat \$s/zustand-vorher.txt)\" = 'alt alt' ] && [ -z \"\$(cat \$s/programm-link.txt)\" ] && [ ! -e \$s/$DESKNAME ] && [ -f \$s/protokoll.txt.vorher ] && [ \"\$(tr '\n' ' ' <\$s/schritte.txt)\" = 'P D ' ]"
pruef "Journal: aktiviert, nennt den Rückfallbefehl" "grep -q 'einstellungen: aktiviert' $U/state/panel-rueckfall.txt && grep -q 'einstellungen-integrieren.sh zurueck' $U/state/panel-rueckfall.txt"
pruef "pruefen danach: Stand neu, installiert" "bash $S pruefen 2>&1 | grep 'Stand: neu' >/dev/null"

echo "=== 4 erneute Aktivierung"
nsich=$(ls -d $U/state/sicherung-einstellungen-* | wc -l); neu=$(stand)
rc=$(lauf aktivieren)
pruef "zweites aktivieren: bereits aktiv, nichts geändert, keine neue Sicherung, kein Protokolleintrag" "[ $rc = 0 ] && grep -q 'bereits aktiv' $U/ausgabe.txt && [ \"\$(stand)\" = \"$neu\" ] && [ \$(ls -d $U/state/sicherung-einstellungen-* | wc -l) = $nsich ]"

echo "=== 5 Rückfall, wiederholter Rückfall"
inode_ablage=$(stat -c %i "$U/opt/einstellungen/$NEU/querencia-einstellungen"); n=$(protokoll_zeilen)
rc=$(lauf zurueck); cat "$U/ausgabe.txt"
pruef "zurueck: Exit 0, alt alt, kein Link, kein Menüeintrag" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$A2' ] && [ ! -e $U/opt/bin/querencia-einstellungen ] && [ ! -L $U/opt/bin/querencia-einstellungen ] && [ ! -e $U/apps/$DESKNAME ]"
pruef "zurueck: Reihenfolge D P; Menüeintrag in der Sicherung; Ablage bleibt liegen" "s=\$(ls -d $U/state/sicherung-einstellungen-zurueck-*) && [ \"\$(tr '\n' ' ' <\$s/schritte.txt)\" = 'D P ' ] && cmp -s \$s/$DESKNAME $V/$DESKNAME && [ -x $U/opt/einstellungen/$NEU/querencia-einstellungen ]"
pruef "zurueck: Protokollzeile „entfernt“" "[ \$(protokoll_zeilen) = $((n+1)) ] && tail -1 $U/opt/protokoll.txt | grep -q 'entfernt: querencia-einstellungen'"
pruef "zurueck: übrige Menüeinträge unverändert" "for f in $EA/*.desktop; do cmp -s \$f $U/apps/\$(basename \$f) || exit 1; done"
rc=$(lauf zurueck)
pruef "zweites zurueck: nichts zu tun" "[ $rc = 0 ] && grep -q 'nichts zu tun' $U/ausgabe.txt && [ \"\$(zustand)\" = '$A2' ]"
rc=$(lauf aktivieren); rc2=$(lauf zurueck); rc3=$(lauf aktivieren)
pruef "aktivieren → zurück → aktivieren wiederholbar, Ablage wiederverwendet (gleiche Inode)" "[ $rc$rc2$rc3 = 000 ] && [ \"\$(zustand)\" = '$N2' ] && [ \$(stat -c %i $U/opt/einstellungen/$NEU/querencia-einstellungen) = $inode_ablage ]"

echo "=== 6 Fehler bei zurueck"
for stelle in zurueck-vor-erstem zurueck-waehrend-menue zurueck-waehrend-menue-nach-rm zurueck-nach-menue \
    zurueck-waehrend-programm zurueck-waehrend-programm-nach-rm \
    signal-zurueck-nach-menue signal-zurueck-waehrend-programm-nach-rm; do
    echo "--- $stelle"
    umgebung "z-$stelle"; lauf aktivieren >/dev/null; vorher=$(stand)
    export QE_FEHLER=$stelle
    rc=$(lauf zurueck)
    unset QE_FEHLER
    pruef "$stelle: Exit 1, Zustand wie vor dem Rückfall (neu neu)" "[ $rc = 1 ] && [ \"\$(stand)\" = \"$vorher\" ] && [ \"\$(zustand)\" = '$N2' ] && keine_reste"
    rc=$(lauf zurueck)
    pruef "$stelle: danach normaler Rückfall möglich" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$A2' ]"
done

echo "=== 7 teilweise integriert"
umgebung t07a
lauf aktivieren >/dev/null; lauf zurueck >/dev/null   # Ablage vorhanden
ln -s "$U/opt/einstellungen/$NEU/querencia-einstellungen" "$U/opt/bin/querencia-einstellungen"
pruef "nur Link neu: pruefen zeigt neu alt (gemischt)" "[ \"\$(zustand)\" = 'neu alt ' ] && bash $S pruefen 2>&1 | grep 'gemischt' >/dev/null"
rc=$(lauf aktivieren)
pruef "aktivieren ergänzt nur D" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$N2' ] && [ \"\$(tr '\n' ' ' <\$(ls -td $U/state/sicherung-einstellungen-aktivieren-* | head -1)/schritte.txt)\" = 'D ' ]"
umgebung t07b
cp "$V/$DESKNAME" "$U/apps/$DESKNAME"; vorher=$(stand)
export QE_FEHLER=nach-programm; rc=$(lauf aktivieren); unset QE_FEHLER
pruef "Menüeintrag schon neu + Fehler nach dem Programm: genau der Ausgangsstand (alt neu)" "[ $rc = 1 ] && [ \"\$(stand)\" = \"$vorher\" ] && [ \"\$(zustand)\" = 'alt neu ' ]"
rc=$(lauf zurueck)
pruef "zurueck aus Teilstand alt neu: alt alt" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$A2' ]"

echo "=== 8 fremde Stände und fremde Änderungen"
umgebung t08a; ln -s /nirgends/querencia-einstellungen "$U/opt/bin/querencia-einstellungen"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "Link auf fremdes Ziel: verweigert (Exit 2), nichts geändert, keine Ablage" "[ $rc = 2 ] && grep -q 'Programm ist fremd' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ] && [ ! -e $U/opt/einstellungen ]"
umgebung t08b; cp "$V/querencia-einstellungen" "$U/opt/bin/querencia-einstellungen"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "Programm als Datei statt Link: verweigert" "[ $rc = 2 ] && grep -q 'Programm ist fremd' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t08c; sed 's/^Name=.*/Name=Anders/' "$V/$DESKNAME" >"$U/apps/$DESKNAME"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "fremder Menüeintrag: verweigert" "[ $rc = 2 ] && grep -q 'Menüeintrag ist fremd' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t08d; mkdir -p "$U/opt/einstellungen/$NEU"; cp "$U/opt/bin/querencia-desktop" "$U/opt/einstellungen/$NEU/querencia-einstellungen" 2>/dev/null || echo x >"$U/opt/einstellungen/$NEU/querencia-einstellungen"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "vorhandene Ablage mit falschem Inhalt: verweigert" "[ $rc = 2 ] && grep -q 'Ablage verändert' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t08e; rm "$U/opt/protokoll.txt"
rc=$(lauf aktivieren)
pruef "Protokoll fehlt (falscher Ablageordner): verweigert" "[ $rc = 2 ] && grep -q 'Protokoll fehlt' $U/ausgabe.txt && [ ! -e $U/opt/bin/querencia-einstellungen ]"
umgebung t08f; rm "$U/opt/bin/querencia-desktop"
rc=$(lauf aktivieren)
pruef "kein Desktop-Link (falscher Ablageordner): verweigert" "[ $rc = 2 ] && grep -q 'kein Desktop-Link' $U/ausgabe.txt && [ ! -e $U/opt/einstellungen ]"
umgebung t08g; lauf aktivieren >/dev/null; sed -i 's/^Name=.*/Name=Anders/' "$U/apps/$DESKNAME"; vorher=$(stand)
rc=$(lauf zurueck)
pruef "fremder Menüeintrag nach Aktivierung: zurueck ändert nichts" "[ $rc = 2 ] && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t08h; lauf aktivieren >/dev/null; ln -sfn /nirgends "$U/opt/bin/querencia-einstellungen"; vorher=$(stand)
rc=$(lauf zurueck)
pruef "fremder Link nach Aktivierung: zurueck ändert nichts" "[ $rc = 2 ] && grep -q 'Programm ist fremd' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t08i
QE_PROBE= rc=$(QE_FEHLER=vor-erstem lauf aktivieren)
pruef "ohne QE_PROBE wirkt QE_FEHLER nicht" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$N2' ]"

echo "=== 9 Sperre"
umgebung t09; vorher=$(stand)
( exec 7>"$U/state/einstellungen-integrieren.lock"; flock 7; sleep 5 ) & sleep 1
rc=$(lauf aktivieren); wait
pruef "zweiter Lauf während eines ersten: verweigert, nichts geändert" "[ $rc = 2 ] && grep -q 'anderer Lauf' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"

echo "=== 10 falscher Aufruf"
umgebung t10
rc=$(lauf); rc2=$(lauf entfernen)
pruef "ohne oder mit unbekanntem Befehl: Exit 2, nichts geändert" "[ $rc = 2 ] && [ $rc2 = 2 ] && [ \"\$(zustand)\" = '$A2' ]"

pruef "echte Dateien (~/opt/querencia, Menüeinträge) unverändert (Inhalt, Inode); live keine Ablage, kein Link, kein Menüeintrag" "[ \"\$echt_vorher\" = \"\$(echt)\" ] && echo \"\$echt_vorher\" | grep -q 'keine Ablage einstellungen'"
echo "ERGEBNIS integration: $ok PASS, $bad FAIL"
[ "$bad" = 0 ]
