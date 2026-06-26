#!/system/bin/sh
# Ravencore v1.0 - Utilities

# Standardize MODDIR for all scripts
MODDIR="/data/adb/modules/ravencore"
RAVENCORE_DIR="/data/media/0/Android/media/.ravencore"
RAVENCORE_LOG="$RAVENCORE_DIR/helper.log"
RAVENCORE_PROFILE="$RAVENCORE_DIR/profile"
RAVENCORE_CUSTOM="$RAVENCORE_DIR/custom"

MAX_LOG_SIZE=102400 # 100KB

alias cmd='/system/bin/cmd'
alias ravencore_helper='/system/bin/ravencore_helper'

write() {
    if [ -f "$1" ]; then
        chmod 644 "$1" 2>/dev/null
        echo "$2" > "$1" 2>/dev/null
    fi
}

log() {
    local level="INFO"
    local msg="$1"
    [ -n "$2" ] && { level="$1"; msg="$2"; }

    [ ! -d "$RAVENCORE_DIR" ] && mkdir -p "$RAVENCORE_DIR" && chmod 775 "$RAVENCORE_DIR"
    if [ -f "$RAVENCORE_LOG" ]; then
        local size=$(wc -c < "$RAVENCORE_LOG" 2>/dev/null || echo 0)
        if [ "$size" -gt "$MAX_LOG_SIZE" ]; then
            tail -n 500 "$RAVENCORE_LOG" > "${RAVENCORE_LOG}.tmp"
            mv "${RAVENCORE_LOG}.tmp" "$RAVENCORE_LOG"
            echo "[$(date +%H:%M:%S)] [SYS] Log auto-trimmed" >> "$RAVENCORE_LOG"
        fi
    fi
    echo "[$(date +%H:%M:%S)] [$level] $msg" >> "$RAVENCORE_LOG" 2>/dev/null
}

notify() {
    local title="$1"
    local body="$2"
    local combined="${title} - ${body}"
    su -lp 2000 -c "am start -n bellavita.toast/.MainActivity -e toasttext '$combined'" >/dev/null 2>&1
}
