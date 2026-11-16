# AGENTS.md — Pedoman Ketat & Prinsip Kerja AI Agent (Rust)

Dokumen ini adalah **kontrak kerja mutlak** untuk semua AI Coding Agent yang bekerja di repositori ini. Tujuan utama Anda adalah menulis kode Rust yang aman (*safe*), idiomatik, terukur, dan benar secara logika—**bukan sekadar membuat kode berhasil dikompilasi atau membuat indikator test berwarna hijau dengan jalan pintas.**

---

## 1. Integritas Pengujian (Zero-Tolerance untuk Manipulasi Test)

Test adalah spesifikasi kebenaran sistem. Jika sebuah test gagal (*FAIL*), itu berarti **implementasi kode yang salah**, bukan test-nya.

### DILARANG KERAS:
- **Mengubah Assertion:** Jangan pernah mengubah nilai ekspektasi pada `assert!`, `assert_eq!`, `assert_ne!`, atau `matches!` hanya agar sesuai dengan output kode yang sedang *buggy*.
- **Menghapus atau Menonaktifkan Test:** Jangan pernah menghapus fungsi `#[test]`, mengomentari baris kode test, atau menambahkan atribut `#[ignore]` / `#[should_panic]` untuk menyembunyikan kegagalan.
- **Hardcoding / Stubbing Palsu:** Jangan menulis logika `if input == test_case { return expected_output }` di dalam kode produksi hanya untuk mengelabui test runner.
- **Melemahkan Validasi di Test:** Jangan mengganti pengecekan spesifik (misal memeriksa varian error tertentu) menjadi pengecekan longgar (misal sekadar `assert!(result.is_err())`).

### YANG HARUS DILAKUKAN:
- Telusuri akar masalah (*root cause*) pada logika bisnis atau alur data di kode produksi.
- Anda **hanya boleh** mengubah test yang sudah ada jika pengguna secara eksplisit meminta perubahan spesifikasi bisnis, dan Anda wajib menjelaskan alasan perubahan test tersebut kepada pengguna.

---

## 2. Disiplin Ownership, Borrowing, dan Lifetimes

Borrow checker di Rust ada untuk menjamin keamanan memori dan konkurensi. Jangan melawannya dengan cara merusak arsitektur kode.

### DILARANG KERAS:
- **Spamming `.clone()`:** Jangan menambahkan `.clone()` atau `.to_owned()` secara membabi buta hanya untuk membungkam error *borrow checker* (`cannot move out of...` atau `borrowed value does not live long enough`) tanpa memahami siapa yang seharusnya menjadi pemilik (*owner*) data tersebut.
- **Penyalahgunaan Smart Pointer:** Jangan langsung membungkus tipe data dengan `Rc<RefCell<T>>` atau `Arc<Mutex<T>>` hanya karena kesulitan mengatur struktur *ownership* atau *mutable reference*, kecuali memang dibutuhkan untuk *shared state* lintas *thread* atau struktur graf yang sah.
- **Memalsukan `'static` Lifetime:** Jangan memaksakan batas `T: 'static` atau menggunakan `Box::leak` hanya karena malas mendefinisikan *lifetime annotations* (`'a`) yang benar.

### YANG HARUS DILAKUKAN:
- Desain alur data dengan jelas: tentukan fungsi mana yang membutuhkan kepemilikan penuh (`T`), mana yang hanya perlu membaca (`&T`), dan mana yang perlu memodifikasi (`&mut T`).
- Gunakan *slices* (`&[T]`, `&str`) daripada referensi ke koleksi penuh (`&Vec<T>`, `&String`) pada parameter fungsi.
- Jika terbentur masalah *borrow checker* yang kompleks, restrukturisasi cakupan (*scope*) variabel atau pecah struct menjadi bagian-bagian yang lebih modular.

---

## 3. Keamanan Memori & Larangan `unsafe`

Keunggulan utama Rust adalah jaminan *memory safety* pada saat kompilasi.

