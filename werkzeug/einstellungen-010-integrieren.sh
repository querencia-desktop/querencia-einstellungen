#!/bin/bash
# Querencia Einstellungen 0.1.0 (deutsch/englisch über gettext, Bereich
# „Sprache und Region“, mehrere Tastaturbelegungen mit Umschalttaste;
# QD-055/056/057): Versionswechsel der Live-Ablage. NUR NACH FREIGABE
# (Status-Session prüft und gibt frei).
#
#   bash einstellungen-010-integrieren.sh pruefen      nur lesen
#   bash einstellungen-010-integrieren.sh aktivieren   0.0.9 → 0.1.0
#   bash einstellungen-010-integrieren.sh zurueck      0.1.0 → Stand vor aktivieren (0.0.9)
#
# Ausgang („alt“) ist der Live-Stand 0.0.9 (G2-AJ, de7e95a52a8eec92, live seit
# 20.10. 14:03): Link ~/opt/querencia/bin/querencia-einstellungen → dessen
# Ablage, im Menüeintrag Exec/TryExec auf dieselbe Ablage. Zulässig ist nur
# dieser Ausgang (alt-erlaubt.txt der Vorlage). aktivieren merkt ihn in
# $STATE/einstellungen-010-vorher.txt und den Menüeintrag byte-genau in
# $STATE/einstellungen-010-menue-vorher.desktop; zurueck stellt genau beides
# wieder her.
#
# Komponenten (jede atomar per mv bzw. Link-Tausch):
#   A  ~/opt/querencia/einstellungen/<NEU16>/: querencia-einstellungen (0555),
#        share/locale/{de,en}/LC_MESSAGES/querencia-einstellungen.mo (0444),
#        SHA256SUMS (0444), Ordner 0555; entsteht vollständig unter einem
#        Hilfsnamen und wird erst geprüft umbenannt (nie eine Binary ohne
#        deutschen Katalog, sonst sähe der Benutzer Englisch); + <NEU16>.herkunft
#   P  ~/opt/querencia/bin/querencia-einstellungen → Ablage A
#   D  ~/.local/share/applications/org.querencia.Einstellungen.desktop: ganz
#        ersetzt durch den zweisprachigen Menüeintrag der Vorlage (Name,
#        Comment, Keywords mit [de], aus po/ erzeugt; Icon= unverändert
#        org.querencia.Einstellungen); Exec/TryExec → <NEU16>
#   Dazu je eine Zeile in ~/opt/querencia/protokoll.txt und im Journal
#   ~/.local/state/querencia/panel-rueckfall.txt.
# NICHT Teil dieses Skripts: gsettings/dconf, ~/.config/querencia/labwc (die
# Seite „Sprache und Region“ schreibt dort erst, wenn der Benutzer übernimmt),
# rc.xml und environment im Runtime-Repo, mimeapps.list, desktop.ini,
# panel.ini, Desktop, Autostart, Panel. Kein Neustart, kein labwc
# --reconfigure, kein gsettings; ein offenes Einstellungsfenster läuft in der
# alten Version weiter, bis es geschlossen wird.
#
# Rückbau-Reihenfolge: zuerst dieses Skript `zurueck` (→ 0.0.9), dann die
# Skripte der vorigen Versionen (G2-AJ → 0.0.8, G2-AG, G2-AF, G2-AA, G2-Z,
# G2-X), ganz zuletzt G2-V.
#
# Aufbau wie einstellungen-g2aj-integrieren.sh: Sperre (gemeinsam mit den
# anderen einstellungen-*-integrieren.sh), Vorprüfung (jede Komponente „alt“
# oder „neu“, sonst nichts geändert), Sicherung, Schritte mit Kontrolle,
# Rückrollen bei Fehler oder INT/TERM/HUP; Exit 0 erledigt, 1 zurückgerollt,
# 2 verweigert, 3 Rückrollen unvollständig.
# Nur für die Probe: QE_OPT, QE_APPS, QE_STATE; QE_PROBE=1 mit QE_FEHLER=<Stelle>;
# QE_VORSCHLAG (eine beschädigte Vorlagenkopie) gilt nur mit QE_PROBE=1 und QE_OPT.
set -uo pipefail
HIER=~/projects/querencia-einstellungen-wt-010/werkzeug
VORSCHLAG=~/projects/querencia-einstellungen-wt-010/integration/010/vorschlag
if [ "${QE_PROBE:-}" = 1 ] && [ -n "${QE_OPT:-}" ] && [ -n "${QE_VORSCHLAG:-}" ]; then VORSCHLAG=$QE_VORSCHLAG; fi
OPT=${QE_OPT:-~/opt/querencia}
APPS=${QE_APPS:-~/.local/share/applications}
STATE=${QE_STATE:-~/.local/state/querencia}

