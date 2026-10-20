#!/bin/bash
# Probe für einstellungen-011-integrieren.sh (0.1.0 → 0.1.1) in Wegwerf-
# Umgebungen nachweis/profile/integration-011/<fall>. Jede Umgebung bildet den
# Live-Stand nach 0.1.0 nach: Ablage df0f666b499ed551 (Kopie der echten, mit
# share/locale), Link darauf, Menüeintrag = Kopie des ECHTEN Menüeintrags,
# protokoll.txt, bin-Links für Desktop/Panel, dazu die 0.0.9-Ablage und die
# gemerkten Zustände des 0.1.0-Skripts (Rückbaukette über
# einstellungen-010-integrieren.sh). Umgelenkt über QE_OPT, QE_APPS, QE_STATE
# (und XDG_CONFIG_HOME); Fehler über QE_PROBE=1 und QE_FEHLER; beschädigte
# Vorlagen über QE_VORSCHLAG. Echte Dateien werden nur gelesen; Fälle, die
# eine Umlenkung auf echte Orte versuchen, laufen nur mit `pruefen`.
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
S=${QE_SKRIPT:-$P/werkzeug/einstellungen-011-integrieren.sh}
S010=~/projects/querencia-einstellungen-wt-010/werkzeug/einstellungen-010-integrieren.sh
V=$P/integration/011/vorschlag
BASIS=$P/nachweis/profile/integration-011
EO=~/opt/querencia
EA=~/.local/share/applications
ES=~/.local/state/querencia
EM=~/.config/mimeapps.list
ALT=df0f666b499ed551
V009=de7e95a52a8eec92
NEU=557957f3f09b7939
DESKNAME=org.querencia.Einstellungen.desktop
MO_DE=share/locale/de/LC_MESSAGES/querencia-einstellungen.mo
MO_EN=share/locale/en/LC_MESSAGES/querencia-einstellungen.mo
export EINSTELLUNGEN_011_FREIGABE=ja
[ "$(grep ' querencia-einstellungen$' "$V/SHA256SUMS" | cut -c1-16)" = $NEU ] || { echo "Vorlage ist nicht $NEU" >&2; exit 2; }
for d in $ALT $V009; do [ -f "$EO/einstellungen/$d/querencia-einstellungen" ] || { echo "Ablage fehlt live: $EO/einstellungen/$d" >&2; exit 2; }; done

if [ -e "$BASIS" ]; then
    [ -f "$BASIS/.querencia-testprofil" ] || { echo "kein Testordner: $BASIS" >&2; exit 2; }
    chmod -R u+w -- "${BASIS:?}"; rm -rf -- "${BASIS:?}"
fi
mkdir -p -- "$BASIS"; : >"$BASIS/.querencia-testprofil"
# Momentaufnahmen (andere Sessions können während der Probe live ändern)
cp -p "$EA/$DESKNAME" "$BASIS/menueeintrag-echt.desktop"
cp -p "$ES/einstellungen-010-menue-vorher.desktop" "$ES/einstellungen-010-vorher.txt" "$BASIS/"
cp -p "$EM" "$BASIS/mimeapps-echt.list" 2>/dev/null || true
ECHT_ICON=$(grep -m1 '^Icon=' "$BASIS/menueeintrag-echt.desktop")
echo "echter Menüeintrag: $(sha256sum <"$BASIS/menueeintrag-echt.desktop" | cut -c1-16), $ECHT_ICON"

echt() {
    sha256sum "$EO/protokoll.txt" "$EA/$DESKNAME" "$EM" "$ES/panel-rueckfall.txt"
    readlink "$EO/bin/querencia-desktop" "$EO/bin/querencia-panel" "$EO/bin/querencia-einstellungen"
    ls -lai "$EO/bin" "$EO/einstellungen"
    (cd "$EO/einstellungen/$ALT" && find . -type f | LC_ALL=C sort | xargs sha256sum)
    [ ! -e "$EO/einstellungen/$NEU" ] && echo "keine 0.1.1-Ablage live"
    (cd ~/.config/querencia 2>/dev/null && find labwc -type f 2>/dev/null | LC_ALL=C sort | xargs -r sha256sum)
    sha256sum ~/.config/dconf/user
    ls "$ES" | grep -c 'einstellungen-011' || true
}
echt_vorher=$(echt)

