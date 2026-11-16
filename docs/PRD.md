# PRD — Multi Stopwatch (Desktop App)

| | |
|---|---|
| **Nama sementara** | Multi Stopwatch |
| **Versi dokumen** | 1.0 |
| **Tanggal** | 16 November 2026 |
| **Status** | Draft untuk implementasi MVP |
| **Platform target** | Windows 10/11 (utama), macOS & Linux X11 (best-effort) |

---

## 1. Ringkasan

Multi Stopwatch adalah aplikasi desktop ringan untuk menjalankan **banyak stopwatch sekaligus**, di mana setiap stopwatch punya **nama sendiri**. Pengguna bisa start, pause, dan reset tiap stopwatch secara independen. Tersedia **Focus Mode**: jendela aplikasi mengecil menjadi widget kecil yang menempel di **pojok kanan atas layar** dan selalu di atas jendela lain, sehingga waktu tetap terlihat tanpa mengganggu pekerjaan.

Hal terpenting: **waktu stopwatch tidak boleh hilang**. Stopwatch yang di-pause tetap menyimpan waktunya saat aplikasi ditutup dan dibuka kembali.

### Prinsip desain

1. **Sederhana** — satu tabel database, tanpa ORM, tanpa state management library, tanpa backend service.
2. **Rust adalah sumber kebenaran** — semua logika waktu dan penyimpanan ada di Rust. Frontend hanya menampilkan.
3. **Akurat karena timestamp, bukan counter** — waktu dihitung dari selisih timestamp, bukan dari `setInterval` yang menambah angka. Jadi tetap akurat walau jendela di-minimize atau tick tertunda.

---

## 2. Tujuan dan Non-Goals

### Tujuan (MVP)

- Membuat, menamai, mengubah nama, dan menghapus banyak stopwatch.
- Start/resume, pause, dan reset tiap stopwatch secara independen.
- Focus Mode: widget kecil di kanan atas layar, always-on-top.
- Data persisten: stopwatch yang di-pause **tidak reset** setelah aplikasi ditutup.
- Aplikasi kecil, cepat dibuka, dan hemat resource.

### Non-Goals (sengaja tidak dikerjakan di MVP)

- Lap / split time.
- Countdown timer.
- Riwayat sesi, statistik, atau laporan.
- Sinkronisasi cloud / akun pengguna.
- Export (CSV/JSON).
- System tray, notifikasi, global hotkey.
- Drag & drop untuk mengurutkan stopwatch.
- Multi-bahasa (UI memakai Bahasa Indonesia).
- Auto-update.

---

## 3. Pengguna dan Use Case

**Pengguna utama:** individu yang perlu melacak durasi beberapa aktivitas paralel, misalnya mahasiswa, freelancer, atau developer.

| # | Use case |
|---|---|
| UC-1 | Freelancer menjalankan stopwatch "Klien A" dan "Klien B" bergantian untuk mencatat jam kerja. |
| UC-2 | Developer mengukur waktu "Coding", "Meeting", dan "Review" dalam satu hari. |
| UC-3 | Pengguna menutup aplikasi di tengah hari, lalu membukanya lagi besok. Semua waktu yang tadi di-pause masih ada. |
| UC-4 | Pengguna sedang mengetik dokumen dan ingin stopwatch tetap terlihat kecil di pojok kanan atas layar. |

---

## 4. Requirement Fungsional

Prioritas: **P0** = wajib MVP, **P1** = sebaiknya ada di MVP, **P2** = nanti.

