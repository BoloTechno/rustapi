# Project Roadmap & Task Checklist: rustapi

Dokumen ini berfungsi sebagai peta jalan (*roadmap*) dan daftar pelacak tugas (*task checklist*) terstruktur untuk proyek **rustapi**. 

> [!IMPORTANT]
> **Aturan Otomatisasi Checklist**:
> Setiap kali fitur baru, perbaikan bug, atau fase kerja selesai diimplementasikan dan diverifikasi, item yang bersangkutan **wajib dicentang `[x]` secara otomatis** oleh agent/developer beserta tanggal atau catatan ringkas.

---

## 📊 Status Ringkasan Proyek

- **Fase Selesai**: 5 dari 9 Fase
- **Status Build & Test**: 37/37 Tests Lulus (100%), 0 Clippy Warnings, Formatting Konsisten
- **Fase Aktif Saat Ini**: **Fase 6: Production Web Middleware & Observability**

---

## 🗺️ Peta Jalan & Daftar Tugas

### [x] Fase 1: Fondasi Clean Architecture & CRUD Dasar Task
- [x] **Domain Layer**: Entitas `Task`, enum `TaskStatus` (`Pending`, `Completed`), validasi judul tidak kosong & maks 120 karakter, unit tests domain.
- [x] **Application Layer**: Port trait `TaskRepository`, use cases `TaskService` (`create`, `get`, `list`, `update`, `complete`, `reopen`, `delete`).
- [x] **Infrastructure Layer**: Adapter in-memory `InMemoryTaskRepository` dan adapter SQLite `SqliteTaskRepository` dengan auto-migrasi skema.
- [x] **Presentation Layer**: Router Axum, DTOs (`CreateTaskRequest`, `UpdateTaskRequest`, `TaskResponse`), handler HTTP, dan translasi error `ApiError`.
- [x] **Composition Root**: Inisialisasi pool SQLite dan dependency injection di `src/main.rs`.
- [x] **Tests & Docs**: Automated integration tests di `tests/api_tests.rs` dan dokumentasi awal.

---

### [x] Fase 2: Operasi Lanjutan, Filter, & Pagination Task
- [x] **Prioritas Task**: Enum `TaskPriority` (`low`, `medium`, `high`, `urgent`) dengan default `medium`.
- [x] **Tenggat Waktu**: Atribut `due_date: Option<DateTime<Utc>>` dengan format ISO-8601 / RFC-3339.
- [x] **Filter Dinamis**: Dukungan filter kombinasi berdasarkan `status`, `priority`, dan pencarian kata kunci (`search`).
- [x] **Pagination Terstruktur**: Parameter query `?page=X&per_page=Y` dengan metadata respons (`total_items`, `total_pages`, `page`, `per_page`).
- [x] **Health Check**: Endpoint `/health` dan `/api/v1/health` dengan metadata versi dan status server.
- [x] **Integration Tests**: Pengujian filter, pagination, dan health check di `tests/api_tests.rs` dan `tests/sqlite_tests.rs`.

---

### [x] Fase 3: Relasi Data Kategori & Validasi Judul Unik
- [x] **Domain Kategori**: Entitas `Category` (`id`, `name`, `color`), validasi nama tidak kosong dan maks 50 karakter.
- [x] **Relasi Task & Category**: Atribut `category_id: Option<Uuid>` pada entitas `Task` (*One-to-Many*).
- [x] **Aturan Judul Unik**: Validasi keunikan judul task (*case-insensitive*) dan kode error `409 Conflict` (`DUPLICATE_TITLE`).
- [x] **Use Cases Kategori**: Port `CategoryRepository`, `CategoryService` (CRUD kategori lengkap).
- [x] **Integritas Foreign Key di Application Layer**: `TaskService` memvalidasi keberadaan `category_id` dan menolak ID kategori fiktif dengan `404 Not Found` (`CATEGORY_NOT_FOUND`).
- [x] **HTTP Endpoints Kategori**: Rute `/api/v1/categories` (POST, GET, PATCH, DELETE) dan filter `?category_id=...` pada daftar task.
- [x] **Database Indexing**: Indeks unik SQLite `idx_tasks_title` untuk performa dan integritas penyimpanan.

