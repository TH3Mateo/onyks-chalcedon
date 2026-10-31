// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind, Debouncer};
use tauri::{Emitter, Manager};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Files that must never be versioned: the DbLib is rewritten locally by this program
// (and by Altium) from the database, so versioning it only produces conflicts; `.tmp`
// are our own atomic-write leftovers. Passed to every svn call (not stored in the
// repository), so `svn add` / `svn status` skip them like a permanent .gitignore.
const SVN_IGNORED: &str = "*.DbLib *.dblib *.tmp Thumbs.db desktop.ini";

fn svn_command() -> Command {
    let mut command = Command::new("svn");
    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000);
    command.arg(format!(
        "--config-option=config:miscellany:global-ignores={}",
        SVN_IGNORED
    ));
    command
}

fn svn_stdout(mut command: Command) -> Result<String, String> {
    let output = command
        .output()
        .map_err(|e| format!("Błąd wykonania komendy: {}", e))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn same_url(a: &str, b: &str) -> bool {
    a.trim().trim_end_matches('/').eq_ignore_ascii_case(b.trim().trim_end_matches('/'))
}

// The working copy remembers the server URL in `.svn`. When the server's address changes
// (another IP after switching between the LAN and a VPN, a new hostname...) every svn call
// fails and the copy looks broken. If the configured address differs from the stored one,
// `svn relocate` repoints the copy (it verifies the repository UUID, so it cannot attach
// the copy to a different repository).
fn svn_ensure_url(svn_folder_path: &str, url: &str, login: &str, password: &str) -> Result<(), String> {
    if url.trim().is_empty() {
        return Ok(());
    }
    let mut info = svn_command();
    info.current_dir(svn_folder_path)
        .args(["info", "--show-item", "url", "--no-newline"]);
    let current = svn_stdout(info)?;
    if same_url(&current, url) {
        return Ok(());
    }

    let mut relocate = svn_command();
    relocate
        .current_dir(svn_folder_path)
        .args([
            "relocate",
            url.trim(),
            "--username",
            login,
            "--password",
            password,
            "--trust-server-cert",
            "--non-interactive",
            "--no-auth-cache",
        ]);
    svn_stdout(relocate).map(|_| ()).map_err(|e| {
        format!(
            "Adres repozytorium zmienił się z {} na {}, ale nie udało się przepiąć kopii roboczej: {}",
            current,
            url.trim(),
            e
        )
    })
}

#[tauri::command(async)]
fn is_svn_repository(svn_folder_path: &str) -> bool {
    let path = Path::new(svn_folder_path).join(".svn");
    path.is_dir()
}

#[tauri::command(async)]
fn svn_status(svn_folder_path: &str) -> Result<Vec<String>, String> {
    let mut command = svn_command();
    let output = command
        .current_dir(svn_folder_path)
        .arg("status")
        .output()
        .map_err(|e| format!("Błąd uruchomienia: {}", e))?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut pliki = Vec::new();

        for linia in stdout.lines() {
            let czesci: Vec<&str> = linia.split_whitespace().collect();

            if czesci.len() >= 2 {
                if let Some(sciezka) = czesci.last() {
                    pliki.push(sciezka.to_string());
                }
            }
        }
        Ok(pliki)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command(async)]
fn svn_checkout(
    svn_folder_path: &str,
    login: &str,
    password: &str,
    url: &str,
) -> Result<Vec<String>, String> {
    let mut command = svn_command();
    let output = command
        .current_dir(svn_folder_path)
        .args([
            "checkout",
            url,
            svn_folder_path,
            "--username",
            login,
            "--password",
            password,
            "--trust-server-cert",
            "--non-interactive",
            "--no-auth-cache",
        ])
        .output()
        .map_err(|e| format!("Błąd wykonania komendy: {}", e))?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut pliki = Vec::new();

        for linia in stdout.lines() {
            let czesci: Vec<&str> = linia.split_whitespace().collect();
            if czesci.len() >= 2 {
                let sciezka = czesci[1..].join(" ");
                pliki.push(sciezka);
            }
        }

        Ok(pliki)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command(async)]
fn svn_add_all(svn_folder_path: &str) -> Result<Vec<String>, String> {
    let mut command = svn_command();
    let output = command
        .current_dir(svn_folder_path)
        .args(["add", "--force", "."])
        .output()
        .map_err(|e| format!("Błąd wykonania komendy: {}", e))?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut pliki = Vec::new();

        for linia in stdout.lines() {
            let czesci: Vec<&str> = linia.split_whitespace().collect();
            if czesci.len() >= 2 {
                let sciezka = czesci[1..].join(" ");

                pliki.push(sciezka);
            }
        }
        Ok(pliki)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command(async)]
fn svn_commit(
    svn_folder_path: &str,
    login: &str,
    password: &str,
    commit_name: &str,
    url: Option<&str>,
) -> Result<String, String> {
    svn_ensure_url(svn_folder_path, url.unwrap_or(""), login, password)?;
    let mut command = svn_command();
    let output = command
        .current_dir(svn_folder_path)
        .args([
            "commit",
            "-m",
            commit_name,
            "--username",
            login,
            "--password",
            password,
            "--trust-server-cert",
            "--non-interactive",
            "--no-auth-cache",
        ])
        .output()
        .map_err(|e| format!("Błąd wykonania komendy: {}", e))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command(async)]
fn svn_update(
    svn_folder_path: &str,
    login: &str,
    password: &str,
    url: Option<&str>,
) -> Result<String, String> {
    svn_ensure_url(svn_folder_path, url.unwrap_or(""), login, password)?;
    let mut command = svn_command();
    let output = command
        .current_dir(svn_folder_path)
        .args([
            "update",
            "--username",
            login,
            "--password",
            password,
            "--trust-server-cert",
            "--non-interactive",
            "--no-auth-cache"
        ])
        .output()
        .map_err(|e| format!("Błąd wykonania komendy: {}", e))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command(async)]
fn svn_revert(svn_folder_path: &str) -> Result<String, String> {
    let mut command = svn_command();

    let output = command
        .current_dir(svn_folder_path)
        .args([
            "revert", 
            "-R", 
            "."
        ])
        .output()
        .map_err(|e| format!("Błąd wykonania komendy: {}", e))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command(async)]
fn svn_cleanup(svn_folder_path: &str) -> Result<String, String> {
    let mut command = svn_command();

    let output = command
        .current_dir(svn_folder_path)
        .args([
            "cleanup",
            "--remove-unversioned",
            ])
        .output()
        .map_err(|e| format!("Błąd wykonania komendy: {}", e))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}


#[tauri::command(async)]
fn svn_delete(svn_folder_path: &str) -> Result<String, String> {
    let mut status_command = svn_command();

    let status_output = status_command
        .current_dir(svn_folder_path)
        .arg("status")
        .output()
        .map_err(|e| format!("Błąd uruchomienia: {}", e))?;

    if !status_output.status.success() {
        return Err(String::from_utf8_lossy(&status_output.stderr).to_string());
    }

    let mut deleted_files = Vec::new();
    for line in String::from_utf8_lossy(&status_output.stdout).lines() {
        if let Some(path) = line.strip_prefix('!').map(str::trim_start) {
            let mut delete_command = svn_command();

            let output = delete_command
                .current_dir(svn_folder_path)
                .args(["rm", path])
                .output()
                .map_err(|e| format!("Błąd wykonania komendy: {}", e))?;

            if !output.status.success() {
                return Err(String::from_utf8_lossy(&output.stderr).to_string());
            }

            deleted_files.push(path.to_string());
        }
    }

    Ok(deleted_files.join("\n"))
}

// Watches a folder (e.g. the SVN repository path) and all its subfolders for changes,
// so the frontend can auto-push on local edits instead of only polling on a timer.
// `.svn` internals and `.tmp` files (our own atomic-write leftovers) are filtered out
// so SVN's own metadata churn during update/commit doesn't retrigger the watch. The
// frontend also stops the watcher around its own push/pull calls (see
// useRepositorySync.js) to avoid feedback loops.
struct WatcherState(Mutex<Option<Debouncer<RecommendedWatcher>>>);

// SVN rewrites its own metadata (`.svn/**`) on every update/commit, and our atomic
// writer leaves `.tmp` files briefly (see write_file_atomically) - neither should
// count as a user edit worth auto-pushing.
fn is_relevant_change(path: &str) -> bool {
    !path.contains(".svn") && !path.ends_with(".tmp")
}

#[tauri::command(async)]
fn start_watch(
    app: tauri::AppHandle,
    state: tauri::State<WatcherState>,
    path: String,
) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    *guard = None;

    let mut debouncer = new_debouncer(Duration::from_secs(3), move |result: notify_debouncer_mini::DebounceEventResult| {
        let events = match result {
            Ok(events) => events,
            Err(_) => return,
        };
        let relevant = events.iter().any(|event| {
            event.kind != DebouncedEventKind::AnyContinuous
                && is_relevant_change(&event.path.to_string_lossy())
        });
        if relevant {
            let _ = app.emit("repository-changed", ());
        }
    })
    .map_err(|e| e.to_string())?;

    debouncer
        .watcher()
        .watch(Path::new(&path), RecursiveMode::Recursive)
        .map_err(|e| e.to_string())?;

    *guard = Some(debouncer);
    Ok(())
}

