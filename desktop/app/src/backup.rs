// SPDX-License-Identifier: GPL-3.0-or-later
//! Backing up the user's choices to one file and restoring them, for a new
//! PC or a reinstalled Windows: the settings, the notification rules and the
//! Deck. Not the device identity or the clipboard history: those are sealed
//! to this Windows account (DPAPI) and can't be opened anywhere else, so the
//! phone pairs again after a restore.
//!
//! A restore doesn't write over files the running app keeps saving: it
//! stages the backup, the app restarts, and [`apply_pending`] puts the files
//! in place before anything reads them.

use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// What a backup file says it is.
const FORMAT: &str = "ulak-backup";
const VERSION: u32 = 1;

/// The files a backup carries, by their key in it.
const FILES: &[(&str, &str)] = &[
    ("settings", "desktop-settings.json"),
    ("notificationRules", "notification-rules.json"),
    ("deck", "deck.json"),
];

/// A staged restore, applied at the next start.
const PENDING: &str = "restore-pending.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Backup {
    format: String,
    version: u32,
    app_version: String,
    /// Unix milliseconds.
    created: i64,
    files: BTreeMap<String, Value>,
}

/// Writes the user's settings, rules and Deck from `data_dir` to `out`.
/// Returns how many of them there were.
pub fn export(data_dir: &Path, out: &Path) -> Result<usize, String> {
    let mut files = BTreeMap::new();
    for (key, name) in FILES {
        match fs::read(data_dir.join(name)) {
            Ok(bytes) => {
                let value: Value =
                    serde_json::from_slice(&bytes).map_err(|e| format!("{name} isn't readable: {e}"))?;
                files.insert((*key).to_owned(), value);
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("can't read {name}: {e}")),
        }
    }
    let count = files.len();
    let backup = Backup {
        format: FORMAT.to_owned(),
        version: VERSION,
        app_version: env!("CARGO_PKG_VERSION").to_owned(),
        created: now_ms(),
        files,
    };
    write_atomically(out, &serde_json::to_vec_pretty(&backup).map_err(|e| e.to_string())?)
        .map_err(|e| format!("can't write {}: {e}", out.display()))?;
    Ok(count)
}

/// Checks the backup at `path` and stages it for the next start. Returns
/// how many files it will restore.
pub fn stage_restore(data_dir: &Path, path: &Path) -> Result<usize, String> {
    let bytes = fs::read(path).map_err(|e| format!("can't read {}: {e}", path.display()))?;
    let backup = parse(&bytes)?;
    write_atomically(&data_dir.join(PENDING), &bytes).map_err(|e| format!("can't stage the restore: {e}"))?;
    Ok(backup.files.len())
}

/// Puts a staged backup's files in place (at start, before anything reads
/// them), and forgets it. A broken one is set aside, not applied.
pub fn apply_pending(data_dir: &Path) {
    let pending = data_dir.join(PENDING);
    let Ok(bytes) = fs::read(&pending) else { return };
    match parse(&bytes) {
        Ok(backup) => {
            for (key, name) in FILES {
                let Some(value) = backup.files.get(*key) else { continue };
                let written = serde_json::to_vec_pretty(value)
                    .map_err(std::io::Error::other)
                    .and_then(|json| write_atomically(&data_dir.join(name), &json));
                match written {
                    Ok(()) => tracing::info!(file = name, "restored from backup"),
                    Err(e) => tracing::warn!(file = name, error = %e, "can't restore"),
                }
            }
            let _ = fs::remove_file(&pending);
        }
        Err(e) => {
            tracing::warn!(error = %e, "the staged backup isn't usable; set aside");
            let _ = fs::rename(&pending, data_dir.join("restore-pending.json.bad"));
        }
    }
}

