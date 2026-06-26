# 🦅 Ravencore

<p align="center">
  <img src="webroot/assets/Luca-v6-AI.jpg" alt="Ravencore Banner" width="100%" style="border-radius: 8px;">
</p>

<p align="center">
  <a href="https://github.com/xzhrael/Ravencore">
    <img src="https://img.shields.io/badge/Status-Stable-green?style=for-the-badge" alt="Status">
  </a>
  <img src="https://img.shields.io/badge/Version-v1.0-blue?style=for-the-badge" alt="Version">
  <img src="https://img.shields.io/badge/License-Apache%202.0-orange?style=for-the-badge" alt="License">
  <img src="https://img.shields.io/badge/Android-10+-blue?style=for-the-badge&logo=android" alt="Android Version">
  <img src="https://img.shields.io/badge/Platform-KernelSU%20%7C%20Magisk%20%7C%20APatch-purple?style=for-the-badge" alt="Platform">
</p>

---

**Ravencore** is a premium, lightweight system utility and optimization module designed for rooted Android devices. Powered by a high-performance **native Rust daemon** (`ravencore_helper`), it delivers granular system-level enhancements—including charging controls, dynamic resolution downscaling, native memory preloading, and safe thermal management—without risky kernel alterations or device instability. It features a modern, high-performance **WebUI Dashboard** for real-time monitoring and seamless utility configuration.

**Ravencore** adalah modul utilitas dan pengoptimalan sistem premium yang ringan untuk perangkat Android dengan akses root. Ditenagai oleh **daemon Rust native** berperforma tinggi (`ravencore_helper`), modul ini menghadirkan peningkatan tingkat sistem yang presisi—seperti kontrol pengisian daya, resolusi downscaling dinamis, pra-pemuatan memori native, serta manajemen termal yang aman—tanpa modifikasi kernel berisiko tinggi atau ketidakstabilan sistem. Modul ini dilengkapi dengan **WebUI Dashboard** modern berperforma tinggi untuk pemantauan waktu nyata dan konfigurasi utilitas yang lancar.

---

## ⚡ Core Features / Fitur Utama

### 🖥️ Premium WebUI Dashboard / Dasbor WebUI Premium
* **Real-Time Monitoring:** Track active CPU/GPU frequencies, RAM footprint, battery health, temperature, charging current, and voltage in real-time.
  **Pemantauan Waktu Nyata:** Pantau frekuensi CPU/GPU aktif, penggunaan RAM, kesehatan baterai, suhu, arus pengisian daya, dan tegangan secara real-time.
* **Interactive Floating Widgets:** Clean, glow-styled floating status indicators displaying active device info, kernel version, and daemon states.
  **Widget Terapung Interaktif:** Indikator status terapung dengan gaya bercahaya yang bersih, menampilkan info perangkat aktif, versi kernel, dan status daemon.
* **Live Log Viewers:** Real-time log capture for background daemon status and asset preloading events with auto-scroll features.
  **Penampil Log Dinamis:** Menangkap log daemon latar belakang dan aktivitas pra-pemuatan aset secara real-time dengan fitur gulir otomatis.

### 🎮 Game Booster & Preloader / Pengoptimal & Pra-pemuat Game
* **Resolution Downscaling:** Adjust custom resolution downscaling ratios per game package to maximize FPS on mid-range hardware.
  **Downscaling Resolusi:** Sesuaikan rasio penurunan resolusi per paket game untuk memaksimalkan FPS pada perangkat kelas menengah.
* **Native RAM Preloading:** Preloads game libraries and assets into RAM using C-level memory mapping and page-locking (`mmap` + `mlock` + `madvise`), preventing Android from paginating out game assets.
  **Pra-pemuatan RAM Native:** Memuat pustaka dan aset game langsung ke RAM menggunakan pemetaan memori tingkat C dan penguncian halaman (`mmap` + `mlock` + `madvise`), mencegah Android melakukan paginasi aset game keluar dari memori.
* **Auto-Launch Cleanup:** Instantly cleans RAM, flushes drop caches, and suspends system battery limits (`com.miui.powerkeeper`) when optimized games start.
  **Pembersihan Otomatis Saat Peluncuran:** Seketika membersihkan RAM, membersihkan cache, dan menonaktifkan pembatas baterai sistem (`com.miui.powerkeeper`) saat game dimulai.

### 🔋 Battery & Thermal Safety / Keamanan Baterai & Termal
* **Smart Fast Charging:** Safely unlocks maximum charging speeds based on real-time temperature (throttles to `1.5A` if battery temp $\ge$ 42°C, restoring full input current under 38°C).
  **Pengisian Cepat Pintar:** Membuka kecepatan pengisian maksimum secara aman berdasarkan suhu waktu nyata (membatasi ke `1.5A` jika suhu baterai $\ge$ 42°C, memulihkan arus input penuh di bawah 38°C).
