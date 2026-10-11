# WinOpt: Universal Hardware-Aware Windows OS Optimization & Cleaner GUI

A high-performance, safety-first, non-destructive systems optimization utility, deep cache cleaner, and debloater for Windows 10 and Windows 11 (x64 and ARM64).

Written in Rust for sub-2MB standalone execution, zero runtime dependencies, and instant startup.

---

## ⚡ Key Features

### 1. Dedicated Executable GUI Desktop Application
- **Zero Configuration**: Simply double-click `winopt.exe` from File Explorer or run it from a terminal without flags. The console window is automatically hidden and a native standalone Fluent-themed desktop application window (1200x850) is launched immediately.
- **Works Offline**: The embedded web GUI engine has zero external internet dependencies and runs locally on `http://127.0.0.1:8765`.

### 2. Deep Junk, Cache & Temp Directories Cleaner
Safely reclaims gigabytes of disk space and eliminates filesystem clutter:
- **User Temporary Files (`%TEMP%`)**: Cleans app installer extracts, abandoned node_modules, and cache files.
- **Windows System Temp (`C:\Windows\Temp`)**: Purges installer logs, system temp files, and driver unpacks.
- **Windows Update Download Cache (`SoftwareDistribution\Download`)**: Safely removes redundant, already-installed patch packages.
- **DirectX Shader Caches (`D3DSCache`, NVIDIA/AMD shader caches)**: Stale or corrupt shader caches frequently cause severe microstutter in games and 3D applications.
- **Windows Prefetch Cache (`C:\Windows\Prefetch`)**: Removes stale prefetch traces for uninstalled applications.
- **Application Crash Dumps (`%LOCALAPPDATA%\CrashDumps`)**: Cleans forgotten `.dmp` files.
- **File Explorer Thumbnail Cache (`thumbcache_*.db`)**: Cleans corrupted thumbnail caches that cause File Explorer to freeze or take seconds to open folders.
- **Delivery Optimization P2P Cache**: Removes cached P2P update parts.
- **Empty Recycle Bin**: Safely purges deleted files across all connected drives.
- **Safe Locked-File Handling**: Actively locked files in use by running processes are gracefully bypassed without interruption.

### 3. Anti-Sluggishness & Responsiveness Engine
Directly targets the primary reasons Windows becomes slow, sluggish, and unresponsive over time:
- **Instant RAM Working Set & DNS Flush (⚡ One-Click Boost)**: Clears DNS resolver caches and trims bloated process working sets, freeing up standby memory immediately.
- **Disable Multimedia Network & CPU Throttling**: By default, Windows throttles non-multimedia network packets to 10,000 packets/sec and reserves 20% CPU for background services. WinOpt sets `NetworkThrottlingIndex = 0xffffffff` and `SystemResponsiveness = 0` (giving 100% CPU priority to foreground applications).
- **Disable GameDVR Background Video Recording**: Stops Windows from constantly polling and buffering video encoding in the background, saving 10–15% GPU/CPU overhead and eliminating frame drops.
- **Eliminate UI Menu & Window Animation Latency**: Removes artificial 400ms menu popup delays (`MenuShowDelay = 0`) and sluggish window minimize/maximize animation pauses (`MinAnimate = 0`).
- **Disable Power Throttling on Foreground Tasks**: Prevents Windows 11 from artificially downclocking and power-throttling active application threads when on AC power.
- **Disable Random Idle Automatic Maintenance**: Stops Windows Maintenance tasks from abruptly spinning up background CPU and disk scans during brief idle pauses.

### 4. Universal Hardware-Aware Intelligence
Rather than blindly applying aggressive registry tweaks that can cause thermal throttling or unstable thread scheduling, WinOpt profiles your machine in real time:
- **Chassis & Power State**: Distinguishes **Desktops vs. Laptops (AC vs. Battery)**. Restrains aggressive CPU priority separations and core unparking on battery power to protect battery life and thermals.
- **Storage Subsystem**: Distinguishes **NVMe / SATA SSDs from mechanical HDDs**. Ensures TRIM is enabled. On ultra-fast flash drives (SSD/NVMe), it tunes `SysMain` (SuperFetch) to prevent unnecessary RAM pressure and redundant NAND flash writes; on HDDs, it keeps sequential caching active.
- **CPU Topology**: Detects heterogeneous hybrid cores (Intel Alder Lake / Raptor Lake P/E cores, AMD 3D V-Cache vs symmetric layouts) and core counts.
- **Memory Capacity**: Tailors background process thresholds and bloatware purging based on RAM capacity (<8 GB Low-RAM, 8–32 GB Standard, 64 GB+ High-Capacity).