### DILARANG KERAS:
- **Menggunakan Blok `unsafe`:** Anda dilarang keras menulis blok `unsafe { ... }`, `unsafe fn`, atau `unsafe impl` untuk melewati aturan kompilator.
- **Transmutasi Tipe:** Jangan pernah menggunakan `std::mem::transmute`, *raw pointers* (`*const T`, `*mut T`), atau `MaybeUninit` sebagai jalan pintas mengatasi masalah tipe data atau *lifetime*.

### PENGECUALIAN:
- Penggunaan `unsafe` hanya diizinkan jika pengguna meminta secara spesifik (misalnya untuk integrasi FFI dengan C) dan wajib disertai komentar `// SAFETY: ...` yang menjelaskan secara rinci mengapa invariannya aman.

---

## 4. Penanganan Error (Error Handling) yang Eksplisit

Kode produksi tidak boleh mengalami *crash* (`panic!`) karena input yang tidak valid atau kegagalan I/O yang bisa diprediksi.

### DILARANG KERAS:
- **Menggunakan `.unwrap()` dan `.expect()` di Kode Produksi:** Jangan pernah menggunakan keduanya di luar kode test (`#[cfg(test)]`), kecuali Anda dapat membuktikan secara matematis/statis bahwa nilainya tidak mungkin `None`/`Err` (dan itu pun lebih disarankan menggunakan *pattern matching*).
- **Menelan Error Secara Diam-diam:** Dilarang mengabaikan `Result` menggunakan `let _ = fungsi_berisiko();` atau mengubahnya menjadi `.ok()` tanpa menangani skenario kegagalannya.
- **Menggunakan `panic!`, `todo!`, atau `unimplemented!`:** Jangan meninggalkan makro ini pada kode yang diklaim sudah selesai.

### YANG HARUS DILAKUKAN:
- Gunakan `Result<T, E>` dan propagasi error menggunakan operator `?`.
- Buat tipe error yang bermakna (menggunakan `enum` khusus, `thiserror` untuk *library*, atau `anyhow` untuk *application layer* sesuai konvensi proyek).
- Gunakan kombinator yang idiomatik seperti `.map_err()`, `.ok_or_else()`, `if let`, atau `let ... else`.

---

## 5. Kepatuhan Terhadap Compiler & Clippy

Peringatan (*warnings*) dari `rustc` dan `clippy` adalah sinyal adanya potensi *bug* atau kode yang tidak idiomatik.

### DILARANG KERAS:
- **Membungkam Linter:** Jangan pernah menambahkan atribut seperti `#[allow(dead_code)]`, `#[allow(unused_variables)]`, `#[allow(clippy::all)]`, atau atribut sejenis hanya untuk membuat *build* bersih dari peringatan.
- **Melewati Pengecekan Tipe dengan `Any`:** Jangan menggunakan `Box<dyn Any>` atau serialisasi ke `serde_json::Value` semata-mata untuk menghindari penulisan tipe generik atau *trait* yang tepat.

### YANG HARUS DILAKUKAN:
- Jika ada `dead_code`, hapus kode yang tidak terpakai tersebut atau hubungkan dengan modul yang seharusnya memanggilnya.
- Jalankan `cargo check`, `cargo test`, dan `cargo clippy -- -D warnings` sebagai standar verifikasi sebelum menyatakan pekerjaan selesai.
- Manfaatkan sistem tipe Rust secara maksimal (*Type-Driven Design*): gunakan `enum` dan `newtype pattern` agar kondisi yang tidak valid tidak mungkin direpresentasikan di dalam kode (*make illegal states unrepresentable*).

---

## 6. Protokol Saat Mengalami Kebuntuan (Stuck Protocol)

Jika Anda gagal mengompilasi kode atau menyelesaikan sebuah *bug* setelah beberapa kali percobaan:
1. **BERHENTI** melakukan perubahan acak (*shotgun debugging*) seperti menambah `.clone()`, mengubah tanda `&`, atau mengutak-atik test.
2. **ANALISIS** pesan error dari `rustc` secara menyeluruh—kompilator Rust memberikan petunjuk struktur yang sangat akurat.
3. **TANYAKAN KEPADA PENGGUNA** jika ada ambiguitas pada desain arsitektur atau logika domain daripada memaksakan kompromi teknis yang buruk.