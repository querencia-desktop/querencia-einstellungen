#!/usr/bin/env python3
# Einstellungen 0.2.0 (Seite „Treiber“): PackageKit-Attrappe für die
# isolierte GUI-Prüfung. Spricht die Teilmenge von org.freedesktop.PackageKit,
# die qs_paketkit (Querencia Software, Tag paketkit/0.1.0) nutzt, auf einem
# PRIVATEN Bus (Adresse aus QS_PK_BUS); verweigert den Systembus. Kein Paket
# wird angefasst: der Bestand steht in <zustand>/bestand.tsv, nach jeder
# Änderung schreibt die Attrappe <zustand>/dpkg.txt neu (Format von
# dpkg-query -W -f '${Package}\t${db:Status-Abbrev}\n'), das der Stub
# dpkg-query der Prüfung ausgibt. Jeder Aufruf steht in <zustand>/pk.log.
# Bringt ein Paket Programme mit, liegen sie unter
# <zustand>/nach-installation/<paket>/ und kommen beim Installieren nach
# <zustand>/bin (z. B. ubuntu-drivers aus ubuntu-drivers-common).
#
#   QS_PK_BUS=unix:path=… python3 -I pk-attrappe-020.py <zustand>
#
# Umgebung: QE_PK_POLKIT=verweigern (schreibend → not-authorized),
# QE_PK_SPERRE=1 (schreibend → cannot-get-lock), QE_PK_SCHRITT_MS (Standard 60).
import os
import sys
import time

from gi.repository import Gio, GLib

ZUSTAND = sys.argv[1]
ADRESSE = os.environ.get("QS_PK_BUS", "")
if not ADRESSE or "system_bus_socket" in ADRESSE:
    sys.exit("pk-attrappe-020: nur auf einem privaten Bus (QS_PK_BUS)")
SCHRITT = int(os.environ.get("QE_PK_SCHRITT_MS", "60"))

PK = "org.freedesktop.PackageKit"
TX = "org.freedesktop.PackageKit.Transaction"
XML = """<node>
 <interface name="org.freedesktop.PackageKit">
  <method name="CreateTransaction"><arg type="o" direction="out"/></method>
  <method name="GetTransactionList"><arg type="ao" direction="out"/></method>
  <method name="CanAuthorize"><arg type="s" direction="in"/><arg type="u" direction="out"/></method>
  <property name="BackendName" type="s" access="read"/>
  <property name="DistroId" type="s" access="read"/>
  <property name="Roles" type="t" access="read"/>
 </interface>
 <interface name="org.freedesktop.PackageKit.Transaction">
  <method name="SetHints"><arg type="as" direction="in"/></method>
  <method name="Resolve"><arg type="t" direction="in"/><arg type="as" direction="in"/></method>
  <method name="SearchNames"><arg type="t" direction="in"/><arg type="as" direction="in"/></method>
  <method name="InstallPackages"><arg type="t" direction="in"/><arg type="as" direction="in"/></method>
  <method name="RemovePackages"><arg type="t" direction="in"/><arg type="as" direction="in"/>
   <arg type="b" direction="in"/><arg type="b" direction="in"/></method>
  <method name="Cancel"/>
  <property name="Percentage" type="u" access="read"/>
  <property name="Status" type="u" access="read"/>
  <property name="Role" type="u" access="read"/>
  <property name="AllowCancel" type="b" access="read"/>
  <signal name="Package"><arg type="u"/><arg type="s"/><arg type="s"/></signal>
  <signal name="ItemProgress"><arg type="s"/><arg type="u"/><arg type="u"/></signal>
  <signal name="ErrorCode"><arg type="u"/><arg type="s"/></signal>
  <signal name="RequireRestart"><arg type="u"/><arg type="s"/></signal>
  <signal name="Finished"><arg type="u"/><arg type="u"/></signal>
  <signal name="Destroy"/>
 </interface>
</node>"""
INFO = Gio.DBusNodeInfo.new_for_xml(XML)

