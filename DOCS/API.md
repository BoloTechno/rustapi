# Spesifikasi & Dokumentasi REST API

Dokumentasi lengkap endpoint API untuk layanan **rustapi**.

- **Base URL:** `http://127.0.0.1:3000/api/v1`
- **Format Payload:** `application/json`
- **Encoding:** UTF-8
- **Autentikasi:** Bearer Token JWT pada header `Authorization: Bearer <token>` untuk seluruh endpoint yang diproteksi (Tasks & Categories).

---

## 1. Format Respon Standar

### Respon Sukses Single Item (200 OK / 201 Created)
Menggunakan envelope konsisten:
```json
{
  "success": true,
  "data": { ... }
}
```

### Respon Sukses Paginated List (200 OK)
Daftar data disertai dengan metadata pagination:
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

### Respon Error (400 / 401 / 404 / 409 / 422 / 500)
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

## 2. Autentikasi Pengguna & Multi-Tenancy

Layanan ini menerapkan isolasi data multi-penyewa (*multi-tenancy*). Pengguna yang terautentikasi hanya dapat mengakses, mengubah, dan menghapus task serta kategori miliknya sendiri. Keunikan judul task dan nama kategori berlaku per pengguna (*user-scoped*).

### A. Registrasi Pengguna Baru
Mendaftarkan akun pengguna baru dan langsung mengembalikan JWT token.

- **Method / URL:** `POST /api/v1/auth/register`
- **Headers:** `Content-Type: application/json`
- **Request Body:**
  ```json
  {
    "email": "user@example.com",
    "password": "mypassword123",
    "name": "Budi Santoso"
  }
  ```
  *Keterangan Atribut:*
  - `email` *(wajib)*: Format email valid (case-insensitive & unik global).
  - `password` *(wajib)*: Minimal 8 karakter.
  - `name` *(opsional)*: Nama lengkap pengguna.
- **Status Code:** `201 Created`
- **Response:**
  ```json
  {
    "success": true,
    "data": {
      "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
      "user": {
        "id": "c1a2b3c4-d5e6-4789-8012-3456789abcde",
        "email": "user@example.com",
        "name": "Budi Santoso",
        "created_at": "2026-10-24T21:00:00Z"
      }
    }
  }
  ```

---

### B. Login Pengguna
Melakukan autentikasi menggunakan email dan password untuk mendapatkan JWT token.

- **Method / URL:** `POST /api/v1/auth/login`
- **Headers:** `Content-Type: application/json`
- **Request Body:**
  ```json
  {
    "email": "user@example.com",
    "password": "mypassword123"
  }
  ```
- **Status Code:** `200 OK`
- **Response:**
  ```json
  {
    "success": true,
    "data": {
      "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
      "user": {
        "id": "c1a2b3c4-d5e6-4789-8012-3456789abcde",
        "email": "user@example.com",
        "name": "Budi Santoso",
        "created_at": "2026-10-24T21:00:00Z"
      }
    }
  }
  ```

---

### C. Profil Pengguna yang Sedang Login
Mengambil data profil pengguna yang terautentikasi berdasarkan Bearer token.

- **Method / URL:** `GET /api/v1/auth/me`
- **Headers:** `Authorization: Bearer <token>`
- **Status Code:** `200 OK`
- **Response:**
  ```json
  {
    "success": true,
    "data": {
      "id": "c1a2b3c4-d5e6-4789-8012-3456789abcde",
      "email": "user@example.com",
      "name": "Budi Santoso",
      "created_at": "2026-10-24T21:00:00Z"
    }
  }
  ```

---

## 3. Daftar Endpoint Sumber Daya

### A. Health Check Server
Pemeriksaan kesehatan dan status server (terbuka publik tanpa token).

- **Method / URL:** `GET /health` atau `GET /api/v1/health`
- **Status Code:** `200 OK`
- **Response:**
  ```json
  {
    "status": "ok",
    "version": "0.1.0",
    "timestamp": "2026-10-24T10:45:00.123456789Z"
  }
  ```

