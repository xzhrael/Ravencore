#!/system/bin/sh
# Ravencore v1.0 - Installer Script

ui_print " "
ui_print "======================================="
ui_print "              RAVENCORE                "
ui_print "             SYSTEM v1.0               "
ui_print "======================================="
ui_print " "

# 1. ENVIRONMENT DETECTION
ui_print "- Detecting environment..."
if [ -d "/data/adb/ksu" ]; then
    ENV="KernelSU"
elif [ -d "/data/adb/ap" ]; then
    ENV="APATCH"
else
    ENV="Magisk/Generic"
fi
ui_print " ROOT MANAGER       : $ENV"

ui_print "[*] Cleaning up old configs & legacy scripts..."
rm -rf /storage/emulated/0/Android/media/.ravencore_* /data/media/0/Android/media/.ravencore_* 2>/dev/null
rm -rf /data/media/0/Android/media/.ravencore/status* /data/media/0/Android/media/.ravencore/sysmon* 2>/dev/null
mkdir -p /data/media/0/Android/media/.ravencore && chmod 775 /data/media/0/Android/media/.ravencore
rm -rf "/data/adb/modules/ravencore/app_mappings.json"
rm -f "$MODPATH/action.sh" 2>/dev/null

# 2. ARCHITECTURE DETECTION & INSTALL
ui_print "- Installing architecture-specific daemon..."
if [ "$ARCH" = "arm64" ]; then
    ui_print "    Platform: ARM64 (arm64-v8a)"
    cp -f "$MODPATH/libs/arm64-v8a/ravencore_helper" "$MODPATH/system/bin/ravencore_helper"
elif [ "$ARCH" = "arm" ]; then
    ui_print "    Platform: ARM32 (armeabi-v7a)"
    cp -f "$MODPATH/libs/armeabi-v7a/ravencore_helper" "$MODPATH/system/bin/ravencore_helper"
else
    ui_print "    Platform: Unknown ($ARCH). Falling back to ARM64."
    cp -f "$MODPATH/libs/arm64-v8a/ravencore_helper" "$MODPATH/system/bin/ravencore_helper"
fi
rm -rf "$MODPATH/libs" 2>/dev/null

ui_print "[*] Setting up core permissions..."
set_perm_recursive "$MODPATH/system/bin" 0 2000 0755 0755
set_perm_recursive "$MODPATH/scripts" 0 0 0755 0755
set_perm "$MODPATH/post-fs-data.sh" 0 0 0755
set_perm "$MODPATH/service.sh" 0 0 0755
set_perm "$MODPATH/system_monitor.apk" 0 0 0644
set_perm "$MODPATH/toast.apk" 0 0 0644
[ -f "$MODPATH/overlay.apk" ] && set_perm "$MODPATH/overlay.apk" 0 0 0644
set_perm "$MODPATH/system.prop" 0 0 0644

ui_print "[*] Patching SELinux contexts..."
chcon -R u:object_r:system_file:s0 "$MODPATH/scripts"
chcon u:object_r:system_file:s0 "$MODPATH/post-fs-data.sh"
chcon u:object_r:system_file:s0 "$MODPATH/service.sh"

ui_print "[*] Installing Ravencore Toast UI..."
if pm install "$MODPATH/toast.apk" >/dev/null 2>&1; then
    ui_print "    [+] SUCCESS: Toast UI installed!"
else
    ui_print "    [-] FAILED: Will auto-install on reboot."
fi

if [ -f "$MODPATH/overlay.apk" ]; then
    ui_print "[*] Installing Ravencore Game Overlay..."
    if pm install "$MODPATH/overlay.apk" >/dev/null 2>&1; then
        cmd appops set ravencore.overlay SYSTEM_ALERT_WINDOW allow 2>/dev/null
        ui_print "    [+] SUCCESS: Game Overlay UI installed!"
    else
        ui_print "    [-] FAILED: Will auto-install on reboot."
    fi
fi

ui_print " "
ui_print "======================================="
ui_print " INSTALLATION COMPLETE!                "
ui_print " No Kernel Tweaks. Pure System only.   "
ui_print "======================================="
