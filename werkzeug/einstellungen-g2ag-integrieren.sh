#!/bin/bash
# Querencia Einstellungen 0.0.8 (G2-AG: Bereiche „Tastatur“, „Maus“,
# „Fensterdesign“, „Arbeitsflächen“ schreiben die labwc-Konfiguration in die
# Benutzerkopie ~/.config/querencia/labwc, QD-046 ②③⑤): Versionswechsel der
# Live-Ablage. NUR NACH FREIGABE (Status-Session prüft und gibt frei).
#
#   bash einstellungen-g2ag-integrieren.sh pruefen      nur lesen
#   bash einstellungen-g2ag-integrieren.sh aktivieren   aktueller Stand → 0.0.8
#   bash einstellungen-g2ag-integrieren.sh zurueck      0.0.8 → Stand vor aktivieren
#
# Ausgang („alt“) ist der dann aktuelle Live-Stand: Link
# ~/opt/querencia/bin/querencia-einstellungen → Ablage einer der Versionen in
# PROG_ALT_LISTE (0.0.7 G2-AF, live seit 20.10. 03:37; 0.0.6 G2-AA; 0.0.5 G2-Z), im
# Menüeintrag Exec/TryExec auf dieselbe Ablage. aktivieren merkt sich diesen
# Stand in $STATE/einstellungen-g2ag-vorher.txt; zurueck stellt genau ihn
# wieder her. Dieses Skript verwaltet im Menüeintrag NUR Exec= und TryExec=;
# Comment= (G2-X), Icon= (G2-V), Name und Kategorien bleiben byte-gleich.
#
# Komponenten (jede atomar per mv bzw. Link-Tausch):
#   A  ~/opt/querencia/einstellungen/<NEU16>/querencia-einstellungen
#        neue schreibgeschützte Ablage (0555) + <NEU16>.herkunft; bleibt liegen
#   P  ~/opt/querencia/bin/querencia-einstellungen → Ablage A
#   D  ~/.local/share/applications/org.querencia.Einstellungen.desktop:
#        Exec/TryExec → <NEU16>
#   Dazu je eine Zeile in ~/opt/querencia/protokoll.txt und im Journal
#   ~/.local/state/querencia/panel-rueckfall.txt.
# NICHT Teil dieses Skripts: ~/.config/querencia/labwc (schreibt nur der
# Benutzer über die Bereiche; Starter-Umstellung = Teil A im Runtime-Repo,
# labwc-benutzer-integrieren.sh), rc.xml im Runtime-Repo, mimeapps.list,
# desktop.ini, Desktop, Autostart, Panel. Kein Neustart, kein labwc
# --reconfigure; ein offenes Einstellungsfenster läuft in der alten Version
# weiter, bis es geschlossen wird. Ohne Teil A zeigen die vier Bereiche
# „wirksam ab der nächsten Anmeldung“ und schreiben trotzdem in die
# Benutzerkopie (wirkt, sobald Teil A aktiv ist).
#
# Rückbau-Reihenfolge: zuerst dieses Skript `zurueck`, dann die Skripte der
# vorigen Versionen (G2-AF, G2-AA, G2-Z, G2-X), ganz zuletzt G2-V.
#
# Aufbau wie einstellungen-g2z-integrieren.sh: Sperre (gemeinsam mit den
# anderen einstellungen-*-integrieren.sh), Vorprüfung (jede Komponente „alt“
# oder „neu“, sonst nichts geändert), Sicherung, Schritte mit Kontrolle,
# Rückrollen bei Fehler oder INT/TERM/HUP; Exit 0 erledigt, 1 zurückgerollt,
# 2 verweigert, 3 Rückrollen unvollständig.
# Nur für die Probe: QE_OPT, QE_APPS, QE_STATE; QE_PROBE=1 mit QE_FEHLER=<Stelle>.
set -uo pipefail
HIER=~/projects/querencia-einstellungen-wt-labwc/werkzeug
VORSCHLAG=~/projects/querencia-einstellungen-wt-labwc/integration/g2ag/vorschlag
OPT=${QE_OPT:-~/opt/querencia}
APPS=${QE_APPS:-~/.local/share/applications}
STATE=${QE_STATE:-~/.local/state/querencia}

PROG=querencia-einstellungen
DESKNAME=org.querencia.Einstellungen.desktop
LINK=$OPT/bin/$PROG
DESK=$APPS/$DESKNAME
PROTO=$OPT/protokoll.txt

