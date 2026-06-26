#!/system/bin/sh
# LucaPro Support v2.1 - Uninstaller Script
# Reverting persistent changes and cleaning residue.

# 1. Kill daemon and monitor if running
pkill -f "lucapro_helper" 2>/dev/null
pkill -f "LucaProSysMon" 2>/dev/null

# Reset Battery Charging nodes to defaults
echo 0 > /sys/class/qcom-battery/idle_mode 2>/dev/null
echo 0 > /sys/class/power_supply/battery/input_suspend 2>/dev/null
echo 1 > /sys/class/power_supply/battery/charging_enabled 2>/dev/null

# 2. Reset Display Settings
cmd settings delete system min_refresh_rate 2>/dev/null
cmd settings delete system peak_refresh_rate 2>/dev/null
cmd settings delete system user_refresh_rate 2>/dev/null
cmd settings delete secure min_refresh_rate 2>/dev/null
cmd settings delete secure peak_refresh_rate 2>/dev/null
cmd settings delete secure user_refresh_rate 2>/dev/null

# 3. Restore App Permissions & Battery Settings (GMS)
cmd appops set com.google.android.gms RUN_IN_BACKGROUND allow 2>/dev/null
cmd appops set com.google.android.gms WAKE_LOCK allow 2>/dev/null
settings put global low_power 0 2>/dev/null
settings delete global battery_saver_constants 2>/dev/null

# 4. Wipe Data Residue (both FUSE and direct paths)
rm -rf /storage/emulated/0/Android/media/.lucapro 2>/dev/null
rm -rf /data/media/0/Android/media/.lucapro 2>/dev/null
rm -f /storage/emulated/0/Android/media/.lucapro_* /data/media/0/Android/media/.lucapro_* 2>/dev/null

# 5. Remove PID file
rm -f /data/adb/modules/lucapro/lucapro.pid 2>/dev/null

# 6. Uninstall Toast UI APK
pm uninstall bellavita.toast 2>/dev/null