* **Automated Bypass Charging:** Draw power directly from the charger instead of the battery during active gaming to minimize heat buildup and maximize battery lifespan.
  **Pengisian Daya Bypass Otomatis:** Mengalirkan daya langsung dari pengisi daya alih-alih baterai selama bermain game untuk meminimalkan panas dan memperpanjang umur baterai.
* **Active Battery Saver:** Allocates background threads/cgroups to efficiency cores (CPU 0-3), whitelists critical messenger services (WhatsApp, Discord, etc.), and forces deep Doze state when the screen is off.
  **Penghemat Baterai Aktif:** Mengalokasikan thread/cgroup latar belakang ke inti efisiensi (CPU 0-3), memasukkan layanan perpesanan penting ke daftar putih (WhatsApp, Discord, dll.), dan memaksa status Doze mendalam saat layar mati.

### ⚙️ Automation & Maintenance / Otomatisasi & Pemeliharaan
* **Flat-Traversal Cache Cleaner:** Executes extremely fast, single-level junk cleaning (cleaning `cache` and `CodeCache` directories in milliseconds instead of lagging recursive shell `find` queries).
  **Pembersih Cache Flat-Traversal:** Melakukan pembersihan junk direktori tunggal dengan sangat cepat (membersihkan folder `cache` dan `CodeCache` dalam hitungan milidetik tanpa lag pemindaian rekursif `find` shell).
* **Safe Background Killer:** Automatically stops background third-party processes on-demand while keeping essential system services and your current game intact.
  **Pemberhentian Proses Latar Belakang:** Menghentikan proses pihak ketiga di latar belakang secara aman sesuai permintaan tanpa mengganggu layanan sistem penting dan game yang sedang berjalan.
* **Daily ART Compiler:** Automatically schedules daily package compilations (`bg-dexopt-job` at 5 AM) to keep applications responsive.
  **Kompilasi ART Harian:** Menjadwalkan kompilasi paket harian secara otomatis (`bg-dexopt-job` pada jam 5 pagi) untuk menjaga aplikasi tetap responsif.

---

## ⚙️ Technical Architecture / Arsitektur Teknis

Ravencore is built with high performance and low overhead in mind. The module transitions performance-critical shell tasks directly into native compiled code:

Ravencore dirancang dengan performa tinggi dan penggunaan resource yang sangat rendah. Modul ini memigrasikan tugas-tugas shell yang kritis langsung ke kode native terkompilasi:

```mermaid
graph TD
    A[WebUI Dashboard] -->|ksu.exec / Commands| B[Rust Helper Daemon: ravencore_helper]
    B -->|Native Sysfs Write| C[Smart Charging & Bypass]
    B -->|mmap + mlock + madvise| D[Native RAM Preloader]
    B -->|O(N) Flat Traversal| E[Flat Cache Cleaner]
    B -->|Sysfs Direct Thermal Zone Toggle| F[Thermal Guard / Safety Brake]
    B -->|am force-stop| G[Background RAM Optimizer]
```

* **$O(N)$ Cache Cleaning vs $O(M)$ Recursive Shell Find:** Native Rust scans `/data/media/0/Android/data` in a flat-level directory traversal, targeting exact folders (`cache`, `CodeCache`) instead of executing shell forks which scan thousands of nested files recursively, reducing execution from ~30s to <50ms.
  **Pembersihan Cache $O(N)$ vs Shell Find Rekursif $O(M)$:** Kode native Rust memindai `/data/media/0/Android/data` dengan penelusuran tingkat datar (flat traversal), langsung menuju folder target (`cache`, `CodeCache`) alih-alih mengeksekusi subshell `find` rekursif yang memindai ribuan subfolder, memangkas waktu dari ~30 detik menjadi <50 milidetik.
* **Sysfs Direct Writing:** Direct interaction with power supply interfaces and thermal zones via standard file writes in Rust threads, bypassing costly process forks (`sh` lifecycle spawning).
  **Penulisan Sysfs Langsung:** Interaksi langsung dengan antarmuka daya dan zona termal melalui penulisan file standar di thread Rust, menghindari overhead pembuatan proses subshell (`sh`).

---

## 📦 Installation / Instalasi

This is a system-level module compatible with modern Android root managers:
Ini adalah modul tingkat sistem yang kompatibel dengan manajer root Android modern:

1. Download the latest `Ravencore-v1.0-release.zip` from your build output directory.
   Unduh berkas `Ravencore-v1.0-release.zip` terbaru dari direktori output build Anda.
2. Open your Root Manager app (Magisk Manager, KernelSU, or APatch).
   Buka aplikasi Root Manager Anda (Magisk Manager, KernelSU, atau APatch).
3. Choose the ZIP file and flash it.
   Pilih berkas ZIP tersebut dan pasang (flash).