| ID | Requirement | Prioritas |
|---|---|---|
| F-01 | Pengguna dapat membuat stopwatch baru. Jika nama kosong, otomatis bernama "Stopwatch N". | P0 |
| F-02 | Setiap stopwatch punya nama yang bisa diubah (inline edit). Nama di-trim, maksimal 50 karakter, boleh duplikat. | P0 |
| F-03 | Tombol **Start/Resume** menjalankan stopwatch. | P0 |
| F-04 | Tombol **Pause** menghentikan sementara dan menyimpan waktu terkumpul. | P0 |
| F-05 | Tombol **Reset** mengembalikan waktu ke 00:00:00 dan menghentikan stopwatch. | P0 |
| F-06 | Beberapa stopwatch bisa berjalan bersamaan dan tidak saling memengaruhi. | P0 |
| F-07 | Pengguna dapat menghapus stopwatch (dengan konfirmasi). | P0 |
| F-08 | **Focus Mode**: jendela mengecil jadi widget di kanan atas layar, always-on-top, menampilkan satu stopwatch terpilih. | P0 |
| F-09 | Keluar dari Focus Mode mengembalikan ukuran, posisi, dan dekorasi jendela seperti sebelumnya. | P0 |
| F-10 | **Persistensi:** semua stopwatch (nama, waktu, urutan) tersimpan di database lokal dan dimuat ulang saat aplikasi dibuka. | P0 |
| F-11 | Saat aplikasi ditutup, stopwatch yang sedang berjalan otomatis di-pause pada saat penutupan dan waktunya tersimpan. | P0 |
| F-12 | Pemulihan setelah crash/mati listrik: stopwatch yang masih berstatus running saat startup di-pause di titik heartbeat terakhir. | P1 |
| F-13 | Hanya satu instance aplikasi yang boleh berjalan (instance kedua memfokuskan jendela yang ada). | P1 |
| F-14 | Tombol `Esc` keluar dari Focus Mode. | P1 |
| F-15 | Konfirmasi singkat sebelum Reset jika waktu stopwatch > 0. | P1 |
| F-16 | Ingat ukuran/posisi jendela mode normal. | P2 |
| F-17 | Urutkan stopwatch dengan drag & drop. | P2 |
| F-18 | Tema gelap/terang mengikuti sistem. | P2 |

---

## 5. Perilaku Detail

### 5.1 Model state stopwatch

Setiap stopwatch hanya menyimpan dua nilai waktu:

- `accumulated_ms` — total waktu yang sudah terkumpul saat tidak berjalan.
- `started_at_ms` — timestamp Unix (ms) saat terakhir di-start; `NULL` jika tidak berjalan.

```
elapsed(now) = accumulated_ms + (started_at_ms != NULL ? now - started_at_ms : 0)
```

Status diturunkan, tidak disimpan:

| Status | Kondisi | Label tombol utama |
|---|---|---|
| **Idle** | `started_at_ms` NULL dan `accumulated_ms` = 0 | Start |
| **Running** | `started_at_ms` tidak NULL | Pause |
| **Paused** | `started_at_ms` NULL dan `accumulated_ms` > 0 | Resume |

### 5.2 Transisi

| Aksi | Dari Idle / Paused | Dari Running |
|---|---|---|
| **Start / Resume** | `started_at_ms = now` | No-op (idempotent) |
| **Pause** | No-op | `accumulated_ms += now - started_at_ms`, `started_at_ms = NULL` |
| **Reset** | `accumulated_ms = 0` | `accumulated_ms = 0`, `started_at_ms = NULL` |

Catatan: `now` diambil **sekali** di Rust per aksi, lalu dipakai untuk seluruh perhitungan aksi itu.

### 5.3 Saat aplikasi ditutup dan dibuka

- **Penutupan normal:** pada event `CloseRequested` (dan `RunEvent::Exit` untuk jaga-jaga, mis. Cmd+Q di macOS), Rust mem-pause semua stopwatch yang running dalam **satu transaksi**, lalu aplikasi keluar.
- **Pembukaan:** semua stopwatch dimuat dari database. Stopwatch yang sebelumnya di-pause tetap di-pause dengan waktu yang sama. Tidak ada yang berjalan otomatis.
- **Crash / mati listrik (F-12):** selama ada stopwatch running, Rust menulis `heartbeat_ms` setiap ±15 detik. Saat startup, setiap baris dengan `started_at_ms` tidak NULL dipulihkan menjadi `accumulated_ms += (heartbeat_ms - started_at_ms)` dan `started_at_ms = NULL`. Kehilangan waktu maksimal ±15 detik, dan tidak pernah menghitung waktu saat aplikasi mati.