PROG=querencia-einstellungen
DESKNAME=org.querencia.Einstellungen.desktop
LINK=$OPT/bin/$PROG
DESK=$APPS/$DESKNAME
PROTO=$OPT/protokoll.txt
MO_DE=share/locale/de/LC_MESSAGES/$PROG.mo
MO_EN=share/locale/en/LC_MESSAGES/$PROG.mo

# Zulässige Ausgangsstände (volle SHA256) stehen in alt-erlaubt.txt der
# Vorlage (nur 0.0.9 G2-AJ); weitere nur nach Prüfung.
PROG_ALT_LISTE=$(grep -v '^#' "$VORSCHLAG/alt-erlaubt.txt" 2>/dev/null | cut -d' ' -f1)
PROG_NEU=$(grep " $PROG\$" "$VORSCHLAG/SHA256SUMS" 2>/dev/null | cut -d' ' -f1)
DE_NEU=$(grep " $MO_DE\$" "$VORSCHLAG/SHA256SUMS" 2>/dev/null | cut -d' ' -f1)
EN_NEU=$(grep " $MO_EN\$" "$VORSCHLAG/SHA256SUMS" 2>/dev/null | cut -d' ' -f1)
DESK_NEU=$(grep " $DESKNAME\$" "$VORSCHLAG/SHA256SUMS" 2>/dev/null | cut -d' ' -f1)
NEU16=${PROG_NEU:0:16}
ABLAGE_NEU=$OPT/einstellungen/$NEU16
VORHER_DATEI=$STATE/einstellungen-010-vorher.txt
MENUE_VORHER=$STATE/einstellungen-010-menue-vorher.desktop
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
    # Exec/TryExec zeigen immer auf die echte Ablage, auch in der Probe
    # (wie G2-C/G2-G/G2-X/G2-Z/G2-AJ).
    EXEC_ALT=~/opt/querencia/einstellungen/$ALT16/$PROG
}
EXEC_NEU=~/opt/querencia/einstellungen/$NEU16/$PROG

summe() { sha256sum "$1" 2>/dev/null | cut -d' ' -f1; }
melde() { echo "einstellungen-010-integrieren: $*"; }
die() { echo "einstellungen-010-integrieren: $*" >&2; exit 2; }

fehlerstelle() {
    [ "${QE_PROBE:-}" = 1 ] && [ -n "${QE_OPT:-}" ] || return 0
    case ${QE_FEHLER:-} in
    "$1") melde "Testfehler an Stelle $1"; return 1 ;;
    "signal-$1") melde "Testsignal TERM an Stelle $1"; kill -TERM $$; sleep 1; return 0 ;;
    esac
    return 0
}

# Ablage vollständig: genau Binary, beide Kataloge und SHA256SUMS, Inhalte
# stimmen, schreibgeschützt.
ablage_vollstaendig() {
    local a=$1
    [ -d "$a" ] && [ ! -L "$a" ] || return 1
    [ "$(cd "$a" && find . -type f | LC_ALL=C sort | tr '\n' ' ')" = "./SHA256SUMS ./$PROG ./$MO_DE ./$MO_EN " ] || return 1
    [ "$(summe "$a/$PROG")" = "$PROG_NEU" ] && [ "$(summe "$a/$MO_DE")" = "$DE_NEU" ] && [ "$(summe "$a/$MO_EN")" = "$EN_NEU" ] || return 1
    (cd "$a" && sha256sum -c --quiet SHA256SUMS >/dev/null 2>&1) || return 1
    [ -z "$(find "$a" -perm /222 2>/dev/null)" ]
}

