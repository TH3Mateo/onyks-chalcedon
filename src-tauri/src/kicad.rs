// KiCad setup for accounts that use KiCad: the ONYKS_LIBRARY path variable in kicad_common.json
// and the global library tables (sym-lib-table, fp-lib-table).
//
// The server generates kicad_tables/sym-lib-table and fp-lib-table in the SVN repository (every
// row's uri starts with ${ONYKS_LIBRARY}/). After each update those rows are copied into the
// tables in KiCad's configuration folder. Only rows with that uri prefix are ever touched: every
// other row of the user is kept byte for byte, and nothing here is committed to SVN.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const VARIABLE: &str = "ONYKS_LIBRARY";
const MANAGED_PREFIX: &str = "${ONYKS_LIBRARY}/";
const DATABASE_LIBRARY_NAME: &str = "Onyks Bloodstone";

// ---------------------------------------------------------------------------
// S-expression reader (just enough for KiCad's library tables)
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum Node {
    Atom { text: String, start: usize, end: usize },
    Str { value: String, start: usize, end: usize },
    List { items: Vec<Node>, start: usize, end: usize },
}

impl Node {
    fn span(&self) -> (usize, usize) {
        match self {
            Node::Atom { start, end, .. } | Node::Str { start, end, .. } | Node::List { start, end, .. } => (*start, *end),
        }
    }

    // The value of an atom or of a quoted string.
    fn text(&self) -> Option<&str> {
        match self {
            Node::Atom { text, .. } => Some(text),
            Node::Str { value, .. } => Some(value),
            Node::List { .. } => None,
        }
    }

    fn items(&self) -> Option<&[Node]> {
        match self {
            Node::List { items, .. } => Some(items),
            _ => None,
        }
    }

    fn head(&self) -> Option<&str> {
        self.items()?.first()?.text()
    }
}

fn parse(text: &str) -> Result<Vec<Node>, String> {
    let bytes = text.as_bytes();
    let mut pos = 0;
    let mut stack: Vec<(usize, Vec<Node>)> = Vec::new();
    let mut top: Vec<Node> = Vec::new();

    while pos < bytes.len() {
        match bytes[pos] {
            b' ' | b'\t' | b'\r' | b'\n' => pos += 1,
            b'(' => {
                stack.push((pos, std::mem::take(&mut top)));
                pos += 1;
            }
            b')' => {
                let (start, parent) = stack.pop().ok_or("niezbalansowany nawias ')'")?;
                let items = std::mem::replace(&mut top, parent);
                top.push(Node::List { items, start, end: pos + 1 });
                pos += 1;
            }
            b'"' => {
                let start = pos;
                pos += 1;
                let mut value: Vec<u8> = Vec::new();
                loop {
                    let byte = *bytes.get(pos).ok_or("niezamknięty cudzysłów")?;
                    pos += 1;
                    match byte {
                        b'"' => break,
                        b'\\' => {
                            let escaped = *bytes.get(pos).ok_or("nieznany koniec escape")?;
                            pos += 1;
                            value.push(match escaped {
                                b'n' => b'\n',
                                b'r' => b'\r',
                                b't' => b'\t',
                                other => other,
                            });
                        }
                        other => value.push(other),
                    }
                }
                top.push(Node::Str { value: String::from_utf8_lossy(&value).to_string(), start, end: pos });
            }
            _ => {
                let start = pos;
                while pos < bytes.len() && !matches!(bytes[pos], b' ' | b'\t' | b'\r' | b'\n' | b'(' | b')' | b'"') {
                    pos += 1;
                }
                top.push(Node::Atom { text: text[start..pos].to_string(), start, end: pos });
            }
        }
    }
    if !stack.is_empty() {
        return Err("niezbalansowany nawias '('".to_string());
    }
    Ok(top)
}

fn quoted(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

// ---------------------------------------------------------------------------
// Library tables
// ---------------------------------------------------------------------------

fn row_uri(node: &Node) -> Option<String> {
    if node.head() != Some("lib") {
        return None;
    }
    node.items()?
        .iter()
        .find(|child| child.head() == Some("uri"))?
        .items()?
        .get(1)?
        .text()
        .map(|s| s.to_string())
}

fn is_managed_row(node: &Node) -> bool {
    row_uri(node).map_or(false, |uri| uri.starts_with(MANAGED_PREFIX))
}

fn root_of<'a>(nodes: &'a [Node], head: &str) -> Result<&'a Node, String> {
    match nodes {
        [root] if root.head() == Some(head) => Ok(root),
        _ => Err(format!("to nie jest tablica ({})", head)),
    }
}