# Zulässige Ausgangsstände (volle SHA256) stehen in alt-erlaubt.txt der
# Vorlage (0.0.7 G2-AF, 0.0.6 G2-AA, 0.0.5 G2-Z); weitere nur nach Prüfung.
PROG_ALT_LISTE=$(grep -v '^#' "$VORSCHLAG/alt-erlaubt.txt" 2>/dev/null | cut -d' ' -f1)
PROG_NEU=$(grep " $PROG\$" "$VORSCHLAG/SHA256SUMS" 2>/dev/null | cut -d' ' -f1)
NEU16=${PROG_NEU:0:16}
ABLAGE_NEU=$OPT/einstellungen/$NEU16
VORHER_DATEI=$STATE/einstellungen-g2ag-vorher.txt
# Ausgangsstand: beim aktivieren der aktuelle Link, sonst der gemerkte.
alt_bestimmen() {
    local s
    PROG_ALT=""
    if [ -L "$LINK" ]; then
        s=$(summe "$LINK")
        if [ "$s" != "$PROG_NEU" ] && echo "$PROG_ALT_LISTE" | grep -qx "$s"; then PROG_ALT=$s; fi
    fi
    if [ -z "$PROG_ALT" ] && [ -f "$VORHER_DATEI" ]; then
        s=$(cut -d' ' -f1 "$VORHER_DATEI")
        if echo "$PROG_ALT_LISTE" | grep -qx "$s"; then PROG_ALT=$s; fi
    fi
    [ -n "$PROG_ALT" ] || PROG_ALT=$(echo "$PROG_ALT_LISTE" | head -1)
    ALT16=${PROG_ALT:0:16}
    ABLAGE_ALT=$OPT/einstellungen/$ALT16
    # Die verwalteten Zeilen des Menüeintrags (Exec/TryExec zeigen immer auf
    # die echte Ablage, auch in der Probe, wie bei G2-C/G2-G/G2-X/G2-Z).
    EXEC_ALT=~/opt/querencia/einstellungen/$ALT16/$PROG
}
EXEC_NEU=~/opt/querencia/einstellungen/$NEU16/$PROG

summe() { sha256sum "$1" 2>/dev/null | cut -d' ' -f1; }
melde() { echo "einstellungen-g2ag-integrieren: $*"; }
die() { echo "einstellungen-g2ag-integrieren: $*" >&2; exit 2; }

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
    if zeile_ist Exec "$EXEC_ALT" "$f" && zeile_ist TryExec "$EXEC_ALT" "$f"; then echo alt
    elif zeile_ist Exec "$EXEC_NEU" "$f" && zeile_ist TryExec "$EXEC_NEU" "$f"; then echo neu
    else echo fremd; fi
}
zustand_d() {
    [ -f "$DESK" ] && [ ! -L "$DESK" ] || { [ -e "$DESK" ] && echo fremd || echo fehlt; return; }
    [ "$(grep -c '^\[' "$DESK")" = 1 ] && grep -qx '\[Desktop Entry\]' "$DESK" || { echo fremd; return; }
    zustand_datei "$DESK"
}
alle() { echo "$(zustand_p) $(zustand_d)"; }

