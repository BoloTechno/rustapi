use rustapi::domain::TaskStatus;
use rustapi::{SqliteTaskRepository, TaskService};
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;

#[tokio::test]
async fn test_sqlite_repository_crud_flow() {
    // Gunakan in-memory SQLite pool untuk pengujian yang terisolasi dan cepat
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    let sqlite_repo = SqliteTaskRepository::new(pool);
    sqlite_repo.init_db().await.unwrap();

    let service = TaskService::new(Arc::new(sqlite_repo));

    // 1. Create Task di SQLite
    let created = service
        .create_task(
            "Belajar SQLite di Rust 🦀".into(),
            Some("Persistensi data Clean Architecture".into()),
        )
        .await
        .unwrap();

    assert_eq!(created.title(), "Belajar SQLite di Rust 🦀");
    assert_eq!(created.status(), TaskStatus::Pending);

    // 2. Read by ID dari SQLite
    let fetched = service.get_task_by_id(created.id()).await.unwrap();
    assert_eq!(fetched.id(), created.id());
    assert_eq!(fetched.title(), "Belajar SQLite di Rust 🦀");
    assert_eq!(
        fetched.description(),
        Some("Persistensi data Clean Architecture")
    );

    // 3. Update task di SQLite
    let updated = service
        .update_task(
            created.id(),
            Some("Belajar SQLite & SQLx di Rust".into()),
            Some("Update deskripsi".into()),
        )
        .await
        .unwrap();
    assert_eq!(updated.title(), "Belajar SQLite & SQLx di Rust");
    assert_eq!(updated.description(), Some("Update deskripsi"));

    // 4. Complete task di SQLite
    let completed = service.complete_task(created.id()).await.unwrap();
    assert_eq!(completed.status(), TaskStatus::Completed);

    // 4b. Reopen task di SQLite
    let reopened = service.reopen_task(created.id()).await.unwrap();
    assert_eq!(reopened.status(), TaskStatus::Pending);

    // Complete lagi untuk lanjut ke pengujian list
    service.complete_task(created.id()).await.unwrap();

    // 5. List tasks dengan filter status di SQLite
    let completed_list = service
        .list_tasks(Some(TaskStatus::Completed))
        .await
        .unwrap();
    assert_eq!(completed_list.len(), 1);
    assert_eq!(completed_list[0].id(), created.id());

    let pending_list = service.list_tasks(Some(TaskStatus::Pending)).await.unwrap();
    assert_eq!(pending_list.len(), 0);

    // 6. Delete task di SQLite
    service.delete_task(created.id()).await.unwrap();

    // 7. Verifikasi setelah dihapus
    let err = service.get_task_by_id(created.id()).await;
    assert!(err.is_err());
}