// The rows of a table that point into the SVN working copy, as written in the text.
fn managed_rows(text: &str, head: &str) -> Result<Vec<String>, String> {
    let nodes = parse(text)?;
    let root = root_of(&nodes, head)?;
    let items = root.items().unwrap_or(&[]);
    Ok(items
        .iter()
        .filter(|row| is_managed_row(row))
        .map(|row| {
            let (start, end) = row.span();
            text[start..end].to_string()
        })
        .collect())
}

// The table with its managed rows replaced by `rows`. Everything else - the user's own libraries,
// their formatting, other entries such as (version) - stays as it is. `existing` None or blank
// gives a new table.
fn merge_table(existing: Option<&str>, head: &str, rows: &[String]) -> Result<String, String> {
    let mut out = format!("({}\n", head);
    match existing.filter(|text| !text.trim().is_empty()) {
        None => out.push_str("  (version 7)\n"),
        Some(text) => {
            let nodes = parse(text)?;
            let root = root_of(&nodes, head)?;
            for child in &root.items().unwrap_or(&[])[1..] {
                if is_managed_row(child) {
                    continue;
                }
                let (start, end) = child.span();
                out.push_str("  ");
                out.push_str(&text[start..end]);
                out.push('\n');
            }
        }
    }
    for row in rows {
        out.push_str("  ");
        out.push_str(row);
        out.push('\n');
    }
    out.push_str(")\n");
    Ok(out)
}

fn lib_row(name: &str, library_type: &str, uri: &str, descr: &str) -> String {
    format!(
        "(lib (name {})(type {})(uri {})(options \"\")(descr {}))",
        quoted(name),
        quoted(library_type),
        quoted(uri),
        quoted(descr)
    )
}

fn normalize(text: &str) -> String {
    text.replace("\r\n", "\n").trim_end().to_string()
}

fn read_optional(path: &Path) -> Result<Option<String>, String> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(String::from_utf8_lossy(&bytes).trim_start_matches('\u{feff}').to_string())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("Błąd odczytu {}: {}", path.display(), e)),
    }
}

// Writes `content` (a copy of the old file is kept once, as <file>.bak).
fn write_with_backup(path: &Path, content: &str) -> Result<(), String> {
    if path.exists() {
        let mut backup = path.as_os_str().to_owned();
        backup.push(".bak");
        let backup = PathBuf::from(backup);
        if !backup.exists() {
            fs::copy(path, &backup).map_err(|e| format!("Błąd kopii zapasowej {}: {}", backup.display(), e))?;
        }
    }
    crate::write_file_atomically(&path.to_string_lossy(), content.as_bytes())
}

struct TableSync {
    changed: bool,
    rows: usize,
}

// `source`: the generated table from the working copy (None when the repository has none yet -
// the managed rows already in the user's table then stay, except the database one).
// `extra`: managed rows made here (the database library).
fn sync_table(config_file: &Path, head: &str, source: Option<&str>, extra: &[String]) -> Result<TableSync, String> {
    let existing = read_optional(config_file)?;

    let mut rows: Vec<String> = match (source, existing.as_deref()) {
        (Some(text), _) => managed_rows(text, head)?,
        (None, Some(text)) if !text.trim().is_empty() => managed_rows(text, head)?
            .into_iter()
            .filter(|row| !row.contains(".kicad_dbl"))
            .collect(),
        _ => Vec::new(),
    };
    let from_source = rows.len();
    rows.extend(extra.iter().cloned());

    let merged = merge_table(existing.as_deref(), head, &rows)?;
    let unchanged = existing.as_deref().map_or(false, |text| normalize(text) == normalize(&merged));
    if !unchanged {
        write_with_backup(config_file, &merged)?;
    }
    Ok(TableSync { changed: !unchanged, rows: from_source })
}

// ---------------------------------------------------------------------------
// kicad_common.json
// ---------------------------------------------------------------------------

