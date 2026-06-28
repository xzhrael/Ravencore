use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write, Read};
use std::os::raw::{c_int, c_void};
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// --- CONSTANTS ---
const MEDIA_DIR: &str = "/data/media/0/Android/media/.ravencore/";
const CUSTOM_CONFIG: &str = "/data/media/0/Android/media/.ravencore/custom";
const LOG_FILE: &str = "/data/media/0/Android/media/.ravencore/helper.log";
const PID_FILE: &str = "/data/adb/modules/ravencore/ravencore.pid";
const MAX_LOG_SIZE: u64 = 102400;

// --- ATOMICS & GLOBALS ---
static SHOULD_EXIT: AtomicBool = AtomicBool::new(false);
static MLBB_PROCESS_ACTIVE: AtomicBool = AtomicBool::new(false);
static GLOBAL_REFRESH_RATE_MODE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(1); // 0 = auto/other, 1 = smart (default)
static LOG_MUTEX: Mutex<()> = Mutex::new(());
static LOG_WRITE_COUNT: Mutex<i32> = Mutex::new(0);
static PRELOADED_MAPPINGS: Mutex<Vec<(usize, usize)>> = Mutex::new(Vec::new());
static CPU_TEMP_PATH: Mutex<Option<String>> = Mutex::new(None);
static FPS_PATH: Mutex<Option<String>> = Mutex::new(None);

// --- LIBC RAW FFI ---
const PROT_READ: c_int = 1;
const MAP_SHARED: c_int = 1;
const MADV_WILLNEED: c_int = 3;
const LOCK_EX: c_int = 2;
const LOCK_NB: c_int = 4;

extern "C" {
    fn flock(fd: c_int, op: c_int) -> c_int;
    fn mmap(addr: *mut c_void, len: usize, prot: c_int, flags: c_int, fd: c_int, offset: i64) -> *mut c_void;
    fn munmap(addr: *mut c_void, len: usize) -> c_int;
    fn madvise(addr: *mut c_void, len: usize, advice: c_int) -> c_int;
    fn mlock(addr: *const c_void, len: usize) -> c_int;
    fn signal(sig: c_int, handler: unsafe extern "C" fn(c_int)) -> *mut c_void;
    fn getpid() -> c_int;
}

// --- SIGNAL HANDLER ---
unsafe extern "C" fn signal_handler(_sig: c_int) {
    let _ = fs::write("/sys/class/qcom-battery/idle_mode", "0");
    let _ = fs::write("/sys/class/power_supply/battery/input_suspend", "0");
    let _ = fs::write("/sys/class/power_supply/battery/charging_enabled", "1");
    let _ = fs::remove_file(PID_FILE);
    let _ = std::process::Command::new("pkill").args(["-f", "RavencoreSysMon"]).output();
    let _ = fs::remove_file("/data/media/0/Android/media/.ravencore/sysmon_status");
    let _ = fs::remove_file("/data/media/0/Android/media/.ravencore/sysmon.lock");
    let _ = fs::remove_file("/data/media/0/Android/media/.ravencore/status");
    std::process::exit(0);
}

// --- LOGGING ---
fn write_log(level: &str, msg: &str) {
    if level == "DETECT" || level == "CONFIG" || level == "SAFETY" || level == "WORKAROUND" {
        if let Ok(content) = fs::read_to_string("/data/media/0/Android/media/.ravencore/custom") {
            if !content.contains("opt_verbose_logs=1") {
                return;
            }
        } else {
            return;
        }
    }
    let _lock = LOG_MUTEX.lock().unwrap();
    let _ = fs::create_dir_all(MEDIA_DIR);
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO).as_secs();
    
    // Format timestamp
    let ts = format_time(now);
    
    if let Ok(mut log) = OpenOptions::new().create(true).append(true).open(LOG_FILE) {
        let _ = writeln!(log, "[{}] [{}] {}", ts, level, msg);
    }
    
    let mut count = LOG_WRITE_COUNT.lock().unwrap();
    *count += 1;
    if *count >= 100 {
        *count = 0;
        if let Ok(metadata) = fs::metadata(LOG_FILE) {
            if metadata.len() > MAX_LOG_SIZE {
                rotate_log();
            }
        }
    }
}

fn format_time(timestamp: u64) -> String {
    let secs = timestamp % 60;
    let mins = (timestamp / 60) % 60;
    let hours = ((timestamp / 3600) + 7) % 24; // +7 for WIB timezone
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}

fn rotate_log() {
    if let Ok(file) = File::open(LOG_FILE) {
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();
        if lines.len() > 500 {
            if let Ok(mut outfile) = File::create(LOG_FILE) {
                for line in &lines[lines.len() - 500..] {
                    let _ = writeln!(outfile, "{}", line);
                }
            }
        }
    }
}

// --- SYSFS HELPERS ---
fn write_node(path: &str, value: &str) -> bool {
    if fs::write(path, value).is_ok() {
        return true;
    }
    if let Ok(meta) = fs::metadata(path) {
        let mut perms = meta.permissions();
        perms.set_readonly(false);
        let _ = fs::set_permissions(path, perms);
    }
    fs::write(path, value).is_ok()
}

fn read_node(path: &str) -> String {
    fs::read_to_string(path)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn safe_stoi(s: &str, fallback: i32) -> i32 {
    s.parse::<i32>().unwrap_or(fallback)
}

fn get_battery_temp_celsius() -> i32 {
    let mut raw_str = read_node("/sys/class/power_supply/battery/temp");
    if raw_str.is_empty() {
        raw_str = read_node("/sys/class/power_supply/battery/batt_temp");
    }
    if raw_str.is_empty() {
        raw_str = read_node("/sys/class/thermal/thermal_zone0/temp");
    }
    let raw = safe_stoi(&raw_str, 0);
    if raw == 0 {
        return 30;
    }
    if raw.abs() > 10000 {
        raw / 1000
    } else if raw.abs() > 1000 {
        raw / 100
    } else if raw.abs() > 100 {
        raw / 10
    } else {
        raw
    }
}

fn get_cpu_temp_celsius() -> i32 {
    // Check cache first
    if let Ok(cache) = CPU_TEMP_PATH.lock() {
        if let Some(ref path) = *cache {
            let raw = read_node(path);
            if !raw.is_empty() {
                if let Ok(val) = raw.parse::<i32>() {
                    return if val.abs() > 1000 { val / 1000 } else { val };
                }
            }
        }
    }

    // Scan all thermal zones and grade them by priority based on their type
    // High priority: gold cores (big CPU), tsens_tz_sensor (SoC junction)
    // Medium priority: silver cores (LITTLE CPU), other cpu-related
    // Low priority: ap-thermal, thermal_zone0
    let mut best_path = None;
    let mut best_priority = -1;

    for i in 0..80 {
        let type_path = format!("/sys/class/thermal/thermal_zone{}/type", i);
        let temp_path = format!("/sys/class/thermal/thermal_zone{}/temp", i);
        if Path::new(&type_path).exists() && Path::new(&temp_path).exists() {
            let tz_type = read_node(&type_path).to_lowercase();
            let temp_content = read_node(&temp_path);
            if let Ok(val) = temp_content.parse::<i32>() {
                let temp = if val.abs() > 1000 { val / 1000 } else { val };
                // Ensure the temp is currently active and within a normal CPU operational range (25°C to 95°C)
                if temp >= 25 && temp <= 95 {
                    let mut priority = 0;
                    if tz_type.contains("gold") || tz_type.contains("big") || tz_type.contains("prime") {
                        priority = 4; // Big/Gold cores are best representation of hot CPU
                    } else if tz_type.contains("cpu-1") || tz_type.contains("cpu1") {
                        priority = 3;
                    } else if tz_type.contains("cpu-0") || tz_type.contains("cpu0") || tz_type.contains("silver") {
                        priority = 2; // LITTLE/Silver cores
                    } else if tz_type.contains("tsens_tz_sensor") || tz_type.contains("soc") {
                        priority = 2; // SoC temperature sensor
                    } else if tz_type.contains("cpu") {
                        priority = 2; // General CPU
                    } else if tz_type.contains("ap-") || tz_type.contains("chg") {
                        priority = 1;
                    }
                    
                    if priority > best_priority {
                        best_priority = priority;
                        best_path = Some(temp_path);
                    }
                }
            }
        }
    }

    if let Some(path) = best_path {
        let raw = read_node(&path);
        if let Ok(val) = raw.parse::<i32>() {
            let temp = if val.abs() > 1000 { val / 1000 } else { val };
            if let Ok(mut cache) = CPU_TEMP_PATH.lock() {
                *cache = Some(path);
            }
            return temp;
        }
    }

    // Fallback to zone 0 or 1 if nothing prioritized matches
    for path in &["/sys/class/thermal/thermal_zone1/temp", "/sys/class/thermal/thermal_zone0/temp"] {
        if Path::new(path).exists() {
            let raw = read_node(path);
            if let Ok(val) = raw.parse::<i32>() {
                let temp = if val.abs() > 1000 { val / 1000 } else { val };
                if temp >= 20 && temp <= 95 {
                    return temp;
                }
            }
        }
    }
    0
}

fn get_fps() -> i32 {
    // Check cache first
    if let Ok(cache) = FPS_PATH.lock() {
        if let Some(ref path) = *cache {
            let content = read_node(path);
            if !content.is_empty() {
                let cleaned = content.replace("fps:", "").replace("fps", "").trim().to_string();
                if let Ok(val) = cleaned.parse::<f32>() {
                    return val as i32;
                }
                if let Ok(val) = cleaned.parse::<i32>() {
                    return val;
                }
            }
        }
    }

    let paths = [
        "/sys/class/drm/sde-crtc-0/measured_fps",
        "/sys/class/mi_display/disp-DSI-0/measured_fps",
        "/sys/class/mi_display/disp-DSI-0/fps-info",
        "/sys/class/graphics/fb0/measured_fps",
        "/sys/devices/platform/soc/ae00000.qcom,mdss_mdp/measured_fps",
        "/sys/class/drm/card0-DSI-1/measured_fps",
        "/sys/class/drm/card0-DSI-1/fps",
        "/sys/class/graphics/fb0/fps",
    ];
    for path in &paths {
        if Path::new(path).exists() {
            let content = read_node(path);
            if !content.is_empty() {
                let cleaned = content.replace("fps:", "").replace("fps", "").trim().to_string();
                if let Ok(val) = cleaned.parse::<f32>() {
                    if let Ok(mut cache) = FPS_PATH.lock() {
                        *cache = Some(path.to_string());
                    }
                    return val as i32;
                }
                if let Ok(val) = cleaned.parse::<i32>() {
                    if let Ok(mut cache) = FPS_PATH.lock() {
                        *cache = Some(path.to_string());
                    }
                    return val;
                }
            }
        }
    }
    0
}

// --- CONFIG PARSER ---
fn parse_config() -> HashMap<String, String> {
    let mut config = HashMap::new();
    if let Ok(file) = File::open(CUSTOM_CONFIG) {
        let reader = BufReader::new(file);
        for line in reader.lines().filter_map(|l| l.ok()) {
            if let Some(sep) = line.find('=') {
                let key = line[..sep].trim().to_string();
                let val = line[sep + 1..].trim().to_string();
                if !key.is_empty() {
                    config.insert(key, val);
                }
            }
        }
    }
    config
}

fn get_config_value(config: &HashMap<String, String>, key: &str, default_val: &str) -> String {
    config.get(key).cloned().unwrap_or_else(|| default_val.to_string())
}

// --- READ LOGS FOR WEBUI ---
fn get_last_logs() -> (Vec<String>, Vec<String>) {
    let mut preload_logs = Vec::new();
    let mut system_logs = Vec::new();
    
    if let Ok(file) = File::open(LOG_FILE) {
        let reader = BufReader::new(file);
        let all_lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).filter(|l| !l.is_empty()).collect();
        
        for line in all_lines.iter().rev() {
            let lower = line.to_lowercase();
            let is_preload = lower.contains("preload") || lower.contains("mlbb");
            
            if is_preload {
                if preload_logs.len() < 2 {
                    preload_logs.push(line.clone());
                }
            } else {
                if system_logs.len() < 6 {
                    system_logs.push(line.clone());
                }
            }
            if preload_logs.len() >= 2 && system_logs.len() >= 6 {
                break;
            }
        }
    }
    preload_logs.reverse();
    system_logs.reverse();
    (preload_logs, system_logs)
}

