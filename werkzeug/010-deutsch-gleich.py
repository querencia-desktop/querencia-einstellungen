#!/usr/bin/python3 -I
"""Einstellungen 0.1.0: Deutsch byte-gleich (Muster §8.3).

Sucht die deutschen Literale im Quelltext von 0.0.9 (Zweig main, ohne Tests,
ohne Protokollzeilen) und meldet die, die nicht als msgstr in po/de.po stehen
(Platzhalter vereinheitlicht). Was übrig bleibt, ist von Hand zu begründen
(Protokoll, Dateiinhalt, zu ganzen Sätzen zusammengezogene Bruchstücke).

  python3 -I werkzeug/010-deutsch-gleich.py   (im Worktree)
"""
import os
import re
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import i18n  # noqa: E402
root = "crates/querencia-einstellungen"
_, de = i18n.read_po(root + "/po/de.po")
ph = lambda s: re.sub(r"\{[a-z_0-9]*(:[^}]*)?\}", "{}", s)
strs = set()
for e in de:
    for s in ([e["str"]] if e.get("plural") is None else e["strs"]):
        strs.add(ph(s))
dateien = subprocess.run(["git", "ls-tree", "--name-only", "main", root + "/src/"], capture_output=True, text=True).stdout.split()
offen = []
gesamt = 0
for d in dateien:
    t = subprocess.run(["git", "show", "main:" + d], capture_output=True, text=True).stdout
    t = t.split("#[cfg(test)]")[0]
    pos = 0
    while True:
        j = t.find('"', pos)
        if j < 0: break
        # Zeichenliterale '"' überspringen
        if t[j-1:j] == "'" and t[j+1:j+2] == "'":
            pos = j + 2; continue
        if t[j-2:j] == "b'" or t[j-1:j] == "\\":
            pos = j + 1; continue
        zs = t.rfind("\n", 0, j) + 1
        zeile = t[zs:t.find("\n", j)]
        try:
            s, ende = i18n.rust_string(t, j)
        except Exception:
            pos = j + 1; continue
        pos = ende
        if zeile.lstrip().startswith("//"):
            continue
        ctx = t[max(0, j-200):j]
        if re.search(r'(eprintln|println|print)!\s*\(\s*$', ctx) or "querencia-einstellungen:" in s:
            continue
        if not re.search(r"[äöüßÄÖÜ„“…–]|[A-ZÄÖÜ][a-zäöüß]+ [a-zäöüß]", s):
            continue
        gesamt += 1
        if ph(s) not in strs:
            offen.append((d.split("/")[-1], s))
print("deutsche Literale in 0.0.9:", gesamt, "– ohne gleichlautenden msgstr:", len(offen))
for d, s in offen:
    print(f"  {d}: {s!r}")
