#!/system/bin/sh
# LucaPro Support v2.1 — Early Boot Setup

MODDIR="/data/adb/modules/lucapro"

# 1. FIX PERMISSIONS & SELINUX (Crucial for Toast & Service)
find "$MODDIR" -type d -exec chmod 755 {} +
find "$MODDIR" -type f -name "*.sh" -exec chmod 755 {} +
find "$MODDIR" -type f -name "*.apk" -exec chmod 644 {} +
find "$MODDIR" -type f -name "*.prop" -exec chmod 644 {} +
chmod 755 "$MODDIR/system/bin/lucapro_helper"
chcon -R u:object_r:system_file:s0 "$MODDIR"
# Specifically for binary to allow it to execute shell commands
chcon u:object_r:toolbox_exec:s0 "$MODDIR/system/bin/lucapro_helper"

# 2. PREVENT KERNEL PANIC REMOVED