> **Keputusan produk:** stopwatch yang running saat aplikasi ditutup **tidak melanjutkan hitungan** selama aplikasi mati. Ini lebih aman dan sesuai ekspektasi "stopwatch". Lihat bagian 13 untuk alternatifnya.

### 5.3.1 Close = quit

Pada MVP, menutup jendela berarti keluar dari aplikasi (tanpa tray). Ini juga berlaku saat Focus Mode.

### 5.4 Focus Mode

**Masuk (`enter_focus_mode(id)`)**, seluruhnya dikerjakan di Rust:

1. Simpan geometri jendela saat ini di memori (ukuran, posisi, status maximized, decorations).
2. Jika maximized, un-maximize dulu.
3. Hilangkan dekorasi window (title bar), nonaktifkan resize, set **always-on-top**.
4. Ubah ukuran jendela menjadi **±280 × 96 px (logical)**.
5. Posisikan di **pojok kanan atas** area kerja (work area) monitor tempat jendela berada, dengan margin 16 px. Pada multi-monitor, gunakan monitor saat ini, bukan monitor utama.
6. Frontend beralih ke tampilan compact.

**Tampilan Focus Mode:** nama stopwatch, waktu `HH:MM:SS`, tombol Pause/Resume, dan tombol keluar. Seluruh widget bisa di-drag (`data-tauri-drag-region`) karena title bar sudah dihilangkan.

**Keluar (`exit_focus_mode()`):** kembalikan semua properti dari geometri yang disimpan (decorations, resizable, always-on-top = false, ukuran, posisi, maximized), lalu frontend kembali ke tampilan normal.

**Aturan tambahan:**

- Stopwatch yang sedang berjalan tetap berjalan saat masuk/keluar Focus Mode.
- Focus Mode **tidak dipersist**: setelah restart, aplikasi selalu mulai di mode normal.
- Menutup jendela saat Focus Mode berperilaku sama seperti 5.3.
- Reset dan hapus tidak tersedia di Focus Mode (mencegah salah klik di widget kecil).

### 5.5 Validasi dan batasan

- Nama: trim spasi, 1–50 karakter. Jika kosong setelah trim, pakai nama default (untuk create) atau tolak perubahan (untuk rename).
- Jumlah stopwatch tidak dibatasi secara keras. Daftar di UI harus bisa di-scroll.
- Format tampilan: `HH:MM:SS` dengan 1 digit desimal (tenth of second) di mode normal. Jika lebih dari 99 jam, jumlah digit jam bertambah (tidak dipotong). Focus Mode menampilkan tanpa desimal.

---

## 6. Tech Stack

| Lapisan | Pilihan | Catatan |
|---|---|---|
| Shell desktop | **Tauri 2** | Binary kecil, memakai WebView bawaan OS. |
| Backend | **Rust** (full) | Seluruh logika waktu, database, dan manipulasi window. |
| Frontend | **SvelteKit** + Svelte 5 + TypeScript | Mode SPA: `@sveltejs/adapter-static`, `ssr = false`. |
| Styling | CSS biasa (scoped style Svelte) | Tanpa UI library/Tailwind agar tetap ringan. |
| State frontend | Svelte 5 runes (`$state`) di modul `.svelte.ts` | Tanpa library tambahan. |
| Database | **SQLite via `rusqlite`** (fitur `bundled`) | Lihat 6.1. |
| Error handling | `thiserror` | Error di-serialize ke string untuk frontend. |
| Plugin Tauri | `tauri-plugin-single-instance` (P1) | Mencegah dua proses menulis ke DB yang sama. |