fn parse(bytes: &[u8]) -> Result<Backup, String> {
    let backup: Backup =
        serde_json::from_slice(bytes).map_err(|_| "this isn't an Ulak backup file".to_owned())?;
    if backup.format != FORMAT {
        return Err("this isn't an Ulak backup file".to_owned());
    }
    if backup.version > VERSION {
        return Err(format!("this backup is from a newer Ulak ({}); update Ulak first", backup.app_version));
    }
    if !backup.files.keys().any(|k| FILES.iter().any(|(key, _)| key == k)) {
        return Err("this backup has nothing to restore".to_owned());
    }
    Ok(backup)
}

/// A whole file or nothing: written beside it, then renamed over it.
fn write_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let partial: PathBuf = path.with_extension("partial");
    let mut file = fs::File::create(&partial)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(&partial, path)
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

/// Starts this program again once this process has quit, with `args`: the
/// new process gets `--wait-pid` and waits (see [`wait_for_exit`]).
pub fn relaunch_after_exit(args: &[String]) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    std::process::Command::new(exe)
        .arg("--wait-pid")
        .arg(std::process::id().to_string())
        .args(args)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Waits until process `pid` has ended, or `timeout` passes.
pub fn wait_for_exit(pid: u32, timeout: std::time::Duration) {
    #[cfg(windows)]
    crate::win::processes::wait_for_exit(pid, timeout);
    #[cfg(not(windows))]
    {
        let deadline = std::time::Instant::now() + timeout;
        while Path::new(&format!("/proc/{pid}")).exists() && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_backup_restores_settings_rules_and_deck_at_the_next_start() {
        let old = tempfile::tempdir().unwrap();
        fs::write(old.path().join("desktop-settings.json"), r#"{"dock":true,"dockEdge":"left"}"#).unwrap();
        fs::write(old.path().join("notification-rules.json"), r#"{"history":true,"apps":{}}"#).unwrap();
        fs::write(old.path().join("identity.key"), b"sealed to this Windows").unwrap();
        let file = old.path().join("ulak-backup.json");
        assert_eq!(export(old.path(), &file).unwrap(), 2, "no deck.json there");

        let new = tempfile::tempdir().unwrap();
        fs::write(new.path().join("desktop-settings.json"), r#"{"dock":false}"#).unwrap();
        assert_eq!(stage_restore(new.path(), &file).unwrap(), 2);
        let untouched = fs::read_to_string(new.path().join("desktop-settings.json")).unwrap();
        assert_eq!(untouched, r#"{"dock":false}"#, "nothing changes until the restart");

        apply_pending(new.path());
        let settings: Value =
            serde_json::from_slice(&fs::read(new.path().join("desktop-settings.json")).unwrap()).unwrap();
        assert_eq!(settings["dockEdge"], "left");
        assert!(new.path().join("notification-rules.json").exists());
        assert!(!new.path().join("deck.json").exists());
        assert!(!new.path().join("identity.key").exists(), "the identity isn't carried");
        assert!(!new.path().join(PENDING).exists(), "applied once");
    }

    #[test]
    fn waiting_for_a_process_that_is_gone_returns_at_once() {
        let started = std::time::Instant::now();
        // No process has this ID (IDs are multiples of 4 on Windows; this isn't).
        wait_for_exit(u32::MAX - 2, std::time::Duration::from_secs(5));
        assert!(started.elapsed() < std::time::Duration::from_secs(2));
    }

    #[test]
    fn other_files_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("x.json");
        fs::write(&file, r#"{"dock":true}"#).unwrap();
        assert!(stage_restore(dir.path(), &file).is_err());
        let newer = format!(
            r#"{{"format":"{FORMAT}","version":{},"appVersion":"9.0.0","created":0,"files":{{"deck":{{}}}}}}"#,
            VERSION + 1
        );
        fs::write(&file, newer).unwrap();
        assert!(stage_restore(dir.path(), &file).unwrap_err().contains("newer"));
        assert!(!dir.path().join(PENDING).exists());
    }
}