// --- WRITE CONSOLIDATED STATUS ---
fn update_status_file(tick: i32, focused_pkg: &str, focused_pid: i32, screen_awake: i32, battery_saver: i32, zen_mode: i32) {
    let mut cpu_freq = read_node("/sys/devices/system/cpu/cpufreq/policy4/scaling_cur_freq");
    if cpu_freq.is_empty() || cpu_freq == "0" {
        cpu_freq = read_node("/sys/devices/system/cpu/cpufreq/policy0/scaling_cur_freq");
    }
    if cpu_freq.is_empty() {
        cpu_freq = "0".to_string();
    }

    let mut gpu_freq = read_node("/sys/class/kgsl/kgsl-3d0/gpuclk");
    if gpu_freq.is_empty() || gpu_freq == "0" {
        gpu_freq = read_node("/sys/class/devfreq/kgsl-3d0/cur_freq");
    }
    if gpu_freq.is_empty() {
        gpu_freq = "0".to_string();
    }

    let mut bat_cap = read_node("/sys/class/power_supply/battery/capacity");
    if bat_cap.is_empty() {
        bat_cap = "0".to_string();
    }

    let mut bat_temp = read_node("/sys/class/power_supply/battery/temp");
    if bat_temp.is_empty() {
        bat_temp = read_node("/sys/class/power_supply/battery/batt_temp");
    }
    if bat_temp.is_empty() {
        bat_temp = read_node("/sys/class/thermal/thermal_zone0/temp");
    }
    if bat_temp.is_empty() {
        bat_temp = "0".to_string();
    }

    let mut health = read_node("/sys/class/power_supply/battery/health");
    if health.is_empty() {
        health = "Good".to_string();
    }

    let mut current = read_node("/sys/class/power_supply/battery/current_now");
    if current.is_empty() {
        current = "0".to_string();
    }

    let mut voltage = read_node("/sys/class/power_supply/battery/voltage_now");
    if voltage.is_empty() {
        voltage = "0".to_string();
    }

    let mut bypass = read_node("/sys/class/qcom-battery/idle_mode");
    if bypass.is_empty() || bypass == "0" {
        bypass = read_node("/sys/class/power_supply/battery/input_suspend");
    }
    if bypass.is_empty() || bypass == "0" {
        let val = read_node("/sys/class/power_supply/battery/charging_enabled");
        if val == "0" {
            bypass = "1".to_string();
        } else if val == "1" {
            bypass = "0".to_string();
        }
    }
    if bypass.is_empty() {
        bypass = "0".to_string();
    }

    // System Meminfo
    let mut mem_total = 0;
    let mut mem_avail = 0;
    let mut swap_total = 0;
    let mut swap_free = 0;
    
    if tick % 5 == 0 {
        if let Ok(file) = File::open("/proc/meminfo") {
            let reader = BufReader::new(file);
            for line in reader.lines().filter_map(|l| l.ok()) {
                if line.starts_with("MemTotal:") {
                    mem_total = parse_mem_value(&line);
                } else if line.starts_with("MemAvailable:") {
                    mem_avail = parse_mem_value(&line);
                } else if line.starts_with("SwapTotal:") {
                    swap_total = parse_mem_value(&line);
                } else if line.starts_with("SwapFree:") {
                    swap_free = parse_mem_value(&line);
                }
            }
        }
    }

    // Storage info via df
    let mut storage_total = 0;
    let mut storage_used = 0;
    if tick % 5 == 0 {
        if let Ok(output) = std::process::Command::new("df").arg("/data").output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                let mut lines = text.lines();
                lines.next();
                if let Some(line) = lines.next() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        storage_total = parts[1].parse::<u64>().unwrap_or(0);
                        storage_used = parts[2].parse::<u64>().unwrap_or(0);
                    }
                }
            }
        }
    }

    // Uptime
    let mut uptime_sec = 0;
    if let Ok(uptime_str) = fs::read_to_string("/proc/uptime") {
        if let Some(first) = uptime_str.split_whitespace().next() {
            uptime_sec = first.parse::<f64>().unwrap_or(0.0) as u64;
        }
    }

    let (preload_logs, system_logs) = if tick % 2 == 0 {
        get_last_logs()
    } else {
        (Vec::new(), Vec::new())
    };

    // Scan for Dexopt log status
    let mut dexopt_time = "None".to_string();
    if let Ok(log_content) = fs::read_to_string(LOG_FILE) {
        for line in log_content.lines().rev() {
            if line.contains("[DEXOPT]") && line.contains("Daily Dexopt job complete") {
                if let Some(idx) = line.find(']') {
                    dexopt_time = line[..idx+1].to_string();
                }
                break;
            }
        }
    }

    // Atomic write to status file
    let tmp_file = format!("{}{}", MEDIA_DIR, "status.tmp");
    let status_file = format!("{}{}", MEDIA_DIR, "status");
    if let Ok(mut out) = File::create(&tmp_file) {
        let _ = writeln!(out, "CPU_FREQ={}", cpu_freq);
        let _ = writeln!(out, "GPU_FREQ={}", gpu_freq);
        let _ = writeln!(out, "BAT_CAP={}", bat_cap);
        let _ = writeln!(out, "BAT_TEMP={}", bat_temp);
        let _ = writeln!(out, "HEALTH={}", health);
        let _ = writeln!(out, "CURRENT={}", current);
        let _ = writeln!(out, "VOLTAGE={}", voltage);
        let _ = writeln!(out, "BYPASS={}", bypass);
        if tick % 5 == 0 {
            let _ = writeln!(out, "MEM_TOTAL={}", mem_total);
            let _ = writeln!(out, "MEM_AVAIL={}", mem_avail);
            let _ = writeln!(out, "SWAP_TOTAL={}", swap_total);
            let _ = writeln!(out, "SWAP_FREE={}", swap_free);
            let _ = writeln!(out, "STORAGE_TOTAL={}", storage_total);
            let _ = writeln!(out, "STORAGE_USED={}", storage_used);
        }
        let _ = writeln!(out, "UPTIME={}", uptime_sec);
        let _ = writeln!(out, "DEXOPT={}", dexopt_time);
        
        let _ = writeln!(out, "PRELOAD_LOG_COUNT={}", preload_logs.len());
        for (i, log) in preload_logs.iter().enumerate() {
            let _ = writeln!(out, "PRELOAD_LOG_{}={}", i, log);
        }
        let _ = writeln!(out, "SYSTEM_LOG_COUNT={}", system_logs.len());
        for (i, log) in system_logs.iter().enumerate() {
            let _ = writeln!(out, "SYSTEM_LOG_{}={}", i, log);
        }
        
        let _ = writeln!(out, "SCREEN_AWAKE={}", screen_awake);
        let _ = writeln!(out, "BATTERY_SAVER={}", battery_saver);
        let _ = writeln!(out, "ZEN_MODE={}", zen_mode);
        let _ = writeln!(out, "FOCUSED_APP={}", focused_pkg);
        let _ = writeln!(out, "FOCUSED_PID={}", focused_pid);
        let _ = writeln!(out, "DAEMON_PID={}", std::process::id());
        let _ = writeln!(out, "MODULE_ACTIVE=1");
        
        drop(out);
        let _ = fs::rename(tmp_file, status_file);
    }
}