invarianten() {
    [ ${#PROG_NEU} = 64 ] || die "PROG_NEU nicht gesetzt (SHA256SUMS der Vorlage)"
    [ -n "$PROG_ALT_LISTE" ] || die "alt-erlaubt.txt fehlt oder leer"
    (cd "$VORSCHLAG" && sha256sum -c --quiet SHA256SUMS >/dev/null 2>&1) || die "Vorlagenordner stimmt nicht mit SHA256SUMS überein: $VORSCHLAG"
    [ "$(summe "$VORSCHLAG/$PROG")" = "$PROG_NEU" ] || die "Vorlage Programm: falsche Prüfsumme ($VORSCHLAG/$PROG)"
    [ -d "$OPT/bin" ] && [ ! -L "$OPT/bin" ] || die "kein Ablageordner: $OPT/bin"
    [ -f "$PROTO" ] || die "Protokoll fehlt: $PROTO"
    [ -L "$OPT/bin/querencia-desktop" ] || die "kein Desktop-Link in $OPT/bin – falscher Ablageordner?"
    [ -d "$APPS" ] || die "kein Anwendungsordner: $APPS"
    [ "$(summe "$ABLAGE_ALT/$PROG")" = "$PROG_ALT" ] || die "Ablage des Ausgangsstands fehlt oder verändert (Rückfallziel): $ABLAGE_ALT"
    if [ -e "$ABLAGE_NEU" ] || [ -L "$ABLAGE_NEU" ]; then
        [ -d "$ABLAGE_NEU" ] && [ ! -L "$ABLAGE_NEU" ] || die "0.0.8-Ablage ist kein Ordner: $ABLAGE_NEU"
        [ "$(summe "$ABLAGE_NEU/$PROG")" = "$PROG_NEU" ] || die "0.0.8-Ablage verändert: $ABLAGE_NEU"
    fi
}

zustaende_zulaessig() {
    local z
    for z in "Programm:$(zustand_p)" "Menüeintrag:$(zustand_d)"; do
        case ${z#*:} in alt | neu) ;; fehlt) die "${z%%:*} fehlt – zuerst die Vorversionen; nichts geändert" ;; *) die "${z%%:*} ist ${z#*:} – nichts geändert" ;; esac
    done
}

sperren() {
    mkdir -p -- "$STATE" || die "kann $STATE nicht anlegen"
    exec 8>"$STATE/einstellungen-integrieren.lock"
    flock -n 8 || die "ein anderer Lauf eines einstellungen-*-integrieren.sh ist aktiv – nichts geändert"
}

journal() { echo "$(date '+%F %T') einstellungen-g2ag: $*" >>"$STATE/panel-rueckfall.txt"; }
protokoll() { echo "$(date '+%F %T') $*" >>"$PROTO"; }

link_setzen() {
    local link=$1 ziel=$2 stelle=$3 tmp
    tmp=$(dirname "$link")/.$(basename "$link").g2ag-tmp
    rm -f -- "$tmp"
    ln -s -- "$ziel" "$tmp" || return 1
    fehlerstelle "$stelle" || { rm -f -- "$tmp"; return 1; }
    mv -f -T -- "$tmp" "$link" || { rm -f -- "$tmp"; return 1; }
    fehlerstelle "$stelle-nach-mv" || return 1
}

# Menüeintrag: nur Exec= und TryExec= auf <nach> setzen (Modus bleibt), über
# eine Kopie im selben Ordner und mv.
menue_umstellen() {
    local nach=$1 stelle=$2 exec_n modus tmp
    if [ "$nach" = neu ]; then exec_n=$EXEC_NEU; else exec_n=$EXEC_ALT; fi
    modus=$(stat -c %a "$DESK")
    tmp=$APPS/.$DESKNAME.g2ag-tmp
    rm -f -- "$tmp"
    awk -v e="$exec_n" '
        /^Exec=/ { print "Exec=" e; next }
        /^TryExec=/ { print "TryExec=" e; next }
        { print }' "$DESK" >"$tmp" && chmod "$modus" "$tmp" || { rm -f -- "$tmp"; return 1; }
    [ "$(zustand_datei "$tmp")" = "$nach" ] || { rm -f -- "$tmp"; return 1; }
    # alles außer den zwei Zeilen byte-gleich?
    diff <(grep -vE '^(Exec|TryExec)=' "$DESK") <(grep -vE '^(Exec|TryExec)=' "$tmp") >/dev/null || { rm -f -- "$tmp"; return 1; }
    fehlerstelle "$stelle" || { rm -f -- "$tmp"; return 1; }
    mv -f -T -- "$tmp" "$DESK" || { rm -f -- "$tmp"; return 1; }
    fehlerstelle "$stelle-nach-mv" || return 1
}

datei_ersetzen() {
    local quelle=$1 ziel=$2 modus tmp
    modus=$(stat -c %a "$ziel" 2>/dev/null || echo 644)
    tmp=$(dirname "$ziel")/.$(basename "$ziel").g2ag-tmp
    rm -f -- "$tmp"
    cp -- "$quelle" "$tmp" && chmod "$modus" "$tmp" && mv -f -T -- "$tmp" "$ziel" || { rm -f -- "$tmp"; return 1; }
}

ablage_anlegen() {
    local tmp
    if [ ! -e "$ABLAGE_NEU" ]; then
        mkdir -p -- "$OPT/einstellungen" || return 1
        tmp=$OPT/einstellungen/.$NEU16.g2ag-tmp
        if [ -e "$tmp" ]; then chmod -R u+w -- "$tmp" && rm -rf -- "$tmp"; fi
        mkdir -- "$tmp" && cp -- "$VORSCHLAG/$PROG" "$tmp/$PROG" &&
            [ "$(summe "$tmp/$PROG")" = "$PROG_NEU" ] &&
            chmod 0555 "$tmp/$PROG" "$tmp" && mv -T -- "$tmp" "$ABLAGE_NEU" || { [ -e "$tmp" ] && chmod -R u+w -- "$tmp" && rm -rf -- "$tmp"; return 1; }
        echo "Quelle: $VORSCHLAG/$PROG (Worktree ~/projects/querencia-einstellungen-wt-labwc, Zweig labwc-baustein, Tag einstellungen/0.0.8)" >"$OPT/einstellungen/$NEU16.herkunft" || return 1
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
    rm -f -- "$OPT/bin/.$PROG.g2ag-tmp" "$APPS/.$DESKNAME.g2ag-tmp"
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
    SICHERUNG=$STATE/sicherung-einstellungen-g2ag-$1-$(date +%Y%m%d-%H%M%S)-$$
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
    "alt alt") melde "Stand: alt – Querencia Einstellungen $ALT16 (Ausgangsstand)" ;;
    "neu neu") melde "Stand: neu – Querencia Einstellungen 0.0.8 ($NEU16, Tastatur, Maus, Fensterdesign, Arbeitsflächen)" ;;
    *) melde "Stand: gemischt ($(alle))" ;;
    esac
    melde "Programm-Link $LINK: $(zustand_p) → $(readlink "$LINK" 2>/dev/null || echo '-')"
    melde "Menüeintrag   $DESK: $(zustand_d) (verwaltet nur Exec/TryExec; $(grep -m1 '^Icon=' "$DESK" 2>/dev/null || echo 'Icon=-') und Comment= werden nie verändert)"
    if [ -e "$ABLAGE_NEU" ]; then melde "Ablage 0.0.8  $ABLAGE_NEU: vorhanden"; else melde "Ablage 0.0.8  $ABLAGE_NEU: fehlt (wird bei aktivieren angelegt)"; fi
    melde "Ausgangsstand $ABLAGE_ALT: vorhanden (Rückfallziel)"
    local l=${XDG_CONFIG_HOME:-$HOME/.config}/querencia/labwc
    melde "labwc-Benutzerkopie $l: $([ -d "$l" ] && echo vorhanden || echo 'fehlt (Teil A legt sie bei der Anmeldung an)'); wird vom Skript nie angefasst"
    melde "unverändert bleiben: Links querencia-desktop/-panel, ~/.config/querencia/labwc, Runtime-Repo, mimeapps.list, desktop.ini, panel.ini, Autostart, Icon= und Comment= im Menüeintrag"
    echo "$(alle)"
}

