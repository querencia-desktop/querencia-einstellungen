#!/usr/bin/env python3
"""Settings-Portal für die isolierte Probe (nur org.freedesktop.portal.Settings).

GTK 4.22 auf Wayland liest Schrift und Zeiger nur über den Settings-Portal.
Der echte xdg-desktop-portal startet nicht ohne Dokumente-Portal, und der
hängt ein FUSE-Dateisystem ein (Falle PT-1). Dieser Ersatz liefert wie
xdg-desktop-portal-gtk die GSettings-Schemata aus dem dconf des Profils
(Read, ReadOne, ReadAll, Signal SettingChanged) und sonst nichts.
Läuft nur auf dem Wegwerf-Bus der Probe (isoliert.py).

Herkunft: querencia-erscheinungsbild `werkzeug/settings-portal.py`
(Thema-3, Commit 789489f). G2-AJ ergänzt wie xdg-desktop-portal-gtk
`org.freedesktop.appearance color-scheme` (0 keine Vorliebe, 1 dunkel,
2 hell), aus dem GTK 4.22 die dunkle Darstellung liest; Aufruf über
`isoliert.sh` mit `QE_PORTAL=settings`.
"""

import fnmatch
import os
import sys

from gi.repository import Gio, GLib

SCHEMATA = ("org.gnome.desktop.interface", "org.gnome.desktop.wm.preferences",
            "org.gnome.desktop.a11y.interface")
XML = """
<node>
  <interface name="org.freedesktop.portal.Settings">
    <method name="ReadAll"><arg type="as" direction="in"/><arg type="a{sa{sv}}" direction="out"/></method>
    <method name="Read"><arg type="s" direction="in"/><arg type="s" direction="in"/><arg type="v" direction="out"/></method>
    <method name="ReadOne"><arg type="s" direction="in"/><arg type="s" direction="in"/><arg type="v" direction="out"/></method>
    <signal name="SettingChanged"><arg type="s"/><arg type="s"/><arg type="v"/></signal>
    <property name="version" type="u" access="read"/>
  </interface>
</node>
"""


def main() -> int:
    if "/run/user/" in os.environ.get("DBUS_SESSION_BUS_ADDRESS", "/run/user/"):
        print("settings-portal: nur auf dem Bus der isolierten Probe", file=sys.stderr)
        return 2
    quelle = Gio.SettingsSchemaSource.get_default()
    settings = {s: Gio.Settings.new(s) for s in SCHEMATA if quelle.lookup(s, True)}
    bus = Gio.bus_get_sync(Gio.BusType.SESSION)
    iface = Gio.DBusNodeInfo.new_for_xml(XML).interfaces[0]

    def erscheinung():
        s = settings.get("org.gnome.desktop.interface")
        wahl = s.get_string("color-scheme") if s else "default"
        return GLib.Variant("u", {"prefer-dark": 1, "prefer-light": 2}.get(wahl, 0))

    def wert(ns, key):
        if (ns, key) == ("org.freedesktop.appearance", "color-scheme"):
            return erscheinung()
        s = settings.get(ns)
        if s is None or key not in s.props.settings_schema.list_keys():
            return None
        return s.get_value(key)

    def aufruf(conn, sender, path, iname, method, params, inv):
        if method == "ReadAll":
            muster = params.unpack()[0] or ["*"]
            aus = {}
            for ns, s in settings.items():
                if any(fnmatch.fnmatch(ns, m) for m in muster):
                    aus[ns] = {k: s.get_value(k) for k in s.props.settings_schema.list_keys()}
            if any(fnmatch.fnmatch("org.freedesktop.appearance", m) for m in muster):
                aus["org.freedesktop.appearance"] = {"color-scheme": erscheinung()}
            inv.return_value(GLib.Variant("(a{sa{sv}})", (aus,)))
            return
        ns, key = params.unpack()
        v = wert(ns, key)
        if v is None:
            inv.return_dbus_error("org.freedesktop.portal.Error.NotFound", f"{ns} {key}")
        elif method == "Read":
            inv.return_value(GLib.Variant("(v)", (GLib.Variant("v", v),)))
        else:
            inv.return_value(GLib.Variant("(v)", (v,)))

    def eigenschaft(conn, sender, path, iname, name):
        return GLib.Variant("u", 2)

    bus.register_object("/org/freedesktop/portal/desktop", iface, aufruf, eigenschaft, None)
    def geaendert(s, k, ns):
        bus.emit_signal(None, "/org/freedesktop/portal/desktop", "org.freedesktop.portal.Settings",
                        "SettingChanged", GLib.Variant("(ssv)", (ns, k, s.get_value(k))))
        if (ns, k) == ("org.gnome.desktop.interface", "color-scheme"):
            bus.emit_signal(None, "/org/freedesktop/portal/desktop", "org.freedesktop.portal.Settings",
                            "SettingChanged", GLib.Variant("(ssv)", ("org.freedesktop.appearance", k, erscheinung())))

    for ns, s in settings.items():
        s.connect("changed", lambda s, k, ns=ns: geaendert(s, k, ns))
    Gio.bus_own_name_on_connection(bus, "org.freedesktop.portal.Desktop",
                                   Gio.BusNameOwnerFlags.NONE, None, None)
    GLib.MainLoop().run()
    return 0


if __name__ == "__main__":
    sys.exit(main())
