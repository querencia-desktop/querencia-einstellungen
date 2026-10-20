//! Übersetzungen über GNU gettext (Textdomäne `querencia-einstellungen`),
//! nach `docs/i18n-muster.md` des Editors (G2-AC).
//!
//! Quellsprache im Code ist Englisch (msgid englisch, wie bei GNOME und
//! Debian); Deutsch kommt aus `po/de.po`. Ohne passenden Katalog zeigt das
//! Programm die englischen Texte. Die Sprache wählt gettext wie üblich über
//! `LANGUAGE`, `LC_ALL`, `LC_MESSAGES` und `LANG`. `po/en.po` liefert einen
//! englischen Katalog, damit `LANGUAGE=en:de` englisch bleibt (ohne `en.mo`
//! fiele gettext dort auf den deutschen Katalog zurück).
//!
//! Die Kataloge (`<lang>/LC_MESSAGES/querencia-einstellungen.mo`) sucht das
//! Programm zuerst neben der Binary (`<ablage>/share/locale`, unveränderliche
//! Ablage unter `~/opt/querencia/einstellungen/<sha16>/`), dann im Präfix der
//! Binary (`/usr/bin/../share/locale`), zuletzt in `/usr/share/locale`.
//! `QUERENCIA_EINSTELLUNGEN_LOCALEDIR` setzt das Verzeichnis ausdrücklich
//! (Tests, GUI-Prüfung).
//!
//! Texte mit Platzhaltern stehen als ein Satz im Katalog („Saved as {name}.“),
//! nie zusammengesetzt; [`gettext_f`] und [`ngettext_f`] setzen die Werte erst
//! nach der Übersetzung ein. Feste Tabellen (Konstanten) markiert
//! [`gettext_noop`] für die Extraktion; übersetzt wird beim Anzeigen mit
//! [`gettext`] auf den Tabellenwert.

use std::path::{Path, PathBuf};

use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};
pub use gettextrs::{gettext, ngettext, pgettext};

/// Textdomäne = Name der .mo-Dateien.
pub const DOMAIN: &str = "querencia-einstellungen";

/// Textdomäne binden; vor `Application::run` aufrufen. Die Locale aus der
/// Umgebung setzt GTK selbst beim Start (`gtk_init` ruft `setlocale(LC_ALL,
/// "")`); alle Oberflächentexte entstehen erst danach (startup/activate).
/// Die Textausgaben ohne Fenster (`--labwc-zeigen` usw.) sind Prüfausgaben
/// und bleiben unübersetzt.
pub fn init() {
    let dir = locale_dir();
    if let Err(err) = bindtextdomain(DOMAIN, dir.as_path()) {
        eprintln!(
            "querencia-einstellungen: Übersetzungen nicht gebunden ({}): {err}",
            dir.display()
        );
    }
    let _ = bind_textdomain_codeset(DOMAIN, "UTF-8");
    let _ = textdomain(DOMAIN);
}

/// Verzeichnis der Kataloge (siehe Modulbeschreibung).
pub fn locale_dir() -> PathBuf {
    if let Some(dir) =
        std::env::var_os("QUERENCIA_EINSTELLUNGEN_LOCALEDIR").filter(|d| !d.is_empty())
    {
        return PathBuf::from(dir);
    }
    // current_exe() löst den Link ~/opt/querencia/bin/querencia-einstellungen auf.
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.canonicalize().ok())
        .and_then(|exe| exe.parent().map(Path::to_path_buf));
    locale_dir_for(exe_dir.as_deref())
}

fn locale_dir_for(exe_dir: Option<&Path>) -> PathBuf {
    if let Some(exe_dir) = exe_dir {
        let beside = exe_dir.join("share/locale");
        if beside.is_dir() {
            return beside;
        }
        if let Some(prefix) = exe_dir.parent() {
            let prefixed = prefix.join("share/locale");
            if prefixed.is_dir() {
                return prefixed;
            }
        }
    }
    PathBuf::from("/usr/share/locale")
}

/// Sprachen, für die ein Katalog dieses Programms vorliegt (z. B. `de`,
/// `en`), sortiert. Englisch ist als Quellsprache immer dabei.
pub fn kataloge() -> Vec<String> {
    kataloge_in(&locale_dir())
}

fn kataloge_in(dir: &Path) -> Vec<String> {
    let mut l: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            e.path()
                .join("LC_MESSAGES")
                .join(format!("{DOMAIN}.mo"))
                .is_file()
        })
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    if !l.iter().any(|s| s == "en") {
        l.push("en".into());
    }
    l.sort();
    l
}

/// Markiert ein Literal für die Extraktion, ohne es zu übersetzen (für
/// Tabellen in Konstanten; wie `N_()` in C).
pub const fn gettext_noop(msgid: &str) -> &str {
    msgid
}

/// Übersetzt `msgid` und setzt danach benannte Platzhalter `{name}` ein.
pub fn gettext_f(msgid: &str, args: &[(&str, &str)]) -> String {
    fill(&gettext(msgid), args)
}

/// Wie [`gettext_f`] mit Pluralform für `n`; `{n}` steht automatisch bereit.
pub fn ngettext_f(msgid: &str, msgid_plural: &str, n: u32, args: &[(&str, &str)]) -> String {
    let count = n.to_string();
    let mut all = vec![("n", count.as_str())];
    all.extend_from_slice(args);
    fill(&ngettext(msgid, msgid_plural, n), &all)
}