#[tauri::command(async)]
fn stop_watch(state: tauri::State<WatcherState>) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    *guard = None;
    Ok(())
}

#[tauri::command(async)]
fn DbLib_ConnectionString(user: &str, source: &str) -> String {
    format!(
        "ConnectionString=Provider=MSDASQL.1;Persist Security Info=False;User ID={};Data Source={}",
        source, user
    )
}

#[tauri::command(async)]
fn DbLib_table(id: u32, name: &str, enabled: bool) -> String {
    
    let enabled_str = if enabled { "True" } else { "False" };
    format!(
        "[Table{id}]\n\
        SchemaName=\n\
        TableName={name}\n\
        Enabled={enabled_str}\n\
        UserWhere=0\n\
        UserWhereText="
    )
}

#[tauri::command(async)]
fn DbLib_FieldMap(index: u32, tableName: &str, fieldName: &str, 
    fieldType: u32, parameterName: &str, visibleOnAdd: bool, 
    addMode: u32, removeMode: u32, updateMode: u32) -> String {
    
    let visible_str = if visibleOnAdd { "True" } else { "False" };
    format!(
        "[FieldMap{index}]\n\
        Options=FieldName={tableName}.{fieldName}|\
        TableNameOnly={tableName}|\
        FieldNameOnly={fieldName}|\
        FieldType={fieldType}|\
        ParameterName={parameterName}|\
        VisibleOnAdd={visible_str}|\
        AddMode={addMode}|\
        RemoveMode={removeMode}|\
        UpdateMode={updateMode}",
    )
}

