#!/bin/bash
# Querencia Einstellungen 0.1.1 (QD-080, Werkstatt QREL-06 A17.6: Rückstellung auf
# den Vorgabewert byte-gleich, eigene Sicherungen *.einstellungen-vorher-*
# höchstens fünf, Hinweis gsettings/GNOME im Erscheinungsbild):
# Versionswechsel der Live-Ablage. NUR NACH FREIGABE (Status-Session prüft und
# gibt frei); aktivieren und zurueck verlangen EINSTELLUNGEN_011_FREIGABE=ja.
#
#   bash einstellungen-011-integrieren.sh pruefen      nur lesen
#   EINSTELLUNGEN_011_FREIGABE=ja bash einstellungen-011-integrieren.sh aktivieren   0.1.0 → 0.1.1
#   EINSTELLUNGEN_011_FREIGABE=ja bash einstellungen-011-integrieren.sh zurueck      0.1.1 → 0.1.0
#
# Ausgang („alt“) ist genau der Live-Stand 0.1.0 (df0f666b499ed551, live seit
# 20.10. 19:49:24): Link ~/opt/querencia/bin/querencia-einstellungen → dessen
# Ablage (Binary, de.mo, en.mo mit den Summen unten), Menüeintrag byte-gleich
# mit der 0.1.0-Vorlage (DESK_ALT). Alle Prüfsummen stehen fest in diesem
# Skript (Wunsch der Status-Session nach 0.1.0): die der Vorlage (PROG_NEU,
# DE_NEU, EN_NEU, DESK_NEU), die ihrer SHA256SUMS (SUMMEN_SHA) und die des
# Ausgangs; nichts davon wird aus der Vorlage gelesen. aktivieren merkt den
# Menüeintrag byte-genau in $STATE/einstellungen-011-menue-vorher.desktop;
# zurueck stellt ihn und den Link wieder her.
#
# Komponenten (jede atomar per mv bzw. Link-Tausch):
#   A  ~/opt/querencia/einstellungen/<NEU16>/: querencia-einstellungen (0555),
#        share/locale/{de,en}/LC_MESSAGES/querencia-einstellungen.mo (0444),
#        SHA256SUMS (0444), Ordner 0555; entsteht vollständig unter einem
#        Hilfsnamen und wird erst geprüft umbenannt (nie eine Binary ohne
#        deutschen Katalog); + <NEU16>.herkunft
#   P  ~/opt/querencia/bin/querencia-einstellungen → Ablage A
#   D  ~/.local/share/applications/org.querencia.Einstellungen.desktop: ganz
#        ersetzt durch den Menüeintrag der Vorlage (wie 0.1.0, nur Kommentar
#        und Exec/TryExec → <NEU16>)
#   Dazu je eine Zeile in ~/opt/querencia/protokoll.txt und im Journal
#   ~/.local/state/querencia/panel-rueckfall.txt.
# NICHT Teil dieses Skripts: gsettings/dconf, ~/.config/querencia/labwc (auch
# nicht die dort liegenden Sicherungen rc.xml.vorher-*: die räumt die
# Status-Session), rc.xml und environment im Runtime-Repo, mimeapps.list,
# desktop.ini, panel.ini, Desktop, Autostart, Panel. Kein Neustart, kein labwc
# --reconfigure, kein gsettings; ein offenes Einstellungsfenster läuft in der
# alten Version weiter, bis es geschlossen wird.
#
# Rückbau-Reihenfolge: zuerst dieses Skript `zurueck` (→ 0.1.0), dann
# ~/projects/querencia-einstellungen-wt-010/werkzeug/einstellungen-010-integrieren.sh
# zurueck (→ 0.0.9) und weiter wie dort beschrieben.
#
# Aufbau wie einstellungen-010-integrieren.sh: Sperre (gemeinsam mit den
# anderen einstellungen-*-integrieren.sh), Vorprüfung (jede Komponente „alt“
# oder „neu“, sonst nichts geändert), Sicherung, Schritte mit Kontrolle,
# Rückrollen bei Fehler oder INT/TERM/HUP; Exit 0 erledigt, 1 zurückgerollt,
# 2 verweigert, 3 Rückrollen unvollständig.
# Nur für die Probe: QE_OPT, QE_APPS, QE_STATE – alle drei oder keiner, und
# keiner auf, in oder über einem echten Ort; nur dann gelten QE_PROBE=1 mit
# QE_FEHLER=<Stelle> und QE_VORSCHLAG (eine beschädigte Vorlagenkopie).
set -uo pipefail
HIER=~/projects/querencia-einstellungen-wt-011/werkzeug
VORSCHLAG=~/projects/querencia-einstellungen-wt-011/integration/011/vorschlag
ECHT_OPT=~/opt/querencia
ECHT_APPS=~/.local/share/applications
ECHT_STATE=~/.local/state/querencia