---

### B. Buat Task Baru
Membuat task baru untuk pengguna yang login dengan status awal `pending`. Judul task (`title`) **wajib unik per pengguna**.

- **Method / URL:** `POST /api/v1/tasks`
- **Headers:** 
  - `Authorization: Bearer <token>`
  - `Content-Type: application/json`
- **Request Body:**
  ```json
  {
    "title": "Belajar Rust Clean Architecture",
    "description": "Membangun REST API dengan Axum dan SQLite",
    "priority": "high",
    "due_date": "2026-12-31T23:59:59Z",
    "category_id": "f8a1a9e2-4567-4abc-8def-123456789abc"
  }
  ```
  *Keterangan Atribut:*
  - `title` *(wajib)*: 1 - 120 karakter Unicode (**harus unik**, case-insensitive).
  - `description` *(opsional)*: Teks deskripsi.
  - `priority` *(opsional)*: `"low"`, `"medium"`, `"high"`, `"urgent"` (default: `"medium"`).
  - `due_date` *(opsional)*: Format ISO-8601 / RFC-3339 UTC.
    - Format standar: `YYYY-MM-DDTHH:MM:SSZ` (contoh: `"2026-12-31T23:59:59Z"`).
    - Format lokal dengan timezone: `YYYY-MM-DDTHH:MM:SS+07:00` (contoh WIB: `"2026-12-31T17:00:00+07:00"`).
  - `category_id` *(opsional)*: UUID dari kategori yang sudah ada. Jika UUID yang dikirim tidak ditemukan di database, request ditolak dengan `404 Not Found` (`CATEGORY_NOT_FOUND`).

- **Status Code:** `201 Created`
- **Response:**
  ```json
  {
    "success": true,
    "data": {
      "id": "a9b1a223-8173-4a6a-8ee8-dceb2dea25ef",
      "title": "Belajar Rust Clean Architecture",
      "description": "Membangun REST API dengan Axum dan SQLite",
      "status": "pending",
      "priority": "high",
      "due_date": "2026-12-31T23:59:59Z",
      "category_id": "f8a1a9e2-4567-4abc-8def-123456789abc",
      "created_at": "2026-10-24T08:18:07.328340601Z",
      "updated_at": "2026-10-24T08:18:07.328340601Z"
    }
  }
  ```

---

### C. Daftar Seluruh Task (dengan Pagination, Filter, & Search)
Mengambil daftar task yang diurutkan dari yang terbaru (`created_at` descending).

- **Method / URL:** `GET /api/v1/tasks`
- **Query Parameters (Opsional):**
  - `page`: Nomor halaman (1-indexed, default `1`). Contoh: `?page=2`
  - `per_page`: Jumlah item per halaman (default `10`, batas maksimal `100`). Contoh: `?per_page=5`
  - `status`: Filter status (`pending` atau `completed`). Contoh: `?status=completed`
  - `priority`: Filter prioritas (`low`, `medium`, `high`, `urgent`). Contoh: `?priority=urgent`
  - `category_id`: Filter berdasarkan UUID kategori. Contoh: `?category_id=f8a1a9e2-...`
  - `search`: Pencarian kata kunci case-insensitive pada judul atau deskripsi. Contoh: `?search=Rust`
- **Status Code:** `200 OK`
- **Response:**
  ```json
  {
    "success": true,
    "data": [
      {
        "id": "a9b1a223-8173-4a6a-8ee8-dceb2dea25ef",
        "title": "Belajar Rust Clean Architecture",
        "description": "Membangun REST API dengan Axum dan SQLite",
        "status": "pending",
        "priority": "high",
        "due_date": "2026-12-31T23:59:59Z",
        "category_id": "f8a1a9e2-4567-4abc-8def-123456789abc",
        "created_at": "2026-10-24T08:18:07.328340601Z",
        "updated_at": "2026-10-24T08:18:07.328340601Z"
      }
    ],
    "pagination": {
      "page": 1,
      "per_page": 10,
      "total_items": 1,
      "total_pages": 1
    }
  }
  ```