#[tauri::command(async)]
fn read_text_file(path: &str) -> Result<Option<String>, String> {
    // Lossy, because a file saved by another program is not necessarily UTF-8.
    match fs::read(path) {
        Ok(content) => Ok(Some(String::from_utf8_lossy(&content).into_owned())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("Błąd odczytu pliku: {}", e)),
    }
}

// Written to a temporary file first and then renamed, so a program reading the file
// at the same moment (e.g. Altium with the DbLib) never sees it half-written.
fn write_file_atomically(path: &str, data: &[u8]) -> Result<(), String> {
    let target = Path::new(path);
    if let Some(parent) = target.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| format!("Błąd tworzenia folderu: {}", e))?;
        }
    }
    let mut temporary = target.as_os_str().to_owned();
    temporary.push(".tmp");
    fs::write(&temporary, data).map_err(|e| format!("Błąd zapisu pliku: {}", e))?;
    if fs::rename(&temporary, target).is_err() {
        // Windows refuses to replace a file another program holds open; writing into
        // it directly may still be allowed.
        let _ = fs::remove_file(&temporary);
        fs::write(target, data).map_err(|e| format!("Błąd zapisu pliku: {}", e))?;
    }
    Ok(())
}

// One entry of a folder listing for the Repository view: `type` is "dir" or "file".
#[derive(serde::Serialize)]
struct DirEntry {
    name: String,
    r#type: &'static str,
}