// Sets environment.vars.<name> = value, leaving every other key as it is. Returns whether the
// file changed.
fn set_path_variable(file: &Path, name: &str, value: &str) -> Result<bool, String> {
    let text = read_optional(file)?.ok_or_else(|| format!("Brak pliku {} (uruchom KiCada raz).", file.display()))?;
    let mut json: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("{} nie jest poprawnym JSON-em: {}", file.display(), e))?;

    let root = json.as_object_mut().ok_or("kicad_common.json nie jest obiektem")?;
    let environment = root
        .entry("environment")
        .or_insert_with(|| serde_json::json!({}));
    if !environment.is_object() {
        *environment = serde_json::json!({});
    }
    let vars = environment
        .as_object_mut()
        .unwrap()
        .entry("vars")
        .or_insert(serde_json::Value::Null);
    if !vars.is_object() {
        // KiCad writes "vars": null until a variable is defined.
        *vars = serde_json::json!({});
    }
    let vars = vars.as_object_mut().unwrap();

    if vars.get(name).and_then(|v| v.as_str()) == Some(value) {
        return Ok(false);
    }
    vars.insert(name.to_string(), serde_json::Value::String(value.to_string()));

    let mut output = serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?;
    output.push('\n');
    write_with_backup(file, &output)?;
    Ok(true)
}

// ---------------------------------------------------------------------------
// Environment
// ---------------------------------------------------------------------------

fn config_dir(version: &str) -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library").join("Preferences"));
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")));

    let dir = base.ok_or("Nie można ustalić katalogu konfiguracji.")?.join("kicad").join(version);
    if dir.is_dir() {
        Ok(dir)
    } else {
        Err(format!(
            "Nie znaleziono konfiguracji KiCada {} ({}). Uruchom KiCada raz i zamknij go, a potem spróbuj ponownie.",
            version,
            dir.display()
        ))
    }
}