fn parse_mem_value(line: &str) -> i64 {
    let mut num = 0;
    for c in line.chars() {
        if c.is_ascii_digit() {
            num = num * 10 + (c as i64 - '0' as i64);
        }
    }
    num
}

// --- CMD WORKER QUEUE ---
fn cmd_worker_thread(rx: std::sync::mpsc::Receiver<String>) {
    while !SHOULD_EXIT.load(Ordering::Relaxed) {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(cmd) => {
                if !cmd.is_empty() {
                    let _ = std::process::Command::new("sh").args(["-c", &cmd]).output();
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                break;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
        }
    }
}

// --- GAME MODE DOWNSCALE ---
fn apply_game_mode(pkg: &str, scale: f32, enable: bool, tx: &Sender<String>) {
    if enable {
        let cmd1 = format!("cmd device_config put game_overlay {} mode=2", pkg);
        let cmd2 = format!("cmd game set --mode 2 --downscale {:.2} {}", scale, pkg);
        let cmd3 = format!("cmd game mode 2 {}", pkg);
        let _ = tx.send(cmd1);
        let _ = tx.send(cmd2);
        let _ = tx.send(cmd3);
    } else {
        let _ = tx.send(format!("cmd device_config delete game_overlay {}", pkg));
        let _ = tx.send(format!("cmd game set --mode 2 --downscale default {}", pkg));
        let _ = tx.send(format!("cmd game mode 0 {}", pkg));
    }
}

fn is_game_optimized(config: &HashMap<String, String>, pkg: &str) -> bool {
    if get_config_value(config, &format!("opt_game_mode_{}", pkg), "0") == "1" {
        return true;
    }
    if get_config_value(config, &format!("opt_preload_{}", pkg), "0") == "1" {
        return true;
    }
    if get_config_value(config, &format!("opt_disable_thermal_{}", pkg), "0") == "1" {
        return true;
    }
    if pkg == "com.mobile.legends" {
        if get_config_value(config, "mlbb_downscale", "0") == "1" {
            return true;
        }
    }
    let key = format!("opt_downscale_{}", pkg);
    get_config_value(config, &key, "0") == "1"
}

fn get_game_scale(config: &HashMap<String, String>, pkg: &str) -> i32 {
    if pkg == "com.mobile.legends" {
        if let Some(scale_str) = config.get("mlbb_scale") {
            return safe_stoi(scale_str, 100);
        }
    }
    let key = format!("opt_scale_{}", pkg);
    if let Some(scale_str) = config.get(&key) {
        return safe_stoi(scale_str, 100);
    }
    100
}

// --- RUST NATIVE PRELOADING ---
fn get_total_ram_kb() -> u64 {
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.contains("MemTotal:") {
                return parse_mem_value(line) as u64;
            }
        }
    }
    4194304
}

fn clear_preloaded_mappings() {
    if let Ok(mut mappings) = PRELOADED_MAPPINGS.lock() {
        for (addr, len) in mappings.drain(..) {
            unsafe {
                munmap(addr as *mut c_void, len);
            }
        }
    }
}

fn preload_file(path: &Path, mlock_enabled: bool, bytes_loaded: &mut u64, files_loaded: &mut i32) {
    if let Ok(file) = File::open(path) {
        if let Ok(meta) = file.metadata() {
            let len = meta.len() as usize;
            if len > 0 {
                let fd = file.as_raw_fd();
                unsafe {
                    let addr = mmap(std::ptr::null_mut(), len, PROT_READ, MAP_SHARED, fd, 0);
                    if addr != std::ptr::null_mut() && addr as isize != -1 {
                        madvise(addr, len, MADV_WILLNEED);
                        if mlock_enabled {
                            mlock(addr, len);
                            if let Ok(mut mappings) = PRELOADED_MAPPINGS.lock() {
                                mappings.push((addr as usize, len));
                            }
                        } else {
                            munmap(addr, len);
                        }
                        *bytes_loaded += len as u64;
                        *files_loaded += 1;
                    }
                }
            }
        }
    }
}

fn preload_dir_recursive(dir_path: &Path, mlock_enabled: bool, limit: i32, files_loaded: &mut i32, bytes_loaded: &mut u64) {
    if *files_loaded >= limit {
        return;
    }
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            if *files_loaded >= limit {
                break;
            }
            let path = entry.path();
            if let Ok(meta) = fs::symlink_metadata(&path) {
                if meta.is_dir() {
                    preload_dir_recursive(&path, mlock_enabled, limit, files_loaded, bytes_loaded);
                } else if meta.is_file() {
                    preload_file(&path, mlock_enabled, bytes_loaded, files_loaded);
                }
            }
        }
    }
}

fn preload_mlbb() {
    let mut base = "/data/media/0/Android/data/com.mobile.legends/files/dragon2017/assets".to_string();
    if !Path::new(&base).exists() {
        base = "/data/media/0/Android/data/com.mobile.legends/files/unity_assets_files".to_string();
        if !Path::new(&base).exists() {
            write_log("WARN", "MLBB assets not found, skip preload");
            return;
        }
    }

    let total_ram = get_total_ram_kb();
    let mut limit = 1000;
    let mut lock_ram = false;
    
    if total_ram > 5500000 { // 6GB RAM or higher (typically reports ~5.6GB available)
        limit = 4000;
        lock_ram = true;
    } else if total_ram > 3500000 { // 4GB RAM
        limit = 2000;
        lock_ram = false;
    }
    
    write_log("INFO", &format!("Starting Native Preload (Limit: {}, Lock: {})...", limit, lock_ram));
    notify("Ravencore", "Preloading assets to RAM...");
    
    let mut files_loaded = 0;
    let mut bytes_loaded = 0;
    
    let scenes_path = format!("{}/Scenes", base);
    let bundles_path = format!("{}/AssetBundles", base);
    
    if Path::new(&scenes_path).exists() {
        preload_dir_recursive(Path::new(&scenes_path), lock_ram, limit, &mut files_loaded, &mut bytes_loaded);
    }
    if files_loaded < limit && Path::new(&bundles_path).exists() {
        preload_dir_recursive(Path::new(&bundles_path), lock_ram, limit, &mut files_loaded, &mut bytes_loaded);
    }
    if files_loaded < limit {
        preload_dir_recursive(Path::new(&base), lock_ram, limit, &mut files_loaded, &mut bytes_loaded);
    }
    
    let mb = bytes_loaded / (1024 * 1024);
    write_log("INFO", &format!("Preload complete: {} files (~{} MB)", files_loaded, mb));
    notify("Ravencore", &format!("Preloaded {} assets (~{} MB) to RAM", files_loaded, mb));
}

