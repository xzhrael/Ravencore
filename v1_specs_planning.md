# Rencana Spesifikasi & Perencanaan Ravencore v1.0

Dokumen ini memuat spesifikasi, arsitektur, dan perencanaan untuk **Ravencore System v1.0**. Fokus utama dari modul ini adalah memberikan pengalaman pengguna yang modern, premium, dan segar melalui antarmuka WebUI yang responsif, tanpa mengorbankan performa perangkat berspesifikasi menengah ke bawah (khususnya Redmi Note 11 / SPES Snapdragon 680).

---

## 1. Filosofi Inti & Batasan Sistem v1.0
Aturan dasar dari modul Ravencore dipertahankan demi menjaga kestabilan dan kepatuhan sistem:
- **Tetap Fokus pada Utilitas**: Fitur terbatas pada Charging Control, Gaming downscale, pembersihan (Maintenance), dan diagnostik sistem.
- **TIDAK ADA Tweaks Kernel**: Tidak ada perubahan parameter kernel CPU/GPU/IO yang berisiko merusak sistem atau membuat tidak stabil.
- **TIDAK ADA Thermal Throttling Control**: Modifikasi dilakukan secara aman dengan monitoring suhu baterai dinamis.
- **Pembersihan Manual**: Pembersihan memori (RAM) dilakukan secara manual atas perintah pengguna untuk menghindari matinya aplikasi sistem penting.

---

## 2. Fitur Utama Visual: Hero Dashboard
Bagian atas halaman utama (*Home Tab*) memiliki **Hero Dashboard** yang menjadi pusat perhatian visual pertama kali ketika WebUI dibuka.

### Desain & Struktur Hero Dashboard
- **Banner Premium**: Menampilkan gambar utama dari `/webroot/assets/Luca-v6-AI.jpg`.
- **Efek Visual Overlay**: Gambar dihiasi dengan gradien gelap linear/radial agar menyatu dengan latar belakang WebUI, serta efek pencahayaan neon tipis.
- **Widget Terapung (Floating Widget)**: Di atas banner, terdapat informasi status dinamis yang bersih:
  - **Nama Modul & Versi**: `Ravencore System v1.0` dengan logo bercahaya (glowing icon).
  - **Device & Kernel**: Deteksi otomatis nama model perangkat dan versi kernel.
  - **Status Daemon**: Indikator LED neon hijau/merah yang menunjukkan status keaktifan daemon Rust `ravencore_helper`.

---

## 3. Antarmuka WebUI v1.0 (Premium & High-Performance)
WebUI dirancang menggunakan pendekatan desain modern berbasis *Vanilla CSS* dan *Modular JavaScript*.

### Alur Pengembangan UI Iteratif (Preview PC)
- **Preview PC**: Semua perombakan UI, gaya, navigasi tab, dan efek transisi dapat diuji secara lokal di [preview_pc.html](file:///D:/Module/LucaPro/preview_pc.html).
- **Sinkronisasi**: File produksi utama [index.html](file:///D:/Module/LucaPro/webroot/index.html) disinkronkan dengan perubahan preview yang telah teruji secara visual.
