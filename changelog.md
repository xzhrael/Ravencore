# Ravencore Module Changelog

This changelog records the development history, features, and release notes for the Ravencore module.

---

## v1.0 (Stable Release)

### 1. Rebranding & Identity Refresh
- **Ravencore V1.0 Branding**: Rebranded the entire module from `LucaPro` to `Ravencore` across all scripts, WebUI elements, configurations, and documentation.
- **Identity Upgrade**: Integrated the new `RC` logo badge, clean `RAVENCORE SYSTEM V1.0` title, and version indicators.
- **Hidden Directory Structure**: Consolidates active status logs, safety locks, and user custom profiles inside a single hidden folder: `/data/media/0/Android/media/.ravencore/` to keep storage clean.

### 2. High-Performance WebUI
- **Modular Interface**: Built a high-performance Vanilla CSS and Javascript Webroot dashboard optimized for Android WebView performance.
- **System Resource Monitor**: Provides real-time CPU utilization, RAM usage, and battery temperature readings.
- **Log Viewers**: Displays active game preload events and helper daemon status logs in real-time with automatic scroll-to-bottom features.

### 3. Thermal & Game Management
- **Thermal Core Watcher**: Automatically detects and controls system thermal engines (`thermal-engine`, `mi_thermald`, `vendor.thermal-engine`, etc.) upon launching active games.
- **Safety Engine**: Battery monitor tracks temperatures every second; automatically temporarily restores standard thermal configurations if the battery reaches a threshold (>= 46°C) for device safety.
- **Active Game Compiler**: Features a one-click ART compiler optimization ("Optimize Active Games") via `bg-dexopt-job` for all games configured in Game Mode.

### 4. Charging & System Utilities
- **Fast Charge / Bypass Controls**: Toggleable high-speed battery charge rates and charging suspend options to bypass direct battery wear during intensive gaming.
- **One-Click Cleaning**: Dedicated manual cache and RAM cleaning tools that safely clean user directories and terminate non-essential background services.
- **Dynamic Refresh Rates**: Provides interface-driven controls to lock or reset refresh rates.