### 6.1 Kenapa SQLite via `rusqlite`

Data aplikasi ini kecil (puluhan baris, satu tabel), tetapi **wajib tahan crash**. Pertimbangannya:

| Opsi | Penilaian |
|---|---|
| **SQLite + `rusqlite`** ✅ | Satu file, tulis atomik dan tahan crash secara otomatis, API sinkron sederhana (tidak perlu async runtime tambahan), hampir nol konfigurasi. Cocok walau data kecil. |
| File JSON | Paling sederhana di awal, tapi perlu menulis sendiri atomic write (tulis ke temp lalu rename) agar tidak korup saat crash. Mudah jadi lebih rumit dari SQLite. |
| `sqlx` | Terlalu berat: async, compile-time check, dan tooling migrasi tidak diperlukan. |
| ORM (Diesel/SeaORM) | Overengineering untuk satu tabel. |
| `tauri-plugin-sql` | Mengekspos SQL ke JavaScript. Bertentangan dengan keputusan "backend full Rust". |

**Migrasi** cukup memakai `PRAGMA user_version`:

```rust
let v: i32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
if v < 1 {
    conn.execute_batch("CREATE TABLE ...; PRAGMA user_version = 1;")?;
}
// versi berikutnya: if v < 2 { ... }
```

**Pengaturan koneksi:** `journal_mode = WAL`, `foreign_keys = ON`. Satu koneksi dibungkus `Mutex<Connection>` dan disimpan di Tauri `State`. Itu cukup, karena beban tulis sangat rendah.

**Lokasi file:** `app_data_dir()` milik Tauri, nama file `multi-stopwatch.db`.

---

## 7. Arsitektur

```
┌─────────────────────── Tauri App ───────────────────────┐
│                                                         │
│  SvelteKit (WebView)            Rust (src-tauri)        │
│  ┌──────────────────┐  invoke  ┌────────────────────┐   │
│  │ UI + ticker 100ms│ ───────► │ commands.rs        │   │
│  │ hitung elapsed   │ ◄─────── │ (validasi, logika) │   │
│  │ dari snapshot    │   DTO    ├────────────────────┤   │
│  └──────────────────┘          │ db.rs (rusqlite)   │──►│ multi-stopwatch.db
│                                │ focus.rs (window)  │   │
│                                │ heartbeat thread   │   │
│                                └────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### Alur data

1. Saat startup, frontend memanggil `list_stopwatches` dan menyimpan hasilnya di store.
2. Setiap aksi (start/pause/reset/rename/delete) adalah **satu command**. Rust memvalidasi, menulis ke DB, lalu mengembalikan DTO terbaru, dan store frontend diperbarui dari DTO itu.
3. Detik berjalan **tidak ditulis ke database**. Frontend memiliki **satu ticker global** (≈100 ms, aktif hanya bila ada stopwatch running) yang menghitung `elapsed = accumulated_ms + (Date.now() - started_at_ms)` untuk tiap kartu.
4. Database hanya ditulis saat ada aksi pengguna, saat penutupan, dan saat heartbeat (±15 detik, hanya jika ada yang running).

Karena waktu dihitung dari timestamp, jendela yang di-minimize atau WebView yang di-throttle **tidak membuat waktu meleset**.

### Struktur proyek

```
multi-stopwatch/
├─ src/                          # SvelteKit
│  ├─ routes/
│  │  ├─ +layout.ts              # export const ssr = false; prerender = false
│  │  ├─ +layout.svelte
│  │  └─ +page.svelte            # list stopwatch + switch mode normal/focus
│  └─ lib/
│     ├─ api.ts                  # wrapper tipis untuk invoke()
│     ├─ time.ts                 # computeElapsed(), formatElapsed()
│     ├─ stores/
│     │  └─ stopwatches.svelte.ts
│     └─ components/
│        ├─ StopwatchCard.svelte
│        └─ FocusView.svelte
├─ src-tauri/
│  ├─ src/
│  │  ├─ main.rs
│  │  ├─ lib.rs                  # builder, setup, window/run events
│  │  ├─ models.rs               # Stopwatch, StopwatchDto
│  │  ├─ stopwatch.rs            # logika murni (start/pause/reset/elapsed)
│  │  ├─ db.rs                   # open, migrate, query
│  │  ├─ commands.rs
│  │  ├─ focus.rs                # enter/exit focus mode
│  │  └─ error.rs
│  ├─ capabilities/default.json  # cukup core:default
│  ├─ tauri.conf.json
│  └─ Cargo.toml
├─ package.json
└─ svelte.config.js              # adapter-static, fallback: 'index.html'
```

> Seluruh manipulasi window dilakukan **dari Rust**, sehingga frontend tidak perlu izin `window:*` tambahan. Capability tetap minimal.

---

## 8. Data Model

```sql
CREATE TABLE stopwatches (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    name           TEXT    NOT NULL,
    accumulated_ms INTEGER NOT NULL DEFAULT 0,   -- waktu terkumpul saat tidak berjalan
    started_at_ms  INTEGER,                      -- unix ms; NULL = tidak berjalan
    heartbeat_ms   INTEGER,                      -- terakhir terlihat hidup (untuk recovery)
    position       INTEGER NOT NULL,             -- urutan tampil
    created_at_ms  INTEGER NOT NULL,
    updated_at_ms  INTEGER NOT NULL
);
```

Satu tabel sudah cukup untuk MVP. Tidak ada tabel pengaturan.

### Logika inti (fungsi murni agar mudah di-test)

```rust
pub fn elapsed_ms(sw: &Stopwatch, now_ms: i64) -> i64 {
    match sw.started_at_ms {
        Some(t) => sw.accumulated_ms + (now_ms - t).max(0),
        None => sw.accumulated_ms,
    }
}