die() { echo "einstellungen-011-integrieren: $*" >&2; exit 2; }

# Umlenkung nur vollständig und nie auf echte Orte (auch nicht darin oder
# darüber): sonst liefe die Probe mit QE_FEHLER gegen die Live-Ablage.
UMGELENKT=
if [ -n "${QE_OPT:-}${QE_APPS:-}${QE_STATE:-}" ]; then
    [ -n "${QE_OPT:-}" ] && [ -n "${QE_APPS:-}" ] && [ -n "${QE_STATE:-}" ] ||
        die "QE_OPT, QE_APPS und QE_STATE nur alle drei zusammen – nichts geändert"
    for v in "$QE_OPT" "$QE_APPS" "$QE_STATE"; do
        r=$(realpath -m -- "$v")
        for e in "$ECHT_OPT" "$ECHT_APPS" "$ECHT_STATE"; do
            case "$r/" in "$e/"*) die "Umlenkung $v liegt auf oder in $e – nichts geändert" ;; esac
            case "$e/" in "$r/"*) die "Umlenkung $v enthält $e – nichts geändert" ;; esac
        done
    done
    UMGELENKT=ja
fi
PROBE=
if [ "${QE_PROBE:-}" = 1 ] && [ -n "$UMGELENKT" ]; then PROBE=ja; fi
if [ -n "$PROBE" ] && [ -n "${QE_VORSCHLAG:-}" ]; then VORSCHLAG=$QE_VORSCHLAG; fi
OPT=${QE_OPT:-$ECHT_OPT}
APPS=${QE_APPS:-$ECHT_APPS}
STATE=${QE_STATE:-$ECHT_STATE}

PROG=querencia-einstellungen
DESKNAME=org.querencia.Einstellungen.desktop
LINK=$OPT/bin/$PROG
DESK=$APPS/$DESKNAME
PROTO=$OPT/protokoll.txt
MO_DE=share/locale/de/LC_MESSAGES/$PROG.mo
MO_EN=share/locale/en/LC_MESSAGES/$PROG.mo

# Feste Prüfsummen. Neu: Vorlage integration/011/vorschlag (Tag einstellungen/0.1.1).
PROG_NEU=557957f3f09b7939f22f41eed9342183ddedd828971293d92a9837c9609af5e5
DE_NEU=eb7f266174838110333ca52af588997b5b53f55aeb37480eb0eb213b9593dda5
EN_NEU=3f35fc107efc99390c904a31b6630cc12918a681a46a0722440d94170c1c7e49
DESK_NEU=d6c74e710035f10992177ff924ca8f8d3fc32abee7dd4773a25b112018e58407
SUMMEN_SHA=3049ff1c2bb00387b93462e5116dad6e14cb4fbff3190e36e94248b066b4837f
# Alt: 0.1.0 (Ablage df0f666b499ed551, Menüeintrag = Vorlage integration/010/vorschlag).
PROG_ALT=df0f666b499ed551ea196b5932262e72526cb13610953efb1088d98f495a144d
DE_ALT=c7976560907c34e47a406544a1c113b6edb7b336c1ba470011cddbdef076ffaf
EN_ALT=0f34b7325b7f2e0d2cbe6aa393823b0286b9604ed6b1216d1f7c52d0e7bd2273
DESK_ALT=9c7e1a20511514a62bfc03260f41aefda07f8092d1b882b7ef29182021fb6173

NEU16=${PROG_NEU:0:16}
ALT16=${PROG_ALT:0:16}
ABLAGE_NEU=$OPT/einstellungen/$NEU16
ABLAGE_ALT=$OPT/einstellungen/$ALT16
# Exec/TryExec zeigen immer auf die echte Ablage, auch in der Probe.
EXEC_NEU=$ECHT_OPT/einstellungen/$NEU16/$PROG
EXEC_ALT=$ECHT_OPT/einstellungen/$ALT16/$PROG
VORHER_DATEI=$STATE/einstellungen-011-vorher.txt
MENUE_VORHER=$STATE/einstellungen-011-menue-vorher.desktop

summe() { sha256sum "$1" 2>/dev/null | cut -d' ' -f1; }
melde() { echo "einstellungen-011-integrieren: $*"; }

