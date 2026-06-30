# Ravencore Module Changelog

This changelog records the development history, features, and release notes for the Ravencore module.

---

## v1.1 (Single APK Consolidation & Performance)

### 1. 100% Self-Contained Architecture (Single APK)
- **Integrated Background Monitor (`SysMonMain`)**: Rewrote the Kotlin background system monitor into an optimized Java 8 class (`SysMonMain`) directly compiled inside `raven_engine.apk`.
- **Hidden API Restrictions Bypass**: Implemented `dalvik.system.VMRuntime` meta-reflection to whitelist all hidden system API lookups dynamically without external wrapper libraries.
- **D8 Compiler Compatibility**: Replaced all anonymous inner classes in Java modules with static nested classes (`ToastRemover`, `ShutdownHook`), bypassing R8/D8 dexing NullPointerExceptions.

### 2. Built-in Premium Toast Engine
- **Nothing OS-Styled Toast notifications**: Implemented custom floating notification drawer drawing logic directly inside `OverlayService` with a translucent background (`#E615161C`), rounded corners (`20dp`), monospace bold typography, and a smooth timeout removal.
- **Direct Broadcast System**: Upgraded `utils.sh` notification system and the Rust daemon's helper notifier to trigger system notifications via `am broadcast -a ravencore.intent.action.SHOW_TOAST --es text "..."`. Eliminates launcher drawer popup delay and Activity stack allocation.

### 3. Smarter CPU Temperature Scanning
- **Graded Priority CPU Thermal Watcher**: Rewrote the Rust helper daemon's `get_cpu_temp_celsius` function to actively loop and grade thermal zones.
- **SoC Junction & Big Core Prioritization**: Automatically prioritizes high-performance Big/Gold CPU cores and Qualcomm SoC junction sensors (`tsens_tz_sensor`, `soc`) over static, battery, or dummy zones, ensuring reactive temperature updates under intense workloads.

### 4. Code & Layout Simplification
- **Background-Only Raven Engine**: Removed all heavy GUI overlays, handle bars, gesture sensors, Choreographer FPS ticks, and overlay config switches from the Java service to minimize memory footprint to absolute zero during gameplay.
- **Instant Game Optimizer Loading**: Replaced the slow, asynchronous chunked application querying loop in the Game Optimizer with direct mapping of preloaded application list data, cutting redundant calls and rendering the interface instantly in under 1ms.
- **Native CSS Scroll-Snapping**: Replaced custom, heavy JavaScript pointer drag events for dashboard swipeable cards with native CSS scroll-snapping, reducing code size by ~50 lines of JS and achieving hardware-accelerated 60fps card transitions.
- **Package ID Migration**: Migrated background daemon and helper service package target to `ravencore.engine` for consistency.
- **Significantly Reduced Package Size**: Reduced workspace files and consolidated runtime processes to run cleanly in the background.

---

## v1.0 (Stable Release)

### 1. Rebranding & Identity Refresh
- **Ravencore V1.0 Branding**: Rebranded the entire module from `LucaPro` to `Ravencore` across all scripts, WebUI elements, configurations, and documentation.
- **Identity Upgrade**: Integrated the new `RC` logo badge, clean `RAVENCORE SYSTEM V1.0` title, and version indicators.
- **Hidden Directory Structure**: Consolidates active status logs, safety locks, and user custom profiles inside a single hidden folder: `/data/media/0/Android/media/.ravencore/` to keep storage clean.

### 2. High-Performance WebUI & Card-Style App List
- **High-Performance App Listing (Encore Method)**: Rewrote game query loading logic in the WebUI to process apps in chunks of 15 and yield the rendering thread for 10ms between cycles. Prevents WebView freezing and ensures smooth 60fps scrolling on mid-range devices.
- **Native Label & Icon Resolvers**: Integrated native KernelSU package batch info APIs (`ksu.getPackagesInfo`) and the native `ksu://icon/` protocol to fetch localized game labels and icons natively without relying on heavy databases.
- **Premium Card Layout (inspired by applist.jpg)**: Game list elements in the Optimizer page now render as premium rounded cards (`14px` border radius) with circular icons, horizontal badges, and clean navigation chevrons.
- **Memory Cleaner Performance Optimization**: Stripped all icon nodes and drawable indicators from the Memory Cleaner app list (`#apps`) to optimize WebView RAM utilization.

### 3. Hidden & Silent Game Overlay ("Raven Engine")
- **Hidden App Launcher Drawer**: Removed the launcher category from the overlay manifest so the app remains completely hidden from the user's home screen and app drawer.
- **Attractive, Unobtrusive Branding**: Rebranded the package to **`Raven Engine`** with a custom glowing neon-red raven head icon. It presents itself as a core system daemon in the Android Settings App Manager list to prevent accidental uninstalls.
- **Silent Root Permission Granting**: Automatically grants the Display over Other Apps (`SYSTEM_ALERT_WINDOW`) permission on module installation and device boot via root appops commands, bypassing annoying user configuration overlays.
- **Conditional Visibility**: The game overlay handle and stats HUD are conditioned in the Rust daemon to only trigger and appear when the "Game Mode" toggle is explicitly enabled for the running package.

### 4. Thermal & Game Management
- **Thermal Core Watcher**: Automatically detects and controls system thermal engines (`thermal-engine`, `mi_thermald`, `vendor.thermal-engine`, etc.) upon launching active games.
- **Safety Engine**: Battery monitor tracks temperatures every second; automatically temporarily restores standard thermal configurations if the battery reaches a threshold (>= 46°C) for device safety.
- **Active Game Compiler**: Features a one-click ART compiler optimization ("Optimize Active Games") via `bg-dexopt-job` for all games configured in Game Mode.

### 5. Charging & System Utilities
- **Fast Charge / Bypass Controls**: Toggleable high-speed battery charge rates and charging suspend options to bypass direct battery wear during intensive gaming.
- **One-Click Cleaning**: Dedicated manual cache and RAM cleaning tools that safely clean user directories and terminate non-essential background services.
- **Dynamic Refresh Rates**: Provides interface-driven controls to lock or reset refresh rates.

### 6. Repository Safety & Git Health
- **Comprehensive Gitignore Mapping**: Excludes heavy Java/Rust build folders (`build/`, `target/`), local debug keystores, certificates, logs, and ZIP rilis from remote pushes to avoid security breaches and credential leaks.