zustand_p() {
    local ziel s
    [ -L "$LINK" ] || { [ -e "$LINK" ] && echo fremd || echo fehlt; return; }
    ziel=$(readlink "$LINK"); s=$(summe "$LINK")
    if [ "$ziel" = "$ABLAGE_ALT/$PROG" ] && [ "$s" = "$PROG_ALT" ]; then echo alt
    elif [ "$ziel" = "$ABLAGE_NEU/$PROG" ] && ablage_vollstaendig "$ABLAGE_NEU"; then echo neu
    else echo fremd; fi
}

# genau eine Zeile „<schluessel>=<wert>“ in der Gruppe [Desktop Entry]?
zeile_ist() { [ "$(grep -c "^$1=" "$3")" = 1 ] && grep -qxF -- "$1=$2" "$3"; }
zustand_datei() {
    local f=$1
    if [ "$(summe "$f")" = "$DESK_NEU" ]; then echo neu
    elif [ "$(grep -c '^\[' "$f")" = 1 ] && grep -qx '\[Desktop Entry\]' "$f" &&
        zeile_ist Exec "$EXEC_ALT" "$f" && zeile_ist TryExec "$EXEC_ALT" "$f"; then echo alt
    else echo fremd; fi
}
zustand_d() {
    [ -f "$DESK" ] && [ ! -L "$DESK" ] || { [ -e "$DESK" ] && echo fremd || echo fehlt; return; }
    zustand_datei "$DESK"
}
alle() { echo "$(zustand_p) $(zustand_d)"; }