# PackageKit-Werte (pk-enum.h)
INFO_INSTALLIERT, INFO_VERFUEGBAR, INFO_INSTALLIERT_WIRD, INFO_ENTFERNT_WIRD = 1, 2, 12, 13
ST_ENTFERNEN, ST_LADEN, ST_INSTALLIEREN, ST_ABH, ST_SPERRE, ST_BERECHTIGUNG = 6, 8, 9, 7, 30, 31
EXIT_ERFOLG, EXIT_FEHLER, EXIT_ABGEBROCHEN = 1, 2, 3
E_NICHT_GEFUNDEN, E_ABGEBROCHEN, E_SPERRE, E_NICHT_BERECHTIGT, E_INTERN = 8, 17, 26, 48, 4
ROLLE_INSTALL, ROLLE_REMOVE, ROLLE_RESOLVE, ROLLE_SEARCH = 11, 14, 17, 21
FILTER_INSTALLIERT, FILTER_NICHT_INSTALLIERT = 1 << 2, 1 << 3
FLAG_SIMULIEREN = 1 << 2


def protokoll(text):
    with open(os.path.join(ZUSTAND, "pk.log"), "a", encoding="utf-8") as f:
        f.write(f"{time.strftime('%H:%M:%S')} {text}\n")


def bestand_lesen():
    """name → [version, installiert, zusammenfassung]"""
    b = {}
    with open(os.path.join(ZUSTAND, "bestand.tsv"), encoding="utf-8") as f:
        for z in f:
            if z.strip() and not z.startswith("#"):
                name, version, inst, zus = z.rstrip("\n").split("\t")
                b[name] = [version, inst == "1", zus]
    return b


def bestand_schreiben(b):
    zeilen = ["# name\tversion\tinstalliert\tzusammenfassung"]
    zeilen += [f"{n}\t{v}\t{'1' if i else '0'}\t{z}" for n, (v, i, z) in sorted(b.items())]
    with open(os.path.join(ZUSTAND, "bestand.tsv"), "w", encoding="utf-8") as f:
        f.write("\n".join(zeilen) + "\n")
    with open(os.path.join(ZUSTAND, "dpkg.txt"), "w", encoding="utf-8") as f:
        for n, (_, i, _) in sorted(b.items()):
            if i:
                f.write(f"{n}\tii \n")


def dateien_bringen(namen):
    """Ein Paket „bringt“ Programme: <zustand>/nach-installation/<paket>/*
    kommt nach <zustand>/bin (im PATH der Prüfsitzung)."""
    for n in namen:
        quelle = os.path.join(ZUSTAND, "nach-installation", n)
        if not os.path.isdir(quelle):
            continue
        os.makedirs(os.path.join(ZUSTAND, "bin"), exist_ok=True)
        for d in os.listdir(quelle):
            ziel = os.path.join(ZUSTAND, "bin", d)
            with open(os.path.join(quelle, d), "rb") as q, open(ziel, "wb") as z:
                z.write(q.read())
            os.chmod(ziel, 0o755)


def paket_id(name, eintrag):
    return f"{name};{eintrag[0]};amd64;{'installed' if eintrag[1] else 'ubuntu'}"