ok=0; bad=0
pruef() { if eval "$2"; then ok=$((ok+1)); echo "PASS   $1"; else bad=$((bad+1)); echo "FAIL   $1"; fi; }
weg() { rm -f -- "${1:?}"; }

ablage_kopieren() {
    local q=$EO/einstellungen/$1 z=$U/opt/einstellungen/$1
    mkdir -p "$z"; (cd "$q" && find . -type d) | while read -r d; do mkdir -p "$z/$d"; done
    (cd "$q" && find . -type f) | while read -r f; do cp -p "$q/$f" "$z/$f"; done
    find "$z" -type d -exec chmod 0555 {} +
    echo "Quelle: Probe" >"$U/opt/einstellungen/$1.herkunft"
}
umgebung() {
    U=$BASIS/$1
    mkdir -p "$U/opt/bin" "$U/opt/einstellungen" "$U/apps" "$U/state" "$U/config"
    cp -P "$EO/bin/querencia-desktop" "$EO/bin/querencia-panel" "$U/opt/bin/"
    ablage_kopieren $ALT; ablage_kopieren $V009
    ln -s "$U/opt/einstellungen/$ALT/querencia-einstellungen" "$U/opt/bin/querencia-einstellungen"
    cp -p "$EO/protokoll.txt" "$U/opt/protokoll.txt"
    cp -p "$BASIS/menueeintrag-echt.desktop" "$U/apps/$DESKNAME"
    cp -p "$U/apps/$DESKNAME" "$U/menueeintrag-ausgang.desktop"
    cp -p "$BASIS/einstellungen-010-menue-vorher.desktop" "$BASIS/einstellungen-010-vorher.txt" "$U/state/"
    : >"$U/state/panel-rueckfall.txt"
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
zustand_010() { bash "$S010" pruefen 2>&1 | grep -oE '(Programm-Link|Menüeintrag) +[^ ]+: [a-z]+' | awk '{print $NF}' | tr '\n' ' '; }
keine_reste() { [ -z "$(find "$U" -name '*.011-tmp*' 2>/dev/null)" ]; }
lauf() { bash "$S" "$@" >"$U/ausgabe.txt" 2>&1; echo $?; }
mime_gleich() { cmp -s "$U/config/mimeapps.list" "$BASIS/mimeapps-echt.list"; }
A2='alt alt '
N2='neu neu '

echo "=== 1 Ausgangslage (wie live nach 0.1.0)"
umgebung t01
rc=$(lauf pruefen); cat "$U/ausgabe.txt"
pruef "pruefen: Exit 0, alt alt, Stand $ALT, Rückfallziel vollständig, letzte Zeile alt alt" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$A2' ] && grep -q 'Stand: alt – Querencia Einstellungen $ALT' $U/ausgabe.txt && grep -q 'Ausgangsstand .*vollständig' $U/ausgabe.txt && [ \"\$(tail -1 $U/ausgabe.txt)\" = 'alt alt' ]"
pruef "0.1.0-Skript sieht denselben Stand als seinen Endstand (neu neu)" "[ \"\$(zustand_010)\" = '$N2' ]"
pruef "Vorlage: Binary 0555, Kataloge und Menüeintrag 0444, Ordner 0555, SHA256SUMS stimmt, keine alt-erlaubt.txt" "[ \$(stat -c %a $V/querencia-einstellungen) = 555 ] && [ \$(stat -c %a $V/$MO_DE) = 444 ] && [ \$(stat -c %a $V/$MO_EN) = 444 ] && [ \$(stat -c %a $V/$DESKNAME) = 444 ] && [ -z \"\$(find $V -type d ! -perm 555)\" ] && (cd $V && sha256sum -c --quiet SHA256SUMS) && [ ! -e $V/alt-erlaubt.txt ]"
pruef "feste Summen im Skript = Vorlage (SHA256SUMS-Datei und jede Zeile)" "grep -q \"^SUMMEN_SHA=\$(sha256sum $V/SHA256SUMS | cut -d' ' -f1)\$\" $S && while read -r h f; do case \$f in querencia-einstellungen) k=PROG_NEU;; *de/*) k=DE_NEU;; *en/*) k=EN_NEU;; *.desktop) k=DESK_NEU;; esac; grep -qx \"\$k=\$h\" $S || exit 1; done <$V/SHA256SUMS"
pruef "Vorlage: Menüeintrag gültig, zweisprachig, Icon wie live, gegenüber 0.1.0 nur Kommentar und Exec/TryExec anders" "desktop-file-validate $V/$DESKNAME && grep -qx 'Name\\[de\\]=Systemeinstellungen' $V/$DESKNAME && grep -qxF '$ECHT_ICON' $V/$DESKNAME && [ \"\$(diff <(grep -v '^#\\|Exec=' $BASIS/menueeintrag-echt.desktop) <(grep -v '^#\\|Exec=' $V/$DESKNAME))\" = '' ]"

echo "=== 2 Freigabe und Umlenkung"
umgebung t02; vorher=$(stand)
rc=$(EINSTELLUNGEN_011_FREIGABE='' lauf aktivieren)
pruef "aktivieren ohne EINSTELLUNGEN_011_FREIGABE=ja: verweigert, nichts geändert, keine Sicherung" "[ $rc = 2 ] && grep -q 'nur mit EINSTELLUNGEN_011_FREIGABE=ja' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ] && [ -z \"\$(ls $U/state | grep sicherung-einstellungen-011)\" ]"
rc=$(EINSTELLUNGEN_011_FREIGABE=nein lauf zurueck)
pruef "zurueck mit FREIGABE=nein: verweigert" "[ $rc = 2 ] && grep -q 'nur mit EINSTELLUNGEN_011_FREIGABE=ja' $U/ausgabe.txt"
rc=$(EINSTELLUNGEN_011_FREIGABE='' lauf pruefen)
pruef "pruefen braucht keine Freigabe" "[ $rc = 0 ] && [ \"\$(tail -1 $U/ausgabe.txt)\" = 'alt alt' ]"
# Umlenkung auf echte Orte: nur mit pruefen (liest nur); der Abbruch kommt vor allem anderen
for fall in "QE_OPT=$EO" "QE_OPT=$EO/einstellungen" "QE_APPS=$EA" "QE_STATE=$ES" "QE_STATE=~/.local/state" "QE_OPT=~" "QE_APPS=$EA/../applications"; do
    rc=$(env "$fall" bash "$S" pruefen >"$U/ausgabe.txt" 2>&1; echo $?)
    pruef "Umlenkung $fall (echter Ort, darin oder darüber): verweigert vor jeder Prüfung" "[ $rc = 2 ] && grep -q 'Umlenkung .* – nichts geändert' $U/ausgabe.txt && [ \$(wc -l <$U/ausgabe.txt) = 1 ]"
done
rc=$(env -u QE_APPS bash "$S" pruefen >"$U/ausgabe.txt" 2>&1; echo $?)
pruef "nur zwei von drei Umlenkungen: verweigert" "[ $rc = 2 ] && grep -q 'nur alle drei zusammen' $U/ausgabe.txt"
rc=$(env -u QE_OPT -u QE_APPS -u QE_STATE QE_VORSCHLAG=/nirgends bash "$S" pruefen >"$U/ausgabe.txt" 2>&1; echo $?)
pruef "QE_PROBE/QE_VORSCHLAG ohne Umlenkung: nicht beachtet (pruefen live wie ohne)" "[ $rc = 0 ] && [ \"\$(tail -1 $U/ausgabe.txt)\" = 'alt alt' ] && grep -q 'Vorlage $V:' $U/ausgabe.txt"
rc=$(QE_PROBE='' QE_VORSCHLAG=/nirgends lauf pruefen)
pruef "QE_VORSCHLAG ohne QE_PROBE=1: nicht beachtet" "[ $rc = 0 ] && [ \"\$(tail -1 $U/ausgabe.txt)\" = 'alt alt' ]"

echo "=== 3 Fehler bei aktivieren"
for stelle in vor-erstem ablage waehrend-programm nach-programm waehrend-menue nach-menue waehrend-programm-nach-mv waehrend-menue-nach-mv signal-vor-erstem signal-nach-programm signal-waehrend-menue-nach-mv; do
    echo "--- $stelle"
    umgebung "f-$stelle"; vorher=$(stand)
    export QE_FEHLER=$stelle
    rc=$(lauf aktivieren)
    unset QE_FEHLER
    if [ "$stelle" = ablage ] || [ "$stelle" = vor-erstem ] || [ "$stelle" = signal-vor-erstem ]; then
        pruef "$stelle: Exit 1, zurückgerollt, alles wie vorher (keine Ablage), keine Reste" "[ $rc = 1 ] && grep -q 'zurückgerollt' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ] && [ \"\$(zustand)\" = '$A2' ] && keine_reste"
    else
        pruef "$stelle: Exit 1, zurückgerollt, Link und Menüeintrag wie vorher, keine Reste" "[ $rc = 1 ] && grep -q 'zurückgerollt' $U/ausgabe.txt && [ \"\$(stand | head -3)\" = \"\$(echo \"$vorher\" | head -3)\" ] && [ \"\$(zustand)\" = '$A2' ] && keine_reste"
    fi
    rc=$(lauf aktivieren)
    pruef "$stelle: danach normale Aktivierung" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$N2' ]"
done

echo "=== 4 normale Aktivierung"
umgebung t04; n=$(wc -l <"$U/opt/protokoll.txt")
rc=$(lauf aktivieren); cat "$U/ausgabe.txt"
pruef "aktivieren: Exit 0, neu neu" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$N2' ]"
pruef "Ablage $NEU: genau Binary, de.mo, en.mo, SHA256SUMS; Ordner 0555, Binary 0555, Rest 0444; byte-gleich mit der Vorlage; Herkunft" "[ \"\$(cd $U/opt/einstellungen/$NEU && find . -type f | LC_ALL=C sort | tr '\n' ' ')\" = './SHA256SUMS ./querencia-einstellungen ./$MO_DE ./$MO_EN ' ] && [ -z \"\$(find $U/opt/einstellungen/$NEU -type d ! -perm 555)\" ] && [ \$(stat -c %a $U/opt/einstellungen/$NEU/querencia-einstellungen) = 555 ] && [ \$(stat -c %a $U/opt/einstellungen/$NEU/$MO_DE) = 444 ] && [ \$(stat -c %a $U/opt/einstellungen/$NEU/SHA256SUMS) = 444 ] && cmp -s $U/opt/einstellungen/$NEU/querencia-einstellungen $V/querencia-einstellungen && cmp -s $U/opt/einstellungen/$NEU/$MO_DE $V/$MO_DE && cmp -s $U/opt/einstellungen/$NEU/$MO_EN $V/$MO_EN && (cd $U/opt/einstellungen/$NEU && sha256sum -c --quiet SHA256SUMS) && grep -q 'Tag einstellungen/0.1.1' $U/opt/einstellungen/$NEU.herkunft"
T=$(mktemp -d "$BASIS/vers-XXXX"); chmod 0700 "$T"
pruef "Link → 0.1.1; --version meldet 0.1.1 (eigenes HOME/XDG_RUNTIME_DIR, ohne Bildschirm)" "[ \"\$(readlink $U/opt/bin/querencia-einstellungen)\" = $U/opt/einstellungen/$NEU/querencia-einstellungen ] && [ \"\$(env -u WAYLAND_DISPLAY -u DISPLAY -u DBUS_SESSION_BUS_ADDRESS HOME=$T XDG_RUNTIME_DIR=$T $U/opt/bin/querencia-einstellungen --version)\" = 'querencia-einstellungen 0.1.1' ]"
pruef "Menüeintrag: byte-gleich mit der Vorlage, gültig, Exec und TryExec auf $NEU, Icon wie vorher" "cmp -s $U/apps/$DESKNAME $V/$DESKNAME && desktop-file-validate $U/apps/$DESKNAME && grep -qx 'Exec=~/opt/querencia/einstellungen/$NEU/querencia-einstellungen' $U/apps/$DESKNAME && grep -qx 'TryExec=~/opt/querencia/einstellungen/$NEU/querencia-einstellungen' $U/apps/$DESKNAME && grep -qxF '$ECHT_ICON' $U/apps/$DESKNAME"
pruef "Menüeintrag des Ausgangs byte-genau gemerkt" "cmp -s $U/state/einstellungen-011-menue-vorher.desktop $U/menueeintrag-ausgang.desktop"
pruef "0.1.0-Ablage bleibt byte-gleich, genau eine Protokollzeile mit Rückfallbefehl" "diff -r $U/opt/einstellungen/$ALT $EO/einstellungen/$ALT && [ \$(wc -l <$U/opt/protokoll.txt) = $((n+1)) ] && tail -1 $U/opt/protokoll.txt | grep -q 'Rückfall: EINSTELLUNGEN_011_FREIGABE=ja bash .*einstellungen-011-integrieren.sh zurueck'"
pruef "Sicherung (alt alt, Linkziel 0.1.0, Schritte P D) und Journal" "s=\$(ls -d $U/state/sicherung-einstellungen-011-aktivieren-*) && [ \"\$(cat \$s/zustand-vorher.txt)\" = 'alt alt' ] && grep -q $ALT \$s/programm-link.txt && grep -q $ALT \$s/ausgangsstand.txt && grep -q $ALT $U/state/einstellungen-011-vorher.txt && cmp -s \$s/$DESKNAME $U/menueeintrag-ausgang.desktop && [ \"\$(tr '\n' ' ' <\$s/schritte.txt)\" = 'P D ' ] && grep -q 'einstellungen-011: aktiviert' $U/state/panel-rueckfall.txt"
pruef "mimeapps.list byte-gleich" "mime_gleich"
pruef "0.1.0-Skript verweigert auf dem 0.1.1-Stand (zurueck), nichts geändert" "v=\$(stand); bash $S010 zurueck >/dev/null 2>&1; [ \$? = 2 ] && [ \"\$(stand)\" = \"\$v\" ]"

echo "=== 5 erneut, zurück, wiederholt, Rückbaukette"
nsich=$(ls -d "$U"/state/sicherung-einstellungen-011-* | wc -l); neu=$(stand)
rc=$(lauf aktivieren)
pruef "zweites aktivieren: bereits aktiv, keine neue Sicherung" "[ $rc = 0 ] && grep -q 'bereits aktiv' $U/ausgabe.txt && [ \"\$(stand)\" = \"$neu\" ] && [ \$(ls -d $U/state/sicherung-einstellungen-011-* | wc -l) = $nsich ]"
rc=$(lauf zurueck)
pruef "zurueck: Exit 0, Link 0.1.0, Menüeintrag byte-gleich mit dem Ausgang, Ablage 0.1.1 bleibt, Reihenfolge D P" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$A2' ] && [ \"\$(readlink $U/opt/bin/querencia-einstellungen)\" = $U/opt/einstellungen/$ALT/querencia-einstellungen ] && cmp -s $U/apps/$DESKNAME $U/menueeintrag-ausgang.desktop && [ -x $U/opt/einstellungen/$NEU/querencia-einstellungen ] && [ \"\$(tr '\n' ' ' <\$(ls -d $U/state/sicherung-einstellungen-011-zurueck-*)/schritte.txt)\" = 'D P ' ]"
pruef "nach zurueck sieht das 0.1.0-Skript seinen Endstand (neu neu)" "[ \"\$(zustand_010)\" = '$N2' ]"
rc=$(lauf zurueck)
pruef "zweites zurueck: nichts zu tun" "[ $rc = 0 ] && grep -q 'nichts zu tun' $U/ausgabe.txt"
rc=$(lauf aktivieren); rc2=$(lauf zurueck); rc3=$(lauf aktivieren)
pruef "aktivieren → zurück → aktivieren wiederholbar" "[ $rc$rc2$rc3 = 000 ] && [ \"\$(zustand)\" = '$N2' ] && cmp -s $U/apps/$DESKNAME $V/$DESKNAME"
pruef "Rückbaukette: 0.1.1 zurueck, dann 0.1.0-Skript zurueck → 0.0.9" "[ \$(lauf zurueck) = 0 ] && bash $S010 zurueck >/dev/null 2>&1 && [ \"\$(zustand_010)\" = '$A2' ] && [ \"\$(readlink $U/opt/bin/querencia-einstellungen)\" = $U/opt/einstellungen/$V009/querencia-einstellungen ] && cmp -s $U/apps/$DESKNAME $BASIS/einstellungen-010-menue-vorher.desktop"
pruef "… danach: 0.0.9 ist kein zulässiger Ausgangsstand, pruefen meldet fremd fremd, aktivieren verweigert" "[ \"\$(zustand)\" = 'fremd fremd ' ] && v=\$(stand) && [ \$(lauf aktivieren) = 2 ] && [ \"\$(stand)\" = \"\$v\" ]"
pruef "… 0.1.0-Skript aktivieren, dann 0.1.1 aktivieren: wieder 0.1.1" "bash $S010 aktivieren >/dev/null 2>&1 && [ \$(lauf aktivieren) = 0 ] && [ \"\$(zustand)\" = '$N2' ]"
pruef "mimeapps.list nach allen Läufen byte-gleich" "mime_gleich"

echo "=== 6 Fehler bei zurueck"
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

echo "=== 7 Teilstände und fremde Stände"
umgebung t07a; sed -i 's/^Icon=.*/Icon=preferences-system/' "$U/apps/$DESKNAME"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "Menüeintrag weicht von 0.1.0 ab (anderes Icon=): fremd, verweigert, nichts geändert" "[ $rc = 2 ] && grep -q 'Menüeintrag ist fremd' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t07b; lauf aktivieren >/dev/null; weg "$U/state/einstellungen-011-menue-vorher.desktop"; vorher=$(stand)
rc=$(lauf zurueck)
pruef "gemerkter Menüeintrag fehlt: zurueck verweigert, nichts geändert" "[ $rc = 2 ] && grep -q 'gemerkter Menüeintrag fehlt' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t07c; lauf aktivieren >/dev/null; printf '# x\n' >>"$U/state/einstellungen-011-menue-vorher.desktop"; vorher=$(stand)
rc=$(lauf zurueck)
pruef "gemerkter Menüeintrag verändert: zurueck verweigert, nichts geändert" "[ $rc = 2 ] && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t07d; lauf aktivieren >/dev/null; lauf zurueck >/dev/null
ln -sfn "$U/opt/einstellungen/$NEU/querencia-einstellungen" "$U/opt/bin/querencia-einstellungen"
rc=$(lauf aktivieren)
pruef "nur Link neu: aktivieren ergänzt nur D" "[ $rc = 0 ] && [ \"\$(zustand)\" = '$N2' ] && [ \"\$(tr '\n' ' ' <\$(ls -td $U/state/sicherung-einstellungen-011-aktivieren-* | head -1)/schritte.txt)\" = 'D ' ]"
umgebung t07e; weg "$U/opt/bin/querencia-einstellungen"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "Link fehlt: verweigert, nichts geändert" "[ $rc = 2 ] && grep -q 'zuerst die Vorversionen' $U/ausgabe.txt && [ ! -e $U/opt/einstellungen/$NEU ] && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t07f; ln -sfn /nirgends "$U/opt/bin/querencia-einstellungen"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "fremder Link: verweigert, nichts geändert" "[ $rc = 2 ] && grep -q 'Programm ist fremd' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t07g; chmod u+w "$U/opt/einstellungen/$ALT" "$U/opt/einstellungen/$ALT/share/locale/de/LC_MESSAGES"; weg "$U/opt/einstellungen/$ALT/$MO_DE"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "Rückfallziel 0.1.0 ohne de.mo: verweigert (Ablage des Ausgangsstands)" "[ $rc = 2 ] && grep -q 'Ablage des Ausgangsstands 0.1.0' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t07h; chmod u+w "$U/opt/einstellungen/$ALT" "$U/opt/einstellungen/$ALT/querencia-einstellungen"; printf 'x' >>"$U/opt/einstellungen/$ALT/querencia-einstellungen"
rc=$(lauf aktivieren)
pruef "Rückfallziel 0.1.0 verändert: verweigert" "[ $rc = 2 ] && grep -q 'Ablage des Ausgangsstands' $U/ausgabe.txt"
umgebung t07i; mkdir -p "$U/opt/einstellungen/$NEU"; cp "$V/querencia-einstellungen" "$U/opt/einstellungen/$NEU/"; vorher=$(stand)
rc=$(lauf aktivieren)
pruef "vorhandene 0.1.1-Ablage ohne Kataloge: verweigert (nie eine Binary ohne de.mo)" "[ $rc = 2 ] && grep -q '0.1.1-Ablage unvollständig' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t07j; lauf aktivieren >/dev/null; printf '\nComment=anders\n' >>"$U/apps/$DESKNAME"; vorher=$(stand)
rc=$(lauf zurueck)
pruef "Menüeintrag nach Aktivierung von Hand geändert: zurueck ändert nichts (fremd)" "[ $rc = 2 ] && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t07k; lauf aktivieren >/dev/null; chmod u+w "$U/opt/einstellungen/$NEU" "$U/opt/einstellungen/$NEU/share/locale/de/LC_MESSAGES"; weg "$U/opt/einstellungen/$NEU/$MO_DE"; vorher=$(stand)
rc=$(lauf pruefen); rc2=$(lauf zurueck)
pruef "de.mo aus der 0.1.1-Ablage verschwunden: pruefen und zurueck verweigern, nichts geändert" "[ $rc = 2 ] && [ $rc2 = 2 ] && grep -q 'unvollständig' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"

echo "=== 8 beschädigte Vorlagen (QE_VORSCHLAG, nur in der Probe)"
kopie() { K=$U/vorlage; cp -a "$V" "$K"; chmod -R u+w "$K"; }
umgebung t08a; kopie; weg "$K/$MO_DE"; (cd "$K" && grep -v " $MO_DE\$" SHA256SUMS >S && mv S SHA256SUMS); vorher=$(stand)
rc=$(QE_VORSCHLAG=$K lauf aktivieren)
pruef "Vorlage ohne de.mo, SHA256SUMS angepasst: verweigert (feste SUMMEN_SHA), nichts geändert" "[ $rc = 2 ] && grep -q 'nicht die geprüfte' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t08b; kopie; printf 'x' >>"$K/$MO_EN"; vorher=$(stand)
rc=$(QE_VORSCHLAG=$K lauf aktivieren)
pruef "Vorlage mit verändertem Katalog: verweigert (SHA256SUMS), nichts geändert" "[ $rc = 2 ] && grep -q 'stimmt nicht mit SHA256SUMS' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"
umgebung t08c; kopie; printf 'x' >>"$K/querencia-einstellungen"; (cd "$K" && sha256sum querencia-einstellungen $MO_DE $MO_EN $DESKNAME >SHA256SUMS); vorher=$(stand)
rc=$(QE_VORSCHLAG=$K lauf aktivieren)
pruef "andere Binary mit stimmiger SHA256SUMS: verweigert (feste Summen), nichts geändert" "[ $rc = 2 ] && grep -q 'nicht die geprüfte' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"

echo "=== 9 Sperre (gemeinsam mit den anderen einstellungen-*-integrieren.sh)"
umgebung t09; vorher=$(stand)
( exec 7>"$U/state/einstellungen-integrieren.lock"; flock 7; sleep 5 ) & sleep 1
rc=$(lauf aktivieren); wait
pruef "zweiter Lauf während eines ersten: verweigert" "[ $rc = 2 ] && grep -q 'anderer Lauf' $U/ausgabe.txt && [ \"\$(stand)\" = \"$vorher\" ]"

echo "=== 10 mimeapps.list, desktop.ini und labwc-Benutzerkopie (mit Sicherungen) werden nie angefasst"
umgebung t10; mkdir -p "$U/config/querencia"
printf '[Background]\nFile=\nMode=zoom\nColor=#1c4e96\n' >"$U/config/querencia/desktop.ini"; d=$(sha256sum <"$U/config/querencia/desktop.ini")
mkdir -p "$U/config/querencia/labwc/environment.d"; printf '<labwc_config/>\n' >"$U/config/querencia/labwc/rc.xml"
for f in rc.xml.vorher-20261020-112812 rc.xml.vorher-20261020-201950 rc.xml.einstellungen-vorher-20261020-220000; do echo alt >"$U/config/querencia/labwc/$f"; done
printf 'XCURSOR_SIZE=32\n' >"$U/config/querencia/labwc/environment.d/zz-querencia-einstellungen.env"
l=$(cd "$U/config/querencia" && find labwc -type f | LC_ALL=C sort | xargs sha256sum)
lauf aktivieren >/dev/null; rc=$(lauf pruefen); lauf zurueck >/dev/null
pruef "mimeapps.list, desktop.ini und labwc-Benutzerkopie samt Sicherungen byte-gleich" "[ $rc = 0 ] && mime_gleich && [ \"\$(sha256sum <$U/config/querencia/desktop.ini)\" = \"$d\" ] && [ \"\$(cd $U/config/querencia && find labwc -type f | LC_ALL=C sort | xargs sha256sum)\" = \"$l\" ]"
unset XDG_CONFIG_HOME

pruef "echte Dateien unverändert (Inhalt, Inode, Ablage 0.1.0, mimeapps.list, labwc-Benutzerkopie, dconf, Journal, kein Zustand von 0.1.1); live keine 0.1.1-Ablage" "[ \"\$echt_vorher\" = \"\$(echt)\" ] && echo \"\$echt_vorher\" | grep -q 'keine 0.1.1-Ablage live'"
# Ändert eine andere Session während der Probe live Menüeintrag, protokoll.txt
# oder Journal, schlagen die beiden letzten Prüfungen an, ohne dass dieses
# Skript etwas geändert hat – dann Zeitstempel im Protokoll ansehen.
pruef "echter Menüeintrag unverändert seit Probebeginn" "cmp -s $EA/$DESKNAME $BASIS/menueeintrag-echt.desktop"
echo "ERGEBNIS integration-011: $ok PASS, $bad FAIL"
[ "$bad" = 0 ]
