use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

const VERSION_FILE_URL: &str =
    "https://raw.githubusercontent.com/The-SudoStart/sysora/main/version.txt";
const RELEASES_API_URL: &str =
    "https://api.github.com/repos/The-SudoStart/sysora/releases/latest";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub update_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogLine {
    pub line: String,
    pub level: String,
}

/// Parse a loose semver ("v0.3.0", "0.3.1-rc.1", "0.3") into a comparable tuple.
pub fn parse_version(v: &str) -> Option<(u64, u64, u64)> {
    let s = v.trim().trim_start_matches('v');
    let core = s.split('-').next().unwrap_or(s);
    let mut it = core.split('.');
    let major = it.next()?.trim().parse().ok()?;
    let minor = it.next().unwrap_or("0").trim().parse().unwrap_or(0);
    let patch = it.next().unwrap_or("0").trim().parse().unwrap_or(0);
    Some((major, minor, patch))
}

/// Fetch version string over HTTPS using curl.
async fn fetch_curl_output(url: &str, headers: &[&str]) -> Option<String> {
    let mut cmd = Command::new("curl");
    cmd.args(["-fsSL", "--proto", "=https", "--tlsv1.2", "-m", "10"]);
    for h in headers {
        cmd.args(["-H", h]);
    }
    cmd.arg(url);

    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);

    let child = cmd.spawn().ok()?;
    let output = tokio::time::timeout(std::time::Duration::from_secs(12), child.wait_with_output())
        .await
        .ok()?
        .ok()?;

    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Fetch the latest version from GitHub version.txt or releases API.
pub async fn fetch_latest_version() -> Option<String> {
    // 1. Try raw version.txt on main branch
    if let Some(content) = fetch_curl_output(VERSION_FILE_URL, &[]).await {
        let first_line = content.lines().next().unwrap_or("").trim().to_string();
        if !first_line.is_empty() && parse_version(&first_line).is_some() {
            return Some(first_line);
        }
    }

    // 2. Fallback: check GitHub releases API for latest tag_name
    if let Some(json_str) = fetch_curl_output(RELEASES_API_URL, &["User-Agent: Sysora"]).await {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
            if let Some(tag) = json.get("tag_name").and_then(|t| t.as_str()) {
                let clean = tag.trim_start_matches('v').trim().to_string();
                if parse_version(&clean).is_some() {
                    return Some(clean);
                }
            }
        }
    }

    None
}

/// Read the local version from app data dir version.txt or default to CARGO_PKG_VERSION.
pub fn get_local_version(app: &AppHandle) -> String {
    if let Ok(data_dir) = app.path().app_data_dir() {
        let path = data_dir.join("version.txt");
        if let Ok(content) = std::fs::read_to_string(path) {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn evaluate_update(current: &str, latest: Option<&str>) -> UpdateInfo {
    let update_available = match (latest, parse_version(current)) {
        (Some(latest_ver), Some(cur)) => {
            parse_version(latest_ver).map(|l| l > cur).unwrap_or(false)
        }
        _ => false,
    };

    UpdateInfo {
        current_version: current.to_string(),
        latest_version: latest.map(|s| s.trim_start_matches('v').to_string()),
        update_available,
    }
}

#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> Result<UpdateInfo, String> {
    let current = get_local_version(&app);
    let latest = fetch_latest_version().await;
    Ok(evaluate_update(&current, latest.as_deref()))
}

fn get_update_script_name() -> &'static str {
    #[cfg(target_os = "windows")]
    { "windows.ps1" }
    #[cfg(target_os = "macos")]
    { "macos.sh" }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    { "ubuntu.sh" }
}

fn resolve_script(app: &AppHandle) -> Result<PathBuf, String> {
    let script_name = get_update_script_name();

    // 1. Try bundled resource directory
    if let Ok(path) = app.path().resolve(format!("install-scripts/{}", script_name), tauri::path::BaseDirectory::Resource) {
        if path.exists() {
            return Ok(path);
        }
    }
    if let Ok(path) = app.path().resolve(script_name, tauri::path::BaseDirectory::Resource) {
        if path.exists() {
            return Ok(path);
        }
    }

    // 2. Try install-scripts relative to current exe or working directory (dev mode)
    let candidates = [
        PathBuf::from("install-scripts").join(script_name),
        PathBuf::from("../install-scripts").join(script_name),
        PathBuf::from("resources").join(script_name),
    ];

    for candidate in candidates {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    // 3. Fallback: check exe parent directory
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let candidate1 = exe_dir.join("install-scripts").join(script_name);
            if candidate1.exists() {
                return Ok(candidate1);
            }
            let candidate2 = exe_dir.join("resources").join(script_name);
            if candidate2.exists() {
                return Ok(candidate2);
            }
        }
    }

    Err(format!(
        "Could not locate update script '{}'. Please ensure install-scripts/ are present.",
        script_name
    ))
}