class Transaktion:
    zaehler = 0

    def __init__(self, bus, eigner):
        Transaktion.zaehler += 1
        self.pfad = f"/org/freedesktop/PackageKit/tid_{Transaktion.zaehler}"
        self.bus, self.eigner = bus, eigner
        self.werte = {"Percentage": 101, "Status": 1, "Role": 0, "AllowCancel": False}
        self.hinweise = []
        self.abbruch = False
        self.start = time.monotonic()
        self.reg = bus.register_object(self.pfad, INFO.interfaces[1], self.methode, self.eigenschaft, None)

    def signal(self, name, typ=None, werte=None):
        v = GLib.Variant(typ, werte) if typ else None
        self.bus.emit_signal(None, self.pfad, TX, name, v)

    def setzen(self, **werte):
        self.werte.update(werte)
        typen = {"Percentage": "u", "Status": "u", "Role": "u", "AllowCancel": "b"}
        geaendert = {k: GLib.Variant(typen[k], v) for k, v in werte.items()}
        self.bus.emit_signal(None, self.pfad, "org.freedesktop.DBus.Properties", "PropertiesChanged",
                             GLib.Variant("(sa{sv}as)", (TX, geaendert, [])))

    def fertig(self, exit_code):
        ms = int((time.monotonic() - self.start) * 1000)
        self.signal("Finished", "(uu)", (exit_code, ms))
        GLib.timeout_add(200, self.zerstoeren)

    def zerstoeren(self):
        self.signal("Destroy")
        self.bus.unregister_object(self.reg)
        return False

    def fehler(self, code, text):
        self.signal("ErrorCode", "(us)", (code, text))
        self.fertig(EXIT_FEHLER)

    def eigenschaft(self, _bus, _sender, _pfad, _iface, name):
        typen = {"Percentage": "u", "Status": "u", "Role": "u", "AllowCancel": "b"}
        return GLib.Variant(typen[name], self.werte[name])

    def methode(self, _bus, sender, _pfad, _iface, name, args, aufruf):
        args = args.unpack()
        protokoll(f"{self.pfad} {name} {args!r}")
        if sender != self.eigner:
            aufruf.return_dbus_error("org.freedesktop.PackageKit.Transaction.RefusedByPolicy", "fremd")
            return
        aufruf.return_value(None)
        if name == "SetHints":
            self.hinweise = list(args[0])
        elif name == "Cancel":
            self.abbruch = True
        elif name in ("Resolve", "SearchNames"):
            self.setzen(Role=ROLLE_RESOLVE if name == "Resolve" else ROLLE_SEARCH)
            GLib.idle_add(self.lesen, name, args[0], args[1])
        elif name == "InstallPackages":
            self.setzen(Role=ROLLE_INSTALL)
            GLib.idle_add(self.schreiben, ROLLE_INSTALL, args[0], list(args[1]))
        elif name == "RemovePackages":
            self.setzen(Role=ROLLE_REMOVE)
            if args[2] or args[3]:
                GLib.idle_add(self.fehler, E_INTERN, "attrappe: allow_deps/autoremove müssen false sein")
            else:
                GLib.idle_add(self.schreiben, ROLLE_REMOVE, args[0], list(args[1]))

    def lesen(self, art, filt, werte):
        b = bestand_lesen()
        for name, e in sorted(b.items()):
            treffer = name in werte if art == "Resolve" else any(w in name for w in werte)
            if not treffer:
                continue
            if filt & FILTER_INSTALLIERT and not e[1]:
                continue
            if filt & FILTER_NICHT_INSTALLIERT and e[1]:
                continue
            self.signal("Package", "(uss)", (INFO_INSTALLIERT if e[1] else INFO_VERFUEGBAR, paket_id(name, e), e[2]))
        self.fertig(EXIT_ERFOLG)
        return False

    def schreiben(self, rolle, flaggen, ids):
        if not all(h in " ".join(self.hinweise) for h in ("locale=", "interactive=", "background=")):
            self.fehler(E_INTERN, "attrappe: SetHints fehlt")
            return False
        b = bestand_lesen()
        namen = [i.split(";")[0] for i in ids]
        for n in namen:
            if n not in b or (rolle == ROLLE_REMOVE and not b[n][1]):
                self.fehler(E_NICHT_GEFUNDEN, f"{n} nicht gefunden")
                return False
        if flaggen & FLAG_SIMULIEREN:
            info = INFO_INSTALLIERT_WIRD if rolle == ROLLE_INSTALL else INFO_ENTFERNT_WIRD
            for i, n in zip(ids, namen):
                self.signal("Package", "(uss)", (info, i, b[n][2]))
            self.fertig(EXIT_ERFOLG)
            return False
        self.setzen(Status=ST_BERECHTIGUNG)
        if os.environ.get("QE_PK_POLKIT") == "verweigern":
            GLib.timeout_add(300, self.fehler, E_NICHT_BERECHTIGT, "attrappe: polkit hat abgelehnt")
            return False
        if os.environ.get("QE_PK_SPERRE") == "1":
            self.setzen(Status=ST_SPERRE)
            GLib.timeout_add(300, self.fehler, E_SPERRE, "Die Paketverwaltung wird von apt (PID 4711) benutzt")
            return False
        self.setzen(Status=ST_ABH, AllowCancel=True, Percentage=0)
        schritte = iter(range(0, 101, 5))

        def weiter():
            if self.abbruch:
                self.signal("ErrorCode", "(us)", (E_ABGEBROCHEN, "Die Transaktion wurde abgebrochen"))
                self.fertig(EXIT_ABGEBROCHEN)
                return False
            p = next(schritte, None)
            if p is None:
                info = INFO_INSTALLIERT_WIRD if rolle == ROLLE_INSTALL else INFO_ENTFERNT_WIRD
                for i, n in zip(ids, namen):
                    self.signal("Package", "(uss)", (info, i, b[n][2]))
                    b[n][1] = rolle == ROLLE_INSTALL
                bestand_schreiben(b)
                if rolle == ROLLE_INSTALL:
                    dateien_bringen(namen)
                protokoll(f"{self.pfad} {'installiert' if rolle == ROLLE_INSTALL else 'entfernt'}: {' '.join(namen)}")
                self.signal("RequireRestart", "(us)", (4, ids[0]))
                self.fertig(EXIT_ERFOLG)
                return False
            phase = ST_ENTFERNEN if rolle == ROLLE_REMOVE else (ST_LADEN if p < 50 else ST_INSTALLIEREN)
            if p in (0, 50):
                self.setzen(Status=phase)
            self.setzen(Percentage=p)
            for i in ids:
                self.signal("ItemProgress", "(suu)", (i, phase, p))
            return True

        GLib.timeout_add(SCHRITT, weiter)
        return False