/// Ersetzt `{schlüssel}` in einem Durchgang; eingesetzte Werte werden nicht
/// erneut durchsucht, unbekannte Platzhalter bleiben stehen.
pub fn fill(template: &str, args: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let value = after.find('}').and_then(|close| {
            let key = &after[..close];
            args.iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| (*v, close))
        });
        match value {
            Some((v, close)) => {
                out.push_str(v);
                rest = &after[close + 1..];
            }
            None => {
                out.push('{');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platzhalter_einmal_ersetzt() {
        assert_eq!(
            fill("Saved as {name}.", &[("name", "a.txt")]),
            "Saved as a.txt."
        );
        // Ein Wert mit Klammern wird nicht noch einmal ersetzt.
        assert_eq!(
            fill("{a} und {b}", &[("a", "{b}"), ("b", "x")]),
            "{b} und x"
        );
        assert_eq!(fill("{unbekannt} {", &[("a", "x")]), "{unbekannt} {");
    }

    #[test]
    fn ohne_katalog_gilt_die_englische_msgid() {
        let n = |k| {
            ngettext_f(
                "Without confirmation the previous settings are restored in {n} second.",
                "Without confirmation the previous settings are restored in {n} seconds.",
                k,
                &[],
            )
        };
        assert_eq!(
            n(3),
            "Without confirmation the previous settings are restored in 3 seconds."
        );
        assert_eq!(
            n(1),
            "Without confirmation the previous settings are restored in 1 second."
        );
    }

    /// Blöcke einer .po/.pot als (Kontext+msgid, Block).
    fn bloecke(text: &str) -> Vec<(String, String)> {
        text.split("\n\n")
            .filter_map(|block| {
                let id: Vec<&str> = block
                    .lines()
                    .skip_while(|l| !l.starts_with("msgid "))
                    .take_while(|l| !l.starts_with("msgstr") && !l.starts_with("msgid_plural"))
                    .collect();
                let ctxt = block.lines().find(|l| l.starts_with("msgctxt "));
                (!id.is_empty() && id != ["msgid \"\""])
                    .then(|| (format!("{ctxt:?}{}", id.join("")), block.to_owned()))
            })
            .collect()
    }

    fn katalog_vollstaendig(name: &str, po: &str) {
        let pot = include_str!("../po/querencia-einstellungen.pot");
        assert!(!po.contains("#, fuzzy"), "{name} enthält fuzzy-Einträge");
        let pot_ids: Vec<String> = bloecke(pot).into_iter().map(|(id, _)| id).collect();
        let po_bloecke = bloecke(po);
        let po_ids: Vec<&String> = po_bloecke.iter().map(|(id, _)| id).collect();
        assert!(pot_ids.len() > 400, "Vorlage zu klein: {}", pot_ids.len());
        for id in &pot_ids {
            assert!(po_ids.contains(&id), "fehlt in {name}: {id}");
        }
        for (_, block) in &po_bloecke {
            if block.lines().all(|l| l.starts_with("#~") || l.is_empty()) {
                continue;
            }
            let leer = block
                .lines()
                .any(|l| l.starts_with("msgstr") && l.ends_with(" \"\""))
                && !block
                    .lines()
                    .skip_while(|l| !l.starts_with("msgstr"))
                    .any(|l| l.starts_with('"'));
            assert!(!leer, "unübersetzt in {name}:\n{block}");
        }
    }

    /// Deutsch hängt an `po/de.po`: jede msgid der Vorlage muss dort
    /// übersetzt sein (sonst sähe die deutsche Sitzung Englisch).
    #[test]
    fn deutscher_katalog_vollstaendig() {
        katalog_vollstaendig("de.po", include_str!("../po/de.po"));
    }

    #[test]
    fn englischer_katalog_vollstaendig() {
        katalog_vollstaendig("en.po", include_str!("../po/en.po"));
    }

    #[test]
    fn kataloge_neben_der_binary_dann_im_praefix() {
        let tmp = std::env::temp_dir().join(format!("qe-i18n-{}", std::process::id()));
        let ablage = tmp.join("ablage");
        std::fs::create_dir_all(ablage.join("share/locale/de/LC_MESSAGES")).unwrap();
        std::fs::write(
            ablage.join("share/locale/de/LC_MESSAGES/querencia-einstellungen.mo"),
            b"",
        )
        .unwrap();
        assert_eq!(locale_dir_for(Some(&ablage)), ablage.join("share/locale"));
        assert_eq!(kataloge_in(&ablage.join("share/locale")), ["de", "en"]);
        let bin = tmp.join("usr/bin");
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::create_dir_all(tmp.join("usr/share/locale")).unwrap();
        assert_eq!(locale_dir_for(Some(&bin)), tmp.join("usr/share/locale"));
        let leer = tmp.join("leer/bin");
        std::fs::create_dir_all(&leer).unwrap();
        assert_eq!(
            locale_dir_for(Some(&leer)),
            PathBuf::from("/usr/share/locale")
        );
        assert_eq!(locale_dir_for(None), PathBuf::from("/usr/share/locale"));
        std::fs::remove_dir_all(&tmp).unwrap();
    }
}
