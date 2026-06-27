# Ravencore Module Changelog

This changelog records the development history, features, and release notes for the Ravencore module.

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
