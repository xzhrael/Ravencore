# LucaPro Module Changelog

This changelog records the complete development history, fixes, and improvements introduced during this engineering cycle.

---

## v2.1 (Latest Release)

### 1. Thermal Core Management
- **Inspired by `Rianixia-ThermalCore`**: Cloned and audited Rianixia's thermal engine control mechanisms to implement high-performance, safe temperature throttling bypasses.
- **Thermal Engine Defeat**: Stops Qualcomm, MediaTek, and Xiaomi system thermal daemons (`thermal-engine`, `mi_thermald`, `vendor.thermal-engine`, etc.) upon active game launching.
- **Sysfs Mitigation**: Automatically disables temperature throttling across all thermal zones (`/sys/class/thermal/thermal_zone*/mode`) during active gameplay.
- **Failsafe Design**: Safely restores all thermal zones to `enabled` and restarts all system thermal engines when the active game loses focus (>8s) or is closed.
- **Dynamic Safety Monitor (Semi-Safe)**: Watcher daemon monitors battery temperature (`BAT_TEMP`) every second. If temperature reaches critical threshold (>= 46°C), thermal engine and throttling zones are temporarily re-enabled (emergency brake) without manual performance/frequency capping, and automatically disabled again once temperature drops below 41°C.

### 2. User App list and Integration
- **Instant Package Resolution**: Integrated high-speed package mapping using POSIX `pm list packages -3` shell command.
- **Offline Resolution**: Resolves package names to app labels locally using a built-in database of 520+ popular packages (`apps.json`) with an automated string cleanup fallback.
- **Optimized UI Performance**: Removed heavy Base64 image decoding and binder cat payloads to guarantee smooth, lag-free UI rendering.

### 3. Dashboard and Identity Refresh
- **LUCAPRO V2.1 Title**: Replaced the legacy `HYBRID ENGINE` banner with the unified module title `LUCAPRO V2.1`.
- **Dynamic Helper Daemon PID**: Changed the sub-title badge from `LucaPro v2.0` to a dynamic `PID: [daemon_pid]` tag. The Rust helper daemon writes its active PID (`std::process::id()`) to `.lucapro/status` which is read and rendered in real-time by the WebUI.

### 4. Interactive UX Animations
- **Modal Transition**: Introduced a smooth opacity fade (`0.25s`) on the modal settings overlay.
- **Spring Entry**: Added a slide-up spring scaling animation (`scale(0.9) translateY(20px)` to `scale(1) translateY(0)`) to the modal settings card for a modern feel.
- **Accordion Expand**: Created a sliding, fade-in accordion transition on the customizations container when enabling `Game Mode`.

### 5. Kernel Compatibility & Bug Fixes
- **Fast Charge Fix**: Addressed charging speed throttling on Redmi Note 11 (SPES/SPESN) devices by adjusting charging control node defaults.
- **Shell Permissions Audit**: Replaced blanket `chmod 755` executions with targeted file and folder permissions (`chmod 755` for scripts, `chmod 644` for prop/apk resources).
- **Log Viewers Auto-scroll**: Implemented auto-scrolling to the bottom of the Preload and System Log viewers on new lines arrival.
- **WebUI Page Height Fix**: Resolved the excessive scrolling space on non-Home tabs by dynamically collapsing height (`height: 0`) and padding of inactive pages in CSS.
- **Generic Preload Logs**: Made the Asset Preload logs viewer generic (supports tracking preloading logs of any game package, not just MLBB) and renamed label to "Game Preload Status".
- **Log Text Styling**: Changed the main text color of the Preload and Realtime Log viewers to white for high contrast and readability, while keeping the tags (`[INFO]`, `[WARN]`, `[ERROR]`, etc. + newly added daemon tags `[SAFETY]`, `[GAME]`, `[SAVER]`) colorized.
- **Storage Subfolder Restructure**: Consolidated all runtime temporary and config files (`status`, `helper.log`, `custom`, `sysmon_status`, `sysmon.lock`) into a single unified hidden directory `/data/media/0/Android/media/.lucapro/` to prevent cluttering the user's main internal storage.
- **Generic ART Compiler**: Replaced the hardcoded MLBB compilation button with a dynamic compiler button ("Optimize Active Games") which compiles all game packages with Game Mode enabled.

---

## v2.0 (Stable release)

### 1. Unified Game Optimizer
- **Suite UI**: Designed a dedicated Game Optimizer tab with instant package searching, filtering, and customization.
- **Per-Game Overlay Settings**: Configures resolution scaling, asset preloading, and active game state mappings individually.

### 2. Bypass Charging & Battery Management
- **Auto Bypass**: Configured automatic bypass charging during active gameplay using vendor charging controls (`idle_mode`, `input_suspend`).
- **Battery Temperature Parsing**: Refactored battery temperature read scripts to support multiple kernel reporting formats.

### 3. Rust Helper Core
- **Engine Refactoring**: Migrated the core watcher daemon to a multithreaded Rust binary (`lucapro_helper`) to ensure memory safety, thread safety, and high-efficiency loops on target devices.
- **GMS and Standby Buckets**: Whitelisted Google Play Services (GMS) to preserve FCM push notifications while optimizing other background standby resources.

---

## v1.1 (Maintenance Release)

- **Apache 2.0 License**: Added official license documentation to the module package.
- **Refactored Utility Scripts**: Addressed compatibility issues across multiple Android versions by replacing syntax with POSIX-compliant logic.
- **Obfuscation**: Applied code protection and packaging constraints.
