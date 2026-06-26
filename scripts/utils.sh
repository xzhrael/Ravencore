#!/system/bin/sh
# LucaPro Support v2.1 - Utilities

# Standardize MODDIR for all scripts
MODDIR="/data/adb/modules/lucapro"
LUCAPRO_DIR="/data/media/0/Android/media/.lucapro"
LUCAPRO_LOG="$LUCAPRO_DIR/helper.log"
LUCAPRO_PROFILE="$LUCAPRO_DIR/profile"
LUCAPRO_CUSTOM="$LUCAPRO_DIR/custom"

MAX_LOG_SIZE=102400 # 100KB

alias cmd='/system/bin/cmd'
alias lucapro_helper='/system/bin/lucapro_helper'

write() {
    if [ -f "$1" ]; then
        chmod 644 "$1" 2>/dev/null
        echo "$2" > "$1" 2>/dev/null
    fi
}

battery_temp_value() {
    local raw=$(cat /sys/class/power_supply/battery/temp 2>/dev/null || echo 300)
    if [ "$raw" -ge 10000 ]; then
        echo $((raw / 1000))
    elif [ "$raw" -ge 1000 ]; then
        echo $((raw / 100))
    elif [ "$raw" -ge 100 ]; then
        echo $((raw / 10))
    else
        echo "$raw"
    fi
}

log() {
    local level="INFO"
    local msg="$1"
    [ -n "$2" ] && { level="$1"; msg="$2"; }

    [ ! -d "$LUCAPRO_DIR" ] && mkdir -p "$LUCAPRO_DIR" && chmod 775 "$LUCAPRO_DIR"
    if [ -f "$LUCAPRO_LOG" ]; then
        local size=$(wc -c < "$LUCAPRO_LOG" 2>/dev/null || echo 0)
        if [ "$size" -gt "$MAX_LOG_SIZE" ]; then
            tail -n 500 "$LUCAPRO_LOG" > "${LUCAPRO_LOG}.tmp"
            mv "${LUCAPRO_LOG}.tmp" "$LUCAPRO_LOG"
            echo "[$(date +%H:%M:%S)] [SYS] Log auto-trimmed" >> "$LUCAPRO_LOG"
        fi
    fi
    echo "[$(date +%H:%M:%S)] [$level] $msg" >> "$LUCAPRO_LOG" 2>/dev/null
}

clean_cache() {
    log "INFO" "Starting Manual Cache Clean..."
    rm -rf /data/cache/*
    find /data/media/0/Android/data -name "cache" -type d -exec rm -rf {} + 2>/dev/null
    find /data/media/0/Android/data -name "CodeCache" -type d -exec rm -rf {} + 2>/dev/null
    log "INFO" "Cache Clean Complete"
    notify "LucaPro" "Junk files & Cache cleaned."
}

kill_bg() {
    log "INFO" "Starting Manual RAM Clean..."
    local exclude="$1"
    
    # Auto-detect currently focused app to prevent force-stopping the active game/app
    if [ -z "$exclude" ]; then
        exclude=$(grep "^FOCUSED_APP=" /data/media/0/Android/media/.lucapro/status 2>/dev/null | cut -d= -f2)
    fi
    if [ -z "$exclude" ]; then
        exclude=$(grep "focused_app" /data/media/0/Android/media/.lucapro/sysmon_status 2>/dev/null | awk '{print $2}')
    fi
    if [ -z "$exclude" ]; then
        exclude=$(dumpsys window 2>/dev/null | grep -E 'mCurrentFocus|mFocusedApp' | grep -oE '[a-zA-Z0-9._]+/[a-zA-Z0-9._]+' | cut -d/ -f1 | head -n 1)
    fi

    local count=0
    for pkg in $(pm list packages -3 | cut -d: -f2); do
        if [ -n "$exclude" ] && [ "$pkg" = "$exclude" ]; then
            continue
        fi
        case "$pkg" in
            *lucapro*|*launcher*|*systemui*|*com.mobile.legends*|*whatsapp*|*instagram*|*discord*|*ksu*|*ksunext*) continue ;;
        esac
        am force-stop "$pkg" 2>/dev/null && count=$((count + 1))
    done
    log "INFO" "RAM Clean Complete ($count apps stopped)"
    notify "LucaPro" "$count background apps stopped."
}

notify() {
    local title="$1"
    local body="$2"
    local combined="${title} - ${body}"
    su -lp 2000 -c "am start -n bellavita.toast/.MainActivity -e toasttext '$combined'" >/dev/null 2>&1
}