fn preload_generic_game(pkg: &str) {
    let base = format!("/data/media/0/Android/data/{}/files", pkg);
    if !Path::new(&base).exists() {
        write_log("WARN", &format!("Game assets not found for {}, skip preload", pkg));
        return;
    }
    let total_ram = get_total_ram_kb();
    let mut limit = 1000;
    let mut lock_ram = false;
    
    if total_ram > 5500000 {
        limit = 4000;
        lock_ram = true;
    } else if total_ram > 3500000 {
        limit = 2000;
        lock_ram = false;
    }
    
    write_log("INFO", &format!("Starting Native Preload for {} (Limit: {}, Lock: {})...", pkg, limit, lock_ram));
    notify("Ravencore", &format!("Preloading {} assets to RAM...", pkg));
    
    let mut files_loaded = 0;
    let mut bytes_loaded = 0;
    
    preload_dir_recursive(Path::new(&base), lock_ram, limit, &mut files_loaded, &mut bytes_loaded);
    
    let mb = bytes_loaded / (1024 * 1024);
    write_log("INFO", &format!("Preload complete for {}: {} files (~{} MB)", pkg, files_loaded, mb));
    notify("Ravencore", &format!("Preloaded {} assets (~{} MB) to RAM", files_loaded, mb));
}

// --- NATIVE NOTIFICATION & UTILITIES ---
fn notify(title: &str, body: &str) {
    if let Ok(content) = fs::read_to_string("/data/media/0/Android/media/.ravencore/custom") {
        if content.contains("opt_notifications=0") {
            return;
        }
    }
    let sanitized = format!("{} - {}", title, body).replace('\'', "");
    let _ = std::process::Command::new("am")
        .args(["broadcast", "-a", "ravencore.intent.action.SHOW_TOAST", "--es", "text", &sanitized])
        .output();
}

fn get_mem_available_kb() -> i64 {
    if let Ok(file) = File::open("/proc/meminfo") {
        let reader = BufReader::new(file);
        for line in reader.lines().filter_map(|l| l.ok()) {
            if line.starts_with("MemAvailable:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return parts[1].parse::<i64>().unwrap_or(0);
                }
            }
        }
    }
    0
}

fn dir_size_bytes(path: &Path) -> u64 {
    let mut total: u64 = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_dir() {
                total += dir_size_bytes(&p);
            } else if let Ok(meta) = p.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

fn kill_bg(exclude: &str) {
    write_log("INFO", "Starting Native RAM Clean...");
    let mut actual_exclude = exclude.to_string();
    
    // Auto-detect currently focused app if exclude is empty
    if actual_exclude.is_empty() {
        if let Ok(content) = fs::read_to_string("/data/media/0/Android/media/.ravencore/status") {
            for line in content.lines() {
                if line.starts_with("FOCUSED_APP=") {
                    actual_exclude = line.replace("FOCUSED_APP=", "").trim().to_string();
                    break;
                }
            }
        }
    }
    if actual_exclude.is_empty() {
        if let Ok(content) = fs::read_to_string("/data/media/0/Android/media/.ravencore/sysmon_status") {
            for line in content.lines() {
                if line.starts_with("focused_app ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        actual_exclude = parts[1].to_string();
                    }
                    break;
                }
            }
        }
    }
    
    let mem_before = get_mem_available_kb();
    let mut count = 0;
    if let Ok(output) = std::process::Command::new("pm").args(["list", "packages", "-3"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some(pkg) = line.strip_prefix("package:") {
                let pkg = pkg.trim();
                if pkg.is_empty() {
                    continue;
                }
                if !actual_exclude.is_empty() && pkg == actual_exclude {
                    continue;
                }
                // Exclude system and critical communication packages
                if pkg.contains("ravencore") || pkg.contains("launcher") || pkg.contains("systemui")
                    || pkg.contains("com.mobile.legends") || pkg.contains("whatsapp") || pkg.contains("instagram")
                    || pkg.contains("discord") || pkg.contains("ksu") || pkg.contains("ksunext") {
                    continue;
                }
                if std::process::Command::new("am").args(["force-stop", pkg]).status().is_ok() {
                    count += 1;
                }
            }
        }
    }
    let mem_after = get_mem_available_kb();
    let freed_mb = (mem_after - mem_before).max(0) / 1024;
    write_log("INFO", &format!("RAM Clean Complete ({} apps stopped, {} MB freed)", count, freed_mb));
    notify("Ravencore", &format!("Cleaned {} MB — {} apps stopped", freed_mb, count));
}

fn clean_cache() {
    write_log("INFO", "Starting Native Cache Clean...");
    
    // Measure size before cleaning
    let mut size_before: u64 = dir_size_bytes(Path::new("/data/cache"));
    let data_dir_path = Path::new("/data/media/0/Android/data");
    if let Ok(entries) = fs::read_dir(data_dir_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let pkg_dir = entry.path();
                let cache_dir = pkg_dir.join("cache");
                if cache_dir.exists() {
                    size_before += dir_size_bytes(&cache_dir);
                }
                let code_cache_dir = pkg_dir.join("CodeCache");
                if code_cache_dir.exists() {
                    size_before += dir_size_bytes(&code_cache_dir);
                }
            }
        }
    }
    
    // Clear /data/cache/*
    if let Ok(entries) = fs::read_dir("/data/cache") {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                let _ = fs::remove_dir_all(&path);
            } else {
                let _ = fs::remove_file(&path);
            }
        }
    }
    
    // Flat scan traversal of Android/data directories to delete cache folders
    let data_dir = Path::new("/data/media/0/Android/data");
    if let Ok(entries) = fs::read_dir(data_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let pkg_dir = entry.path();
                
                let cache_dir = pkg_dir.join("cache");
                if cache_dir.exists() && cache_dir.is_dir() {
                    let _ = fs::remove_dir_all(&cache_dir);
                }
                
                let code_cache_dir = pkg_dir.join("CodeCache");
                if code_cache_dir.exists() && code_cache_dir.is_dir() {
                    let _ = fs::remove_dir_all(&code_cache_dir);
                }
            }
        }
    }
    
    // Calculate total storage freed
    let size_after: u64 = ["/data/cache"].iter().map(|p| dir_size_bytes(Path::new(p))).sum();
    let cleaned_mb = size_before.saturating_sub(size_after) / (1024 * 1024);
    write_log("INFO", &format!("Cache Clean Complete ({} MB cleaned)", cleaned_mb));
    notify("Ravencore", &format!("Cleaned {} MB cache", cleaned_mb));
}

fn apply_disable_thermal(enable: bool) {
    if enable {
        write_log("WARN", "Disabling system thermal engines & zones...");
        let services = [
            "thermal-engine", "vendor.thermal-engine", "mi_thermald", "thermald",
            "thermal", "thermal_manager", "vendor.thermal", "vendor.thermal-hal",
            "vendor.thermal-hal-1-0", "vendor.thermal-hal-2-0"
        ];
        for srv in &services {
            let _ = std::process::Command::new("stop").arg(srv).output();
        }
        
        // Traverse thermal zones
        if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with("thermal_zone") {
                    let mode_path = format!("/sys/class/thermal/{}/mode", name);
                    if Path::new(&mode_path).exists() {
                        let _ = write_node(&mode_path, "disabled");
                    }
                }
            }
        }
        notify("Ravencore", "Thermal Core Disabled. Overheat Warning!");
    } else {
        write_log("INFO", "Restoring system thermal configurations...");
        if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with("thermal_zone") {
                    let mode_path = format!("/sys/class/thermal/{}/mode", name);
                    if Path::new(&mode_path).exists() {
                        let _ = write_node(&mode_path, "enabled");
                    }
                }
            }
        }
        
        let services = [
            "thermal-engine", "vendor.thermal-engine", "mi_thermald", "thermald",
            "thermal", "thermal_manager", "vendor.thermal", "vendor.thermal-hal",
            "vendor.thermal-hal-1-0", "vendor.thermal-hal-2-0"
        ];
        for srv in &services {
            let _ = std::process::Command::new("start").arg(srv).output();
        }
        notify("Ravencore", "Thermal Core Restored.");
    }
}

