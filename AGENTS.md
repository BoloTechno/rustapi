# Project Guidelines & Memory: rustapi

Dokumen ini dimuat secara otomatis oleh Antigravity pada setiap sesi baru untuk memastikan agent selalu memahami konteks, arsitektur, dan aturan proyek **rustapi** tanpa perlu penjelasan ulang dari user.

---

## 1. Arsitektur: Clean Architecture (Hexagonal Architecture)

Proyek ini menggunakan **Clean Architecture** murni di bahasa Rust (Axum + SQLx SQLite).

```text
src/
├── domain/            # Layer 1: Core business logic (Task & Category entities, TaskPriority, TaskStatus, DomainError)
├── application/       # Layer 2: Use cases (TaskService, CategoryService) & Ports (TaskRepository, CategoryRepository)
├── infrastructure/    # Layer 3: Adapters (SqliteTaskRepository, SqliteCategoryRepository, InMemory counterparts)
├── presentation/      # Layer 4: HTTP Routes, Handlers, DTOs, Error Mapping (Axum)
├── lib.rs             # Module re-exports untuk library & tests
└── main.rs            # Composition Root: Dependency Injection & Server runner (port 3000)
```

### Aturan Arsitektur Wajib:
- **Dependency Rule**: Dependensi hanya boleh mengarah ke dalam. Domain sama sekali tidak boleh bergantung pada layer di luarnya (Axum, SQLx, dll).
- **Abstraksi Database**: `TaskService` hanya berinteraksi dengan database melalui *trait* `TaskRepository`.
- **Database Safety**: Database SQLite berada di file `tasks.db`. Semua perubahan skema harus menerapkan auto-migrasi non-destruktif (`PRAGMA table_info` + `ALTER TABLE`) di `SqliteTaskRepository::init_db()`. Jangan pernah menghapus tabel atau data yang ada.

---

## 2. Alur Pengembangan (Domain First / Inside-Out)

Setiap penambahan atau perubahan fitur wajib mengikuti urutan:
1. `src/domain/` (Entity, enum, validation rules, unit tests)
2. `src/application/` (Port trait, Use Cases di `TaskService`)
3. `src/infrastructure/` (Implementasi konkret di `SqliteTaskRepository` & `InMemoryTaskRepository`)
4. `src/presentation/` (DTO, Handler HTTP, Routing, Error response)
5. `tests/` (Automated integration tests di `tests/api_tests.rs` dan `tests/sqlite_tests.rs`)
6. `DOCS/` (Selalu perbarui `DOCS/API.md` dan `DOCS/WORKFLOW.md` secara otomatis)

---

## 3. Tiga Serangkai Verifikasi Wajib

Setelah melakukan perubahan kode, selalu jalankan dan pastikan 100% lulus:
```bash
cargo test                                # Menjalankan seluruh test suite
cargo clippy --all-targets -- -D warnings # Standar kualitas & keamanan kode (0 warnings)
cargo fmt --check                         # Konsistensi formatting
```

---

## 4. Standar Respons REST API

- **Single Item Success**: `{ "success": true, "data": { ... } }`
- **Paginated List Success**:
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
- **Error Response**: `{ "success": false, "error": { "code": "ERROR_CODE", "message": "..." } }`
- **Tenggat Waktu (`due_date`)**: Format ISO-8601 / RFC-3339 UTC (`YYYY-MM-DDTHH:MM:SSZ` atau `+07:00`).
- **Pagination Query**: Parameter `?page=X&per_page=Y&status=...&priority=...&search=...`.

---

## 5. Dokumentasi Lengkap Proyek

- [**`DOCS/AGENT.md`**](file:///home/tipol/Projects/rustapi/DOCS/AGENT.md): Panduan lengkap agent & prinsip Clean Architecture.
- [**`DOCS/API.md`**](file:///home/tipol/Projects/rustapi/DOCS/API.md): Spesifikasi lengkap seluruh endpoint, format request/response, dan contoh cURL.
- [**`DOCS/WORKFLOW.md`**](file:///home/tipol/Projects/rustapi/DOCS/WORKFLOW.md): Panduan alur kerja vibe coding, diagram arsitektur mermaid, dan pengujian.
