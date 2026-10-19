#!/bin/bash
# Querencia Einstellungen 0.0.4 (G2-X: Bereich „Symbole der Arbeitsfläche“ – Rechner,
# Persönlicher Ordner, Papierkorb, Symbolgröße, Laufwerkstypen; schreibt
# desktop.ini [Icons]/[Volumes], QD-042): Versionswechsel der Live-Ablage.
# NUR NACH FREIGABE (Status-Session prüft und gibt frei).
#
#   bash einstellungen-g2x-integrieren.sh pruefen      nur lesen
#   bash einstellungen-g2x-integrieren.sh aktivieren   0.0.2 (G2-G) → 0.0.4 (G2-X)
#   bash einstellungen-g2x-integrieren.sh zurueck      0.0.4 → 0.0.2
#
# Ausgang ist der Endstand von einstellungen-g2g-integrieren.sh: Link
# ~/opt/querencia/bin/querencia-einstellungen → Ablage 03c6dec5e63966b9, im
# Menüeintrag Exec/TryExec auf diese Ablage. Anders als G2-C/G2-G vergleicht
# dieses Skript den Menüeintrag NICHT als Ganzes: Es verwaltet nur die Zeilen
# Exec=, TryExec= und Comment=. Alles andere (Icon= von G2-V, Name, Kategorien,
# Kommentare) bleibt byte-gleich, auch bei zurueck.
#
# Komponenten (jede atomar per mv bzw. Link-Tausch):
#   A  ~/opt/querencia/einstellungen/<NEU16>/querencia-einstellungen
#        neue schreibgeschützte Ablage (0555) + <NEU16>.herkunft; bleibt liegen
#   P  ~/opt/querencia/bin/querencia-einstellungen → Ablage A
#   D  ~/.local/share/applications/org.querencia.Einstellungen.desktop:
#        Exec/TryExec → <NEU16>, Comment nennt die Arbeitsfläche
#   Dazu je eine Zeile in ~/opt/querencia/protokoll.txt und im Journal
#   ~/.local/state/querencia/panel-rueckfall.txt.
# NICHT Teil dieses Skripts: ~/.config/querencia/desktop.ini (schreibt nur
# der Benutzer über „Übernehmen“), der Desktop (eigene Integration QE-04),
# Autostart, Panel. Kein Neustart, kein labwc --reconfigure; ein offenes
# Einstellungsfenster läuft als 0.0.2 weiter, bis es geschlossen wird.
#
# Aufbau wie einstellungen-g2g-integrieren.sh: Sperre (gemeinsam mit den
# anderen einstellungen-*-integrieren.sh), Vorprüfung (jede Komponente „alt“
# oder „neu“, sonst nichts geändert), Sicherung, Schritte mit Kontrolle,
# Rückrollen bei Fehler oder INT/TERM/HUP; Exit 0 erledigt, 1 zurückgerollt,
# 2 verweigert, 3 Rückrollen unvollständig.
# Nur für die Probe: QE_OPT, QE_APPS, QE_STATE; QE_PROBE=1 mit QE_FEHLER=<Stelle>.
set -uo pipefail
HIER=~/projects/querencia-einstellungen/werkzeug
VORSCHLAG=~/projects/querencia-einstellungen/integration/g2x/vorschlag
OPT=${QE_OPT:-~/opt/querencia}
APPS=${QE_APPS:-~/.local/share/applications}
STATE=${QE_STATE:-~/.local/state/querencia}

PROG=querencia-einstellungen
DESKNAME=org.querencia.Einstellungen.desktop
LINK=$OPT/bin/$PROG
DESK=$APPS/$DESKNAME
PROTO=$OPT/protokoll.txt

PROG_ALT=03c6dec5e63966b9088b97c02311d6e3cec0f662831a315e3a436669e0fd0a9b
PROG_NEU=e5acdbead9555b6f9ca1578b27f7a07086531bb01a2b4a0d59b072b7a4a52d1a
ALT16=${PROG_ALT:0:16}
NEU16=${PROG_NEU:0:16}
ABLAGE_ALT=$OPT/einstellungen/$ALT16
ABLAGE_NEU=$OPT/einstellungen/$NEU16
# Die verwalteten Zeilen des Menüeintrags (Exec/TryExec zeigen immer auf die
# echte Ablage, auch in der Probe, wie bei G2-C/G2-G).
EXEC_ALT=~/opt/querencia/einstellungen/$ALT16/$PROG
EXEC_NEU=~/opt/querencia/einstellungen/$NEU16/$PROG
COMMENT_ALT="Hintergrund der Arbeitsfläche, Bildschirmauflösung und Monitoranordnung von Querencia ändern"
COMMENT_NEU="Hintergrund und Symbole der Arbeitsfläche, Bildschirmauflösung und Monitoranordnung von Querencia ändern"

