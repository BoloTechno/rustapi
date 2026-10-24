use chrono::Utc;
use rustapi::application::ports::CategoryRepository;
use rustapi::domain::{TaskPriority, TaskStatus};
use rustapi::{Category, SqliteCategoryRepository, SqliteTaskRepository, TaskFilter, TaskService};
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_sqlite_repository_crud_flow() {
    // Gunakan in-memory SQLite pool untuk pengujian yang terisolasi dan cepat
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    let cat_repo = SqliteCategoryRepository::new(pool.clone());
    cat_repo.init_db().await.unwrap();

    let sqlite_repo = SqliteTaskRepository::new(pool.clone());
    sqlite_repo.init_db().await.unwrap();

    let service = TaskService::new(Arc::new(sqlite_repo), Arc::new(cat_repo));

    let due_date = Utc::now();

    // 1. Create Task di SQLite dengan Priority & Due Date
    let created = service
        .create_task(
            "Belajar SQLite di Rust 🦀".into(),
            Some("Persistensi data Clean Architecture".into()),
            Some(TaskPriority::High),
            Some(due_date),
            None,
        )
        .await
        .unwrap();

    assert_eq!(created.title(), "Belajar SQLite di Rust 🦀");
    assert_eq!(created.status(), TaskStatus::Pending);
    assert_eq!(created.priority(), TaskPriority::High);
    assert!(created.due_date().is_some());

    // 2. Read by ID dari SQLite
    let fetched = service.get_task_by_id(created.id()).await.unwrap();
    assert_eq!(fetched.id(), created.id());
    assert_eq!(fetched.title(), "Belajar SQLite di Rust 🦀");
    assert_eq!(
        fetched.description(),
        Some("Persistensi data Clean Architecture")
    );
    assert_eq!(fetched.priority(), TaskPriority::High);

    // 3. Update task di SQLite (ubah judul, deskripsi, dan prioritas)
    let updated = service
        .update_task(
            created.id(),
            Some("Belajar SQLite & SQLx di Rust".into()),
            Some("Update deskripsi".into()),
            Some(TaskPriority::Urgent),
            None,
            None,
        )
        .await
        .unwrap();
    assert_eq!(updated.title(), "Belajar SQLite & SQLx di Rust");
    assert_eq!(updated.description(), Some("Update deskripsi"));
    assert_eq!(updated.priority(), TaskPriority::Urgent);

    // 4. Complete task di SQLite
    let completed = service.complete_task(created.id()).await.unwrap();
    assert_eq!(completed.status(), TaskStatus::Completed);

    // 4b. Reopen task di SQLite
    let reopened = service.reopen_task(created.id()).await.unwrap();
    assert_eq!(reopened.status(), TaskStatus::Pending);

    // Complete lagi untuk lanjut ke pengujian list
    service.complete_task(created.id()).await.unwrap();

    // 5. List tasks dengan filter status di SQLite
    let (completed_list, total_completed) = service
        .list_tasks(TaskFilter {
            status: Some(TaskStatus::Completed),
            page: 1,
            per_page: 10,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(completed_list.len(), 1);
    assert_eq!(total_completed, 1);
    assert_eq!(completed_list[0].id(), created.id());

    let (pending_list, total_pending) = service
        .list_tasks(TaskFilter {
            status: Some(TaskStatus::Pending),
            page: 1,
            per_page: 10,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(pending_list.len(), 0);
    assert_eq!(total_pending, 0);

    // 6. Delete task di SQLite
    service.delete_task(created.id()).await.unwrap();

    // 7. Verifikasi setelah dihapus
    let err = service.get_task_by_id(created.id()).await;
    assert!(err.is_err());
}

#[tokio::test]
async fn test_sqlite_pagination_and_search() {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    let cat_repo = SqliteCategoryRepository::new(pool.clone());
    cat_repo.init_db().await.unwrap();

    let sqlite_repo = SqliteTaskRepository::new(pool);
    sqlite_repo.init_db().await.unwrap();

    let service = TaskService::new(Arc::new(sqlite_repo), Arc::new(cat_repo));

    // Buat 5 task dengan judul dan deskripsi bervariasi
    for i in 1..=5 {
        service
            .create_task(
                format!("Task #{i} Rust Clean Architecture"),
                Some(format!("Deskripsi nomor {i}")),
                if i % 2 == 0 {
                    Some(TaskPriority::Urgent)
                } else {
                    Some(TaskPriority::Low)
                },
                None,
                None,
            )
            .await
            .unwrap();
    }

    // 1. Pagination: Page 1, limit 2
    let (page_1, total_1) = service
        .list_tasks(TaskFilter {
            page: 1,
            per_page: 2,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(page_1.len(), 2);
    assert_eq!(total_1, 5);

    // 2. Pagination: Page 2, limit 2
    let (page_2, total_2) = service
        .list_tasks(TaskFilter {
            page: 2,
            per_page: 2,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(page_2.len(), 2);
    assert_eq!(total_2, 5);
    // Pastikan item tidak tumpang tindih
    assert_ne!(page_1[0].id(), page_2[0].id());

    // 3. Search: cari kata kunci "nomor 3"
    let (search_res, search_total) = service
        .list_tasks(TaskFilter {
            search: Some("nomor 3".into()),
            page: 1,
            per_page: 10,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(search_res.len(), 1);
    assert_eq!(search_total, 1);
    assert_eq!(search_res[0].title(), "Task #3 Rust Clean Architecture");

    // 4. Filter by Priority Urgent (i = 2, 4 -> 2 items)
    let (urgent_res, urgent_total) = service
        .list_tasks(TaskFilter {
            priority: Some(TaskPriority::Urgent),
            page: 1,
            per_page: 10,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(urgent_res.len(), 2);
    assert_eq!(urgent_total, 2);
}

#[tokio::test]
async fn test_sqlite_category_relation_and_foreign_key() {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    let cat_repo = Arc::new(SqliteCategoryRepository::new(pool.clone()));
    cat_repo.init_db().await.unwrap();

    let task_repo = Arc::new(SqliteTaskRepository::new(pool));
    task_repo.init_db().await.unwrap();

    let service = TaskService::new(task_repo.clone(), cat_repo.clone());

    // 0. Coba buat task dengan category_id yang tidak ada di SQLite
    let fake_cat_id = Uuid::new_v4();
    let err = service
        .create_task(
            "Task Kategori Gaib".into(),
            None,
            None,
            None,
            Some(fake_cat_id),
        )
        .await;
    assert!(matches!(
        err,
        Err(rustapi::ApplicationError::CategoryNotFound(id)) if id == fake_cat_id
    ));

    // 1. Simpan kategori ke database
    let category = Category::new("Testing SQLite".into(), Some("#123456".into())).unwrap();
    rustapi::application::ports::CategoryRepository::save(cat_repo.as_ref(), &category)
        .await
        .unwrap();

    // 2. Buat task dengan category_id
    let task = service
        .create_task(
            "Task Dengan Kategori".into(),
            None,
            None,
            None,
            Some(category.id()),
        )
        .await
        .unwrap();
    assert_eq!(task.category_id(), Some(category.id()));

    // 3. Filter task berdasarkan category_id
    let (list, count) = service
        .list_tasks(TaskFilter {
            category_id: Some(category.id()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(count, 1);
    assert_eq!(list[0].id(), task.id());

    // 4. Hapus kategori via CategoryService dan verifikasi category_id di task jadi None
    let cat_service = rustapi::CategoryService::new(cat_repo.clone(), task_repo.clone());
    cat_service.delete_category(category.id()).await.unwrap();
    let reloaded = service.get_task_by_id(task.id()).await.unwrap();
    assert_eq!(reloaded.category_id(), None);

    // 5. Verifikasi bahwa SQLite index idx_categories_name menolak nama kategori kembar case-insensitive
    let cat1 = Category::new("CaseSensitive".into(), None).unwrap();
    cat_repo.as_ref().save(&cat1).await.unwrap();

    let cat2 = Category::new("casesensitive".into(), None).unwrap();
    let err_dup = cat_repo.as_ref().save(&cat2).await;
    assert!(matches!(
        err_dup,
        Err(rustapi::ApplicationError::CategoryNameAlreadyExists(_))
    ));
}

#[tokio::test]
async fn test_legacy_schema_migration_preserves_existing_data() {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    // 1. Simulasikan skema database lama (sebelum ada kolom priority, due_date, category_id)
    sqlx::query(
        r#"
        CREATE TABLE tasks (
            id TEXT PRIMARY KEY NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // 2. Masukkan data lawas (legacy data)
    let legacy_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO tasks (id, title, description, status, created_at, updated_at)
        VALUES (?, 'Legacy Task Data', 'Deskripsi lama sebelum migrasi', 'pending', '2026-10-20T10:00:00Z', '2026-10-20T10:00:00Z');
        "#,
    )
    .bind(legacy_id.to_string())
    .execute(&pool)
    .await
    .unwrap();

    // 3. Jalankan auto-migrasi repository baru
    let cat_repo = SqliteCategoryRepository::new(pool.clone());
    cat_repo.init_db().await.unwrap();

    let sqlite_repo = SqliteTaskRepository::new(pool);
    sqlite_repo.init_db().await.unwrap();

    let service = TaskService::new(Arc::new(sqlite_repo), Arc::new(cat_repo));

    // 4. Baca task lawas melalui service: data lama utuh & mendapatkan nilai default
    let fetched = service.get_task_by_id(legacy_id).await.unwrap();
    assert_eq!(fetched.id(), legacy_id);
    assert_eq!(fetched.title(), "Legacy Task Data");
    assert_eq!(
        fetched.description(),
        Some("Deskripsi lama sebelum migrasi")
    );
    assert_eq!(fetched.status(), TaskStatus::Pending);
    assert_eq!(fetched.priority(), TaskPriority::Medium); // Nilai default otomatis
    assert_eq!(fetched.due_date(), None); // None jika belum diisi
    assert_eq!(fetched.category_id(), None); // None jika belum diisi

    // 5. Verifikasi bahwa task lawas bisa di-update ke prioritas baru
    let updated = service
        .update_task(
            legacy_id,
            None,
            None,
            Some(TaskPriority::Urgent),
            None,
            None,
        )
        .await
        .unwrap();
    assert_eq!(updated.priority(), TaskPriority::Urgent);
}