// --- PROCESS RUNNING CHECK ---
fn check_process_running(name: &str) -> bool {
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.filter_map(|e| e.ok()) {
            let name_str = entry.file_name().to_string_lossy().into_owned();
            if name_str.chars().all(|c| c.is_ascii_digit()) {
                let cmdline_path = format!("/proc/{}/cmdline", name_str);
                if let Ok(mut file) = File::open(&cmdline_path) {
                    let mut buf = vec![0u8; 256];
                    if let Ok(bytes) = file.read(&mut buf) {
                        if bytes > 0 {
                            let cmdline = match buf.iter().position(|&x| x == 0) {
                                Some(pos) => String::from_utf8_lossy(&buf[..pos]),
                                None => String::from_utf8_lossy(&buf[..bytes])
                            };
                            if cmdline.contains(name) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

// --- SYSTEM MONITOR SCHEDULING ---
fn ensure_sysmon_running() {
    // Periodically grant overlay permission to avoid OS reset overrides
    let _ = std::process::Command::new("cmd").args(["appops", "set", "ravencore.overlay", "SYSTEM_ALERT_WINDOW", "allow"]).output();
    let _ = std::process::Command::new("appops").args(["set", "ravencore.overlay", "SYSTEM_ALERT_WINDOW", "allow"]).output();
    let _ = std::process::Command::new("cmd").args(["appops", "set", "bellavita.toast", "SYSTEM_ALERT_WINDOW", "allow"]).output();
    let _ = std::process::Command::new("appops").args(["set", "bellavita.toast", "SYSTEM_ALERT_WINDOW", "allow"]).output();

    // Ensure ravencore.overlay service is running
    if !check_process_running("ravencore.overlay") {
        write_log("INFO", "Spawning ravencore.overlay service...");
        let _ = std::process::Command::new("am")
            .args(["startforegroundservice", "-n", "ravencore.overlay/.OverlayService"])
            .output();
    }

    if !check_process_running("RavencoreSysMon") {
        write_log("INFO", "Spawning Raven Engine SysMon daemon...");
        let _ = std::process::Command::new("pkill").args(["-f", "RavencoreSysMon"]).output();
        thread::sleep(Duration::from_millis(100));
        
        let spawned = std::process::Command::new("app_process")
            .args([
                "-Djava.class.path=/data/adb/modules/ravencore/raven_engine.apk",
                "/",
                "--nice-name=RavencoreSysMon",
                "ravencore.overlay.SysMonMain",
                "/data/media/0/Android/media/.ravencore/sysmon_status",
                "/data/media/0/Android/media/.ravencore/sysmon.lock"
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
            
        match spawned {
            Ok(_) => write_log("INFO", "Raven Engine SysMon spawned successfully"),
            Err(e) => write_log("ERROR", &format!("Failed to spawn Raven Engine SysMon: {}", e)),
        }
    }
}

// --- PARSE SYSMON STATUS FILE ---
struct SysMonStatus {
    focused_pkg: String,
    focused_pid: i32,
    _focused_uid: i32,
    screen_awake: i32,
    battery_saver: i32,
    zen_mode: i32,
}

fn is_valid_package_name(pkg: &str) -> bool {
    if pkg.is_empty() || pkg.len() > 128 {
        return false;
    }
    pkg.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_')
}

fn read_sysmon_status() -> Option<SysMonStatus> {
    let path = "/data/media/0/Android/media/.ravencore/sysmon_status";
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);
    let mut status = SysMonStatus {
        focused_pkg: "none".to_string(),
        focused_pid: 0,
        _focused_uid: 0,
        screen_awake: 1,
        battery_saver: 0,
        zen_mode: 0,
    };
    
    let mut found_focused = false;
    for line_result in reader.lines() {
        if let Ok(line) = line_result {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            match parts[0] {
                "focused_app" => {
                    if parts.len() >= 2 {
                        let pkg_name = parts[1].to_string();
                        if is_valid_package_name(&pkg_name) {
                            status.focused_pkg = pkg_name;
                            found_focused = true;
                        }
                    }
                    if parts.len() >= 3 {
                        status.focused_pid = parts[2].parse::<i32>().unwrap_or(0);
                    }
                    if parts.len() >= 4 {
                        status._focused_uid = parts[3].parse::<i32>().unwrap_or(0);
                    }
                }
                "screen_awake" => {
                    if parts.len() >= 2 {
                        status.screen_awake = parts[1].parse::<i32>().unwrap_or(1);
                    }
                }
                "battery_saver" => {
                    if parts.len() >= 2 {
                        status.battery_saver = parts[1].parse::<i32>().unwrap_or(0);
                    }
                }
                "zen_mode" => {
                    if parts.len() >= 2 {
                        status.zen_mode = parts[1].parse::<i32>().unwrap_or(0);
                    }
                }
                _ => {}
            }
        }
    }
    if !found_focused {
        return None;
    }
    Some(status)
}

fn apply_active_saver_cgroups(enable: bool, default_bg: &str, default_sys_bg: &str) {
    if enable {
        // Restrict background CPU cgroups
        let _ = fs::write("/dev/cpuctl/background/cpu.shares", "52");
        let _ = fs::write("/dev/cpuctl/background/cpu.weight", "1");
        let _ = fs::write("/dev/cpuctl/background/cpu.uclamp.max", "20");
        let _ = fs::write("/dev/cpuctl/background/cpu.uclamp.min", "0");
        
        let _ = fs::write("/dev/cpuctl/system-background/cpu.shares", "104");
        let _ = fs::write("/dev/cpuctl/system-background/cpu.weight", "10");
        let _ = fs::write("/dev/cpuctl/system-background/cpu.uclamp.max", "30");
        let _ = fs::write("/dev/cpuctl/system-background/cpu.uclamp.min", "0");
        
        // Lock background to efficiency cores (Core 0-3)
        let _ = fs::write("/dev/cpuset/background/cpus", "0-3");
        let _ = fs::write("/dev/cpuset/system-background/cpus", "0-3");
    } else {
        // Restore background defaults
        let _ = fs::write("/dev/cpuctl/background/cpu.shares", "1024");
        let _ = fs::write("/dev/cpuctl/background/cpu.weight", "100");
        let _ = fs::write("/dev/cpuctl/background/cpu.uclamp.max", "max");
        
        let _ = fs::write("/dev/cpuctl/system-background/cpu.shares", "1024");
        let _ = fs::write("/dev/cpuctl/system-background/cpu.weight", "100");
        let _ = fs::write("/dev/cpuctl/system-background/cpu.uclamp.max", "max");
        
        let _ = fs::write("/dev/cpuset/background/cpus", default_bg);
        let _ = fs::write("/dev/cpuset/system-background/cpus", default_sys_bg);
    }
}

// --- MAIN LOOP ---
fn monitor_loop() {
    let pid_file_path = Path::new(PID_FILE);
    
    // Load gamelist.txt for auto bypass charging
    let mut gamelist = Vec::new();
    if let Ok(file) = File::open("/data/adb/modules/ravencore/gamelist.txt") {
        let reader = BufReader::new(file);
        for line in reader.lines().filter_map(|l| l.ok()) {
            let trimmed = line.trim().to_lowercase();
            if !trimmed.is_empty() {
                gamelist.push(trimmed);
            }
        }
    }
    
    let mut last_bypass_state = false;
    
    let file_lock = OpenOptions::new()
        .write(true)
        .create(true)
        .open(pid_file_path);
        
    if file_lock.is_err() {
        eprintln!("Failed to open PID file: {}", PID_FILE);
        return;
    }
    let mut file = file_lock.unwrap();
    let fd = file.as_raw_fd();
    
    unsafe {
        if flock(fd, LOCK_EX | LOCK_NB) < 0 {
            eprintln!("Already running (cannot lock PID file)");
            return;
        }
    }
    
    let pid = unsafe { getpid() };
    let _ = file.set_len(0);
    let _ = writeln!(file, "{}", pid);
    
    let (tx, rx) = channel::<String>();
    let tx_clone1 = tx.clone();
    
    thread::spawn(move || cmd_worker_thread(rx));
    
    write_log("INFO", "Engine Started v1.1 (Rust)");
    notify("Ravencore", "Engine Active");
    
    // Spawn Java monitor daemon immediately
    ensure_sysmon_running();
    
    let default_bg_cpus = fs::read_to_string("/dev/cpuset/background/cpus").unwrap_or_else(|_| "0-7".to_string()).trim().to_string();
    let default_sys_bg_cpus = fs::read_to_string("/dev/cpuset/system-background/cpus").unwrap_or_else(|_| "0-7".to_string()).trim().to_string();
    let mut last_screen_state = 1;
    let mut last_active_saver_enabled = false;
    
    let mut tick = 0;
    let mut last_refresh_pkg = String::new();
    let mut last_config_mtime: i64 = 0;
    let mut cached_config: HashMap<String, String> = HashMap::new();
    let mut prev_game_states: HashMap<String, (String, String)> = HashMap::new();
    let mut prev_zram_optimize: Option<String> = None;
    let mut prev_refresh_rate: Option<String> = None;
    let mut last_dexopt_day: i32 = -1;
    let mut throttled = false;
    let mut prev_fast_charge_state: Option<bool> = None;
    let mut prev_throttled_state: Option<bool> = None;
    let mut mlbb_logic_running = false;
    let mut not_active_ticks = 0;
    let mut last_sysmon_check = 0;
    
    let mut prev_focused_pkg = "none".to_string();
    let mut prev_focused_pid = 0;
    let mut prev_screen_awake = 1;
    let mut prev_battery_saver = 0;
    let mut prev_zen_mode = 0;
    let mut last_optimized_pid = 0;
    let mut thermal_currently_disabled = false;
    
    while !SHOULD_EXIT.load(Ordering::Relaxed) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO).as_secs() as i64;
        
        // Ensure system_monitor is running (check every 10s)
        if tick - last_sysmon_check >= 10 {
            ensure_sysmon_running();
            last_sysmon_check = tick;
        }
        
        // Parse current system state from system_monitor output (with fallback cache to prevent file lock OOM loops)
        let sysmon = read_sysmon_status();
        let (focused_pkg, focused_pid, screen_awake, battery_saver, zen_mode) = match sysmon {
            Some(s) => {
                prev_focused_pkg = s.focused_pkg.clone();
                prev_focused_pid = s.focused_pid;
                prev_screen_awake = s.screen_awake;
                prev_battery_saver = s.battery_saver;
                prev_zen_mode = s.zen_mode;
                (s.focused_pkg, s.focused_pid, s.screen_awake, s.battery_saver, s.zen_mode)
            }
            None => (prev_focused_pkg.clone(), prev_focused_pid, prev_screen_awake, prev_battery_saver, prev_zen_mode),
        };
        
        // Configuration caching to reduce I/O
        let mut config_changed = false;
        if let Ok(metadata) = fs::metadata(CUSTOM_CONFIG) {
            let mtime = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH)
                .duration_since(SystemTime::UNIX_EPOCH).unwrap_or(Duration::ZERO).as_secs() as i64;
                
            if mtime != last_config_mtime {
                last_config_mtime = mtime;
                cached_config = parse_config();
                config_changed = true;
            }
        } else {
            if !cached_config.is_empty() {
                cached_config.clear();
                config_changed = true;
            }
            last_config_mtime = 0;
        }
        
        let config = cached_config.clone();

        // Game Active Check
        let active = is_game_optimized(&config, &focused_pkg);
        let prev_active = MLBB_PROCESS_ACTIVE.load(Ordering::Relaxed);
        if active && !prev_active {
            write_log("DETECT", &format!("Game active: {}", focused_pkg));
        } else if !active && prev_active {
            write_log("DETECT", "Game gone");
        }
        MLBB_PROCESS_ACTIVE.store(active, Ordering::SeqCst);
        
        // Smart Refresh Rate based on focused app
        let mode = GLOBAL_REFRESH_RATE_MODE.load(Ordering::Relaxed);
        if mode == 1 {
            if focused_pkg != last_refresh_pkg {
                last_refresh_pkg = focused_pkg.clone();
                let is_high_refresh = gamelist.contains(&focused_pkg.to_lowercase());
                    
                if is_high_refresh {
                    let _ = std::process::Command::new("cmd").args(["settings", "put", "system", "min_refresh_rate", "90"]).output();
                    let _ = std::process::Command::new("cmd").args(["settings", "put", "system", "peak_refresh_rate", "90"]).output();
                    let _ = std::process::Command::new("cmd").args(["settings", "put", "system", "user_refresh_rate", "90"]).output();
                } else {
                    let _ = std::process::Command::new("cmd").args(["settings", "put", "system", "min_refresh_rate", "60"]).output();
                    let _ = std::process::Command::new("cmd").args(["settings", "put", "system", "peak_refresh_rate", "60"]).output();
                    let _ = std::process::Command::new("cmd").args(["settings", "put", "system", "user_refresh_rate", "60"]).output();
                }
            }
        }
        
        // Apply Config Changes
        if config_changed {
            let mut current_game_states: HashMap<String, (String, String)> = HashMap::new();
            
            // Handle backwards compatibility for com.mobile.legends
            let mlbb_ds = get_config_value(&config, "mlbb_downscale", "0");
            let mlbb_sc = get_config_value(&config, "mlbb_scale", "100");
            current_game_states.insert("com.mobile.legends".to_string(), (mlbb_ds, mlbb_sc));
            
            // Handle dynamic game packages
            for (key, val) in &config {
                if let Some(pkg) = key.strip_prefix("opt_downscale_") {
                    let scale_key = format!("opt_scale_{}", pkg);
                    let scale_val = get_config_value(&config, &scale_key, "100");
                    current_game_states.insert(pkg.to_string(), (val.clone(), scale_val));
                } else if let Some(pkg) = key.strip_prefix("opt_scale_") {
                    let ds_key = format!("opt_downscale_{}", pkg);
                    let ds_val = get_config_value(&config, &ds_key, "0");
                    current_game_states.insert(pkg.to_string(), (ds_val, val.clone()));
                }
            }

            for (pkg, (cur_ds, cur_sc)) in &current_game_states {
                let (prev_ds, prev_sc) = prev_game_states.get(pkg).cloned().unwrap_or_else(|| ("0".to_string(), "100".to_string()));
                if cur_ds != &prev_ds || cur_sc != &prev_sc {
                    if cur_ds == "1" {
                        let scale = safe_stoi(cur_sc, 100) as f32 / 100.0;
                        let scale = scale.max(0.50).min(1.0);
                        apply_game_mode(pkg, scale, true, &tx_clone1);
                        write_log("CONFIG", &format!("Applied game downscale for {}: {}", pkg, scale));
                    } else if prev_ds == "1" {
                        apply_game_mode(pkg, 1.0, false, &tx_clone1);
                        write_log("CONFIG", &format!("Removed game downscale for {}", pkg));
                    }
                }
            }
            for (pkg, (prev_ds, _)) in &prev_game_states {
                if !current_game_states.contains_key(pkg) && prev_ds == "1" {
                    apply_game_mode(pkg, 1.0, false, &tx_clone1);
                    write_log("CONFIG", &format!("Removed game downscale for {}", pkg));
                }
            }
            prev_game_states = current_game_states;
            
            // ZRAM / Swappiness Optimization
            let cur_zram_optimize = get_config_value(&config, "zram_optimize", "0");
            if Some(&cur_zram_optimize) != prev_zram_optimize.as_ref() {
                prev_zram_optimize = Some(cur_zram_optimize.clone());
                if cur_zram_optimize == "1" {
                    write_node("/proc/sys/vm/swappiness", "80");
                    write_log("CONFIG", "ZRAM optimization applied (swappiness=80)");
                } else {
                    write_node("/proc/sys/vm/swappiness", "60");
                    write_log("CONFIG", "ZRAM optimization disabled (swappiness=60)");
                }
            }
            
            // Refresh rate mode
            let cur_refresh_rate = get_config_value(&config, "refresh_rate", "smart");
            if Some(&cur_refresh_rate) != prev_refresh_rate.as_ref() {
                prev_refresh_rate = Some(cur_refresh_rate.clone());
                if cur_refresh_rate == "smart" {
                    GLOBAL_REFRESH_RATE_MODE.store(1, Ordering::SeqCst);
                } else {
                    GLOBAL_REFRESH_RATE_MODE.store(0, Ordering::SeqCst);
                    if cur_refresh_rate == "60" || cur_refresh_rate == "90" {
                        let _ = std::process::Command::new("cmd").args(["settings", "put", "system", "min_refresh_rate", &cur_refresh_rate]).output();
                        let _ = std::process::Command::new("cmd").args(["settings", "put", "system", "peak_refresh_rate", &cur_refresh_rate]).output();
                        let _ = std::process::Command::new("cmd").args(["settings", "put", "system", "user_refresh_rate", &cur_refresh_rate]).output();
                        write_log("CONFIG", &format!("Locked system refresh rate to {}Hz", cur_refresh_rate));
                    } else {
                        // "auto" or empty
                        let _ = std::process::Command::new("cmd").args(["settings", "delete", "system", "min_refresh_rate"]).output();
                        let _ = std::process::Command::new("cmd").args(["settings", "delete", "system", "peak_refresh_rate"]).output();
                        let _ = std::process::Command::new("cmd").args(["settings", "delete", "system", "user_refresh_rate"]).output();
                        write_log("CONFIG", "Restored system default refresh rate (Auto)");
                    }
                }
            }
        }
        
        // DEXOPT Daily 5 AM check
        let day = format_time(now as u64);
        if day.starts_with("05:") {
            let day_num = (now / 86400) as i32;
            if day_num != last_dexopt_day {
                last_dexopt_day = day_num;
                write_log("DEXOPT", "Starting daily 5 AM Dexopt job...");
                thread::spawn(move || {
                    let _ = std::process::Command::new("cmd").args(["package", "bg-dexopt-job"]).output();
                    write_log("DEXOPT", "Daily Dexopt job complete.");
                });
            }
        }
        
        // Game Lifecycle active logic (with 8-second focus loss debouncer)
        let is_game_mode_enabled = if focused_pid != 0 {
            get_config_value(&config, &format!("opt_game_mode_{}", focused_pkg), "0") == "1"
                || (focused_pkg == "com.mobile.legends" && (
                    get_config_value(&config, "opt_game_mode_com.mobile.legends", "0") == "1"
                    || get_config_value(&config, "mlbb_downscale", "0") == "1"
                    || get_config_value(&config, "opt_preload_com.mobile.legends", "0") == "1"
                    || get_config_value(&config, "opt_disable_thermal_com.mobile.legends", "0") == "1"
                ))
        } else {
            false
        };
        let is_running_game = screen_awake == 1 && focused_pid != 0 && gamelist.contains(&focused_pkg.to_lowercase()) && is_game_mode_enabled;
        let game_active = active || is_running_game;
        if game_active && focused_pid != 0 {
            not_active_ticks = 0;
            if !mlbb_logic_running {
                write_log("ACTIVATE", &format!("Game logic ON for {} (PID: {})", focused_pkg, focused_pid));
                mlbb_logic_running = true;
                
                // Only run launch-time optimizations if this PID hasn't been optimized yet
                if focused_pid != last_optimized_pid {
                    // Clear previous game's memory mappings
                    clear_preloaded_mappings();
                    
                    // Restore thermal first to clean up any state from previous game
                    apply_disable_thermal(false);
                    thermal_currently_disabled = false;
                    
                    last_optimized_pid = focused_pid;
                    
                    // Apply disable thermal if enabled for this game
                    let disable_thermal_enabled = get_config_value(&config, &format!("opt_disable_thermal_{}", focused_pkg), "0") == "1";
                    if disable_thermal_enabled {
                        apply_disable_thermal(true);
                        thermal_currently_disabled = true;
                        write_log("GAME", &format!("Thermal Core disabled for active game: {}", focused_pkg));
                    }
                    
                    // pre-game cache flush + Auto RAM Clean
                    let mem_before_flush = get_mem_available_kb();
                    write_node("/proc/sys/vm/drop_caches", "1");
                    
                    let pkg_for_kill = focused_pkg.clone();
                    
                    thread::spawn(move || {
                        kill_bg(&pkg_for_kill);
                        let mem_after = get_mem_available_kb();
                        let total_freed_mb = (mem_after - mem_before_flush).max(0) / 1024;
                        write_log("GAME", &format!("Pre-game cleanup done: {} MB freed", total_freed_mb));
                        notify("Ravencore", &format!("Cleaned {} MB — Game Ready!", total_freed_mb));
                    });
                    
                    // Clear Powerkeeper Data
                    thread::spawn(|| {
                        let _ = std::process::Command::new("pm").args(["clear", "com.miui.powerkeeper"]).output();
                    });
                    
                    let preload_enabled = get_config_value(&config, &format!("opt_preload_{}", focused_pkg), "0") == "1";
                    
                    if preload_enabled {
                        let pkg_clone = focused_pkg.clone();
                        thread::spawn(move || {
                            if pkg_clone == "com.mobile.legends" {
                                preload_mlbb();
                            } else {
                                preload_generic_game(&pkg_clone);
                            }
                        });
                    }
                    
                    // Only apply resolution downscale if game downscaling is enabled
                    let downscale_enabled = if focused_pkg == "com.mobile.legends" {
                        get_config_value(&config, "mlbb_downscale", "0") == "1"
                    } else {
                        get_config_value(&config, &format!("opt_downscale_{}", focused_pkg), "0") == "1"
                    };
                    if downscale_enabled {
                        let scale_pct = get_game_scale(&config, &focused_pkg);
                        let scale = scale_pct as f32 / 100.0;
                        let scale = scale.max(0.50).min(1.0);
                        
                        // Show downscale toast
                        let _ = tx_clone1.send(format!("sh -c '. /data/adb/modules/ravencore/scripts/utils.sh && notify \"Ravencore\" \"Resolution Downscaled to {}%!\"'", scale_pct));
                        
                        let tx_clone_workaround = tx_clone1.clone();
                        let pkg_clone = focused_pkg.clone();
                        thread::spawn(move || {
                            thread::sleep(Duration::from_millis(1000));
                            apply_game_mode(&pkg_clone, scale, true, &tx_clone_workaround);
                            thread::sleep(Duration::from_millis(2000));
                            apply_game_mode(&pkg_clone, scale, true, &tx_clone_workaround);
                            thread::sleep(Duration::from_millis(2000));
                            apply_game_mode(&pkg_clone, scale, true, &tx_clone_workaround);
                            write_log("WORKAROUND", "Re-applied downscale forced layouts refresh");
                        });
                    }
                } else {
                    write_log("GAME", "Re-focused same game process. Skipping redundant launch optimizations.");
                }
            }
            
            // Dynamic Safety Monitor: Check battery temperature while gaming
            let disable_thermal_enabled = get_config_value(&config, &format!("opt_disable_thermal_{}", focused_pkg), "0") == "1";
            if disable_thermal_enabled {
                let temp = get_battery_temp_celsius();
                if temp >= 46 {
                    if thermal_currently_disabled {
                        apply_disable_thermal(false);
                        thermal_currently_disabled = false;
                        write_log("SAFETY", &format!("Overheat detected ({}°C). Re-enabling thermal core to protect device.", temp));
                        notify("Ravencore Safety", &format!("Baterai panas ({}°C)! Thermal Engine dinyalakan kembali.", temp));
                    }
                } else if temp <= 41 {
                    if !thermal_currently_disabled {
                        apply_disable_thermal(true);
                        thermal_currently_disabled = true;
                        write_log("SAFETY", &format!("Device cooled down ({}°C). Disabling thermal core again.", temp));
                        notify("Ravencore Safety", &format!("Suhu aman ({}°C). Thermal Engine dimatikan kembali.", temp));
                    }
                }
            }
        } else {
            if mlbb_logic_running {
                not_active_ticks += 1;
                if not_active_ticks >= 8 {
                    write_log("DEACTIVATE", "Game logic OFF");
                    mlbb_logic_running = false;
                    not_active_ticks = 0;
                    
                    // Clear preloaded locked memory mappings
                    clear_preloaded_mappings();
                    
                    // Restore thermal safely on game exit
                    apply_disable_thermal(false);
                    thermal_currently_disabled = false;
                    write_log("GAME", "Restored system thermal configurations");
                }
            }
        }
        
        // Apply Idle / Active charging controls
        let is_game_running_and_listed = screen_awake == 1 && focused_pid != 0 && gamelist.contains(&focused_pkg.to_lowercase());
        let bypass_supported = get_config_value(&config, "bypass_supported", "1") == "1";
        
        if bypass_supported && (mlbb_logic_running || is_game_running_and_listed) {
            if !last_bypass_state {
                last_bypass_state = true;
                write_log("ACTIVATE", &format!("Bypass Charging ON (Game Active: {})", focused_pkg));
            }
            // Unconditional Auto Bypass Charging during gameplay
            write_node("/sys/class/qcom-battery/idle_mode", "1");
            write_node("/sys/class/power_supply/battery/input_suspend", "1");
            write_node("/sys/class/power_supply/battery/charging_enabled", "0");
        } else {
            if last_bypass_state {
                last_bypass_state = false;
                write_log("DEACTIVATE", "Bypass Charging OFF");
            }
            let fast_charge_on = get_config_value(&config, "fast_charge", "0") == "1";
            if fast_charge_on {
                let temp = get_battery_temp_celsius();
                
                // Hysteresis: throttle at 42°C, recover at 38°C
                if temp >= 42 {
                    throttled = true;
                } else if temp <= 38 {
                    throttled = false;
                }
                
                // Critical safety: hard cutoff at 46°C — revert to stock charging
                if temp >= 46 {
                    if prev_fast_charge_state != Some(false) || prev_throttled_state.is_some() {
                        write_node("/sys/class/power_supply/battery/step_charging_enabled", "1");
                        write_node("/sys/class/power_supply/battery/sw_jeita_enabled", "1");
                        write_node("/sys/class/power_supply/battery/fastcharge_mode", "0");
                        write_node("/sys/class/power_supply/battery/fast_charge", "0");
                        write_node("/sys/class/power_supply/battery/restrict_chg", "0");
                        write_node("/sys/class/power_supply/battery/restrict_cur", "0");
                        write_node("/sys/class/power_supply/battery/input_current_settled", "1500000");
                        write_log("CHARGE", &format!("Fast Charge EMERGENCY CUTOFF — Temp: {}°C", temp));
                        
                        prev_fast_charge_state = Some(false);
                        prev_throttled_state = None;
                    }
                } else {
                    if prev_fast_charge_state != Some(true) || prev_throttled_state != Some(throttled) {
                        // --- Step 1: Disable all kernel charging controllers that fight back ---
                        write_node("/sys/class/power_supply/battery/step_charging_enabled", "0");
                        write_node("/sys/class/power_supply/battery/sw_jeita_enabled", "0");
                        write_node("/sys/class/power_supply/battery/fastcharge_mode", "1");
                        write_node("/sys/class/power_supply/battery/fast_charge", "1");
                        write_node("/sys/class/power_supply/battery/restrict_chg", "0");
                        write_node("/sys/class/power_supply/battery/restrict_cur", "0");
                        write_node("/sys/class/power_supply/battery/system_temp_level", "0");
                        write_node("/sys/class/power_supply/battery/rerun_aicl", "0");
                        
                        // --- Step 2: Set current limits ---
                        let limit_current = if throttled { "2000000" } else { "5000000" };
                        let input_current = if throttled { "1500000" } else { "3000000" };
                        
                        write_node("/sys/class/power_supply/battery/current_max", limit_current);
                        write_node("/sys/class/power_supply/battery/constant_charge_current_max", limit_current);
                        
                        write_node("/sys/class/power_supply/usb/current_max", input_current);
                        write_node("/sys/class/power_supply/usb/input_current_limit", input_current);
                        write_node("/sys/class/power_supply/usb/hw_current_max", input_current);
                        write_node("/sys/class/power_supply/main/current_max", input_current);
                        write_node("/sys/class/power_supply/main/constant_charge_current_max", limit_current);
                        write_node("/sys/class/power_supply/dc/current_max", input_current);
                        write_node("/sys/class/power_supply/battery/input_current_settled", input_current);
                        
                        write_node("/sys/class/power_supply/battery/charge_control_limit", "0");
                        write_node("/sys/class/power_supply/usb/voltage_max", "12000000");
                        
                        write_log("CHARGE", &format!("Fast Charge Applied (Throttled: {}, Temp: {}°C)", throttled, temp));
                        
                        prev_fast_charge_state = Some(true);
                        prev_throttled_state = Some(throttled);
                    }
                }
            } else {
                // Restore stock charging behavior ONCE on state transition
                if prev_fast_charge_state != Some(false) {
                    write_node("/sys/class/power_supply/battery/step_charging_enabled", "1");
                    write_node("/sys/class/power_supply/battery/sw_jeita_enabled", "1");
                    write_node("/sys/class/power_supply/battery/fastcharge_mode", "0");
                    write_node("/sys/class/power_supply/battery/fast_charge", "0");
                    write_node("/sys/class/power_supply/battery/restrict_chg", "0");
                    write_node("/sys/class/power_supply/battery/restrict_cur", "0");
                    
                    // Restore current limits to safe stock defaults so kernel can manage them
                    write_node("/sys/class/power_supply/battery/current_max", "2000000");
                    write_node("/sys/class/power_supply/battery/constant_charge_current_max", "2000000");
                    write_node("/sys/class/power_supply/usb/current_max", "1500000");
                    write_node("/sys/class/power_supply/usb/input_current_limit", "1500000");
                    write_node("/sys/class/power_supply/usb/hw_current_max", "1500000");
                    write_node("/sys/class/power_supply/main/current_max", "1500000");
                    write_node("/sys/class/power_supply/dc/current_max", "1500000");
                    write_node("/sys/class/power_supply/battery/input_current_settled", "1500000");
                    write_node("/sys/class/power_supply/usb/voltage_max", "9000000");
                    
                    write_node("/sys/class/power_supply/battery/rerun_aicl", "1");
                    
                    write_log("CHARGE", "Fast Charge Restored to Stock defaults");
                    
                    prev_fast_charge_state = Some(false);
                    prev_throttled_state = None;
                }
            }
            
            let limit = safe_stoi(&get_config_value(&config, "charge_limit", "100"), 100);
            let level = safe_stoi(&read_node("/sys/class/power_supply/battery/capacity"), 100);
            
            if bypass_supported {
                if get_config_value(&config, "bypass_charge", "0") == "1" {
                    write_node("/sys/class/qcom-battery/idle_mode", "1");
                    write_node("/sys/class/power_supply/battery/input_suspend", "1");
                    write_node("/sys/class/power_supply/battery/charging_enabled", "0");
                } else {
                    if level >= limit {
                        write_node("/sys/class/qcom-battery/idle_mode", "1");
                        write_node("/sys/class/power_supply/battery/input_suspend", "1");
                        write_node("/sys/class/power_supply/battery/charging_enabled", "0");
                    } else if level <= limit - 2 {
                        write_node("/sys/class/qcom-battery/idle_mode", "0");
                        write_node("/sys/class/power_supply/battery/input_suspend", "0");
                        write_node("/sys/class/power_supply/battery/charging_enabled", "1");
                    }
                }
            }
        }
        
        let active_saver_enabled = get_config_value(&config, "active_saver", "0") == "1";
        
        // Handle active_saver state transitions
        if active_saver_enabled != last_active_saver_enabled {
            last_active_saver_enabled = active_saver_enabled;
            if active_saver_enabled {
                write_log("SAVER", "Active Battery Saver enabled");
                if screen_awake == 1 {
                    apply_active_saver_cgroups(true, &default_bg_cpus, &default_sys_bg_cpus);
                }
                // Apply whitelist and battery saver constants via worker
                let cmds = "cmd appops set com.google.android.gms RUN_IN_BACKGROUND allow 2>/dev/null; \
                            cmd appops set com.google.android.gms WAKE_LOCK allow 2>/dev/null; \
                            for p in com.whatsapp com.instagram.android com.zhiliaoapp.musically com.ss.android.ugc.trill com.google.android.gm; do \
                                dumpsys deviceidle whitelist +$p 2>/dev/null; \
                                am set-standby-bucket $p active 2>/dev/null; \
                            done; \
                            settings put global battery_saver_constants 'advertising_is_enabled=false,datasaver_is_enabled=false,enable_night_mode=true,gps_mode=2,force_all_apps_standby=false,enable_firewall=false,vibration_disabled=true,animation_disabled=false,launch_boost_disabled=false,optional_sensors_disabled=true,force_background_check=true' 2>/dev/null; \
                            settings put global low_power 1 2>/dev/null;";
                let _ = tx_clone1.send(cmds.to_string());
            } else {
                write_log("SAVER", "Active Battery Saver disabled");
                apply_active_saver_cgroups(false, &default_bg_cpus, &default_sys_bg_cpus);
                let _ = tx_clone1.send("cmd deviceidle unforce >/dev/null 2>&1".to_string());
                
                // Revert whitelist and battery saver constants via worker
                let cmds = "for p in com.whatsapp com.instagram.android com.zhiliaoapp.musically com.ss.android.ugc.trill com.google.android.gm; do \
                                dumpsys deviceidle whitelist -$p 2>/dev/null; \
                            done; \
                            cmd appops set com.google.android.gms RUN_IN_BACKGROUND allow 2>/dev/null; \
                            cmd appops set com.google.android.gms WAKE_LOCK allow 2>/dev/null; \
                            settings put global low_power 0 2>/dev/null; \
                            settings delete global battery_saver_constants 2>/dev/null;";
                let _ = tx_clone1.send(cmds.to_string());
                last_screen_state = 1;
            }
        }
        
        // Handle screen state transitions when active_saver is enabled
        if active_saver_enabled {
            if screen_awake == 0 && last_screen_state == 1 {
                write_log("SAVER", "Screen OFF: Entering Deep Sleep...");
                let _ = tx_clone1.send("cmd deviceidle force-idle deep >/dev/null 2>&1".to_string());
                write_node("/proc/sys/vm/drop_caches", "3");
                last_screen_state = 0;
            } else if screen_awake == 1 && last_screen_state == 0 {
                write_log("SAVER", "Screen ON: Exiting Deep Sleep...");
                let _ = tx_clone1.send("cmd deviceidle unforce >/dev/null 2>&1".to_string());
                apply_active_saver_cgroups(true, &default_bg_cpus, &default_sys_bg_cpus);
                last_screen_state = 1;
            }
        }
        
        if mlbb_logic_running {
            let fps = get_fps();
            let cpu_temp = get_cpu_temp_celsius();
            let bat_temp = get_battery_temp_celsius();
            let ram_free = get_mem_available_kb() / 1024;
            
            let _ = std::process::Command::new("am")
                .args([
                    "broadcast", "-a", "ravencore.intent.action.UPDATE_STATS",
                    "--ei", "fps", &fps.to_string(),
                    "--ei", "cpu_temp", &cpu_temp.to_string(),
                    "--ei", "bat_temp", &bat_temp.to_string(),
                    "--ei", "ram_free", &ram_free.to_string()
                ])
                .output();
        }

        update_status_file(tick, &focused_pkg, focused_pid, screen_awake, battery_saver, zen_mode);
        tick += 1;
        thread::sleep(Duration::from_secs(1));
    }
    
    let _ = fs::remove_file(PID_FILE);
    let _ = std::process::Command::new("pkill").args(["-f", "RavencoreSysMon"]).output();
    let _ = fs::remove_file("/data/media/0/Android/media/.ravencore/sysmon_status");
    let _ = fs::remove_file("/data/media/0/Android/media/.ravencore/sysmon.lock");
    let _ = fs::remove_file("/data/media/0/Android/media/.ravencore/status");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: ravencore_helper <monitor|kill_bg|clean_cache|disable_thermal>");
        std::process::exit(1);
    }
    
    unsafe {
        signal(15, signal_handler); // SIGTERM
        signal(2, signal_handler);  // SIGINT
    }
    
    let cmd = &args[1];
    if cmd == "monitor" {
        monitor_loop();
    } else if cmd == "kill_bg" {
        kill_bg("");
    } else if cmd == "clean_cache" {
        clean_cache();
    } else if cmd == "disable_thermal" {
        if args.len() >= 3 {
            let enable = args[2] == "1";
            apply_disable_thermal(enable);
        } else {
            eprintln!("Usage: ravencore_helper disable_thermal <0|1>");
            std::process::exit(1);
        }
    } else {
        eprintln!("Unknown command: {}", cmd);
        std::process::exit(1);
    }
}