summe() { sha256sum "$1" 2>/dev/null | cut -d' ' -f1; }
melde() { echo "einstellungen-g2x-integrieren: $*"; }
die() { echo "einstellungen-g2x-integrieren: $*" >&2; exit 2; }

fehlerstelle() {
    [ "${QE_PROBE:-}" = 1 ] && [ -n "${QE_OPT:-}" ] || return 0
    case ${QE_FEHLER:-} in
    "$1") melde "Testfehler an Stelle $1"; return 1 ;;
    "signal-$1") melde "Testsignal TERM an Stelle $1"; kill -TERM $$; sleep 1; return 0 ;;
    esac
    return 0
}

zustand_p() {
    local ziel s
    [ -L "$LINK" ] || { [ -e "$LINK" ] && echo fremd || echo fehlt; return; }
    ziel=$(readlink "$LINK"); s=$(summe "$LINK")
    if [ "$ziel" = "$ABLAGE_ALT/$PROG" ] && [ "$s" = "$PROG_ALT" ]; then echo alt
    elif [ "$ziel" = "$ABLAGE_NEU/$PROG" ] && [ "$s" = "$PROG_NEU" ]; then echo neu
    else echo fremd; fi
}

# genau eine Zeile „<schluessel>=<wert>“ in der Gruppe [Desktop Entry]?
zeile_ist() { [ "$(grep -c "^$1=" "$3")" = 1 ] && grep -qxF -- "$1=$2" "$3"; }
zustand_datei() {
    local f=$1
    if zeile_ist Exec "$EXEC_ALT" "$f" && zeile_ist TryExec "$EXEC_ALT" "$f" && zeile_ist Comment "$COMMENT_ALT" "$f"; then echo alt
    elif zeile_ist Exec "$EXEC_NEU" "$f" && zeile_ist TryExec "$EXEC_NEU" "$f" && zeile_ist Comment "$COMMENT_NEU" "$f"; then echo neu
    else echo fremd; fi
}
zustand_d() {
    [ -f "$DESK" ] && [ ! -L "$DESK" ] || { [ -e "$DESK" ] && echo fremd || echo fehlt; return; }
    [ "$(grep -c '^\[' "$DESK")" = 1 ] && grep -qx '\[Desktop Entry\]' "$DESK" || { echo fremd; return; }
    zustand_datei "$DESK"
}
alle() { echo "$(zustand_p) $(zustand_d)"; }

