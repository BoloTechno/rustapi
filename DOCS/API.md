# Spesifikasi & Dokumentasi REST API

Dokumentasi lengkap endpoint API untuk layanan **rustapi**.

- **Base URL:** `http://127.0.0.1:3000/api/v1`
- **Format Payload:** `application/json`
- **Encoding:** UTF-8

---

## 1. Format Respon Standar

### Respon Sukses (200 OK / 201 Created)
Menggunakan envelope konsisten:
```json
{
  "success": true,
  "data": { ... }
}
```

### Respon Error (400 / 404 / 422 / 500)
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

## 2. Daftar Endpoint

### A. Buat Task Baru
Membuat task baru dengan status awal `pending`.

- **Method / URL:** `POST /api/v1/tasks`
- **Headers:** `Content-Type: application/json`
- **Request Body:**
  ```json
  {
    "title": "Belajar Rust Clean Architecture",
    "description": "Membangun REST API dengan Axum dan SQLite"
  }
  ```
  *Keterangan:* `title` wajib (1 - 120 karakter Unicode), `description` opsional.

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
      "created_at": "2026-10-22T08:18:07.328340601Z",
      "updated_at": "2026-10-22T08:18:07.328340601Z"
    }
  }
  ```

---

### B. Daftar Seluruh Task
Mengambil daftar task yang diurutkan dari yang terbaru (`created_at` descending).

- **Method / URL:** `GET /api/v1/tasks`
- **Query Parameters (Opsional):**
  - `status`: filter status (`pending` atau `completed`). Contoh: `?status=completed`
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
        "created_at": "2026-10-22T08:18:07.328340601Z",
        "updated_at": "2026-10-22T08:18:07.328340601Z"
      }
    ]
  }
  ```

---

### C. Ambil Detail Task Berdasarkan ID
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
      "created_at": "2026-10-22T08:18:07.328340601Z",
      "updated_at": "2026-10-22T08:18:07.328340601Z"
    }
  }
  ```

---

### D. Perbarui Judul / Deskripsi Task
Memperbarui sebagian atribut task secara parsial.

- **Method / URL:** `PATCH /api/v1/tasks/{id}`
- **Path Parameters:** `id` (UUID string)
- **Request Body:**
  ```json
  {
    "title": "Judul Baru yang Diperbarui",
    "description": "Deskripsi baru"
  }
  ```
  *(Semua field bersifat opsional. Masukkan hanya atribut yang ingin diubah).*
- **Status Code:** `200 OK`
- **Response:**
  ```json
  {
    "success": true,
    "data": {
      "id": "a9b1a223-8173-4a6a-8ee8-dceb2dea25ef",
      "title": "Judul Baru yang Diperbarui",
      "description": "Deskripsi baru",
      "status": "pending",
      "created_at": "2026-10-22T08:18:07.328340601Z",
      "updated_at": "2026-10-22T08:25:12.441102910Z"
    }
  }
  ```

---

### E. Selesaikan Task (Complete)
Mengubah status task menjadi `completed`. Bersifat **Idempotent** (aman dipanggil berulang kali).

- **Method / URL:** `PATCH /api/v1/tasks/{id}/complete`
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
      "status": "completed",
      "created_at": "2026-10-22T08:18:07.328340601Z",
      "updated_at": "2026-10-22T08:30:19.248391577Z"
    }
  }
  ```

---

### F. Buka Kembali Task (Reopen)
Mengembalikan status task yang sudah selesai menjadi `pending`. Bersifat **Idempotent**.

- **Method / URL:** `PATCH /api/v1/tasks/{id}/reopen`
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
      "created_at": "2026-10-22T08:18:07.328340601Z",
      "updated_at": "2026-10-22T08:35:10.129384912Z"
    }
  }
  ```

---

### G. Hapus Task (Delete)
Menghapus task dari penyimpanan permanen.

- **Method / URL:** `DELETE /api/v1/tasks/{id}`
- **Path Parameters:** `id` (UUID string)
- **Status Code:** `204 No Content`
- **Response Body:** *Kosong (0 byte sesuai standar resmi HTTP RFC 9110).*

---

## 3. Daftar Kode Error

| Kode Error | HTTP Status | Keterangan |
| :--- | :--- | :--- |
| `EMPTY_TITLE` | `400 Bad Request` | Judul task kosong atau hanya terdiri dari spasi. |
| `TITLE_TOO_LONG` | `400 Bad Request` | Judul melebihi batas maksimum 120 karakter Unicode. |
| `TASK_NOT_FOUND` | `404 Not Found` | ID task yang diminta tidak ditemukan di database. |
| `INVALID_STATE_TRANSITION` | `422 Unprocessable Entity` | Transisi status tidak sah menurut aturan domain. |
| `INTERNAL_SERVER_ERROR` | `500 Internal Server Error` | Terjadi kegagalan koneksi database atau sistem internal. |

---

## 4. Contoh Skrip Pengujian Cepat (cURL)

```bash
# 1. Tambah task
TASK_ID=$(curl -s -X POST http://127.0.0.1:3000/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{"title": "Membaca Buku Rust 🦀", "description": "Bab 1-5"}' \
  | grep -o '"id":"[^"]*' | cut -d'"' -f4)

echo "Created Task ID: $TASK_ID"

# 2. Ambil detail task
curl -s http://127.0.0.1:3000/api/v1/tasks/$TASK_ID

# 3. Update task
curl -s -X PATCH http://127.0.0.1:3000/api/v1/tasks/$TASK_ID \
  -H "Content-Type: application/json" \
  -d '{"title": "Membaca Buku Rust 🦀 (Revisi Bab 1-7)"}'

# 4. Selesaikan task (Complete)
curl -s -X PATCH http://127.0.0.1:3000/api/v1/tasks/$TASK_ID/complete

# 5. Filter task completed
curl -s "http://127.0.0.1:3000/api/v1/tasks?status=completed"

# 6. Buka kembali task (Reopen)
curl -s -X PATCH http://127.0.0.1:3000/api/v1/tasks/$TASK_ID/reopen

# 7. Hapus task
curl -s -i -X DELETE http://127.0.0.1:3000/api/v1/tasks/$TASK_ID
```
