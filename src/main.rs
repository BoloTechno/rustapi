use std::sync::Arc;
use tracing::info;

use rustapi::{AppState, SqliteTaskRepository, TaskService, create_router};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Inisialisasi logging tracing
    tracing_subscriber::fmt::init();

    info!("Memulai server Rust Clean Architecture API...");

    // 2. Setup SQLite Database Pool & Auto-migration
    let db_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://tasks.db?mode=rwc".to_string());
    info!("Menghubungkan ke SQLite database di: {}", db_url);

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    let sqlite_repo = SqliteTaskRepository::new(pool);
    sqlite_repo.init_db().await?;
    info!("Tabel database SQLite berhasil diverifikasi/diinisialisasi");

    // 3. Dependency Injection:
    //    SqliteTaskRepository di-inject ke TaskService via Arc<dyn TaskRepository>
    let task_repository = Arc::new(sqlite_repo);
    let task_service = Arc::new(TaskService::new(task_repository));

    // 4. Setup Presentation State dan Router Axum
    let app_state = AppState::new(task_service);
    let app = create_router(app_state);

    // 5. Binding TCP listener & Menjalankan HTTP Server
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Server aktif dan berjalan di http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