// Folders first, then files, both alphabetical; the `.svn` metadata folder is hidden.
#[tauri::command(async)]
fn list_directory(path: &str) -> Result<Vec<DirEntry>, String> {
    let mut entries: Vec<DirEntry> = fs::read_dir(path)
        .map_err(|e| format!("Błąd odczytu folderu: {}", e))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name == ".svn" {
                return None;
            }
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            Some(DirEntry { name, r#type: if is_dir { "dir" } else { "file" } })
        })
        .collect();
    entries.sort_by(|a, b| {
        (a.r#type != "dir", a.name.to_lowercase()).cmp(&(b.r#type != "dir", b.name.to_lowercase()))
    });
    Ok(entries)
}

#[tauri::command(async)]
fn write_text_file(path: &str, content: &str) -> Result<(), String> {
    write_file_atomically(path, content.as_bytes())
}

#[tauri::command(async)]
fn write_binary_file(path: &str, data: Vec<u8>) -> Result<(), String> {
    write_file_atomically(path, &data)
}

// ---------------------------------------------------------------------------
// Tailscale
// ---------------------------------------------------------------------------

// `tailscale` from PATH first, then the default install location.
fn tailscale_candidates() -> Vec<String> {
    let mut candidates = vec!["tailscale".to_string()];
    for variable in ["ProgramFiles", "ProgramW6432"] {
        if let Ok(dir) = std::env::var(variable) {
            candidates.push(format!("{}\\Tailscale\\tailscale.exe", dir));
        }
    }
    candidates
}

fn tailscale_command(program: &str) -> Command {
    let mut command = Command::new(program);
    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000);
    command
}