---

### D. Ambil Detail Task Berdasarkan ID
- **Method / URL:** `GET /api/v1/tasks/{id}`
- **Path Parameters:** `id` (UUID string)
- **Status Code:** `200 OK`
- **Response:**
  ```json
  {
    "success": true,
    "data": {
      "id": "a9b1a223-8173-4a6a-8ee8-dceb2dea25ef",
      "title": "Belajar Rust Clean Architecture",
      "description": "Membangun REST API dengan Axum dan SQLite",
      "status": "pending",
      "priority": "high",
      "due_date": "2026-12-31T23:59:59Z",
      "category_id": "f8a1a9e2-4567-4abc-8def-123456789abc",
      "created_at": "2026-10-24T08:18:07.328340601Z",
      "updated_at": "2026-10-24T08:18:07.328340601Z"
    }
  }
  ```

---

### E. Perbarui Task (Parsial)
Memperbarui sebagian atribut task secara parsial. Jika judul diubah, judul baru harus unik.

- **Method / URL:** `PATCH /api/v1/tasks/{id}`
- **Path Parameters:** `id` (UUID string)
- **Request Body:**
  ```json
  {
    "title": "Judul Baru yang Diperbarui",
    "description": "Deskripsi baru",
    "priority": "urgent",
    "due_date": "2027-01-01T00:00:00Z",
    "clear_due_date": false,
    "category_id": "f8a1a9e2-4567-4abc-8def-123456789abc",
    "clear_category_id": false
  }
  ```
  *(Semua field opsional. Set `"clear_due_date": true` untuk menghapus tenggat waktu, atau `"clear_category_id": true` untuk melepas kategori. Jika `category_id` diberikan nilai baru, ID tersebut divalidasi dan akan mengembalikan `404 Not Found` (`CATEGORY_NOT_FOUND`) jika kategori tidak ada).*
- **Status Code:** `200 OK`

---

### F. Selesaikan Task (Complete)
- **Method / URL:** `PATCH /api/v1/tasks/{id}/complete`
- **Status Code:** `200 OK`

---

### G. Buka Kembali Task (Reopen)
- **Method / URL:** `PATCH /api/v1/tasks/{id}/reopen`
- **Status Code:** `200 OK`

---

### H. Hapus Task (Delete)
- **Method / URL:** `DELETE /api/v1/tasks/{id}`
- **Status Code:** `204 No Content`

---

### I. Buat Kategori Baru
Membuat kategori untuk pengelompokan task.

- **Method / URL:** `POST /api/v1/categories`
- **Headers:** `Content-Type: application/json`
- **Request Body:**
  ```json
  {
    "name": "Pekerjaan",
    "color": "#007ACC"
  }
  ```
  *Keterangan:* `name` wajib unik (1 - 50 karakter), `color` opsional kode hex (maksimal 16 karakter).
- **Status Code:** `201 Created`
- **Response:**
  ```json
  {
    "success": true,
    "data": {
      "id": "f8a1a9e2-4567-4abc-8def-123456789abc",
      "name": "Pekerjaan",
      "color": "#007ACC",
      "created_at": "2026-10-24T11:00:00.000Z",
      "updated_at": "2026-10-24T11:00:00.000Z"
    }
  }
  ```

---

### J. Daftar Seluruh Kategori
- **Method / URL:** `GET /api/v1/categories`
- **Status Code:** `200 OK`
- **Response:**
  ```json
  {
    "success": true,
    "data": [
      {
        "id": "f8a1a9e2-4567-4abc-8def-123456789abc",
        "name": "Pekerjaan",
        "color": "#007ACC",
        "created_at": "2026-10-24T11:00:00.000Z",
        "updated_at": "2026-10-24T11:00:00.000Z"
      }
    ]
  }
  ```