freigabe() {
    [ "${EINSTELLUNGEN_011_FREIGABE:-}" = ja ] ||
        die "$1 nur mit EINSTELLUNGEN_011_FREIGABE=ja (nach Freigabe der Status-Session) – nichts geändert"
}

fehlerstelle() {
    [ -n "$PROBE" ] || return 0
    case ${QE_FEHLER:-} in
    "$1") melde "Testfehler an Stelle $1"; return 1 ;;
    "signal-$1") melde "Testsignal TERM an Stelle $1"; kill -TERM $$; sleep 1; return 0 ;;
    esac
    return 0
}

# Ablage vollständig: genau Binary, beide Kataloge und SHA256SUMS, Inhalte
# stimmen mit den festen Summen, schreibgeschützt.
ablage_vollstaendig() {
    local a=$1 p=$2 d=$3 e=$4
    [ -d "$a" ] && [ ! -L "$a" ] || return 1
    [ "$(cd "$a" && find . -type f | LC_ALL=C sort | tr '\n' ' ')" = "./SHA256SUMS ./$PROG ./$MO_DE ./$MO_EN " ] || return 1
    [ "$(summe "$a/$PROG")" = "$p" ] && [ "$(summe "$a/$MO_DE")" = "$d" ] && [ "$(summe "$a/$MO_EN")" = "$e" ] || return 1
    (cd "$a" && sha256sum -c --quiet SHA256SUMS >/dev/null 2>&1) || return 1
    [ -z "$(find "$a" -perm /222 2>/dev/null)" ]
}
neu_vollstaendig() { ablage_vollstaendig "$1" "$PROG_NEU" "$DE_NEU" "$EN_NEU"; }

zustand_p() {
    local ziel
    [ -L "$LINK" ] || { [ -e "$LINK" ] && echo fremd || echo fehlt; return; }
    ziel=$(readlink "$LINK")
    if [ "$ziel" = "$ABLAGE_ALT/$PROG" ] && ablage_vollstaendig "$ABLAGE_ALT" "$PROG_ALT" "$DE_ALT" "$EN_ALT"; then echo alt
    elif [ "$ziel" = "$ABLAGE_NEU/$PROG" ] && neu_vollstaendig "$ABLAGE_NEU"; then echo neu
    else echo fremd; fi
}

zustand_datei() {
    case $(summe "$1") in
    "$DESK_NEU") echo neu ;;
    "$DESK_ALT") echo alt ;;
    *) echo fremd ;;
    esac
}
zustand_d() {
    [ -f "$DESK" ] && [ ! -L "$DESK" ] || { [ -e "$DESK" ] && echo fremd || echo fehlt; return; }
    zustand_datei "$DESK"
}
alle() { echo "$(zustand_p) $(zustand_d)"; }