4. Reboot your device to apply system changes.
   Muat ulang (reboot) perangkat Anda untuk menerapkan perubahan sistem.
5. Open the WebUI Dashboard directly from the module card.
   Buka WebUI Dashboard secara langsung dari kartu modul.

---

## 🛠️ Configuration Backups / Cadangan Konfigurasi

Ravencore stores user configurations using a lightweight, optimized Base64 profile string.
Ravencore menyimpan konfigurasi pengguna menggunakan string profil Base64 yang dioptimalkan dan sangat ringan.

* **Ultra-Compact Profile:** Automatically prunes default values to keep backup strings within **16-32 characters**, saving space and making sharing easy.
  **Profil Ultra-Ringkas:** Secara otomatis memotong nilai default untuk menjaga string cadangan berada dalam **16-32 karakter**, menghemat ruang dan memudahkan pembagian profil.
* **Import / Export:** Head to the **About** tab in the WebUI to copy your profile or paste a new profile string.
  **Ekspor / Impor:** Buka tab **About** di WebUI untuk menyalin profil Anda atau menempelkan string profil baru.

---

## 🤝 Contributing & Bug Reports / Kontribusi & Laporan Bug

We welcome contributions to make **Ravencore** even better!
Kami menerima kontribusi untuk membuat **Ravencore** menjadi lebih baik!

* **Report Bugs:** Open an [Issue](https://github.com/xzhrael/Ravencore/issues) and attach relevant logs if you encounter any bugs.
  **Laporkan Bug:** Buka [Issue](https://github.com/xzhrael/Ravencore/issues) dan lampirkan log terkait jika Anda menemukan bug.
* **Pull Requests:** Fork the repository, create a feature/bugfix branch, and submit a PR detailing your changes.
  **Pull Request:** Fork repositori ini, buat branch fitur/perbaikan bug, dan ajukan PR dengan penjelasan detail perubahan Anda.

---

## 👨‍💻 Core Developers & Credits / Pengembang & Referensi

### Core Developers / Pengembang Utama

| Role / Peran | Developer / Pengembang |
| :--- | :--- |
| **Lead Developer / Arsitek Utama** | [@xzhrael](https://github.com/xzhrael) (Luca Azhrael) |

### Sources & References / Sumber & Referensi
* **README Template & Layout:** Inspired by [AZenith](https://github.com/Liliya2727/AZenith) by [@Liliya2727](https://github.com/Liliya2727).
  **Templat & Tata Letak README:** Terinspirasi dari [AZenith](https://github.com/Liliya2727/AZenith) oleh [@Liliya2727](https://github.com/Liliya2727).
* **Thermal Core Management:** Inspired by [Rianixia-ThermalCore](https://github.com/ryanistr/Rianixia-ThermalCore) by [@ryanistr](https://github.com/ryanistr).
  **Manajemen Thermal Core:** Terinspirasi dari [Rianixia-ThermalCore](https://github.com/ryanistr/Rianixia-ThermalCore) oleh [@ryanistr](https://github.com/ryanistr).
* **System Monitor Engine:** Java background daemon powered by [system_monitor](https://github.com/Rem01Gaming/system_monitor) by [@Rem01Gaming](https://github.com/Rem01Gaming).
  **Mesin Pemantau Sistem:** Daemon latar belakang Java ditenagai oleh [system_monitor](https://github.com/Rem01Gaming/system_monitor) oleh [@Rem01Gaming](https://github.com/Rem01Gaming).
* **Encore Tweaks:** Optimization methods and WebUI layout references inspired by [encore](https://github.com/Rem01Gaming/encore) by [@Rem01Gaming](https://github.com/Rem01Gaming).
  **Encore Tweaks:** Metode optimasi dan referensi tata letak WebUI yang terinspirasi dari [encore](https://github.com/Rem01Gaming/encore) oleh [@Rem01Gaming](https://github.com/Rem01Gaming).

---

## 📢 Stay Updated / Tetap Terupdate

Get the latest updates and support by checking our channels:
Dapatkan pembaruan terbaru dan bantuan dengan memantau saluran kami:

<p align="left">
  <a href="https://github.com/xzhrael/Ravencore">
    <img src="https://img.shields.io/badge/GitHub-Repository-black?style=for-the-badge&logo=github" alt="GitHub Repo">
  </a>
  <a href="https://github.com/xzhrael/Ravencore/issues">
    <img src="https://img.shields.io/badge/Support-Issues-red?style=for-the-badge&logo=github" alt="Issues">
  </a>
</p>

---

## ⚖️ License / Lisensi

This project is licensed under the **Apache License 2.0**.
Proyek ini dilisensikan di bawah **Apache License 2.0**.

> Licensed under the Apache License, Version 2.0 (the "License");
> you may not use this file except in compliance with the License.
> You may obtain a copy of the License at [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0)
