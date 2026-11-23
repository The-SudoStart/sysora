use std::process::Command;

#[cfg(target_os = "linux")]
fn get_gtk_theme() -> Option<String> {
    Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "gtk-theme"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                let theme = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .trim_matches('\'')
                    .to_string();
                if !theme.is_empty() {
                    Some(theme)
                } else {
                    None
                }
            } else {
                None
            }
        })
}

#[cfg(target_os = "linux")]
pub fn build_pkexec_command(
    exe: &std::path::Path,
    launcher_pid: u32,
    args: &[String],
    theme: Option<&str>,
) -> Command {
    let display = std::env::var("DISPLAY").unwrap_or_default();
    let xauthority = std::env::var("XAUTHORITY").unwrap_or_default();
    let wayland = std::env::var("WAYLAND_DISPLAY").unwrap_or_default();
    let xdg_runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_default();
    let home = std::env::var("HOME").unwrap_or_default();
    let xdg_data_dirs = std::env::var("XDG_DATA_DIRS").unwrap_or_default();
    let dbus = std::env::var("DBUS_SESSION_BUS_ADDRESS").unwrap_or_default();

    let xdg_current_desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    let xdg_session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
    let xdg_session_desktop = std::env::var("XDG_SESSION_DESKTOP").unwrap_or_default();

    let mut cmd = Command::new("pkexec");
    cmd.arg("env")
        .arg(format!("DISPLAY={display}"))
        .arg(format!("XAUTHORITY={xauthority}"))
        .arg(format!("WAYLAND_DISPLAY={wayland}"))
        .arg(format!("XDG_RUNTIME_DIR={xdg_runtime}"))
        .arg(format!("HOME={home}"))
        .arg(format!("XDG_DATA_DIRS={xdg_data_dirs}"))
        .arg(format!("DBUS_SESSION_BUS_ADDRESS={dbus}"))
        .arg(format!("XDG_CURRENT_DESKTOP={xdg_current_desktop}"))
        .arg(format!("XDG_SESSION_TYPE={xdg_session_type}"))
        .arg(format!("XDG_SESSION_DESKTOP={xdg_session_desktop}"))
        .arg("GTK_DECORATION_LAYOUT=menu:minimize,maximize,close");

    if let Some(t) = theme {
        cmd.arg(format!("GTK_THEME={t}"));
    }

    cmd.arg(exe)
        .arg("--parent-pid")
        .arg(launcher_pid.to_string())
        .args(args);

    cmd
}

#[cfg(unix)]
pub fn parse_parent_pid<I, S>(args: I) -> Option<libc::pid_t>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args: Vec<S> = args.into_iter().collect();
    let pos = args.iter().position(|a| a.as_ref() == "--parent-pid")?;
    let pid_str = args.get(pos + 1)?.as_ref();
    pid_str.parse::<libc::pid_t>().ok()
}

#[cfg(unix)]
pub fn spawn_watchdog() {
    if let Some(parent_pid) = parse_parent_pid(std::env::args()) {
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_millis(500));
            if unsafe { libc::kill(parent_pid, 0) } != 0 {
                std::process::exit(0);
            }
        });
    }
}

#[cfg(unix)]
fn should_skip_elevation() -> bool {
    // Tests or explicit skip flags
    if cfg!(test) {
        return true;
    }
    if std::env::args().any(|a| a == "--no-elevation")
        || std::env::var("SYSORA_NO_ELEVATION").is_ok()
    {
        return true;
    }
    if std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok() {
        return true;
    }
    #[cfg(target_os = "linux")]
    {
        // If headless (no X11 / Wayland), don't pop GUI pkexec
        if std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err() {
            return true;
        }
    }
    false
}

#[cfg(target_os = "linux")]
pub fn elevate_privileges() {
    if unsafe { libc::geteuid() } != 0 {
        if should_skip_elevation() {
            return;
        }
        let launcher_pid = std::process::id();
        let exe = std::env::current_exe().expect("cannot get executable path");
        let args: Vec<String> = std::env::args()
            .skip(1)
            .filter(|a| a != "--parent-pid")
            .collect();
        let theme = get_gtk_theme();
        let mut cmd = build_pkexec_command(&exe, launcher_pid, &args, theme.as_deref());
        let status = cmd.status();
        let code = match status {
            Ok(s) => s.code().unwrap_or(1),
            Err(e) => {
                eprintln!("pkexec failed to launch: {e}");
                1
            }
        };
        std::process::exit(code);
    }
    spawn_watchdog();
}

#[cfg(target_os = "macos")]
pub fn elevate_privileges() {
    if unsafe { libc::geteuid() } != 0 {
        if should_skip_elevation() {
            return;
        }
        let launcher_pid = std::process::id();
        let exe = std::env::current_exe()
            .expect("cannot get executable path")
            .to_string_lossy()
            .to_string();
        let args: Vec<String> = std::env::args()
            .skip(1)
            .filter(|a| a != "--parent-pid")
            .collect();

        let sq = |s: &str| format!("'{}'", s.replace('\'', "'\\''"));
        let mut parts = vec![sq(&exe), sq("--parent-pid"), sq(&launcher_pid.to_string())];
        for a in &args {
            parts.push(sq(a));
        }
        let home = std::env::var("HOME").unwrap_or_default();
        let shell_cmd = format!("export HOME={}; {}", sq(&home), parts.join(" "));

        let apple_script_cmd = shell_cmd.replace('\\', "\\\\").replace('"', "\\\"");

        let result = Command::new("osascript")
            .args([
                "-e",
                &format!(
                    "do shell script \"{}\" with administrator privileges",
                    apple_script_cmd
                ),
            ])
            .status();
        let code = match result {
            Ok(s) => s.code().unwrap_or(1),
            Err(e) => {
                eprintln!("osascript relaunch failed: {e}");
                1
            }
        };
        std::process::exit(code);
    }
    spawn_watchdog();
}

#[cfg(not(unix))]
pub fn elevate_privileges() {
    // Windows elevation is handled via manifest or UAC when needed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_parent_pid() {
        let args = vec!["sysora".to_string(), "--parent-pid".to_string(), "9999".to_string()];
        assert_eq!(parse_parent_pid(&args), Some(9999));

        let args_none = vec!["sysora".to_string(), "--minimized".to_string()];
        assert_eq!(parse_parent_pid(&args_none), None);
    }

    #[test]
    fn test_should_skip_elevation_in_tests() {
        assert!(should_skip_elevation());
    }
}