invarianten() {
    [ ${#PROG_NEU} = 64 ] || die "PROG_NEU nicht gesetzt"
    [ "$(summe "$VORSCHLAG/$PROG")" = "$PROG_NEU" ] || die "Vorlage Programm: falsche Prüfsumme ($VORSCHLAG/$PROG)"
    [ -d "$OPT/bin" ] && [ ! -L "$OPT/bin" ] || die "kein Ablageordner: $OPT/bin"
    [ -f "$PROTO" ] || die "Protokoll fehlt: $PROTO"
    [ -L "$OPT/bin/querencia-desktop" ] || die "kein Desktop-Link in $OPT/bin – falscher Ablageordner?"
    [ -d "$APPS" ] || die "kein Anwendungsordner: $APPS"
    [ "$(summe "$ABLAGE_ALT/$PROG")" = $PROG_ALT ] || die "0.0.2-Ablage fehlt oder verändert (Rückfallziel): $ABLAGE_ALT"
    if [ -e "$ABLAGE_NEU" ] || [ -L "$ABLAGE_NEU" ]; then
        [ -d "$ABLAGE_NEU" ] && [ ! -L "$ABLAGE_NEU" ] || die "0.0.4-Ablage ist kein Ordner: $ABLAGE_NEU"
        [ "$(summe "$ABLAGE_NEU/$PROG")" = "$PROG_NEU" ] || die "0.0.4-Ablage verändert: $ABLAGE_NEU"
    fi
}

zustaende_zulaessig() {
    local z
    for z in "Programm:$(zustand_p)" "Menüeintrag:$(zustand_d)"; do
        case ${z#*:} in alt | neu) ;; fehlt) die "${z%%:*} fehlt – zuerst 0.0.2 (einstellungen-g2g-integrieren.sh); nichts geändert" ;; *) die "${z%%:*} ist ${z#*:} – nichts geändert" ;; esac
    done
}

sperren() {
    mkdir -p -- "$STATE" || die "kann $STATE nicht anlegen"
    exec 8>"$STATE/einstellungen-integrieren.lock"
    flock -n 8 || die "ein anderer Lauf eines einstellungen-*-integrieren.sh ist aktiv – nichts geändert"
}

journal() { echo "$(date '+%F %T') einstellungen-g2x: $*" >>"$STATE/panel-rueckfall.txt"; }
protokoll() { echo "$(date '+%F %T') $*" >>"$PROTO"; }

link_setzen() {
    local link=$1 ziel=$2 stelle=$3 tmp
    tmp=$(dirname "$link")/.$(basename "$link").g2x-tmp
    rm -f -- "$tmp"
    ln -s -- "$ziel" "$tmp" || return 1
    fehlerstelle "$stelle" || { rm -f -- "$tmp"; return 1; }
    mv -f -T -- "$tmp" "$link" || { rm -f -- "$tmp"; return 1; }
    fehlerstelle "$stelle-nach-mv" || return 1
}

# Menüeintrag: nur die drei verwalteten Zeilen von <von> auf <nach> setzen
# (Modus bleibt), über eine Kopie im selben Ordner und mv.
menue_umstellen() {
    local nach=$1 stelle=$2 exec_n comment_n modus tmp
    if [ "$nach" = neu ]; then exec_n=$EXEC_NEU comment_n=$COMMENT_NEU; else exec_n=$EXEC_ALT comment_n=$COMMENT_ALT; fi
    modus=$(stat -c %a "$DESK")
    tmp=$APPS/.$DESKNAME.g2x-tmp
    rm -f -- "$tmp"
    awk -v e="$exec_n" -v c="$comment_n" '
        /^Exec=/ { print "Exec=" e; next }
        /^TryExec=/ { print "TryExec=" e; next }
        /^Comment=/ { print "Comment=" c; next }
        { print }' "$DESK" >"$tmp" && chmod "$modus" "$tmp" || { rm -f -- "$tmp"; return 1; }
    [ "$(zustand_datei "$tmp")" = "$nach" ] || { rm -f -- "$tmp"; return 1; }
    # alles außer den drei Zeilen byte-gleich?
    diff <(grep -vE '^(Exec|TryExec|Comment)=' "$DESK") <(grep -vE '^(Exec|TryExec|Comment)=' "$tmp") >/dev/null || { rm -f -- "$tmp"; return 1; }
    fehlerstelle "$stelle" || { rm -f -- "$tmp"; return 1; }
    mv -f -T -- "$tmp" "$DESK" || { rm -f -- "$tmp"; return 1; }
    fehlerstelle "$stelle-nach-mv" || return 1
}

datei_ersetzen() {
    local quelle=$1 ziel=$2 modus tmp
    modus=$(stat -c %a "$ziel" 2>/dev/null || echo 644)
    tmp=$(dirname "$ziel")/.$(basename "$ziel").g2x-tmp
    rm -f -- "$tmp"
    cp -- "$quelle" "$tmp" && chmod "$modus" "$tmp" && mv -f -T -- "$tmp" "$ziel" || { rm -f -- "$tmp"; return 1; }
}

ablage_anlegen() {
    local tmp
    if [ ! -e "$ABLAGE_NEU" ]; then
        mkdir -p -- "$OPT/einstellungen" || return 1
        tmp=$OPT/einstellungen/.$NEU16.g2x-tmp
        if [ -e "$tmp" ]; then chmod -R u+w -- "$tmp" && rm -rf -- "$tmp"; fi
        mkdir -- "$tmp" && cp -- "$VORSCHLAG/$PROG" "$tmp/$PROG" &&
            [ "$(summe "$tmp/$PROG")" = "$PROG_NEU" ] &&
            chmod 0555 "$tmp/$PROG" "$tmp" && mv -T -- "$tmp" "$ABLAGE_NEU" || { [ -e "$tmp" ] && chmod -R u+w -- "$tmp" && rm -rf -- "$tmp"; return 1; }
        echo "Quelle: $VORSCHLAG/$PROG (Repo ~/projects/querencia-einstellungen, Tag einstellungen/0.0.4)" >"$OPT/einstellungen/$NEU16.herkunft" || return 1
    fi
    [ "$(summe "$ABLAGE_NEU/$PROG")" = "$PROG_NEU" ] && [ -x "$ABLAGE_NEU/$PROG" ]
}

GEAENDERT=()
REIHENFOLGE=""
SICHERUNG=""
AKTION=""

zurueckrollen() {
    local grund=$1 k ok=1 soll
    trap '' INT TERM HUP
    melde "FEHLER: $grund – rolle zurück (Sicherung $SICHERUNG)"
    rm -f -- "$OPT/bin/.$PROG.g2x-tmp" "$APPS/.$DESKNAME.g2x-tmp"
    for k in $(echo "$REIHENFOLGE" | rev); do
        case $k in
        P)
            soll=$(cat "$SICHERUNG/programm-link.txt")
            [ "$(readlink "$LINK" 2>/dev/null)" = "$soll" ] || link_setzen "$LINK" "$soll" - || ok=0
            ;;
        D)
            [ "$(summe "$DESK")" = "$(summe "$SICHERUNG/$DESKNAME")" ] ||
                datei_ersetzen "$SICHERUNG/$DESKNAME" "$DESK" || ok=0
            ;;
        esac
    done
    if [ "$ok" = 1 ] && [ "$(alle)" = "$(cat "$SICHERUNG/zustand-vorher.txt")" ] &&
        [ "$(readlink "$LINK")" = "$(cat "$SICHERUNG/programm-link.txt")" ] &&
        [ "$(summe "$DESK")" = "$(summe "$SICHERUNG/$DESKNAME")" ]; then
        journal "$AKTION abgebrochen ($grund), Ausgangsstand wiederhergestellt ($(alle)); Sicherung $SICHERUNG"
        melde "zurückgerollt: Zustand wie vor dem Aufruf ($(alle))"
        exit 1
    fi
    journal "$AKTION abgebrochen ($grund), RÜCKROLLEN UNVOLLSTÄNDIG; Sicherung $SICHERUNG"
    melde "RÜCKROLLEN UNVOLLSTÄNDIG – Ausgangsstand liegt in $SICHERUNG; Zustand jetzt: $(alle)" >&2
    exit 3
}

schritt() {
    local k=$1 soll=$2 ist
    shift 2
    GEAENDERT+=("$k")
    echo "$k" >>"$SICHERUNG/schritte.txt"
    "$@" || zurueckrollen "Schritt $k fehlgeschlagen"
    case $k in P) ist=$(zustand_p) ;; D) ist=$(zustand_d) ;; esac
    [ "$ist" = "$soll" ] || zurueckrollen "Kontrolle nach Schritt $k: $ist statt $soll"
}