fn build_update_command(resolved_path: &Path) -> Command {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new("powershell");
        cmd.args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &resolved_path.to_string_lossy(),
        ]);
        cmd
    }

    #[cfg(not(target_os = "windows"))]
    {
        let mut cmd = Command::new("bash");
        cmd.arg(resolved_path);
        cmd
    }
}

#[tauri::command]
pub async fn run_app_update(app: AppHandle) -> Result<(), String> {
    let script_path = resolve_script(&app)?;
    let mut cmd = build_update_command(&script_path);

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn update script: {}", e))?;

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    let (tx_log, mut rx_log) = tokio::sync::mpsc::unbounded_channel::<String>();

    // Forward stdout
    let app_out = app.clone();
    let tx1 = tx_log.clone();
    tauri::async_runtime::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = tx1.send(line.clone());
            let _ = app_out.emit(
                "update-log",
                UpdateLogLine {
                    line,
                    level: "info".into(),
                },
            );
        }
    });

    // Forward stderr
    let app_err = app.clone();
    let tx2 = tx_log;
    tauri::async_runtime::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = tx2.send(line.clone());
            let _ = app_err.emit(
                "update-log",
                UpdateLogLine {
                    line,
                    level: "warn".into(),
                },
            );
        }
    });

    // Append to update.log file
    let log_file = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("update.log");

    tauri::async_runtime::spawn(async move {
        if let Ok(mut file) = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .await
        {
            while let Some(line) = rx_log.recv().await {
                let _ = file.write_all(format!("{}\n", line).as_bytes()).await;
            }
            let _ = file.flush().await;
        }
    });

    let status = child
        .wait()
        .await
        .map_err(|e| format!("Update process error: {}", e))?;

    if !status.success() {
        return Err(format!(
            "Update script failed with exit code: {}",
            status.code().unwrap_or(-1)
        ));
    }

    let _ = app.emit(
        "update-log",
        UpdateLogLine {
            line: "Update completed successfully!".to_string(),
            level: "success".into(),
        },
    );

    Ok(())
}

#[tauri::command]
pub fn restart_app(app: AppHandle) {
    tauri::process::restart(&app.env());
}

/// Send an update notification to the user.
pub fn notify_update_available(#[allow(unused_variables)] app: &AppHandle, latest: &str) {
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("notify-send")
            .arg("-a")
            .arg("Sysora")
            .arg("Sysora Update Available")
            .arg(format!("A new version (v{}) is ready to install.", latest))
            .spawn();
    }

    // Cross-platform fallback via tauri notification plugin if enabled
    #[cfg(not(target_os = "linux"))]
    {
        use tauri_plugin_notification::NotificationExt;
        let _ = app
            .notification()
            .builder()
            .title("Sysora Update Available")
            .body(format!("A new version (v{}) is ready to install.", latest))
            .show();
    }
}

/// Spawns a background check on startup.
pub fn start_background_update_checker(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        // Wait 5 seconds after startup before checking
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        let current = get_local_version(&app);
        if let Some(latest) = fetch_latest_version().await {
            let info = evaluate_update(&current, Some(&latest));
            if info.update_available {
                notify_update_available(&app, &latest);
                let _ = app.emit("update-available", &info);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("0.3.0"), Some((0, 3, 0)));
        assert_eq!(parse_version("v0.3.1"), Some((0, 3, 1)));
        assert_eq!(parse_version("1.0.0-rc.1"), Some((1, 0, 0)));
        assert_eq!(parse_version("v2.1"), Some((2, 1, 0)));
        assert_eq!(parse_version("invalid"), None);
    }

    #[test]
    fn test_evaluate_update() {
        let update = evaluate_update("0.3.0", Some("0.3.1"));
        assert!(update.update_available);
        assert_eq!(update.current_version, "0.3.0");
        assert_eq!(update.latest_version, Some("0.3.1".to_string()));

        let same = evaluate_update("0.3.0", Some("0.3.0"));
        assert!(!same.update_available);

        let older = evaluate_update("0.3.2", Some("0.3.1"));
        assert!(!older.update_available);

        let none = evaluate_update("0.3.0", None);
        assert!(!none.update_available);
    }
}
