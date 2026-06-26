#!/system/bin/sh
# LucaPro Support v2.1 - Core Wrapper
# Specialized for Downscale, Preload, and Charging

MODDIR="/data/adb/modules/lucapro"
. "$MODDIR/scripts/utils.sh"

# ═══════════════════════════════════════════════════════════════
# 1. UTILITY INTERCEPTOR (Charging & Refresh Rate)
# ═══════════════════════════════════════════════════════════════
if [ "$1" = "apply_utility" ]; then
    [ ! -d "$LUCAPRO_DIR" ] && mkdir -p "$LUCAPRO_DIR" && chmod 775 "$LUCAPRO_DIR"
    [ ! -f "$LUCAPRO_CUSTOM" ] && touch "$LUCAPRO_CUSTOM"
    grep -v "^${2}=" "$LUCAPRO_CUSTOM" > "${LUCAPRO_CUSTOM}.tmp" 2>/dev/null
    mv "${LUCAPRO_CUSTOM}.tmp" "$LUCAPRO_CUSTOM" 2>/dev/null
    echo "$2=$3" >> "$LUCAPRO_CUSTOM"

    case "$2" in
        "refresh_rate")
            if [ "$3" = "60" ] || [ "$3" = "90" ]; then
                cmd settings put system min_refresh_rate $3 2>/dev/null
                cmd settings put system peak_refresh_rate $3 2>/dev/null
                cmd settings put system user_refresh_rate $3 2>/dev/null
            else
                cmd settings delete system min_refresh_rate 2>/dev/null
                cmd settings delete system peak_refresh_rate 2>/dev/null
                cmd settings delete system user_refresh_rate 2>/dev/null
            fi
            ;;
        "mlbb_downscale")
            if [ "$3" = "0" ]; then
                cmd device_config delete game_overlay com.mobile.legends
                cmd game mode 0 com.mobile.legends
            fi
            ;;
        opt_downscale_*)
            if [ "$3" = "0" ]; then
                pkg=$(echo "$2" | sed 's/^opt_downscale_//')
                cmd device_config delete game_overlay "$pkg" 2>/dev/null
                cmd game mode 0 "$pkg" 2>/dev/null
            fi
            ;;
        "active_saver")
            if [ "$3" = "1" ]; then
                # Apply battery saver tweaks (<8%/h active drain optimization)
                # GMS (com.google.android.gms) MUST remain allowed for instant FCM push notifications
                cmd appops set com.google.android.gms RUN_IN_BACKGROUND allow 2>/dev/null
                cmd appops set com.google.android.gms WAKE_LOCK allow 2>/dev/null
                
                # Whitelist crucial communication apps to avoid Doze and standby bucket limits
                for pkg in com.whatsapp com.instagram.android com.zhiliaoapp.musically com.ss.android.ugc.trill com.google.android.gm; do
                    dumpsys deviceidle whitelist +$pkg 2>/dev/null
                    am set-standby-bucket $pkg active 2>/dev/null
                done
                
                # Safe battery saver constants (Vibration off, animations/launch-boost on, background restrictions on)
                settings put global battery_saver_constants "advertising_is_enabled=false,datasaver_is_enabled=false,enable_night_mode=true,gps_mode=2,force_all_apps_standby=false,enable_firewall=false,vibration_disabled=true,animation_disabled=false,launch_boost_disabled=false,optional_sensors_disabled=true,force_background_check=true" 2>/dev/null
                settings put global low_power 1 2>/dev/null
            else
                # Revert tweaks (100% Compatibility / Disabled state)
                for pkg in com.whatsapp com.instagram.android com.zhiliaoapp.musically com.ss.android.ugc.trill com.google.android.gm; do
                    dumpsys deviceidle whitelist -$pkg 2>/dev/null
                done
                cmd appops set com.google.android.gms RUN_IN_BACKGROUND allow 2>/dev/null
                cmd appops set com.google.android.gms WAKE_LOCK allow 2>/dev/null
                settings put global low_power 0 2>/dev/null
                settings delete global battery_saver_constants 2>/dev/null
            fi
            ;;
        "disable_thermal")
            if [ "$3" = "1" ]; then
                log "WARN" "Disabling system thermal engines & zones..."
                stop thermal-engine 2>/dev/null
                stop vendor.thermal-engine 2>/dev/null
                stop mi_thermald 2>/dev/null
                stop thermald 2>/dev/null
                stop thermal 2>/dev/null
                stop thermal_manager 2>/dev/null
                stop vendor.thermal 2>/dev/null
                stop vendor.thermal-hal 2>/dev/null
                stop vendor.thermal-hal-1-0 2>/dev/null
                stop vendor.thermal-hal-2-0 2>/dev/null
                
                for zone in /sys/class/thermal/thermal_zone*; do
                    if [ -f "$zone/mode" ]; then
                        chmod 644 "$zone/mode" 2>/dev/null
                        echo "disabled" > "$zone/mode" 2>/dev/null
                    fi
                done
                notify "LucaPro" "Thermal Core Disabled. Overheat Warning!"
            else
                log "INFO" "Restoring system thermal configurations..."
                for zone in /sys/class/thermal/thermal_zone*; do
                    if [ -f "$zone/mode" ]; then
                        chmod 644 "$zone/mode" 2>/dev/null
                        echo "enabled" > "$zone/mode" 2>/dev/null
                    fi
                done
                
                start thermal-engine 2>/dev/null
                start vendor.thermal-engine 2>/dev/null
                start mi_thermald 2>/dev/null
                start thermald 2>/dev/null
                start thermal 2>/dev/null
                start thermal_manager 2>/dev/null
                start vendor.thermal 2>/dev/null
                start vendor.thermal-hal 2>/dev/null
                start vendor.thermal-hal-1-0 2>/dev/null
                start vendor.thermal-hal-2-0 2>/dev/null
                notify "LucaPro" "Thermal Core Restored."
            fi
            ;;
    esac
    exit 0
fi

# ═══════════════════════════════════════════════════════════════
# 2. LEGACY FALLBACK (Disabled)
# ═══════════════════════════════════════════════════════════════
case "$1" in
    eco|balanced|game|custom|apply_support)
        log "INFO" "Legacy/Unsupported call ignored ($1)"
        ;;
esac