### 5. Adware & Promotional Content Stripper
- **Start Menu Ads**: Disables suggested applications, promoted apps, and background cloud consumer features (`SilentInstalledAppsEnabled = 0`, `SystemPaneSuggestionsEnabled = 0`).
- **File Explorer Ads**: Strips OneDrive and Microsoft 365 promotional sync provider banners inside Explorer (`ShowSyncProviderNotifications = 0`).
- **Windows Search Ads**: Disables Bing web results within the taskbar search bar (`BingSearchEnabled = 0`, `ConnectedSearchUseWeb = 0`), returning instant, private, offline local file search.
- **Windows Tips & Lock Screen**: Disables lock screen advertising banners, rotating spotlight ads, and "Welcome Experience" popups.
- **Taskbar Widgets**: Disables the MSN News & Interests widget button from Windows 11 taskbars (`TaskbarDa = 0`).

### 6. Diagnostic Data & Telemetry Mitigation
- **Data Collection**: Restricts Windows diagnostic data transmission (`AllowTelemetry = 0` / Basic).
- **Telemetry Services**: Disables `DiagTrack` (Connected User Experiences and Telemetry) and `dmwappushservice`.
- **Advertising Tracking**: Disables Windows Advertising ID (`Enabled = 0`).
- **Telemetry Scheduled Tasks**: Disables CEIP (`Consolidator`, `UsbCeip`) and Compatibility Appraiser background tasks.

### 7. Curated UWP & Provisioned App Debloater
- **Safe Targets**: Scans and removes pre-installed bloatware (Cortana, Clipchamp, MSN News/Weather, Solitaire with ads, Mixed Reality, Feedback Hub, TikTok/Disney/Spotify preloads).
- **Core Whitelist Protection**: Strictly preserves and protects core system components (`Microsoft.WindowsStore`, `Microsoft.DesktopAppInstaller`, `Microsoft.WindowsTerminal`, `Microsoft.WindowsCalculator`, `Microsoft.WindowsCamera`, and UI XAML runtimes).

### 8. Safety & 100% Reversibility
- **Automated Restore Points**: Requests a Windows System Restore Point prior to applying any batch optimization preset.
- **Transactional State Store**: Every registry modification and service change captures a pre-change snapshot in `backups/backup_*.json`.
- **One-Click Rollback**: Fully undo all tweaks back to the exact previous values at any time.

---

## 🚀 Usage

### 1. Launching as GUI Application (Default)

Simply run or double-click the binary:
```powershell
.\target\release\winopt.exe
```
This automatically opens the standalone application window with tabs for:
- 🧹 **Deep Junk & Temp Cleaner** (User Temp, System Temp, Update Cache, Shader Caches, Recycle Bin)
- ⚡ **Performance & Anti-Sluggishness** (Hardware-aware tweaks with live status)
- 📦 **UWP Debloater** (Scan and batch uninstall preloaded sponsored apps)
- 🚀 **Startup Optimization** (Audit registered autorun items)
- ⚡ **Flush RAM & DNS** (One-click latency boost)

### 2. Headless CLI Automation & SysPrep

All operations can also be run headlessly from PowerShell or scripts:

```powershell
# Deep clean temporary files, %TEMP%, Prefetch, and system caches:
.\target\release\winopt.exe --clean

# Flush DNS resolver cache and trim RAM process working sets:
.\target\release\winopt.exe --boost

# Display hardware profile, recommendations, and current tweak statuses:
.\target\release\winopt.exe --status

# Apply all safe hardware-aware optimizations immediately:
.\target\release\winopt.exe --optimize

# Scan and purge detected UWP bloatware packages:
.\target\release\winopt.exe --debloat

# Undo / Rollback all applied optimizations to their original state:
.\target\release\winopt.exe --rollback

# Force interactive console menu:
.\target\release\winopt.exe --cli
```

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