---

### [x] Fase 4: Audit Kode & Penguatan Keandalan Produksi (Ponytail Audit)
- [x] **Auto-Nullify Relasi Kategori**: Menghapus kategori otomatis menyetel `category_id = NULL` pada task terkait melalui `clear_category_id`.
- [x] **Pencegahan Integer Overflow**: Kalkulasi offset pagination menggunakan `saturating_mul` dan clamping `page` untuk mencegah panic pada query angka besar.
- [x] **Database Reliability (WAL Mode)**: Mengaktifkan SQLite `SqliteJournalMode::Wal` dan `busy_timeout(5 detik)` untuk mencegah `database is locked (5)` saat request konkuren.
- [x] **Indeks Unik Kategori**: Menambahkan indeks unik case-insensitive `idx_categories_name` pada tabel SQLite categories.
- [x] **Validasi Warna Kategori**: Menambahkan batas panjang maksimal warna 16 karakter (`MAX_COLOR_LENGTH = 16`, `CATEGORY_COLOR_TOO_LONG`).
- [x] **Fitur Reset Warna Kategori**: Menambahkan opsi `clear_color: true` pada `PATCH /api/v1/categories/:id`.
- [x] **Hygiene Repositori**: Menghapus `tasks.db` dari pelacakan git, memperbarui `.gitignore` (`*.db*`, `.env`), dan membuat file [**`README.md`**](../README.md) standar pada root.

---

### [x] Fase 5: Autentikasi Pengguna & Multi-Tenancy (Data Isolation)
- [x] **5.1 Dependensi Keamanan**: Menambahkan `argon2 = "0.5"`, `jsonwebtoken = "9.3"`, dan `rand = "0.8"`.
- [x] **5.2 Domain User**: Entitas `User` (`id`, `email`, `name`, `password_hash`), validasi format email & password minimum 8 karakter.
- [x] **5.3 Domain Scoping**: Menambahkan relasi kepemilikan `user_id: Uuid` pada entitas `Task` dan `Category`.
- [x] **5.4 Port Abstraksi**:
  - [x] Port `UserRepository` (`save`, `find_by_id`, `find_by_email`).
  - [x] Port `PasswordHasher` (`hash_password`, `verify_password`).
  - [x] Port `TokenService` (`generate_token`, `verify_token`).
- [x] **5.5 Application Use Cases**:
  - [x] `AuthService` (`register`, `login`, `get_me`).
  - [x] Refactor `TaskService`: seluruh operasi dibatasi per `user_id`. Keunikan judul task berlaku per pengguna (*user-scoped*).
  - [x] Refactor `CategoryService`: seluruh operasi dibatasi per `user_id`. Keunikan nama kategori berlaku per pengguna (*user-scoped*).
- [x] **5.6 Infrastructure Adapters**:
  - [x] `Argon2PasswordHasher` (implementasi `PasswordHasher` dengan salt kriptografis).
  - [x] `JwtTokenService` (implementasi `TokenService` dengan masa aktif 24 jam).
  - [x] `SqliteUserRepository` & `InMemoryUserRepository`.
  - [x] Auto-migrasi SQLite: pembuatan tabel `users`, penambahan kolom `user_id` pada `tasks` & `categories`, backfill nilai default baris lawas, dan pembuatan indeks unik per user (`idx_tasks_user_title`, `idx_categories_user_name`).