fn find_tailscale_cli() -> Option<String> {
    tailscale_candidates().into_iter().find(|program| {
        tailscale_command(program)
            .arg("version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    })
}

// Starts the Tailscale tray client (tailscale-ipn.exe) unless it already runs.
// Best effort: the background service works without it.
fn start_tailscale_client() {
    let Ok(dir) = std::env::var("ProgramFiles") else {
        return;
    };
    let client = format!("{}\\Tailscale\\tailscale-ipn.exe", dir);
    if !Path::new(&client).is_file() {
        return;
    }

    let mut tasklist = Command::new("tasklist");
    #[cfg(target_os = "windows")]
    tasklist.creation_flags(0x08000000);
    let running = tasklist
        .args(["/FI", "IMAGENAME eq tailscale-ipn.exe", "/NH"])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).contains("tailscale-ipn.exe"))
        .unwrap_or(false);

    if !running {
        let _ = Command::new(client).spawn();
    }
}

// The auth key must never end up in an error shown in the UI or in logs.
fn redact(text: &str, secret: &str) -> String {
    if secret.is_empty() {
        text.to_string()
    } else {
        text.replace(secret, "***")
    }
}

fn tailscale_backend_state(cli: &str) -> Option<String> {
    let output = tailscale_command(cli).args(["status", "--json"]).output().ok()?;
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    json.get("BackendState")?.as_str().map(|s| s.to_string())
}

// Starts the Tailscale client and logs in with the auth key stored in the given
// text file (`tailscale login --auth-key=<key>`). Returns a short status message.
#[tauri::command(async)]
fn tailscale_connect(auth_key_path: &str) -> Result<String, String> {
    if auth_key_path.trim().is_empty() {
        return Err("Nie wybrano pliku z kluczem Tailscale.".to_string());
    }
    let content = fs::read(auth_key_path)
        .map_err(|e| format!("Nie można odczytać pliku z kluczem Tailscale: {}", e))?;
    let key = String::from_utf8_lossy(&content)
        .trim_start_matches('\u{feff}')
        .trim()
        .to_string();
    if key.is_empty() {
        return Err("Plik z kluczem Tailscale jest pusty.".to_string());
    }
    if key.lines().count() > 1 {
        return Err("Plik z kluczem Tailscale powinien zawierać tylko jeden klucz.".to_string());
    }

    let cli = find_tailscale_cli().ok_or_else(|| {
        "Nie znaleziono programu Tailscale. Zainstaluj go (jest dołączony do instalatora)."
            .to_string()
    })?;

    // The client (tray app) is started only AFTER the login: launched while logged
    // out it opens the browser sign-in page by itself. The background service that
    // `tailscale login` talks to does not need the client.
    // The service may still be starting (e.g. right after boot).
    let mut state = None;
    for _ in 0..10 {
        state = tailscale_backend_state(&cli);
        if state.is_some() {
            break;
        }
        std::thread::sleep(Duration::from_secs(1));
    }

    if state.as_deref() == Some("Running") {
        start_tailscale_client();
        return Ok("Tailscale jest już połączony.".to_string());
    }

    let login = || {
        tailscale_command(&cli)
            .args(["login", "--timeout=60s"])
            .arg(format!("--auth-key={}", key))
            .output()
            .map_err(|e| format!("Błąd uruchomienia tailscale: {}", e))
    };

    let mut output = login()?;

    // On Linux the daemon only accepts commands from root until an "operator" is set.
    // `tailscale up --operator=$USER --auth-key=...` as root logs in and sets the operator
    // in one step (graphical password prompt through polkit).
    #[cfg(target_os = "linux")]
    if !output.status.success() && tailscale_needs_operator(&output) {
        output = tailscale_up_as_operator(&cli, &key)?;
    }

    if output.status.success() {
        start_tailscale_client();
        Ok("Zalogowano do Tailscale.".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let message = if stderr.trim().is_empty() { stdout } else { stderr };
        Err(format!("tailscale login: {}", redact(message.trim(), &key)))
    }
}

#[cfg(target_os = "linux")]
fn tailscale_needs_operator(output: &std::process::Output) -> bool {
    let text = format!(
        "{} {}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    )
    .to_lowercase();
    text.contains("access denied") || text.contains("--operator")
}

// `sudo tailscale up --operator=$USER --auth-key=<key>` through polkit, so it works from the app.
#[cfg(target_os = "linux")]
fn tailscale_up_as_operator(cli: &str, key: &str) -> Result<std::process::Output, String> {
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_default();
    let valid = !user.is_empty()
        && user != "root"
        && user.chars().all(|c| c.is_ascii_alphanumeric() || "._-".contains(c));
    if !valid {
        return Err("Tailscale odmówił dostępu. Wykonaj w terminalu: sudo tailscale up --operator=$USER --auth-key=<klucz>".to_string());
    }

    Command::new("pkexec")
        .arg(cli)
        .arg("up")
        .arg("--timeout=60s")
        .arg(format!("--operator={}", user))
        .arg(format!("--auth-key={}", key))
        .output()
        .map_err(|e| {
            format!(
                "Tailscale odmówił dostępu, a pkexec nie działa ({}). Wykonaj w terminalu: sudo tailscale up --operator=$USER --auth-key=<klucz>",
                e
            )
        })
}

// ---------------------------------------------------------------------------
// Server discovery
// ---------------------------------------------------------------------------

// Tailscale (and similar VPNs) use 100.64.0.0/10; scanning such a range blindly is
// pointless, its peers are listed from `tailscale status` instead.
fn is_vpn_range(ip: &std::net::Ipv4Addr) -> bool {
    let o = ip.octets();
    o[0] == 100 && (64..128).contains(&o[1])
}

// Hosts of the /24 network of every local IPv4 address (LAN, Wi-Fi, VPN adapters),
// without this computer's own addresses.
fn local_subnet_hosts() -> Vec<std::net::Ipv4Addr> {
    let mut own = Vec::new();
    let mut networks = Vec::new();
    for interface in if_addrs::get_if_addrs().unwrap_or_default() {
        if let std::net::IpAddr::V4(ip) = interface.ip() {
            if ip.is_loopback() || ip.is_link_local() || is_vpn_range(&ip) {
                continue;
            }
            own.push(ip);
            let o = ip.octets();
            if !networks.contains(&[o[0], o[1], o[2]]) {
                networks.push([o[0], o[1], o[2]]);
            }
        }
    }
    let mut hosts = Vec::new();
    for n in networks {
        for last in 1..=254u8 {
            let ip = std::net::Ipv4Addr::new(n[0], n[1], n[2], last);
            if !own.contains(&ip) {
                hosts.push(ip);
            }
        }
    }
    hosts
}

// IPv4 addresses of the other devices of the Tailscale network.
fn tailscale_peer_hosts() -> Vec<std::net::Ipv4Addr> {
    let Some(cli) = find_tailscale_cli() else {
        return Vec::new();
    };
    let Ok(output) = tailscale_command(&cli).args(["status", "--json"]).output() else {
        return Vec::new();
    };
    let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) else {
        return Vec::new();
    };
    let mut hosts = Vec::new();
    if let Some(peers) = json.get("Peer").and_then(|p| p.as_object()) {
        for peer in peers.values() {
            if let Some(ips) = peer.get("TailscaleIPs").and_then(|i| i.as_array()) {
                for ip in ips.iter().filter_map(|i| i.as_str()) {
                    if let Ok(ip) = ip.parse::<std::net::Ipv4Addr>() {
                        hosts.push(ip);
                    }
                }
            }
        }
    }
    hosts
}

fn open_hosts(hosts: Vec<std::net::Ipv4Addr>, port: u16, timeout: Duration) -> Vec<String> {
    let found = Mutex::new(Vec::new());
    // Probed in batches, so a /24 network does not open hundreds of sockets at once.
    for batch in hosts.chunks(128) {
        std::thread::scope(|scope| {
            for ip in batch {
                let found = &found;
                scope.spawn(move || {
                    let address = std::net::SocketAddr::from((*ip, port));
                    if std::net::TcpStream::connect_timeout(&address, timeout).is_ok() {
                        found.lock().unwrap().push(ip.to_string());
                    }
                });
            }
        });
    }
    let mut found = found.into_inner().unwrap();
    found.sort_by_key(|ip| ip.parse::<std::net::Ipv4Addr>().map(|a| a.octets()).unwrap_or_default());
    found.dedup();
    found
}

// Looks for computers listening on `port` (the web manager's) in the local networks of
// this computer and among the Tailscale peers. The frontend then checks which of them
// really is the ONYKS server. Returns the IP addresses found.
#[tauri::command(async)]
fn discover_servers(port: u16) -> Vec<String> {
    let mut hosts = local_subnet_hosts();
    hosts.extend(tailscale_peer_hosts());
    open_hosts(hosts, port, Duration::from_millis(400))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
// KiCad -> Altium conversion: runs the bundled kicad2altium.py (copy of
// onyks-bloodstone/converter, pure standard library) with the system Python.
fn python_candidates() -> &'static [&'static str] {
    if cfg!(target_os = "windows") {
        &["python", "py", "python3"]
    } else {
        &["python3", "python"]
    }
}

#[tauri::command]
fn convert_kicad_to_altium(
    app: tauri::AppHandle,
    sym: Vec<String>,
    fp: Vec<String>,
    out: String,
    name: Option<String>,
) -> Result<String, String> {
    if sym.is_empty() && fp.is_empty() {
        return Err("Wybierz pliki .kicad_sym i/lub .kicad_mod / katalog .pretty.".to_string());
    }
    if out.trim().is_empty() {
        return Err("Wybierz katalog wyjściowy.".to_string());
    }

    let script = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Brak katalogu zasobów: {}", e))?
        .join("converter")
        .join("kicad2altium.py");
    if !script.exists() {
        return Err(format!("Nie znaleziono konwertera: {}", script.display()));
    }

    let mut last_error = String::new();
    for python in python_candidates() {
        let mut command = Command::new(python);
        #[cfg(target_os = "windows")]
        command.creation_flags(0x08000000);
        command.env("PYTHONIOENCODING", "utf-8").arg(&script);
        if !sym.is_empty() {
            command.arg("--sym").args(&sym);
        }
        if !fp.is_empty() {
            command.arg("--fp").args(&fp);
        }
        command.arg("-o").arg(&out);
        if let Some(name) = name.as_ref().filter(|n| !n.trim().is_empty()) {
            command.arg("--name").arg(name.trim());
        }

        match command.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let text = format!("{}{}", stdout, stderr).trim().to_string();
                if output.status.success() {
                    return Ok(text);
                }
                // The Microsoft Store stub of "python" exits non-zero without running anything.
                if text.is_empty() || text.contains("Python was not found") {
                    last_error = format!("{} nie uruchomił konwertera.", python);
                    continue;
                }
                return Err(text);
            }
            Err(e) => last_error = format!("{}: {}", python, e),
        }
    }
    Err(format!(
        "Nie znaleziono Pythona 3 (wymagany do konwersji). Zainstaluj go i spróbuj ponownie. ({})",
        last_error
    ))
}