---

### K. Ambil Detail Kategori
- **Method / URL:** `GET /api/v1/categories/{id}`
- **Status Code:** `200 OK`

---

### L. Perbarui Kategori
- **Method / URL:** `PATCH /api/v1/categories/{id}`
- **Request Body:**
  ```json
  {
    "name": "Pekerjaan Kantor",
    "color": "#10B981",
    "clear_color": false
  }
  ```
  *(Semua field opsional. Set `"clear_color": true` untuk menghapus kode warna kategori).*
- **Status Code:** `200 OK`

---

### M. Hapus Kategori
Menghapus kategori. Task yang terhubung ke kategori ini **tidak akan dihapus**, melainkan `category_id`-nya diset ke `null`.

- **Method / URL:** `DELETE /api/v1/categories/{id}`
- **Status Code:** `204 No Content`

---

## 3. Daftar Kode Error

| Kode Error | HTTP Status | Keterangan |
| :--- | :--- | :--- |
| `EMPTY_TITLE` | `400 Bad Request` | Judul task kosong atau hanya terdiri dari spasi. |
| `TITLE_TOO_LONG` | `400 Bad Request` | Judul melebihi batas maksimum 120 karakter Unicode. |
| `DUPLICATE_TITLE` | `409 Conflict` | Judul task sudah ada (duplikat). Judul task wajib unik. |
| `INVALID_PRIORITY` | `400 Bad Request` | Nilai prioritas tidak valid (harus `low`, `medium`, `high`, atau `urgent`). |
| `TASK_NOT_FOUND` | `404 Not Found` | ID task yang diminta tidak ditemukan di database. |
| `EMPTY_CATEGORY_NAME` | `400 Bad Request` | Nama kategori kosong atau hanya spasi. |
| `CATEGORY_NAME_TOO_LONG` | `400 Bad Request` | Nama kategori melebihi 50 karakter Unicode. |
| `CATEGORY_COLOR_TOO_LONG` | `400 Bad Request` | Kode warna kategori melebihi 16 karakter Unicode. |
| `CATEGORY_NOT_FOUND` | `404 Not Found` | ID kategori tidak ditemukan. |
| `DUPLICATE_CATEGORY_NAME` | `409 Conflict` | Nama kategori sudah digunakan oleh kategori lain. |
| `INVALID_STATE_TRANSITION` | `422 Unprocessable Entity` | Transisi status tidak sah menurut aturan domain. |
| `INTERNAL_SERVER_ERROR` | `500 Internal Server Error` | Terjadi kegagalan koneksi database atau sistem internal. |

---

## 4. Contoh Skrip Pengujian Cepat (cURL)

```bash
# 1. Buat kategori baru
CAT_ID=$(curl -s -X POST http://127.0.0.1:3000/api/v1/categories \
  -H "Content-Type: application/json" \
  -d '{"name": "Pekerjaan", "color": "#007ACC"}' | jq -r '.data.id')

# 2. Buat task dengan category_id
TASK_ID=$(curl -s -X POST http://127.0.0.1:3000/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d "{
    \"title\": \"Implementasi Modul Kategori\",
    \"priority\": \"high\",
    \"category_id\": \"$CAT_ID\"
  }" | jq -r '.data.id')

# 3. Uji penolakan judul kembar (harus return 409 Conflict)
curl -i -X POST http://127.0.0.1:3000/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{"title": "Implementasi Modul Kategori"}'

# 4. Filter task berdasarkan category_id
curl -s "http://127.0.0.1:3000/api/v1/tasks?category_id=$CAT_ID" | jq .

# 5. Hapus kategori (task tetap ada dengan category_id = null)
curl -i -X DELETE "http://127.0.0.1:3000/api/v1/categories/$CAT_ID"
curl -s "http://127.0.0.1:3000/api/v1/tasks/$TASK_ID" | jq .
```
