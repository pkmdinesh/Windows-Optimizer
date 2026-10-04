# WinOpt: Universal Hardware-Aware Windows OS Optimization Tool

A high-performance, safety-first, non-destructive systems utility and debloater for Windows 10 and Windows 11 (x64 and ARM64).

Written in Rust for sub-2MB standalone execution, zero runtime dependencies, and instant startup.

---

## ⚡ Key Features

### 1. Universal Hardware-Aware Intelligence
Rather than blindly applying aggressive registry tweaks that can cause thermal throttling or unstable thread scheduling, WinOpt profiles your machine in real time:
- **Chassis & Power State**: Distinguishes **Desktops vs. Laptops (AC vs. Battery)**. Automatically restrains aggressive CPU priority separations and core unparking on battery power to protect battery life and thermals.
- **Storage Subsystem**: Distinguishes **NVMe / SATA SSDs from mechanical HDDs**. Ensures TRIM is enabled. On ultra-fast flash drives (SSD/NVMe), it tunes `SysMain` (SuperFetch) to prevent unnecessary RAM pressure and redundant NAND flash writes; on HDDs, it keeps sequential caching active.
- **CPU Topology**: Detects heterogeneous hybrid cores (Intel Alder Lake / Raptor Lake P/E cores, AMD 3D V-Cache vs symmetric layouts) and core counts.
- **Memory Capacity**: Tailors background process thresholds and bloatware purging based on RAM capacity (<8 GB Low-RAM, 8–32 GB Standard, 64 GB+ High-Capacity).

### 2. Adware & Promotional Content Stripper
- **Start Menu Ads**: Disables suggested applications, promoted apps, and background cloud consumer features (`SilentInstalledAppsEnabled = 0`, `SystemPaneSuggestionsEnabled = 0`).
- **File Explorer Ads**: Strips OneDrive and Microsoft 365 promotional sync provider banners inside Explorer (`ShowSyncProviderNotifications = 0`).
- **Windows Search Ads**: Disables Bing web results within the taskbar search bar (`BingSearchEnabled = 0`, `ConnectedSearchUseWeb = 0`), returning instant, private, offline local file search.
- **Windows Tips & Lock Screen**: Disables lock screen advertising banners, rotating spotlight ads, and "Welcome Experience" popups.
- **Taskbar Widgets**: Disables the MSN News & Interests widget button from Windows 11 taskbars (`TaskbarDa = 0`).

### 3. Diagnostic Data & Telemetry Mitigation
- **Data Collection**: Restricts Windows diagnostic data transmission (`AllowTelemetry = 0` / Basic).
- **Telemetry Services**: Disables `DiagTrack` (Connected User Experiences and Telemetry) and `dmwappushservice`.
- **Advertising Tracking**: Disables Windows Advertising ID (`Enabled = 0`).
- **Telemetry Scheduled Tasks**: Disables CEIP (`Consolidator`, `UsbCeip`) and Compatibility Appraiser background tasks.

### 4. Curated UWP & Provisioned App Debloater
- **Safe Targets**: Scans and removes pre-installed bloatware (Cortana, Clipchamp, MSN News/Weather, Solitaire with ads, Mixed Reality, Feedback Hub, TikTok/Disney/Spotify preloads).
- **Core Whitelist Protection**: Strictly preserves and protects core system components (`Microsoft.WindowsStore`, `Microsoft.DesktopAppInstaller`, `Microsoft.WindowsTerminal`, `Microsoft.WindowsCalculator`, `Microsoft.WindowsCamera`, and UI XAML runtimes).

### 5. Startup & Boot Delay Optimization
- **Startup Delay Removal**: Eliminates the artificial 5–10 second delay Windows introduces before launching desktop startup programs (`StartupDelayInMSec = 0`).
- **Diagnostic Logon Tasks**: Defer or disable non-critical diagnostic disk analyzers that fire at user logon.
- **Startup Audit**: Enumerates programs registered in `HKLM:Run` and `HKCU:Run`.

### 6. Safety & 100% Reversibility
- **Automated Restore Points**: Requests a Windows System Restore Point prior to applying any batch optimization preset.
- **Transactional State Store**: Every registry modification and service change captures a pre-change snapshot in `backups/backup_*.json`.
- **One-Click Rollback**: Fully undo all tweaks back to the exact previous values at any time.

---

## 🚀 Usage

### 1. Command Line Interface (CLI)

```powershell
# Display hardware profile, recommendations, and current tweak statuses
.\target\release\winopt.exe --status

# Apply all safe hardware-aware optimizations immediately (headless)
.\target\release\winopt.exe --optimize

# Scan and purge detected UWP bloatware packages
.\target\release\winopt.exe --debloat

# Undo / Rollback all applied optimizations to their original state
.\target\release\winopt.exe --rollback

# Launch interactive console menu
.\target\release\winopt.exe
```

### 2. Modern Embedded Web Dashboard (GUI)

Run:
```powershell
.\target\release\winopt.exe --web
```
This automatically launches your default browser at `http://127.0.0.1:8765`, providing a sleek, dark-mode Fluent UI dashboard with:
- Live hardware cards (OS build, CPU cores/topology, RAM, SSD/NVMe status, TRIM).
- Interactive tweak toggles and one-click optimization.
- Visual UWP debloater scanner and batch uninstaller.
- Full backup and rollback management.

---

## 🛠️ Building from Source

Requirements:
- Rust (1.75+ or 1.97+) with MSVC toolchain

```powershell
# Build optimized release binary
cargo build --release

# Output binary located at:
# .\target\release\winopt.exe (1.7 MB standalone)
```