// ODBC data source for the CAD tools. The .DbLib / .kicad_dbl only name the source
// ("bloodstone"); the connection and the login live here, in the user's own data source
// (registry on Windows, ~/.odbc.ini on Linux), so no password reaches the library file
// or the repository.
fn odbc_value_ok(value: &str) -> bool {
    !value.contains(['\n', '\r', '\0'])
}

#[cfg(target_os = "windows")]
fn setup_odbc_source_impl(name: &str, host: &str, port: &str, database: &str, username: &str, password: &str) -> Result<String, String> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;

    let drivers = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SOFTWARE\ODBC\ODBCINST.INI\ODBC Drivers")
        .map_err(|_| "Nie znaleziono sterowników ODBC (czy zainstalowano psqlODBC?).".to_string())?;
    let names: Vec<String> = drivers.enum_values().filter_map(|v| v.ok()).map(|(n, _)| n).collect();
    let driver = names
        .iter()
        .find(|n| n.starts_with("PostgreSQL Unicode"))
        .or_else(|| names.iter().find(|n| n.contains("PostgreSQL")))
        .ok_or("Nie znaleziono sterownika ODBC PostgreSQL (psqlODBC).")?
        .clone();
    let dll: String = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(format!(r"SOFTWARE\ODBC\ODBCINST.INI\{}", driver))
        .and_then(|k| k.get_value("Driver"))
        .map_err(|e| format!("Brak ścieżki sterownika {}: {}", driver, e))?;

    let base = RegKey::predef(HKEY_CURRENT_USER);
    let (sources, _) = base
        .create_subkey(r"Software\ODBC\ODBC.INI\ODBC Data Sources")
        .map_err(|e| e.to_string())?;
    sources.set_value(name, &driver).map_err(|e| e.to_string())?;

    let (key, _) = base
        .create_subkey(format!(r"Software\ODBC\ODBC.INI\{}", name))
        .map_err(|e| e.to_string())?;
    for (k, v) in [
        ("Driver", dll.as_str()),
        ("Description", "ONYKS Bloodstone"),
        ("Servername", host),
        ("Port", port),
        ("Database", database),
        ("Username", username),
        ("Password", password),
    ] {
        key.set_value(k, &v).map_err(|e| e.to_string())?;
    }
    Ok(format!("Użytkownik ODBC DSN \"{}\" ({}) zapisany w rejestrze.", name, driver))
}