pub fn start(sw: &mut Stopwatch, now_ms: i64) {
    if sw.started_at_ms.is_none() {
        sw.started_at_ms = Some(now_ms);
    }
}

pub fn pause(sw: &mut Stopwatch, now_ms: i64) {
    if let Some(t) = sw.started_at_ms.take() {
        sw.accumulated_ms += (now_ms - t).max(0);
    }
}

pub fn reset(sw: &mut Stopwatch) {
    sw.accumulated_ms = 0;
    sw.started_at_ms = None;
}
```

Parameter `now_ms` dioper dari luar supaya logika bisa di-unit-test tanpa menunggu waktu nyata.

### DTO ke frontend

```ts
type StopwatchDto = {
  id: number;
  name: string;
  accumulated_ms: number;
  started_at_ms: number | null;
  status: "idle" | "running" | "paused";
  position: number;
};
```

---

## 9. Kontrak Command (IPC)

Semua command mengembalikan `Result<T, String>`. Error ditampilkan frontend sebagai toast.

| Command | Argumen | Return | Keterangan |
|---|---|---|---|
| `list_stopwatches` | — | `StopwatchDto[]` | Diurutkan berdasarkan `position`. |
| `create_stopwatch` | `name?: string` | `StopwatchDto` | Nama default "Stopwatch N". |
| `rename_stopwatch` | `id, name` | `StopwatchDto` | Validasi 1–50 karakter. |
| `start_stopwatch` | `id` | `StopwatchDto` | Idempotent. |
| `pause_stopwatch` | `id` | `StopwatchDto` | Idempotent. |
| `reset_stopwatch` | `id` | `StopwatchDto` | Menghentikan dan mengosongkan. |
| `delete_stopwatch` | `id` | `()` | |
| `enter_focus_mode` | `id` | `()` | Mengubah window (lihat 5.4). |
| `exit_focus_mode` | — | `()` | Mengembalikan window. |

Hook di sisi Rust (bukan command):

- `on_window_event(CloseRequested)` dan `RunEvent::Exit` memanggil `pause_all_running()`.
- Thread heartbeat memperbarui `heartbeat_ms` setiap ±15 detik jika ada yang running.
- Startup menjalankan migrasi, lalu recovery crash (F-12).

---

## 10. Spesifikasi UI

### Jendela utama

Ukuran awal 480 × 640 px, ukuran minimum 360 × 400 px.

```
┌──────────────────────────────────────────────┐
│  Multi Stopwatch                  [ + Baru ] │
├──────────────────────────────────────────────┤
│ ┌──────────────────────────────────────────┐ │
│ │ ✎ Belajar Rust                      [🗑]  │ │
│ │                                          │ │
│ │        01 : 23 : 45 .6                   │ │
│ │                                          │ │
│ │  [ ⏸ Pause ]   [ ↺ Reset ]   [ ⛶ Focus ] │ │
│ └──────────────────────────────────────────┘ │
│ ┌──────────────────────────────────────────┐ │
│ │ ✎ Meeting                           [🗑]  │ │
│ │        00 : 12 : 03 .0   (paused)        │ │
│ │  [ ▶ Resume ]  [ ↺ Reset ]   [ ⛶ Focus ] │ │
│ └──────────────────────────────────────────┘ │
└──────────────────────────────────────────────┘
```

- **Empty state:** pesan singkat "Belum ada stopwatch" dan tombol "+ Buat stopwatch".
- Stopwatch yang sedang running diberi penanda visual (misalnya titik/aksen warna) agar mudah dibedakan dari yang paused.
- Nama diedit inline: klik ikon ✎ atau double-click nama, `Enter` menyimpan, `Esc` membatalkan.
- Angka waktu memakai `font-variant-numeric: tabular-nums` agar lebar tidak bergoyang saat berubah.
- Hapus dan Reset (jika waktu > 0) meminta konfirmasi sederhana.

### Focus Mode

```
┌──────────────────────────────┐
│ Belajar Rust            [✕]  │   ← seluruh area bisa di-drag
│ 01:23:45             [ ⏸ ]   │
└──────────────────────────────┘
        280 × 96 px, pojok kanan atas layar