aktivieren() {
    AKTION=aktivieren
    sperren
    invarianten
    zustaende_zulaessig
    if [ "$(alle)" = "neu neu" ]; then melde "bereits aktiv, nichts geändert"; return 0; fi
    desktop-file-validate "$DESK" >/dev/null 2>&1 || die "Menüeintrag schon vorher ungültig – nichts geändert"
    sichern aktivieren
    echo "$PROG_ALT $ABLAGE_ALT" >"$SICHERUNG/ausgangsstand.txt"
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
    echo "$PROG_ALT $ABLAGE_ALT" >"$VORHER_DATEI"
    protokoll "$PROG -> $ABLAGE_NEU/$PROG (aus $VORSCHLAG/$PROG; 0.0.8 Tastatur, Maus, Fensterdesign, Arbeitsflächen über ~/.config/querencia/labwc; Menüeintrag $DESK nur Exec/TryExec; vorher $ALT16; G2-AG); Rückfall: bash $HIER/einstellungen-g2ag-integrieren.sh zurueck"
    journal "aktiviert (Querencia Einstellungen 0.0.8 $NEU16, vorher $ALT16, Link, Menüeintrag; geändert: ${GEAENDERT[*]:-nichts}); Sicherung $SICHERUNG; Rückfall: bash $HIER/einstellungen-g2ag-integrieren.sh zurueck"
    melde "aktiviert (geändert: ${GEAENDERT[*]:-nichts}). Gilt für jeden neuen Start; ein offenes Fenster läuft in der alten Version weiter."
}

zurueck() {
    AKTION=zurueck
    sperren
    invarianten
    zustaende_zulaessig
    if [ "$(alle)" = "alt alt" ]; then melde "nichts zu tun (Stand ist bereits $ALT16)"; return 0; fi
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
    protokoll "zurück: $PROG -> $ABLAGE_ALT/$PROG ($ALT16; Ablage $NEU16 bleibt; G2-AG)"
    journal "zurück auf $ALT16 (geändert: ${GEAENDERT[*]:-nichts}); Sicherung $SICHERUNG; Ablage $NEU16 bleibt liegen"
    melde "zurück auf $ALT16 (geändert: ${GEAENDERT[*]:-nichts}). ~/.config/querencia/labwc bleibt, wie der Benutzer es eingestellt hat; die alte Version zeigt die Werte nur an."
}

alt_bestimmen
case ${1:-} in
pruefen) pruefen ;;
aktivieren) aktivieren ;;
zurueck) zurueck ;;
*) echo "Aufruf: bash $HIER/einstellungen-g2ag-integrieren.sh pruefen|aktivieren|zurueck" >&2; exit 2 ;;
esac