sichern() {
    SICHERUNG=$STATE/sicherung-einstellungen-g2x-$1-$(date +%Y%m%d-%H%M%S)-$$
    mkdir -p -- "$SICHERUNG" || die "Sicherung $SICHERUNG nicht anlegbar – nichts geändert"
    readlink "$LINK" >"$SICHERUNG/programm-link.txt" &&
        cp -p -- "$DESK" "$SICHERUNG/$DESKNAME" &&
        alle >"$SICHERUNG/zustand-vorher.txt" &&
        cp -p -- "$PROTO" "$SICHERUNG/protokoll.txt.vorher" &&
        (cd "$SICHERUNG" && sha256sum -- * >sha256-vorher.txt) ||
        die "Sicherung unvollständig – nichts geändert"
    : >"$SICHERUNG/schritte.txt"
}

transaktion_beginnen() {
    REIHENFOLGE=$1
    trap 'zurueckrollen "Signal empfangen"' INT TERM HUP
}

pruefen() {
    invarianten
    case $(alle) in
    "alt alt") melde "Stand: alt – Querencia Einstellungen 0.0.2 ($ALT16, Hintergrund + Bildschirm)" ;;
    "neu neu") melde "Stand: neu – Querencia Einstellungen 0.0.4 ($NEU16, dazu Symbole der Arbeitsfläche)" ;;
    *) melde "Stand: gemischt ($(alle))" ;;
    esac
    melde "Programm-Link $LINK: $(zustand_p) → $(readlink "$LINK" 2>/dev/null || echo '-')"
    melde "Menüeintrag   $DESK: $(zustand_d) ($(grep -m1 '^Icon=' "$DESK" 2>/dev/null || echo 'Icon=-'), wird nie verändert)"
    if [ -e "$ABLAGE_NEU" ]; then melde "Ablage 0.0.4  $ABLAGE_NEU: vorhanden"; else melde "Ablage 0.0.4  $ABLAGE_NEU: fehlt (wird bei aktivieren angelegt)"; fi
    melde "Ablage 0.0.2  $ABLAGE_ALT: vorhanden (Rückfallziel)"
    local ini=${XDG_CONFIG_HOME:-$HOME/.config}/querencia/desktop.ini
    if [ -f "$ini" ]; then
        melde "desktop.ini vorhanden, Gruppen: $(grep -o '^\[[^]]*\]' "$ini" | tr '\n' ' ')(wird vom Skript nie angefasst)"
    else
        melde "keine desktop.ini (entsteht erst durch „Übernehmen“ in der App)"
    fi
    melde "unverändert bleiben: Links querencia-desktop/-panel, panel.ini, desktop.ini, desktop-symbole, Autostart, Icon= im Menüeintrag"
}

