---
name: rustapi-guide
description: >-
  Panduan arsitektur, alur kerja, standar API, dan pengembangan Clean Architecture untuk proyek rustapi. Gunakan saat bekerja dengan repositori rustapi.
---

# Panduan Proyek rustapi

Dokumen ini berisi panduan lengkap pengembangan proyek **rustapi** (REST API Clean Architecture berbasis Rust).

## 1. Arsitektur Proyek
- **Domain (`src/domain/`)**: Entitas bisnis murni (`Task`), enum (`TaskStatus`, `TaskPriority`), dan error domain.
- **Application (`src/application/`)**: Layanan use cases (`TaskService`) dan port abstraksi (`TaskRepository`, `TaskFilter`).
- **Infrastructure (`src/infrastructure/`)**: Implementasi konkret adapter (`SqliteTaskRepository` dengan auto-migrasi aman & `InMemoryTaskRepository`).
- **Presentation (`src/presentation/`)**: Framework web Axum 0.8, routing, handler, DTO, pagination, dan health check.

## 2. Alur Vibe Coding & Perintah Verifikasi
Sebelum menyelesaikan tugas apa pun, selalu jalankan:
```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## 3. Dokumentasi Referensi
- [**`DOCS/TODO.md`**](../../../DOCS/TODO.md): Roadmap proyek & checklist pelacak tugas otomatis.
- [**`DOCS/API.md`**](../../../DOCS/API.md): Dokumentasi spesifikasi REST API.
- [**`DOCS/WORKFLOW.md`**](../../../DOCS/WORKFLOW.md): Panduan alur kerja dan diagram sistem.
- [**`DOCS/AGENT.md`**](../../../DOCS/AGENT.md): Instruksi detail agent.
