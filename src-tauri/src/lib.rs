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

#[tauri::command(async)]
fn is_svn_repository(svn_folder_path: &str) -> bool {
    let path = Path::new(svn_folder_path).join(".svn");
    path.is_dir()
}

#[tauri::command(async)]
fn svn_status(svn_folder_path: &str) -> Result<Vec<String>, String> {
    let mut command = Command::new("svn");
    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000);
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
    let mut command = Command::new("svn");
    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000);
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
    let mut command = Command::new("svn");
    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000);
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
) -> Result<String, String> {
    let mut command = Command::new("svn");
    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000);
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
fn svn_update(svn_folder_path: &str, login: &str, password: &str) -> Result<String, String> {
    let mut command = Command::new("svn");
    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000);
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
    let mut command = Command::new("svn");
    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000);

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
    let mut command = Command::new("svn");
    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000);

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
    let mut status_command = Command::new("svn");
    #[cfg(target_os = "windows")]
    status_command.creation_flags(0x08000000);

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
            let mut delete_command = Command::new("svn");
            #[cfg(target_os = "windows")]
            delete_command.creation_flags(0x08000000);

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

#[tauri::command(async)]
fn write_text_file(path: &str, content: &str) -> Result<(), String> {
    write_file_atomically(path, content.as_bytes())
}

#[tauri::command(async)]
fn write_binary_file(path: &str, data: Vec<u8>) -> Result<(), String> {
    write_file_atomically(path, &data)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
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
            write_binary_file
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