aktivieren() {
    AKTION=aktivieren
    sperren
    invarianten
    zustaende_zulaessig
    if [ "$(alle)" = "neu neu" ]; then melde "bereits aktiv, nichts geändert"; return 0; fi
    desktop-file-validate "$DESK" >/dev/null 2>&1 || die "Menüeintrag schon vorher ungültig – nichts geändert"
    sichern aktivieren
    melde "Sicherung: $SICHERUNG (vorher: $(alle))"
    transaktion_beginnen "P D"
    fehlerstelle vor-erstem || zurueckrollen "Testfehler vor dem ersten Schritt"
    ablage_anlegen || zurueckrollen "Ablage nicht anlegbar"
    [ "$(zustand_p)" = alt ] && schritt P neu link_setzen "$LINK" "$ABLAGE_NEU/$PROG" waehrend-programm
    fehlerstelle nach-programm || zurueckrollen "Testfehler nach dem Programm"
    [ "$(zustand_d)" = alt ] && schritt D neu menue_umstellen neu waehrend-menue
    fehlerstelle nach-menue || zurueckrollen "Testfehler nach dem Menüeintrag"
    [ "$(alle)" = "neu neu" ] || zurueckrollen "Schlusskontrolle: $(alle)"
    (invarianten) || zurueckrollen "Invarianten nach der Aktivierung verletzt"
    desktop-file-validate "$DESK" >/dev/null 2>&1 || zurueckrollen "Menüeintrag ungültig"
    trap - INT TERM HUP
    protokoll "$PROG -> $ABLAGE_NEU/$PROG (aus $VORSCHLAG/$PROG; 0.0.4 Symbole der Arbeitsfläche; Menüeintrag $DESK nur Exec/TryExec/Comment; G2-X)"
    journal "aktiviert (Querencia Einstellungen 0.0.4 $NEU16, Link, Menüeintrag; geändert: ${GEAENDERT[*]:-nichts}); Sicherung $SICHERUNG; Rückfall: bash $HIER/einstellungen-g2x-integrieren.sh zurueck"
    melde "aktiviert (geändert: ${GEAENDERT[*]:-nichts}). Gilt für jeden neuen Start; ein offenes Fenster läuft als 0.0.2 weiter."
}

zurueck() {
    AKTION=zurueck
    sperren
    invarianten
    zustaende_zulaessig
    if [ "$(alle)" = "alt alt" ]; then melde "nichts zu tun (Stand ist bereits 0.0.2)"; return 0; fi
    sichern zurueck
    melde "Sicherung: $SICHERUNG (vorher: $(alle))"
    transaktion_beginnen "D P"
    fehlerstelle zurueck-vor-erstem || zurueckrollen "Testfehler vor dem ersten Rückfall-Schritt"
    [ "$(zustand_d)" = neu ] && schritt D alt menue_umstellen alt zurueck-waehrend-menue
    fehlerstelle zurueck-nach-menue || zurueckrollen "Testfehler nach dem Menüeintrag"
    [ "$(zustand_p)" = neu ] && schritt P alt link_setzen "$LINK" "$ABLAGE_ALT/$PROG" zurueck-waehrend-programm
    [ "$(alle)" = "alt alt" ] || zurueckrollen "Schlusskontrolle: $(alle)"
    (invarianten) || zurueckrollen "Invarianten nach dem Rückfall verletzt"
    trap - INT TERM HUP
    protokoll "zurück: $PROG -> $ABLAGE_ALT/$PROG (0.0.2; Ablage $NEU16 bleibt; G2-X)"
    journal "zurück auf 0.0.2 (geändert: ${GEAENDERT[*]:-nichts}); Sicherung $SICHERUNG; Ablage $NEU16 bleibt liegen"
    melde "zurück auf 0.0.2 (geändert: ${GEAENDERT[*]:-nichts}). [Icons]/[Volumes] in desktop.ini bleiben liegen; Achtung: 0.0.2 schreibt beim Hintergrund-Übernehmen die ganze Datei neu und entfernt sie dabei."
}

case ${1:-} in
pruefen) pruefen ;;
aktivieren) aktivieren ;;
zurueck) zurueck ;;
*) echo "Aufruf: bash $HIER/einstellungen-g2x-integrieren.sh pruefen|aktivieren|zurueck" >&2; exit 2 ;;
esac