#[cfg(target_os = "linux")]
fn setup_odbc_source_impl(name: &str, host: &str, port: &str, database: &str, username: &str, password: &str) -> Result<String, String> {
    use std::os::unix::fs::PermissionsExt;

    let listed = Command::new("odbcinst").args(["-q", "-d"]).output().ok();
    let names: Vec<String> = listed
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().map(|l| l.trim().trim_matches(['[', ']']).to_string()).collect())
        .unwrap_or_default();
    let driver = names
        .iter()
        .find(|n| n.starts_with("PostgreSQL Unicode"))
        .or_else(|| names.iter().find(|n| n.contains("PostgreSQL")))
        .cloned()
        .ok_or("Nie znaleziono sterownika ODBC PostgreSQL (zainstaluj odbc-postgresql / postgresql-odbc / psqlodbc).")?;

    let home = std::env::var("HOME").map_err(|_| "Brak $HOME.".to_string())?;
    let path = Path::new(&home).join(".odbc.ini");
    let existing = fs::read_to_string(&path).unwrap_or_default();

    // Drop the old section of this source, keep everything else the user has.
    let header = format!("[{}]", name);
    let mut kept = String::new();
    let mut skipping = false;
    for line in existing.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            skipping = t.eq_ignore_ascii_case(&header);
        }
        if !skipping {
            kept.push_str(line);
            kept.push('\n');
        }
    }
    let kept = kept.trim_end().to_string();
    let section = format!(
        "{}\nDescription = ONYKS Bloodstone\nDriver = {}\nServername = {}\nPort = {}\nDatabase = {}\nUsername = {}\nPassword = {}\n",
        header, driver, host, port, database, username, password
    );
    let content = if kept.is_empty() { section } else { format!("{}\n\n{}", kept, section) };

    let tmp = path.with_extension("ini.tmp");
    fs::write(&tmp, content).map_err(|e| format!("Błąd zapisu {}: {}", tmp.display(), e))?;
    fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(format!("ODBC DSN \"{}\" ({}) zapisany w {}.", name, driver, path.display()))
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn setup_odbc_source_impl(_: &str, _: &str, _: &str, _: &str, _: &str, _: &str) -> Result<String, String> {
    Err("Konfiguracja ODBC jest obsługiwana tylko na Windows i Linux.".to_string())
}