invarianten() {
    [ "$(summe "$VORSCHLAG/SHA256SUMS")" = "$SUMMEN_SHA" ] || die "SHA256SUMS der Vorlage ist nicht die geprüfte ($SUMMEN_SHA): $VORSCHLAG"
    (cd "$VORSCHLAG" && sha256sum -c --quiet SHA256SUMS >/dev/null 2>&1) || die "Vorlagenordner stimmt nicht mit SHA256SUMS überein: $VORSCHLAG"
    [ "$(summe "$VORSCHLAG/$PROG")" = "$PROG_NEU" ] || die "Vorlage Programm: falsche Prüfsumme ($VORSCHLAG/$PROG)"
    [ "$(summe "$VORSCHLAG/$MO_DE")" = "$DE_NEU" ] || die "deutscher Katalog der Vorlage fehlt oder ist falsch ($MO_DE) – ohne ihn sähe der Benutzer Englisch"
    [ "$(summe "$VORSCHLAG/$MO_EN")" = "$EN_NEU" ] || die "englischer Katalog der Vorlage fehlt oder ist falsch ($MO_EN)"
    [ "$(summe "$VORSCHLAG/$DESKNAME")" = "$DESK_NEU" ] || die "Menüeintrag der Vorlage fehlt oder ist falsch ($DESKNAME)"
    grep -qxF "Exec=$EXEC_NEU" "$VORSCHLAG/$DESKNAME" && grep -qxF "TryExec=$EXEC_NEU" "$VORSCHLAG/$DESKNAME" || die "Menüeintrag der Vorlage zeigt nicht auf $EXEC_NEU"
    desktop-file-validate "$VORSCHLAG/$DESKNAME" >/dev/null 2>&1 || die "Menüeintrag der Vorlage ungültig"
    [ -d "$OPT/bin" ] && [ ! -L "$OPT/bin" ] || die "kein Ablageordner: $OPT/bin"
    [ -f "$PROTO" ] || die "Protokoll fehlt: $PROTO"
    [ -L "$OPT/bin/querencia-desktop" ] || die "kein Desktop-Link in $OPT/bin – falscher Ablageordner?"
    [ -d "$APPS" ] || die "kein Anwendungsordner: $APPS"
    ablage_vollstaendig "$ABLAGE_ALT" "$PROG_ALT" "$DE_ALT" "$EN_ALT" || die "Ablage des Ausgangsstands 0.1.0 fehlt, ist unvollständig oder verändert (Rückfallziel): $ABLAGE_ALT"
    if [ -e "$ABLAGE_NEU" ] || [ -L "$ABLAGE_NEU" ]; then
        neu_vollstaendig "$ABLAGE_NEU" || die "0.1.1-Ablage unvollständig oder verändert: $ABLAGE_NEU"
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

journal() { echo "$(date '+%F %T') einstellungen-011: $*" >>"$STATE/panel-rueckfall.txt"; }
protokoll() { echo "$(date '+%F %T') $*" >>"$PROTO"; }

link_setzen() {
    local link=$1 ziel=$2 stelle=$3 tmp
    tmp=$(dirname "$link")/.$(basename "$link").011-tmp
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
    tmp=$APPS/.$DESKNAME.011-tmp
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
    tmp=$(dirname "$ziel")/.$(basename "$ziel").011-tmp
    rm -f -- "$tmp"
    cp -- "$quelle" "$tmp" && chmod "$modus" "$tmp" && mv -f -T -- "$tmp" "$ziel" || { rm -f -- "$tmp"; return 1; }
}

ablage_anlegen() {
    local tmp
    if [ ! -e "$ABLAGE_NEU" ]; then
        mkdir -p -- "$OPT/einstellungen" || return 1
        tmp=$OPT/einstellungen/.$NEU16.011-tmp
        if [ -e "$tmp" ]; then chmod -R u+w -- "$tmp" && rm -rf -- "$tmp"; fi
        mkdir -p -- "$tmp/share/locale/de/LC_MESSAGES" "$tmp/share/locale/en/LC_MESSAGES" &&
            cp -- "$VORSCHLAG/$PROG" "$tmp/$PROG" &&
            cp -- "$VORSCHLAG/$MO_DE" "$tmp/$MO_DE" &&
            cp -- "$VORSCHLAG/$MO_EN" "$tmp/$MO_EN" &&
            (cd "$tmp" && sha256sum -- "$PROG" "$MO_DE" "$MO_EN" >SHA256SUMS) &&
            chmod 0555 "$tmp/$PROG" && chmod 0444 "$tmp/$MO_DE" "$tmp/$MO_EN" "$tmp/SHA256SUMS" &&
            find "$tmp" -type d -exec chmod 0555 {} + &&
            neu_vollstaendig "$tmp" &&
            fehlerstelle ablage &&
            mv -T -- "$tmp" "$ABLAGE_NEU" || { [ -e "$tmp" ] && chmod -R u+w -- "$tmp" && rm -rf -- "$tmp"; return 1; }
        echo "Quelle: $VORSCHLAG (Worktree ~/projects/querencia-einstellungen-wt-011, Zweig v0.1.1, Tag einstellungen/0.1.1)" >"$OPT/einstellungen/$NEU16.herkunft" || return 1
    fi
    neu_vollstaendig "$ABLAGE_NEU" && [ -x "$ABLAGE_NEU/$PROG" ]
}

GEAENDERT=()
REIHENFOLGE=""
SICHERUNG=""
AKTION=""

zurueckrollen() {
    local grund=$1 k ok=1 soll
    trap '' INT TERM HUP
    melde "FEHLER: $grund – rolle zurück (Sicherung $SICHERUNG)"
    rm -f -- "$OPT/bin/.$PROG.011-tmp" "$APPS/.$DESKNAME.011-tmp"
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
    SICHERUNG=$STATE/sicherung-einstellungen-011-$1-$(date +%Y%m%d-%H%M%S)-$$
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
    "neu neu") melde "Stand: neu – Querencia Einstellungen 0.1.1 ($NEU16, Rückstellung byte-gleich, eigene Sicherungen)" ;;
    *) melde "Stand: gemischt ($(alle))" ;;
    esac
    melde "Programm-Link $LINK: $(zustand_p) → $(readlink "$LINK" 2>/dev/null || echo '-')"
    melde "Menüeintrag   $DESK: $(zustand_d) (neu = Menüeintrag der Vorlage ${DESK_NEU:0:16}, $(grep -m1 '^Icon=' "$VORSCHLAG/$DESKNAME"); alt = Menüeintrag 0.1.0 ${DESK_ALT:0:16})"
    if [ -e "$ABLAGE_NEU" ]; then melde "Ablage 0.1.1  $ABLAGE_NEU: $(neu_vollstaendig "$ABLAGE_NEU" && echo 'vollständig (Binary, de.mo, en.mo)' || echo UNVOLLSTÄNDIG)"; else melde "Ablage 0.1.1  $ABLAGE_NEU: fehlt (wird bei aktivieren angelegt)"; fi
    melde "Ausgangsstand $ABLAGE_ALT: vollständig (Binary, de.mo, en.mo; Rückfallziel 0.1.0)"
    melde "Vorlage $VORSCHLAG: SHA256SUMS ${SUMMEN_SHA:0:16} (fest im Skript)"
    [ -n "$UMGELENKT" ] && melde "umgelenkt (Probe): OPT=$OPT APPS=$APPS STATE=$STATE"
    if [ -f "$MENUE_VORHER" ]; then melde "Rückfall-Kopie des Menüeintrags $MENUE_VORHER: $(zustand_datei "$MENUE_VORHER")"; fi
    local l=${XDG_CONFIG_HOME:-$HOME/.config}/querencia/labwc
    melde "labwc-Benutzerkopie $l: $([ -d "$l" ] && echo vorhanden || echo 'fehlt (Teil A legt sie bei der Anmeldung an)'); wird vom Skript nie angefasst"
    melde "unverändert bleiben: gsettings/dconf, Links querencia-desktop/-panel, ~/.config/querencia/labwc, Runtime-Repo, mimeapps.list, desktop.ini, panel.ini, Autostart"
    echo "$(alle)"
}

