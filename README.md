# Rust Clean Architecture REST API 🦀

REST API manajemen tugas (*Task Management*) dan kategori yang dibangun menggunakan bahasa pemrograman **Rust** dengan framework **Axum**, driver database asinkron **SQLx**, dan **SQLite**. 

Proyek ini menerapkan **Clean Architecture (Hexagonal Architecture / Ports and Adapters)** secara murni: memisahkan aturan bisnis inti (*Domain*) dari use case (*Application*), adapter database (*Infrastructure*), dan endpoint HTTP (*Presentation*).

---

## 🏛️ Arsitektur Proyek

```text
src/
├── domain/            # Layer 1: Core business logic (Task, Category, Priorities, Domain Errors)
├── application/       # Layer 2: Use cases (TaskService, CategoryService) & Ports (Traits)
├── infrastructure/    # Layer 3: Adapters (SqliteTaskRepository, SqliteCategoryRepository, InMemory)
├── presentation/      # Layer 4: HTTP Routes, Handlers, DTOs, Error Mapping (Axum)
├── lib.rs             # Module re-exports untuk library & integration tests
└── main.rs            # Composition Root: Dependency Injection & Server runner (port 3000)
```

### Prinsip Utama:
- **Dependency Rule**: Ketergantungan kode hanya mengarah ke dalam. Domain sama sekali tidak bergantung pada database, framework, atau library eksternal.
- **Portability**: Sangat mudah mengganti database SQLite ke PostgreSQL atau MySQL di masa depan tanpa mengubah satu baris pun kode di Domain atau Application Layer.
- **Reliability & Concurrency**: SQLite diinisialisasi dengan mode **WAL (Write-Ahead Logging)**, **Foreign Keys**, dan **Busy Timeout 5s** untuk performa konkuren yang handal.

---

## 🚀 Memulai (Quick Start)

### Prasyarat
- [Rust & Cargo](https://www.rust-lang.org/tools/install) (versi 1.80+ / Edition 2024).

### Menjalankan Server Lokal
```bash
# Menjalankan HTTP server di port 3000
cargo run
```

Server akan aktif di `http://0.0.0.0:3000`.

### Menjalankan Pengujian (Test Suite)
```bash
# Menjalankan seluruh unit test dan integration test (27+ tests)
cargo test

# Menjalankan linter Clippy dengan standar 0 warning
cargo clippy --all-targets -- -D warnings

# Memeriksa konsistensi formatting kode
cargo fmt --check
```

---

## ⚙️ Konfigurasi Lingkungan

Konfigurasi dibaca dari Environment Variables (opsional):
- `DATABASE_URL`: URL koneksi SQLite (default: `sqlite://tasks.db?mode=rwc`).

---

## 📖 Dokumentasi Lengkap

- [**`DOCS/TODO.md`**](DOCS/TODO.md): Roadmap proyek & checklist pelacak tugas otomatis.
- [**`DOCS/API.md`**](DOCS/API.md): Spesifikasi lengkap seluruh endpoint REST API, parameter query, schema request/response, dan contoh cURL.
- [**`DOCS/WORKFLOW.md`**](DOCS/WORKFLOW.md): Panduan alur kerja vibe coding, diagram mermaid arsitektur, dan instruksi pengujian.
- [**`DOCS/AGENT.md`**](DOCS/AGENT.md) & [**`AGENTS.md`**](AGENTS.md): Panduan dan memori agent Antigravity untuk Clean Architecture.

---

## 📄 Ringkasan Fitur API

| Fitur | Keterangan |
| :--- | :--- |
| **Health Check** | `GET /health` & `GET /api/v1/health` status server dan versi build. |
| **Task Management** | CRUD task lengkap dengan validasi judul unik (*case-insensitive*). |
| **Task Priority** | 4 tingkatan prioritas: `low`, `medium`, `high`, `urgent`. |
| **Due Dates** | Tenggat waktu task dengan format ISO-8601 UTC. |
| **Filter & Pagination** | Parameter `?page=X&per_page=Y&status=...&priority=...&category_id=...&search=...`. |
| **Category Management** | CRUD kategori task dengan nama unik dan kode warna opsional. |
| **Relasi Integritas Data** | Auto-nullify relasi task saat kategori dihapus & validasi ID kategori saat task dibuat/diupdate. |