#[tauri::command]
fn setup_odbc_source(name: String, host: String, port: String, database: String, username: String, password: String) -> Result<String, String> {
    for value in [&name, &host, &port, &database, &username, &password] {
        if !odbc_value_ok(value) {
            return Err("Wartość zawiera niedozwolony znak.".to_string());
        }
    }
    if name.trim().is_empty() || host.trim().is_empty() || database.trim().is_empty() || username.is_empty() {
        return Err("Brak nazwy źródła, serwera, bazy lub loginu.".to_string());
    }
    setup_odbc_source_impl(name.trim(), host.trim(), port.trim(), database.trim(), &username, &password)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .manage(WatcherState(Mutex::new(None)))
        .setup(|app| {
            let salt_path = app
                .path()
                .app_local_data_dir()
                .expect("could not resolve app local data path")
                .join("salt.txt");

            app.handle()
                .plugin(tauri_plugin_stronghold::Builder::with_argon2(&salt_path).build())?;
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            is_svn_repository,
            svn_status,
            svn_checkout,
            svn_add_all,
            svn_commit,
            svn_update,
            svn_delete,
            svn_revert,
            svn_cleanup,
            start_watch,
            stop_watch,
            read_text_file,
            write_text_file,
            write_binary_file,
            list_directory,
            tailscale_connect,
            discover_servers,
            convert_kicad_to_altium,
            setup_odbc_source
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn same_url_ignores_trailing_slash_and_case() {
        assert!(same_url("http://10.0.0.5/svn/repo/", "http://10.0.0.5/svn/repo"));
        assert!(same_url("http://Server/svn", "http://server/svn"));
        assert!(!same_url("http://10.0.0.5/svn", "http://100.64.0.2/svn"));
    }

    #[test]
    fn vpn_range_is_100_64_to_100_127() {
        assert!(is_vpn_range(&"100.64.0.1".parse().unwrap()));
        assert!(is_vpn_range(&"100.127.255.1".parse().unwrap()));
        assert!(!is_vpn_range(&"100.128.0.1".parse().unwrap()));
        assert!(!is_vpn_range(&"192.168.1.1".parse().unwrap()));
    }

    #[test]
    fn open_hosts_finds_a_listening_port() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let found = open_hosts(vec!["127.0.0.1".parse().unwrap(), "127.0.0.2".parse().unwrap()], port, Duration::from_millis(300));
        assert_eq!(found, vec!["127.0.0.1".to_string()]);
    }

    #[test]
    fn redact_hides_the_auth_key() {
        assert_eq!(redact("bad key tskey-abc123 given", "tskey-abc123"), "bad key *** given");
        assert_eq!(redact("nothing", ""), "nothing");
    }

    #[test]
    fn tailscale_connect_rejects_missing_or_empty_key_file() {
        assert!(tailscale_connect("").is_err());
        assert!(tailscale_connect("Z:/definitely/not/here.txt").is_err());
    }

    #[test]
    fn is_relevant_change_filters_svn_and_tmp() {
        assert!(is_relevant_change("C:/repo/fpga/fpga.PcbLib"));
        assert!(!is_relevant_change("C:/repo/.svn/wc.db"));
        assert!(!is_relevant_change("C:/repo/onyks_bloodstone.DbLib.tmp"));
    }

    // Exercises the real notify + notify-debouncer-mini watch, bypassing Tauri's
    // AppHandle (not needed outside a running app), to prove a change in a *subfolder*
    // of the watched path is actually detected end to end - not just that it compiles.
    #[test]
    fn watcher_detects_change_in_subfolder() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("chalcedon-watch-test-{nanos}"));
        let sub = dir.join("nested");
        fs::create_dir_all(&sub).unwrap();

        let (tx, rx) = mpsc::channel();
        let mut debouncer = new_debouncer(
            Duration::from_millis(200),
            move |result: notify_debouncer_mini::DebounceEventResult| {
                if let Ok(events) = result {
                    let relevant = events
                        .iter()
                        .any(|e| is_relevant_change(&e.path.to_string_lossy()));
                    if relevant {
                        let _ = tx.send(());
                    }
                }
            },
        )
        .unwrap();
        debouncer
            .watcher()
            .watch(&dir, RecursiveMode::Recursive)
            .unwrap();

        // Give the OS watch a moment to fully register before writing.
        std::thread::sleep(Duration::from_millis(200));
        fs::write(sub.join("changed.txt"), b"hello").unwrap();

        let received = rx.recv_timeout(Duration::from_secs(5)).is_ok();
        let _ = fs::remove_dir_all(&dir);
        assert!(
            received,
            "expected a change event for a file created in a watched subfolder"
        );
    }
}