def main():
    bus = Gio.DBusConnection.new_for_address_sync(
        ADRESSE,
        Gio.DBusConnectionFlags.AUTHENTICATION_CLIENT | Gio.DBusConnectionFlags.MESSAGE_BUS_CONNECTION,
        None, None)
    offen = {}

    def methode(_bus, sender, _pfad, _iface, name, args, aufruf):
        protokoll(f"PackageKit {name} {args.unpack()!r}")
        if name == "CreateTransaction":
            t = Transaktion(bus, sender)
            offen[t.pfad] = t
            aufruf.return_value(GLib.Variant("(o)", (t.pfad,)))
        elif name == "GetTransactionList":
            aufruf.return_value(GLib.Variant("(ao)", ([],)))
        else:
            aufruf.return_value(GLib.Variant("(u)", (1,)))

    def eigenschaft(_bus, _sender, _pfad, _iface, name):
        return {"BackendName": GLib.Variant("s", "attrappe-020"),
                "DistroId": GLib.Variant("s", "ubuntu;26.04;x86_64"),
                "Roles": GLib.Variant("t", (1 << 11) | (1 << 14) | (1 << 17) | (1 << 21))}[name]

    bus.register_object("/org/freedesktop/PackageKit", INFO.interfaces[0], methode, eigenschaft, None)
    antwort = bus.call_sync("org.freedesktop.DBus", "/org/freedesktop/DBus", "org.freedesktop.DBus",
                            "RequestName", GLib.Variant("(su)", (PK, 4)), None, 0, -1, None)
    if antwort.unpack()[0] != 1:
        sys.exit("pk-attrappe-020: Name org.freedesktop.PackageKit nicht bekommen")
    bestand_schreiben(bestand_lesen())
    protokoll(f"bereit auf {ADRESSE}")
    print("bereit", flush=True)
    GLib.MainLoop().run()


main()