aktivieren() {
    AKTION=aktivieren
    freigabe aktivieren
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
    protokoll "$PROG -> $ABLAGE_NEU/$PROG (aus $VORSCHLAG, SHA256SUMS ${SUMMEN_SHA:0:16}; 0.1.1 Rückstellung auf den Vorgabewert byte-gleich, eigene Sicherungen *.einstellungen-vorher-* höchstens 5, Hinweis gsettings/GNOME; Menüeintrag $DESK ersetzt, vorher gemerkt in $MENUE_VORHER; vorher $ALT16 0.1.0; Einstellungen 0.1.1); Rückfall: EINSTELLUNGEN_011_FREIGABE=ja bash $HIER/einstellungen-011-integrieren.sh zurueck"
    journal "aktiviert (Querencia Einstellungen 0.1.1 $NEU16, vorher $ALT16, Link, Menüeintrag; geändert: ${GEAENDERT[*]:-nichts}); Sicherung $SICHERUNG; Rückfall: EINSTELLUNGEN_011_FREIGABE=ja bash $HIER/einstellungen-011-integrieren.sh zurueck"
    melde "aktiviert (geändert: ${GEAENDERT[*]:-nichts}). Gilt für jeden neuen Start; ein offenes Fenster läuft in der alten Version weiter."
}

zurueck() {
    AKTION=zurueck
    freigabe zurueck
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
    protokoll "zurück: $PROG -> $ABLAGE_ALT/$PROG ($ALT16 0.1.0; Menüeintrag wie vor dem aktivieren; Ablage $NEU16 bleibt; Einstellungen 0.1.1)"
    journal "zurück auf $ALT16 (geändert: ${GEAENDERT[*]:-nichts}); Sicherung $SICHERUNG; Ablage $NEU16 bleibt liegen"
    melde "zurück auf $ALT16 (geändert: ${GEAENDERT[*]:-nichts}). gsettings und ~/.config/querencia/labwc bleiben, wie der Benutzer sie eingestellt hat (auch eine dort gewählte Sprache)."
}

case ${1:-} in
pruefen) pruefen ;;
aktivieren) aktivieren ;;
zurueck) zurueck ;;
*) echo "Aufruf: bash $HIER/einstellungen-011-integrieren.sh pruefen | EINSTELLUNGEN_011_FREIGABE=ja … aktivieren|zurueck" >&2; exit 2 ;;
esac