fn kicad_running() -> bool {
    #[cfg(target_os = "windows")]
    {
        let mut tasklist = Command::new("tasklist");
        tasklist.creation_flags(0x08000000);
        let output = match tasklist.args(["/FO", "CSV", "/NH"]).output() {
            Ok(output) => String::from_utf8_lossy(&output.stdout).to_lowercase(),
            Err(_) => return false,
        };
        ["kicad.exe", "eeschema.exe", "pcbnew.exe"].iter().any(|name| output.contains(&format!("\"{}\"", name)))
    }
    #[cfg(not(target_os = "windows"))]
    {
        ["kicad", "eeschema", "pcbnew"].iter().any(|name| {
            Command::new("pgrep")
                .args(["-x", name])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        })
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KicadSyncResult {
    config_dir: String,
    library: String,
    symbols: usize,
    footprints: usize,
    database: bool,
    changed: bool,
    kicad_running: bool,
    notes: Vec<String>,
}

// `working_dir`: the SVN working copy. `version`: KiCad's configuration folder name (e.g. "9.0").
// `database_file`: name of the .kicad_dbl inside the working copy (added when it exists).
#[tauri::command(async)]
pub fn kicad_sync(working_dir: String, version: String, database_file: String) -> Result<KicadSyncResult, String> {
    let working = working_dir.trim().trim_end_matches(['/', '\\']).to_string();
    if working.is_empty() {
        return Err("Folder biblioteki jest pusty.".to_string());
    }
    let root = Path::new(&working);
    if !root.is_dir() {
        return Err(format!("Folder biblioteki nie istnieje: {}", working));
    }
    let config = config_dir(&version)?;
    let library = working.replace('\\', "/");
    let mut notes = Vec::new();
    let mut changed = false;

    changed |= set_path_variable(&config.join("kicad_common.json"), VARIABLE, &library)?;

    let tables = root.join("kicad_tables");
    let database_present = !database_file.is_empty() && root.join(&database_file).is_file();
    let database_row = lib_row(
        DATABASE_LIBRARY_NAME,
        "Database",
        &format!("{}{}", MANAGED_PREFIX, database_file),
        "ONYKS Bloodstone component database",
    );

    let mut counts = [0usize; 2];
    for (index, (file, head)) in [("sym-lib-table", "sym_lib_table"), ("fp-lib-table", "fp_lib_table")].iter().enumerate() {
        let source = read_optional(&tables.join(file))?;
        if source.is_none() {
            notes.push(format!("Brak kicad_tables/{} w katalogu roboczym (serwer jeszcze go nie wygenerował lub brak Pull).", file));
        }
        // The database library can only be added to the symbol libraries.
        let extra: Vec<String> = if index == 0 && database_present { vec![database_row.clone()] } else { Vec::new() };
        let result = sync_table(&config.join(file), head, source.as_deref(), &extra)?;
        changed |= result.changed;
        counts[index] = result.rows;
    }
    if !database_present {
        notes.push(format!("Brak pliku {} - biblioteka bazodanowa nie została dodana.", database_file));
    }

    let running = kicad_running();
    Ok(KicadSyncResult {
        config_dir: config.to_string_lossy().to_string(),
        library,
        symbols: counts[0],
        footprints: counts[1],
        database: database_present,
        changed,
        kicad_running: running,
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    const SERVER_SYM: &str = "(sym_lib_table\n  (version 7)\n  (lib (name \"Parts\")(type \"Altium\")(uri \"${ONYKS_LIBRARY}/symbols/Parts.SchLib\")(options \"\")(descr \"\"))\n  (lib (name \"Other\")(type \"Altium\")(uri \"${ONYKS_LIBRARY}/symbols/Other.SchLib\")(options \"\")(descr \"\"))\n)\n";

    const USER_SYM: &str = "(sym_lib_table\n  (version 7)\n  (lib (name \"4xxx\")(type \"KiCad\")(uri \"${KICAD9_SYMBOL_DIR}/4xxx.kicad_sym\")(options \"\")(descr \"4xxx series\"))\n  (lib (name Mine)(type Kicad)(uri /home/me/mine.kicad_sym)(options \"\")(descr \"a \\\"quoted\\\" word\"))\n  (lib (name \"Old\")(type \"Altium\")(uri \"${ONYKS_LIBRARY}/symbols/Old.SchLib\")(options \"\")(descr \"\"))\n)\n";

    fn temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("chalcedon-kicad-{label}-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn parses_quotes_escapes_and_bare_atoms() {
        let nodes = parse(USER_SYM).unwrap();
        let root = root_of(&nodes, "sym_lib_table").unwrap();
        let rows: Vec<_> = root.items().unwrap().iter().filter(|n| n.head() == Some("lib")).collect();
        assert_eq!(rows.len(), 3);
        assert_eq!(row_uri(rows[1]).unwrap(), "/home/me/mine.kicad_sym");
        assert!(is_managed_row(rows[2]) && !is_managed_row(rows[0]));
        assert!(parse("(lib (name \"x\"").is_err());
    }

    #[test]
    fn merge_replaces_only_managed_rows() {
        let rows = managed_rows(SERVER_SYM, "sym_lib_table").unwrap();
        assert_eq!(rows.len(), 2);
        let merged = merge_table(Some(USER_SYM), "sym_lib_table", &rows).unwrap();
        // the user's rows are untouched, byte for byte
        assert!(merged.contains("(lib (name \"4xxx\")(type \"KiCad\")(uri \"${KICAD9_SYMBOL_DIR}/4xxx.kicad_sym\")(options \"\")(descr \"4xxx series\"))"));
        assert!(merged.contains("(lib (name Mine)(type Kicad)(uri /home/me/mine.kicad_sym)(options \"\")(descr \"a \\\"quoted\\\" word\"))"));
        // the removed server library is gone, the new ones are in
        assert!(!merged.contains("Old.SchLib"));
        assert!(merged.contains("Parts.SchLib") && merged.contains("Other.SchLib"));
        assert!(merged.contains("(version 7)"));
        // running it again changes nothing
        assert_eq!(merge_table(Some(&merged), "sym_lib_table", &rows).unwrap(), merged);
    }

    #[test]
    fn empty_or_missing_table_gets_a_header() {
        let rows = managed_rows(SERVER_SYM, "sym_lib_table").unwrap();
        let fresh = merge_table(None, "sym_lib_table", &rows).unwrap();
        assert!(fresh.starts_with("(sym_lib_table\n  (version 7)\n"));
        assert_eq!(merge_table(Some("  \n"), "sym_lib_table", &rows).unwrap(), fresh);
        let none = merge_table(None, "fp_lib_table", &[]).unwrap();
        assert_eq!(none, "(fp_lib_table\n  (version 7)\n)\n");
    }

    #[test]
    fn a_foreign_file_is_refused_not_overwritten() {
        assert!(merge_table(Some("(fp_lib_table (version 7))"), "sym_lib_table", &[]).is_err());
        assert!(merge_table(Some("not a table"), "sym_lib_table", &[]).is_err());
    }

    #[test]
    fn sync_table_writes_once_and_keeps_a_backup() {
        let dir = temp_dir("table");
        let file = dir.join("sym-lib-table");
        fs::write(&file, USER_SYM).unwrap();

        let first = sync_table(&file, "sym_lib_table", Some(SERVER_SYM), &[]).unwrap();
        assert!(first.changed && first.rows == 2);
        let backup = dir.join("sym-lib-table.bak");
        assert_eq!(fs::read_to_string(&backup).unwrap(), USER_SYM);

        let written = fs::read_to_string(&file).unwrap();
        let second = sync_table(&file, "sym_lib_table", Some(SERVER_SYM), &[]).unwrap();
        assert!(!second.changed);
        assert_eq!(fs::read_to_string(&file).unwrap(), written);

        // no server table: managed rows stay, the database row is dropped
        let db = lib_row(DATABASE_LIBRARY_NAME, "Database", "${ONYKS_LIBRARY}/onyks_bloodstone.kicad_dbl", "");
        sync_table(&file, "sym_lib_table", Some(SERVER_SYM), &[db]).unwrap();
        assert!(fs::read_to_string(&file).unwrap().contains("(type \"Database\")"));
        sync_table(&file, "sym_lib_table", None, &[]).unwrap();
        let kept = fs::read_to_string(&file).unwrap();
        assert!(!kept.contains("Database") && kept.contains("Parts.SchLib"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn database_row_is_recognised_as_managed() {
        let db = lib_row(DATABASE_LIBRARY_NAME, "Database", "${ONYKS_LIBRARY}/onyks_bloodstone.kicad_dbl", "x");
        assert!(db.contains("(type \"Database\")"));
        let table = merge_table(Some(USER_SYM), "sym_lib_table", &[db.clone()]).unwrap();
        let again = merge_table(Some(&table), "sym_lib_table", &[db]).unwrap();
        assert_eq!(table, again);
        assert_eq!(table.matches("onyks_bloodstone.kicad_dbl").count(), 1);
    }

    #[test]
    fn common_json_keeps_other_keys() {
        let dir = temp_dir("json");
        let file = dir.join("kicad_common.json");
        fs::write(&file, "{\n  \"environment\": {\n    \"vars\": null\n  },\n  \"git\": {\n    \"enableGit\": true\n  },\n  \"zzz\": [1, 2]\n}\n").unwrap();

        assert!(set_path_variable(&file, "ONYKS_LIBRARY", "D:/part_library").unwrap());
        let json: serde_json::Value = serde_json::from_str(&fs::read_to_string(&file).unwrap()).unwrap();
        assert_eq!(json["environment"]["vars"]["ONYKS_LIBRARY"], "D:/part_library");
        assert_eq!(json["git"]["enableGit"], true);
        assert_eq!(json["zzz"], serde_json::json!([1, 2]));
        assert!(dir.join("kicad_common.json.bak").exists());

        // same value: nothing is written; another variable already there survives a change
        assert!(!set_path_variable(&file, "ONYKS_LIBRARY", "D:/part_library").unwrap());
        let mut json = json;
        json["environment"]["vars"]["OTHER"] = serde_json::json!("keep");
        fs::write(&file, serde_json::to_string(&json).unwrap()).unwrap();
        assert!(set_path_variable(&file, "ONYKS_LIBRARY", "E:/lib").unwrap());
        let json: serde_json::Value = serde_json::from_str(&fs::read_to_string(&file).unwrap()).unwrap();
        assert_eq!(json["environment"]["vars"]["OTHER"], "keep");
        assert_eq!(json["environment"]["vars"]["ONYKS_LIBRARY"], "E:/lib");

        assert!(set_path_variable(&dir.join("missing.json"), "X", "y").is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    // The tables of a real KiCad installation (if there is one) survive a merge unchanged.
    #[test]
    fn real_user_tables_are_preserved() {
        let dir = match config_dir("9.0") {
            Ok(dir) => dir,
            Err(_) => return,
        };
        for (file, head) in [("sym-lib-table", "sym_lib_table"), ("fp-lib-table", "fp_lib_table")] {
            let text = match fs::read_to_string(dir.join(file)) {
                Ok(text) if !text.contains("${ONYKS_LIBRARY}") => text,
                _ => continue,
            };
            let merged = merge_table(Some(&text), head, &[]).unwrap();
            let count = |t: &str| parse(t).unwrap()[0].items().unwrap().iter().filter(|n| n.head() == Some("lib")).count();
            assert_eq!(count(&text), count(&merged), "{}", file);
            // every user row is still there, verbatim
            for node in parse(&text).unwrap()[0].items().unwrap().iter().filter(|n| n.head() == Some("lib")) {
                let (start, end) = node.span();
                assert!(merged.contains(&text[start..end]), "{}: {}", file, &text[start..end]);
            }
        }
    }
}