invarianten() {
    [ ${#PROG_NEU} = 64 ] || die "PROG_NEU nicht gesetzt (SHA256SUMS der Vorlage)"
    [ ${#DE_NEU} = 64 ] || die "deutscher Katalog fehlt in der Vorlage ($MO_DE) – ohne ihn sähe der Benutzer Englisch"
    [ ${#EN_NEU} = 64 ] || die "englischer Katalog fehlt in der Vorlage ($MO_EN)"
    [ ${#DESK_NEU} = 64 ] || die "Menüeintrag fehlt in der Vorlage ($DESKNAME)"
    [ -n "$PROG_ALT_LISTE" ] || die "alt-erlaubt.txt fehlt oder leer"
    (cd "$VORSCHLAG" && sha256sum -c --quiet SHA256SUMS >/dev/null 2>&1) || die "Vorlagenordner stimmt nicht mit SHA256SUMS überein: $VORSCHLAG"
    [ "$(summe "$VORSCHLAG/$PROG")" = "$PROG_NEU" ] || die "Vorlage Programm: falsche Prüfsumme ($VORSCHLAG/$PROG)"
    zeile_ist Exec "$EXEC_NEU" "$VORSCHLAG/$DESKNAME" && zeile_ist TryExec "$EXEC_NEU" "$VORSCHLAG/$DESKNAME" || die "Menüeintrag der Vorlage zeigt nicht auf $EXEC_NEU"
    desktop-file-validate "$VORSCHLAG/$DESKNAME" >/dev/null 2>&1 || die "Menüeintrag der Vorlage ungültig"
    [ -d "$OPT/bin" ] && [ ! -L "$OPT/bin" ] || die "kein Ablageordner: $OPT/bin"
    [ -f "$PROTO" ] || die "Protokoll fehlt: $PROTO"
    [ -L "$OPT/bin/querencia-desktop" ] || die "kein Desktop-Link in $OPT/bin – falscher Ablageordner?"
    [ -d "$APPS" ] || die "kein Anwendungsordner: $APPS"
    [ "$(summe "$ABLAGE_ALT/$PROG")" = "$PROG_ALT" ] || die "Ablage des Ausgangsstands fehlt oder verändert (Rückfallziel): $ABLAGE_ALT"
    if [ -e "$ABLAGE_NEU" ] || [ -L "$ABLAGE_NEU" ]; then
        ablage_vollstaendig "$ABLAGE_NEU" || die "0.1.0-Ablage unvollständig oder verändert: $ABLAGE_NEU"
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

journal() { echo "$(date '+%F %T') einstellungen-010: $*" >>"$STATE/panel-rueckfall.txt"; }
protokoll() { echo "$(date '+%F %T') $*" >>"$PROTO"; }

link_setzen() {
    local link=$1 ziel=$2 stelle=$3 tmp
    tmp=$(dirname "$link")/.$(basename "$link").010-tmp
    rm -f -- "$tmp"
    ln -s -- "$ziel" "$tmp" || return 1
    fehlerstelle "$stelle" || { rm -f -- "$tmp"; return 1; }
    mv -f -T -- "$tmp" "$link" || { rm -f -- "$tmp"; return 1; }
    fehlerstelle "$stelle-nach-mv" || return 1
}

# Menüeintrag ganz ersetzen (Modus bleibt), über eine Kopie im selben Ordner
# und mv: neu = Vorlage, alt = gemerkter Menüeintrag vor dem aktivieren.
menue_setzen() {
    local nach=$1 stelle=$2 quelle modus tmp
    if [ "$nach" = neu ]; then quelle=$VORSCHLAG/$DESKNAME; else quelle=$MENUE_VORHER; fi
    modus=$(stat -c %a "$DESK")
    tmp=$APPS/.$DESKNAME.010-tmp
    rm -f -- "$tmp"
    cp -- "$quelle" "$tmp" && chmod "$modus" "$tmp" || { rm -f -- "$tmp"; return 1; }
    [ "$(zustand_datei "$tmp")" = "$nach" ] || { rm -f -- "$tmp"; return 1; }
    fehlerstelle "$stelle" || { rm -f -- "$tmp"; return 1; }
    mv -f -T -- "$tmp" "$DESK" || { rm -f -- "$tmp"; return 1; }
    fehlerstelle "$stelle-nach-mv" || return 1
}

datei_ersetzen() {
    local quelle=$1 ziel=$2 modus tmp
    modus=$(stat -c %a "$ziel" 2>/dev/null || echo 644)
    tmp=$(dirname "$ziel")/.$(basename "$ziel").010-tmp
    rm -f -- "$tmp"
    cp -- "$quelle" "$tmp" && chmod "$modus" "$tmp" && mv -f -T -- "$tmp" "$ziel" || { rm -f -- "$tmp"; return 1; }
}

ablage_anlegen() {
    local tmp
    if [ ! -e "$ABLAGE_NEU" ]; then
        mkdir -p -- "$OPT/einstellungen" || return 1
        tmp=$OPT/einstellungen/.$NEU16.010-tmp
        if [ -e "$tmp" ]; then chmod -R u+w -- "$tmp" && rm -rf -- "$tmp"; fi
        mkdir -p -- "$tmp/share/locale/de/LC_MESSAGES" "$tmp/share/locale/en/LC_MESSAGES" &&
            cp -- "$VORSCHLAG/$PROG" "$tmp/$PROG" &&
            cp -- "$VORSCHLAG/$MO_DE" "$tmp/$MO_DE" &&
            cp -- "$VORSCHLAG/$MO_EN" "$tmp/$MO_EN" &&
            (cd "$tmp" && sha256sum -- "$PROG" "$MO_DE" "$MO_EN" >SHA256SUMS) &&
            chmod 0555 "$tmp/$PROG" && chmod 0444 "$tmp/$MO_DE" "$tmp/$MO_EN" "$tmp/SHA256SUMS" &&
            find "$tmp" -type d -exec chmod 0555 {} + &&
            ablage_vollstaendig "$tmp" &&
            fehlerstelle ablage &&
            mv -T -- "$tmp" "$ABLAGE_NEU" || { [ -e "$tmp" ] && chmod -R u+w -- "$tmp" && rm -rf -- "$tmp"; return 1; }
        echo "Quelle: $VORSCHLAG (Worktree ~/projects/querencia-einstellungen-wt-010, Zweig v0.1.0, Tag einstellungen/0.1.0)" >"$OPT/einstellungen/$NEU16.herkunft" || return 1
    fi
    ablage_vollstaendig "$ABLAGE_NEU" && [ -x "$ABLAGE_NEU/$PROG" ]
}

GEAENDERT=()
REIHENFOLGE=""
SICHERUNG=""
AKTION=""

zurueckrollen() {
    local grund=$1 k ok=1 soll
    trap '' INT TERM HUP
    melde "FEHLER: $grund – rolle zurück (Sicherung $SICHERUNG)"
    rm -f -- "$OPT/bin/.$PROG.010-tmp" "$APPS/.$DESKNAME.010-tmp"
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
    SICHERUNG=$STATE/sicherung-einstellungen-010-$1-$(date +%Y%m%d-%H%M%S)-$$
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
    "neu neu") melde "Stand: neu – Querencia Einstellungen 0.1.0 ($NEU16, deutsch/englisch, Sprache und Region, mehrere Tastaturbelegungen)" ;;
    *) melde "Stand: gemischt ($(alle))" ;;
    esac
    melde "Programm-Link $LINK: $(zustand_p) → $(readlink "$LINK" 2>/dev/null || echo '-')"
    melde "Menüeintrag   $DESK: $(zustand_d) (neu = zweisprachiger Menüeintrag der Vorlage, $(grep -m1 '^Icon=' "$VORSCHLAG/$DESKNAME"); alt = Exec/TryExec auf $ALT16)"
    if [ -e "$ABLAGE_NEU" ]; then melde "Ablage 0.1.0  $ABLAGE_NEU: $(ablage_vollstaendig "$ABLAGE_NEU" && echo 'vollständig (Binary, de.mo, en.mo)' || echo UNVOLLSTÄNDIG)"; else melde "Ablage 0.1.0  $ABLAGE_NEU: fehlt (wird bei aktivieren angelegt)"; fi
    melde "Ausgangsstand $ABLAGE_ALT: vorhanden (Rückfallziel)"
    if [ -f "$MENUE_VORHER" ]; then melde "Rückfall-Kopie des Menüeintrags $MENUE_VORHER: $(zustand_datei "$MENUE_VORHER")"; fi
    local l=${XDG_CONFIG_HOME:-$HOME/.config}/querencia/labwc
    melde "labwc-Benutzerkopie $l: $([ -d "$l" ] && echo vorhanden || echo 'fehlt (Teil A legt sie bei der Anmeldung an)'); wird vom Skript nie angefasst"
    melde "unverändert bleiben: gsettings/dconf, Links querencia-desktop/-panel, ~/.config/querencia/labwc, Runtime-Repo, mimeapps.list, desktop.ini, panel.ini, Autostart"
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
    # Menüeintrag des Ausgangs byte-genau merken (Rückfallziel von D)
    if [ "$(zustand_d)" = alt ]; then
        cp -p -- "$DESK" "$MENUE_VORHER.tmp" && mv -f -T -- "$MENUE_VORHER.tmp" "$MENUE_VORHER" || die "Menüeintrag nicht merkbar ($MENUE_VORHER) – nichts geändert"
    fi
    [ "$(zustand_datei "$MENUE_VORHER" 2>/dev/null)" = alt ] || die "gemerkter Menüeintrag fehlt oder ist nicht der Ausgang: $MENUE_VORHER – nichts geändert"
    transaktion_beginnen "P D"
    fehlerstelle vor-erstem || zurueckrollen "Testfehler vor dem ersten Schritt"
    ablage_anlegen || zurueckrollen "Ablage nicht anlegbar"
    [ "$(zustand_p)" = alt ] && schritt P neu link_setzen "$LINK" "$ABLAGE_NEU/$PROG" waehrend-programm
    fehlerstelle nach-programm || zurueckrollen "Testfehler nach dem Programm"
    [ "$(zustand_d)" = alt ] && schritt D neu menue_setzen neu waehrend-menue
    fehlerstelle nach-menue || zurueckrollen "Testfehler nach dem Menüeintrag"
    [ "$(alle)" = "neu neu" ] || zurueckrollen "Schlusskontrolle: $(alle)"
    (invarianten) || zurueckrollen "Invarianten nach der Aktivierung verletzt"
    desktop-file-validate "$DESK" >/dev/null 2>&1 || zurueckrollen "Menüeintrag ungültig"
    trap - INT TERM HUP
    echo "$PROG_ALT $ABLAGE_ALT" >"$VORHER_DATEI"
    protokoll "$PROG -> $ABLAGE_NEU/$PROG (aus $VORSCHLAG; 0.1.0 deutsch/englisch über gettext mit share/locale in der Ablage, Sprache und Region, mehrere Tastaturbelegungen; Menüeintrag $DESK zweisprachig ersetzt, vorher gemerkt in $MENUE_VORHER; vorher $ALT16; Einstellungen 0.1.0); Rückfall: bash $HIER/einstellungen-010-integrieren.sh zurueck"
    journal "aktiviert (Querencia Einstellungen 0.1.0 $NEU16, vorher $ALT16, Link, Menüeintrag; geändert: ${GEAENDERT[*]:-nichts}); Sicherung $SICHERUNG; Rückfall: bash $HIER/einstellungen-010-integrieren.sh zurueck"
    melde "aktiviert (geändert: ${GEAENDERT[*]:-nichts}). Gilt für jeden neuen Start; ein offenes Fenster läuft in der alten Version weiter."
}

zurueck() {
    AKTION=zurueck
    sperren
    invarianten
    zustaende_zulaessig
    if [ "$(alle)" = "alt alt" ]; then melde "nichts zu tun (Stand ist bereits $ALT16)"; return 0; fi
    if [ "$(zustand_d)" = neu ]; then
        [ "$(zustand_datei "$MENUE_VORHER" 2>/dev/null)" = alt ] || die "gemerkter Menüeintrag fehlt oder ist nicht der Ausgang: $MENUE_VORHER – nichts geändert"
    fi
    sichern zurueck
    melde "Sicherung: $SICHERUNG (vorher: $(alle))"
    transaktion_beginnen "D P"
    fehlerstelle zurueck-vor-erstem || zurueckrollen "Testfehler vor dem ersten Rückfall-Schritt"
    [ "$(zustand_d)" = neu ] && schritt D alt menue_setzen alt zurueck-waehrend-menue
    fehlerstelle zurueck-nach-menue || zurueckrollen "Testfehler nach dem Menüeintrag"
    [ "$(zustand_p)" = neu ] && schritt P alt link_setzen "$LINK" "$ABLAGE_ALT/$PROG" zurueck-waehrend-programm
    [ "$(alle)" = "alt alt" ] || zurueckrollen "Schlusskontrolle: $(alle)"
    (invarianten) || zurueckrollen "Invarianten nach dem Rückfall verletzt"
    trap - INT TERM HUP
    protokoll "zurück: $PROG -> $ABLAGE_ALT/$PROG ($ALT16; Menüeintrag wie vor dem aktivieren; Ablage $NEU16 bleibt; Einstellungen 0.1.0)"
    journal "zurück auf $ALT16 (geändert: ${GEAENDERT[*]:-nichts}); Sicherung $SICHERUNG; Ablage $NEU16 bleibt liegen"
    melde "zurück auf $ALT16 (geändert: ${GEAENDERT[*]:-nichts}). gsettings und ~/.config/querencia/labwc bleiben, wie der Benutzer sie eingestellt hat (auch eine dort gewählte Sprache)."
}

alt_bestimmen
case ${1:-} in
pruefen) pruefen ;;
aktivieren) aktivieren ;;
zurueck) zurueck ;;
*) echo "Aufruf: bash $HIER/einstellungen-010-integrieren.sh pruefen|aktivieren|zurueck" >&2; exit 2 ;;
esac