- [x] **5.7 Presentation Layer**:
  - [x] DTOs: `RegisterRequest`, `LoginRequest`, `AuthResponse`, `UserResponse`.
  - [x] Handlers: `POST /api/v1/auth/register`, `POST /api/v1/auth/login`, `GET /api/v1/auth/me`.
  - [x] Axum Extractor `AuthenticatedUser(pub Uuid)` untuk memvalidasi header `Authorization: Bearer <token>` dan menyuntikkan `user_id` ke handler yang diproteksi.
  - [x] Error mapping: `401 Unauthorized` (`INVALID_CREDENTIALS`, `UNAUTHORIZED`), `409 Conflict` (`EMAIL_ALREADY_EXISTS`).
- [x] **5.8 Tests & Verification**:
  - [x] Unit tests format email, password length, dan hashing.
  - [x] Integration tests register, login, profil, dan penolakan kredensial salah.
  - [x] Multi-tenant isolation test: User A tidak dapat membaca/mengubah/menghapus task atau kategori milik User B.
  - [x] User-scoped uniqueness test: User A dan User B dapat membuat judul task yang sama tanpa konflik.
- [x] **5.9 Dokumentasi**: Perbarui `DOCS/API.md` dengan endpoint autentikasi dan contoh request/response.

---

### [x] Fase 6: Production Web Middleware & Observability
- [x] **6.1 CORS Middleware**: Konfigurasi `tower-http::cors` agar REST API siap diakses dari browser frontend (React, Next.js, Vue, mobile).
- [x] **6.2 Request Tracing & Latency Logger**: Integrasi `tower-http::trace` untuk mencatat durasi eksekusi dan latency setiap request HTTP.
- [x] **6.3 Request Timeout Layer**: Mencegah request menggantung dengan timeout middleware (misal 30 detik).
- [x] **6.4 Graceful Shutdown**: Menangani sinyal `SIGINT` dan `SIGTERM` di `src/main.rs` agar koneksi database ditutup dengan aman saat server dihentikan.

---

### [ ] Fase 7: Fitur Bisnis Lanjutan (Subtasks & Tagging System)
- [ ] **7.1 Subtasks / Checklist**:
  - [ ] Entitas `Subtask` (`id`, `task_id`, `title`, `is_completed`).
  - [ ] Endpoint menambah, mencentang, dan menghapus subtask di dalam sebuah task.
- [ ] **7.2 Tagging System (Many-to-Many)**:
  - [ ] Entitas `Tag` (`id`, `user_id`, `name`, `color`).
  - [ ] Tabel relasi `task_tags` (`task_id`, `tag_id`).
  - [ ] Filter task berdasarkan multiple tags.

---

### [ ] Fase 8: Deployment & CI/CD
- [ ] **8.1 Multi-Stage Dockerfile**: Menghasilkan binary Rust *distroless/scratch* berukuran < 30MB yang aman dan siap produksi.
- [ ] **8.2 Docker Compose**: File `docker-compose.yml` untuk menjalankan API dan mengelola persistence database SQLite melalui volume.
- [ ] **8.3 GitHub Actions Workflow**: Pipeline `.github/workflows/ci.yml` yang menjalankan otomatis `cargo test`, `cargo clippy`, dan `cargo fmt` pada setiap *pull request* atau *push*.

---

### [ ] Fase 9: Dukungan Multi-Database (PostgreSQL Adapter)
- [ ] **9.1 Adapter PostgreSQL**:
  - [ ] `PostgresTaskRepository` (implementasi `TaskRepository` dengan `sqlx::PgPool`).
  - [ ] `PostgresCategoryRepository` (implementasi `CategoryRepository` dengan `sqlx::PgPool`).
  - [ ] `PostgresUserRepository` (implementasi `UserRepository` dengan `sqlx::PgPool`).
- [ ] **9.2 Dynamic Storage Engine Selection**:
  - [ ] Membaca prefix `DATABASE_URL` (`sqlite://` vs `postgres://`) pada saat startup di `main.rs` dan menyuntikkan adapter yang sesuai secara dinamis.
