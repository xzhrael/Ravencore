#!/system/bin/sh
# Ravencore v1.0 — Boot Orchestrator

MODDIR="/data/adb/modules/ravencore"
. "$MODDIR/scripts/utils.sh"

# Override storage paths for boot context (FUSE not mounted in daemon namespace)
RAVENCORE_DIR="/data/media/0/Android/media/.ravencore"
RAVENCORE_LOG="$RAVENCORE_DIR/helper.log"
RAVENCORE_CUSTOM="$RAVENCORE_DIR/custom"
export PATH="/system/bin:/vendor/bin:/system/xbin:$PATH"

# Wait for Android UI
while [ "$(getprop sys.boot_completed)" != "1" ]; do sleep 5; done
sleep 5

# Wait for storage decryption / mount (FBE)
count=0
while [ ! -d "/storage/emulated/0/Android/media" ] && [ $count -lt 30 ]; do
    sleep 2
    count=$((count + 1))
done

# Ensure storage dir exists for daemon
mkdir -p "$RAVENCORE_DIR" 2>/dev/null

# Reset hardware nodes to ensure start-up safety (Fix manual start issue)
if [ -f "/sys/class/power_supply/battery/step_charging_enabled" ]; then
    chmod 644 /sys/class/power_supply/battery/step_charging_enabled 2>/dev/null
    echo 1 > /sys/class/power_supply/battery/step_charging_enabled 2>/dev/null
fi

if [ -f "/sys/class/power_supply/battery/fast_charge" ]; then
    chmod 644 /sys/class/power_supply/battery/fast_charge 2>/dev/null
    echo 0 > /sys/class/power_supply/battery/fast_charge 2>/dev/null
fi

if [ -f "/sys/class/power_supply/battery/fastcharge_mode" ]; then
    chmod 644 /sys/class/power_supply/battery/fastcharge_mode 2>/dev/null
    echo 0 > /sys/class/power_supply/battery/fastcharge_mode 2>/dev/null
fi

# Restore kernel charging controllers to default
if [ -f "/sys/class/power_supply/battery/sw_jeita_enabled" ]; then
    chmod 644 /sys/class/power_supply/battery/sw_jeita_enabled 2>/dev/null
    echo 1 > /sys/class/power_supply/battery/sw_jeita_enabled 2>/dev/null
fi

if [ -f "/sys/class/power_supply/battery/restrict_chg" ]; then
    chmod 644 /sys/class/power_supply/battery/restrict_chg 2>/dev/null
    echo 0 > /sys/class/power_supply/battery/restrict_chg 2>/dev/null
fi

if [ -f "/sys/class/power_supply/battery/system_temp_level" ]; then
    chmod 644 /sys/class/power_supply/battery/system_temp_level 2>/dev/null
    echo 0 > /sys/class/power_supply/battery/system_temp_level 2>/dev/null
fi

if [ -f "/sys/class/qcom-battery/idle_mode" ]; then
    chmod 644 /sys/class/qcom-battery/idle_mode 2>/dev/null
    echo 0 > /sys/class/qcom-battery/idle_mode 2>/dev/null
fi
if [ -f "/sys/class/power_supply/battery/input_suspend" ]; then
    chmod 644 /sys/class/power_supply/battery/input_suspend 2>/dev/null
    echo 0 > /sys/class/power_supply/battery/input_suspend 2>/dev/null
fi
if [ -f "/sys/class/power_supply/battery/charging_enabled" ]; then
    chmod 644 /sys/class/power_supply/battery/charging_enabled 2>/dev/null
    echo 1 > /sys/class/power_supply/battery/charging_enabled 2>/dev/null
fi

# Restore GMS permissions to default at boot based on active_saver state
ACTIVE_SAVER=$(grep "^active_saver=" "$RAVENCORE_CUSTOM" 2>/dev/null | cut -d= -f2)
if [ "$ACTIVE_SAVER" = "1" ]; then
    cmd appops set com.google.android.gms RUN_IN_BACKGROUND allow 2>/dev/null
    cmd appops set com.google.android.gms WAKE_LOCK allow 2>/dev/null
    for pkg in com.whatsapp com.instagram.android com.zhiliaoapp.musically com.ss.android.ugc.trill com.google.android.gm; do
        dumpsys deviceidle whitelist +$pkg 2>/dev/null
        am set-standby-bucket $pkg active 2>/dev/null
    done
    settings put global battery_saver_constants "advertising_is_enabled=false,datasaver_is_enabled=false,enable_night_mode=true,gps_mode=2,force_all_apps_standby=false,enable_firewall=false,vibration_disabled=true,animation_disabled=false,launch_boost_disabled=false,optional_sensors_disabled=true,force_background_check=true" 2>/dev/null
    settings put global low_power 1 2>/dev/null
else
    for pkg in com.whatsapp com.instagram.android com.zhiliaoapp.musically com.ss.android.ugc.trill com.google.android.gm; do
        dumpsys deviceidle whitelist -$pkg 2>/dev/null
    done
    cmd appops set com.google.android.gms RUN_IN_BACKGROUND allow 2>/dev/null
    cmd appops set com.google.android.gms WAKE_LOCK allow 2>/dev/null
    settings put global low_power 0 2>/dev/null
    settings delete global battery_saver_constants 2>/dev/null
fi

# Install Toast UI if missing
if ! pm path bellavita.toast >/dev/null 2>&1; then
    log "INFO" "Installing Ravencore Toast UI..."
    pm install "$MODDIR/toast.apk" >/dev/null 2>&1
fi

# Install Overlay App if missing
if ! pm path ravencore.overlay >/dev/null 2>&1; then
    if [ -f "$MODDIR/overlay.apk" ]; then
        log "INFO" "Installing Ravencore Overlay App..."
        pm install "$MODDIR/overlay.apk" >/dev/null 2>&1
        cmd appops set ravencore.overlay SYSTEM_ALERT_WINDOW allow 2>/dev/null
    fi
fi

# Start Overlay service if installed
if pm path ravencore.overlay >/dev/null 2>&1; then
    cmd appops set ravencore.overlay SYSTEM_ALERT_WINDOW allow 2>/dev/null
    log "INFO" "Starting Ravencore Overlay Service..."
    am startforegroundservice -n ravencore.overlay/.OverlayService >/dev/null 2>&1
fi


# 1. START SYSTEM DAEMON
pkill -f "ravencore_helper" 2>/dev/null
pkill -f "RavencoreSysMon" 2>/dev/null
sleep 1
# Launch from MODDIR path (correct SELinux context vs /system/bin overlay)
nohup "$MODDIR/system/bin/ravencore_helper" monitor >/dev/null 2>&1 &

log "INFO" "Boot orchestration complete (v1.0)"
notify "Ravencore" "Ravencore v1.0 Ignited!"
