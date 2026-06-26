#!/system/bin/sh
# LucaPro Support v2.1 - Installer Script

ui_print " "
ui_print " _                                     "
ui_print "| |                                    "
ui_print "| |    _   _  ___ __ _ _ __  _ __ ___  "
ui_print "| |   | | | |/ __/ _\` | '_ \\| '__/ _ \\ "
ui_print "| |___| |_| | (_| (_| | |_) | | | (_) |"
ui_print "\\_____/\\__,_|\\___\\__,_| .__/|_|  \\___/ "
ui_print "                      | |              "
ui_print "                      |_|              "
ui_print " "
ui_print "======================================="
ui_print "       LUCAPRO   S U P P O R T         "
ui_print "           PURE UTILITY v2.1           "
ui_print "======================================="

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
rm -rf /storage/emulated/0/Android/media/.lucapro_* /data/media/0/Android/media/.lucapro_* 2>/dev/null
rm -rf /data/media/0/Android/media/.lucapro/status* /data/media/0/Android/media/.lucapro/sysmon* 2>/dev/null
mkdir -p /data/media/0/Android/media/.lucapro && chmod 775 /data/media/0/Android/media/.lucapro
rm -rf "/data/adb/modules/lucapro/app_mappings.json"
rm -f "$MODPATH/action.sh" 2>/dev/null

# 2. ARCHITECTURE DETECTION & INSTALL
ui_print "- Installing architecture-specific daemon..."
if [ "$ARCH" = "arm64" ]; then
    ui_print "    Platform: ARM64 (arm64-v8a)"
    cp -f "$MODPATH/libs/arm64-v8a/lucapro_helper" "$MODPATH/system/bin/lucapro_helper"
elif [ "$ARCH" = "arm" ]; then
    ui_print "    Platform: ARM32 (armeabi-v7a)"
    cp -f "$MODPATH/libs/armeabi-v7a/lucapro_helper" "$MODPATH/system/bin/lucapro_helper"
else
    ui_print "    Platform: Unknown ($ARCH). Falling back to ARM64."
    cp -f "$MODPATH/libs/arm64-v8a/lucapro_helper" "$MODPATH/system/bin/lucapro_helper"
fi
rm -rf "$MODPATH/libs" 2>/dev/null

ui_print "[*] Setting up core permissions..."
set_perm_recursive "$MODPATH/system/bin" 0 2000 0755 0755
set_perm_recursive "$MODPATH/scripts" 0 0 0755 0755
set_perm "$MODPATH/post-fs-data.sh" 0 0 0755
set_perm "$MODPATH/service.sh" 0 0 0755
set_perm "$MODPATH/system_monitor.apk" 0 0 0644
set_perm "$MODPATH/toast.apk" 0 0 0644
set_perm "$MODPATH/system.prop" 0 0 0644

ui_print "[*] Patching SELinux contexts..."
chcon -R u:object_r:system_file:s0 "$MODPATH/scripts"
chcon u:object_r:system_file:s0 "$MODPATH/post-fs-data.sh"
chcon u:object_r:system_file:s0 "$MODPATH/service.sh"

ui_print "[*] Installing LucaPro Toast UI..."
if pm install "$MODPATH/toast.apk" >/dev/null 2>&1; then
    ui_print "    [+] SUCCESS: Toast UI installed!"
else
    ui_print "    [-] FAILED: Will auto-install on reboot."
fi

ui_print " "
ui_print "======================================="
ui_print " INSTALLATION COMPLETE!                "
ui_print " No Kernel Tweaks. Pure Support only.  "
ui_print "======================================="