```

- Tanpa title bar bawaan OS, always-on-top.
- `[✕]` (atau `Esc`) keluar dari Focus Mode, bukan menutup aplikasi.
- Tombol Pause/Resume besar dan mudah diklik.

---

## 11. Requirement Non-Fungsional

| Area | Target |
|---|---|
| **Akurasi** | Selisih waktu hanya bergantung pada jam sistem; tidak ada drift akumulatif dari ticker UI. |
| **Ketahanan data** | Tidak ada kehilangan data pada penutupan normal. Pada crash, kehilangan maksimal ±15 detik waktu running. Waktu paused tidak pernah hilang. |
| **Performa** | Startup < 1 detik pada mesin umum. Idle CPU mendekati 0% saat tidak ada stopwatch running; ringan saat ada yang running. |
| **Ukuran** | Installer di kisaran beberapa MB (khas Tauri). |
| **Privasi** | Semua data lokal. Tidak ada jaringan, telemetri, atau akun. |
| **Keamanan** | Capability Tauri minimal (`core:default`). Tidak ada akses filesystem/shell dari frontend. SQL hanya dari Rust dengan parameterized query. |
| **Aksesibilitas** | Tombol punya label teks/`aria-label`, kontras cukup, bisa dioperasikan dengan keyboard (Tab/Enter/Esc). |

---

## 12. Edge Case

| Kasus | Perilaku |
|---|---|
| Klik Start dua kali cepat | Aman: command idempotent, waktu tidak ter-reset atau berlipat. |
| Komputer sleep/hibernate saat stopwatch running | Waktu tetap dihitung berdasarkan jam dinding (seperti stopwatch pada umumnya). Didokumentasikan sebagai perilaku yang diketahui. |
| Pengguna mengubah jam sistem saat running | Dapat memengaruhi hasil. `(now - started_at).max(0)` mencegah nilai negatif. Dianggap di luar cakupan MVP. |
| Dua instance aplikasi dibuka | Instance kedua tidak membuka jendela baru; jendela pertama difokuskan (F-13). |
| File database tidak ada | Dibuat otomatis beserta skema. |
| File database korup/tidak bisa dibuka | Tampilkan dialog error yang jelas. Jangan menimpa file secara diam-diam. |
| Penghapusan stopwatch yang sedang di-focus | Tidak mungkin, karena tombol hapus tidak tersedia di Focus Mode. |
| Monitor di kanan atas berbeda DPI / layout multi-monitor | Posisi dihitung dari work area monitor saat ini memakai satuan logical. |
| Aplikasi ditutup saat Focus Mode | Semua yang running di-pause dan disimpan. Pembukaan berikutnya selalu di mode normal. |
| Nama sangat panjang | Dibatasi 50 karakter; UI memotong dengan ellipsis. |

---

## 13. Pertanyaan Terbuka dan Asumsi

| # | Pertanyaan | Default yang dipakai |
|---|---|---|
| 1 | OS target utama? | Windows 10/11. macOS dan Linux (X11) dikompilasi, tetapi dukungan Focus Mode best-effort. |
| 2 | Stopwatch yang running saat aplikasi ditutup: lanjut hitung atau di-pause? | **Di-pause** saat penutupan. Alternatif: tetap menghitung berdasarkan jam dinding sehingga waktu saat aplikasi mati ikut terhitung (cukup dengan tidak memanggil `pause_all_running` saat exit). Bisa dijadikan opsi pengaturan di versi berikutnya. |
| 3 | Perlu system tray agar app tetap hidup saat jendela ditutup? | Tidak untuk MVP. Close = quit. |
| 4 | Apakah Focus Mode perlu menampilkan lebih dari satu stopwatch? | Tidak. MVP menampilkan satu stopwatch. Beralih antar-stopwatch (prev/next) bisa jadi P2. |
| 5 | Nama dan ikon aplikasi final? | "Multi Stopwatch" dan ikon default Tauri untuk sementara. |

---

## 14. Rencana Pengujian

### Unit test (Rust, `stopwatch.rs`)

- `elapsed` pada idle, running, dan paused.
- Start → pause → start → pause: waktu terakumulasi dengan benar.
- Start ganda dan pause ganda bersifat no-op.
- Reset dari running dan dari paused.
- `now < started_at` tidak menghasilkan nilai negatif.
- Recovery crash: `accumulated += heartbeat - started_at`.

### Integration test (Rust, `db.rs`)

- Migrasi dari database kosong menghasilkan `user_version = 1`.
- Create → list → rename → delete.
- `pause_all_running` dalam satu transaksi.
- Data tetap ada setelah koneksi ditutup dan dibuka ulang (memakai file temp).

### Checklist uji manual

- [ ] Buat 3 stopwatch, jalankan 2 sekaligus; waktunya independen.
- [ ] Pause stopwatch A, tutup aplikasi, buka lagi: A masih di-pause dengan waktu yang sama.
- [ ] Biarkan stopwatch B running, tutup aplikasi: B tersimpan dalam keadaan paused.
- [ ] Matikan proses lewat Task Manager saat ada yang running, buka lagi: waktu pulih dengan selisih ≤ 15 detik.
- [ ] Masuk Focus Mode: widget di kanan atas, selalu di atas jendela lain, bisa di-drag.
- [ ] Keluar Focus Mode: ukuran dan posisi kembali seperti semula.
- [ ] Focus Mode dengan monitor ganda dan scaling DPI berbeda.
- [ ] Minimize jendela 10 menit saat running: waktu tetap benar setelah dibuka kembali.
- [ ] Buka aplikasi dua kali: hanya satu jendela.

---

## 15. Acceptance Criteria MVP

MVP dianggap selesai jika:

1. Semua requirement **P0** terpenuhi.
2. Skenario *"pause → tutup aplikasi → buka lagi"* selalu mempertahankan waktu persis.
3. Focus Mode menempatkan widget di kanan atas layar, always-on-top, dan bisa dikembalikan ke mode normal tanpa mengubah ukuran/posisi semula.
4. Seluruh unit/integration test pada bagian 14 lulus.
5. Build installer berhasil untuk Windows (`npm run tauri build`) dan bisa dipasang serta dijalankan pada mesin bersih.

---

## 16. Milestone

| Milestone | Isi | Hasil |
|---|---|---|
| **M0 — Scaffold** | `create-tauri-app` (SvelteKit + TS), adapter-static, `ssr = false`, struktur folder. | App kosong berjalan di `tauri dev`. |
| **M1 — Core Rust** | `stopwatch.rs` + unit test, `db.rs` + migrasi, semua command CRUD/start/pause/reset. | Logika lengkap, teruji tanpa UI. |
| **M2 — UI utama** | Store, ticker global, `StopwatchCard`, inline rename, empty state, konfirmasi. | Fitur inti bisa dipakai. |
| **M3 — Persistensi** | Hook close/exit, heartbeat, recovery crash, single-instance. | Waktu aman saat app ditutup atau crash. |
| **M4 — Focus Mode** | `focus.rs`, `FocusView`, drag region, `Esc`, penyimpanan geometri. | Widget kanan atas berfungsi. |
| **M5 — Polish & rilis** | Ikon, uji manual lintas monitor/DPI, perbaikan bug, build installer. | Installer siap dipakai. |

---

## 17. Risiko dan Mitigasi

| Risiko | Dampak | Mitigasi |
|---|---|---|
| Linux Wayland tidak mendukung `set_position` dan always-on-top secara andal | Focus Mode tidak presisi di Linux | Windows adalah target utama. Di Linux sebutkan sebagai keterbatasan atau sarankan sesi X11. |
| Plugin `window-state` (jika dipakai di P2) bentrok dengan geometri Focus Mode | Ukuran widget ikut tersimpan | Nonaktifkan penyimpanan state selama Focus Mode, atau simpan geometri sendiri. |
| Urutan operasi window (resize, resizable, decorations) berbeda antar-OS | Ukuran/posisi widget tidak sesuai | Bungkus di `focus.rs` dan uji di tiap OS target. |
| Perubahan jam sistem memengaruhi stopwatch running | Hasil waktu keliru | Clamp nilai negatif. Jika perlu nanti, tambahkan `Instant` monotonic untuk sesi berjalan. |
| WebView di-throttle saat tersembunyi | Tampilan tertunda | Tidak memengaruhi nilai waktu karena dihitung dari timestamp. Tampilan akan sinkron kembali saat tick berikutnya. |
| Over-engineering seiring berjalannya proyek | Waktu pengerjaan membengkak | Pegang daftar Non-Goals. Fitur baru masuk lewat revisi PRD. |

---

## 18. Dependensi Utama

**Rust (`Cargo.toml`)**

```toml
tauri = { version = "2" }
tauri-plugin-single-instance = "2"          # P1
rusqlite = { version = "*", features = ["bundled"] }  # pakai versi stabil terbaru
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
```

**Frontend (`package.json`)**

```
svelte (v5), @sveltejs/kit, @sveltejs/adapter-static,
vite, typescript, @tauri-apps/api (v2), @tauri-apps/cli (v2)
```

**`tauri.conf.json` (poin penting)**

- `build.frontendDist = "../build"`, `build.devUrl = "http://localhost:5173"`
- `build.beforeDevCommand = "npm run dev"`, `build.beforeBuildCommand = "npm run build"`
- Jendela utama: 480 × 640, `minWidth` 360, `minHeight` 400
- `identifier` unik, misalnya `com.example.multistopwatch`
