# PT-1 (18.10.2026): Portal-Falle QD-037

`isoliert.sh` wurde geändert (kein Git hier). Original: `isoliert.sh.vor-pt1`.

- Portaldienste im Wegwerf-Bus nicht aktivierbar (D-Bus-Dienstdateien mit Exec=/bin/false)
- GDK_DEBUG=no-portals, GTK_USE_PORTAL=0
- am Ende nur den eigenen `<laufzeit>/doc` aushängen (Prüfung über /proc/self/mounts)
- Schrift der echten Sitzung (gsettings font-name, nur gelesen) als gtk-4.0/settings.ini ins Testprofil, außer es gibt eine eigene

Rückfall: `cp -p isoliert.sh.vor-pt1 isoliert.sh`. Diff und Prüfläufe: 
