# Changelog

All notable changes to Sysora will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.4.0] - 2026-11-24

### Added
- **Automated In-App Updates**: Periodic and startup background checks against remote `version.txt` and GitHub Releases API (`#16`).
- **Interactive Update Modal**: Displays new version release notes, real-time terminal output during download and package installation, and a one-click restart button (`#16`).
- **Platform Install Scripts**: Architecture-aware installation and upgrade scripts for Ubuntu (`install-scripts/ubuntu.sh`), macOS (`install-scripts/macos.sh`), and Windows (`install-scripts/windows.ps1`).
- **Update Settings & Indicators**: "Software Updates" status card in the Settings tab and animated top bar notification badge when updates are ready.
- **Startup Privilege Elevation**: Elevated launch prompt (`pkexec` on Linux, `osascript` on macOS) so Sysora runs as a privileged user throughout the session (`#18`).
- **Child Watchdog**: Monitors parent launcher PID and cleanly exits when parent process terminates.
- **Robust Process Termination**: Terminates entire process groups (`SIGKILL` to `-pgid`) and recursive descendant trees (renderers, zygotes, child processes) (`#18`).
- **Intelligent Process Stem Matching**: Automatically matches browser wrappers to binary instances (e.g., killing `brave-browser-stable` or `zen-bin` cleanly terminates all `brave` or `zen` instances and helper tabs) (`#18`).
- **Process Verification**: Verifies process termination in the system process table before reporting success to the UI (`#18`).
- **Centralized Version Tracking**: Root-level `version.txt` file for lightweight remote version checks.

### Fixed
- **macOS APFS Storage Doubling**: Resolved issue where macOS APFS subvolumes under `/System/Volumes/*` (Data, VM, Preboot) sharing the same container were summed together, displaying 512 GB disks as 1 TB (`#17`).
- **macOS Hardware RAM Accuracy**: Resolved RAM doubling (32 GB $\rightarrow$ 64 GB) by querying authoritative Darwin kernel physical memory via `libc::HW_MEMSIZE` (`sysctl -n hw.memsize`) (`#17`).
- **Tokio Reactor Context**: Resolved panic when spawning background update checker by migrating to Tauri's native async runtime (`tauri::async_runtime::spawn`).
- **Resilient Tray Initialization**: Handled elevated Linux D-Bus session bus restrictions gracefully without interrupting GUI startup.

---

## [0.3.0] - 2026-06-15

### Added
- **Network Monitoring**: Live upload and download speed sparklines with per-interface metrics (`#9`).
- **Historical Snapshots**: SQLite database storing system metrics with trend charts for 1h, 6h, 24h, and 7d intervals (`#14`).
- **Theme Switcher**: Dark, Light, and System theme preferences with persistent state (`#13`).
- **Sensors & Fans**: Hardware temperature readouts and motherboard fan speed monitoring (`#11`).
- **PDF Report Export**: Full system specification and health PDF generation (`#10`).
- **Sysora CLI**: Terminal interface supporting `sysora status`, `sysora processes`, `sysora disk`, `sysora battery`, and `sysora fans` with JSON output support (`#12`).

---

## [0.2.0] - 2026-05-13

### Added
- **Installed Apps Manager**: Cross-platform application listing and uninstaller for Linux, macOS, and Windows.
- **Disk Space Scanner**: Directory tree scanner identifying largest files and folders with direct deletion.
- **Battery Health**: Hardware capacity comparison (design capacity vs. full charge capacity) and cycle count tracking (`#3`).
- **Process Manager**: Sortable process list by memory usage with one-click kill.

---

## [0.1.0] - 2026-05-01

### Added
- Initial open-source release of Sysora.
- Cross-platform system monitor for Linux, macOS, and Windows built with Tauri and Rust.
- Real-time CPU usage, RAM utilization, and system tray monitor.
