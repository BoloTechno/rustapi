# Project Instruction & Guidelines: rustapi (Clean Architecture in Rust)

Dokumen ini adalah instruksi panduan (*agent guidelines*) untuk AI coding assistant pada repositori **rustapi**. Dokumen ini memastikan setiap sesi baru memahami arsitektur, standar kode, dan alur kerja proyek secara mandiri tanpa perlu penjelasan ulang dari user.

---

## 1. Ikhtisar Proyek

- **Nama Proyek:** `rustapi`
- **Tujuan:** REST API Task Management berperforma tinggi yang mematuhi prinsip **Clean Architecture (Hexagonal Architecture / Ports & Adapters)**.
- **Tech Stack:**
  - Bahasa: **Rust** (Edisi 2024)
  - Web Framework: **Axum 0.8** & **Tower**
  - Async Runtime: **Tokio 1.43**
  - Database: **SQLite** via **SQLx 0.8** (file `tasks.db` di root proyek)
  - Serialisasi: **Serde** & **serde_json**
  - Penanganan Error: **thiserror 2.0**
  - Identifier & Waktu: **Uuid 1.13** (v4) & **Chrono 0.4** (Utc)
  - Logging & Observability: **tracing** & **tracing-subscriber**

---

## 2. Struktur Arsitektur & Dependency Rule

Proyek ini menerapkan **Clean Architecture** secara ketat dengan hierarki dependensi satu arah dari luar ke dalam:

```text
Presentation Layer (src/presentation)   --> Axum HTTP Routes, Handlers (Task, Category, Health), DTOs
       ↓
Application Layer (src/application)     --> Use Cases (TaskService, CategoryService), Ports (TaskRepository, CategoryRepository)
       ↓
Domain Layer (src/domain)               --> Core Entities (Task, Category), Enums (TaskStatus, TaskPriority), DomainError
       ↑
Infrastructure Layer (src/infrastructure) --> Adapters: Sqlite (Task, Category), InMemory (Task, Category)
```

### Aturan Wajib per Layer:
1. **Domain (`src/domain/`)**:
   - Lapisan paling murni. **DILARANG** mengimpor `axum`, `sqlx`, atau library luar selain yang esensial (`chrono`, `serde`, `uuid`, `thiserror`).
   - Bertanggung jawab atas validasi entitas (misal: panjang judul maksimal 120 karakter Unicode), transisi status yang valid, dan enkapsulasi state.
2. **Application (`src/application/`)**:
   - Mengorkestrasi alur bisnis (*Use Cases*) melalui struct [`TaskService`](file:///home/tipol/Projects/rustapi/src/application/use_cases/task_use_cases.rs).
   - Berinteraksi dengan database **HANYA** melalui trait abstraksi [`TaskRepository`](file:///home/tipol/Projects/rustapi/src/application/ports/task_repository.rs).
3. **Infrastructure (`src/infrastructure/`)**:
   - Implementasi konkret port.
   - [`SqliteTaskRepository`](file:///home/tipol/Projects/rustapi/src/infrastructure/repositories/sqlite_task_repository.rs): Menangani koneksi database riil, query SQL dinamis (`LIMIT`, `OFFSET`, `LIKE`), dan auto-migrasi aman (`PRAGMA table_info` + `ALTER TABLE`).
   - [`InMemoryTaskRepository`](file:///home/tipol/Projects/rustapi/src/infrastructure/repositories/in_memory_task_repository.rs): Menangani penyimpanan memori (`RwLock<HashMap>`) untuk pengujian otomatis yang cepat dan terisolasi.
4. **Presentation (`src/presentation/`)**:
   - Menangani protokol HTTP (Axum routes, DTO request/response, mapping error domain/aplikasi ke status code HTTP).
   - Membungkus output API ke format envelope konsisten.

---

## 3. Siklus Pengembangan (Inside-Out Development)

Saat mengimplementasikan fitur atau perubahan baru, selalu terapkan urutan **Domain First**:
1. **Domain Layer**: Tambahkan field/validasi pada entitas domain dan unit test-nya.
2. **Application Layer**: Perbarui port trait dan logika pada service use case.
3. **Infrastructure Layer**: Implementasikan pada adapter SQLite dan In-Memory repository.
4. **Presentation Layer**: Buat/perbarui DTO, handler HTTP, routing, dan penanganan error.
5. **Integration Tests**: Tambahkan automated test di folder [`tests/`](file:///home/tipol/Projects/rustapi/tests).
6. **Dokumentasi**: Perbarui dokumentasi di [`DOCS/API.md`](file:///home/tipol/Projects/rustapi/DOCS/API.md), [`DOCS/WORKFLOW.md`](file:///home/tipol/Projects/rustapi/DOCS/WORKFLOW.md), dan checklist tugas di [`DOCS/TODO.md`](file:///home/tipol/Projects/rustapi/DOCS/TODO.md).

---

## 4. Tiga Serangkai Verifikasi Kualitas Wajib

Setiap kali agent selesai melakukan perubahan kode, wajib menjalankan dan memastikan ketiga perintah ini lulus 100%:

```bash
# 1. Jalankan seluruh unit dan integration test suite
cargo test

# 2. Periksa linter dan kualitas kode (harus 0 warnings/errors)
cargo clippy --all-targets -- -D warnings

# 3. Verifikasi format kode konsisten
cargo fmt --check
```

---

## 5. Standar Format Respon API

### Respon Sukses Single Item:
```json
{
  "success": true,
  "data": { ... }
}
```

### Respon Sukses Paginated List:
```json
{
  "success": true,
  "data": [ ... ],
  "pagination": {
    "page": 1,
    "per_page": 10,
    "total_items": 42,
    "total_pages": 5
  }
}
```

### Respon Error:
```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Deskripsi kesalahan manusiawi"
  }
}
```

---

## 6. Pemeliharaan Checklist Proyek (`DOCS/TODO.md`)

- Setiap kali fitur, perbaikan bug, atau tugas selesai diimplementasikan dan diverifikasi, agent **wajib memberikan tanda centang `[x]` secara otomatis** pada item yang relevan di [**`DOCS/TODO.md`**](file:///home/tipol/Projects/rustapi/DOCS/TODO.md).
- Jika ada fase baru atau sub-tugas yang disepakati bersama pengguna, cantumkan di `DOCS/TODO.md` agar progres proyek selalu terarah dan terukur.

---

## 7. Referensi Dokumentasi Lengkap

- [**`DOCS/TODO.md`**](file:///home/tipol/Projects/rustapi/DOCS/TODO.md): Roadmap proyek & checklist pelacak tugas otomatis.
- [**`DOCS/API.md`**](file:///home/tipol/Projects/rustapi/DOCS/API.md): Spesifikasi lengkap seluruh endpoint API, query params (`page`, `per_page`, `status`, `priority`, `search`), payload JSON, kode error semantik, dan contoh cURL.
- [**`DOCS/WORKFLOW.md`**](file:///home/tipol/Projects/rustapi/DOCS/WORKFLOW.md): Panduan alur kerja vibe coding, diagram arsitektur mermaid, dan cara menjalankan server/database.
