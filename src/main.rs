use std::str::FromStr;
use std::sync::Arc;
use tracing::{info, warn};

use rustapi::{
    AppState, Argon2PasswordHasher, AuthService, CategoryService, JwtTokenService,
    SqliteCategoryRepository, SqliteTaskRepository, SqliteUserRepository, TaskService,
    create_router,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Muat konfigurasi .env jika tersedia
    dotenvy::dotenv().ok();

    // 2. Inisialisasi logging tracing
    tracing_subscriber::fmt::init();

    info!("Memulai server Rust Clean Architecture API...");

    // 3. Setup SQLite Database Pool & Auto-migration
    let db_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://tasks.db?mode=rwc".to_string());
    info!("Menghubungkan ke SQLite database di: {}", db_url);

    let connection_options = sqlx::sqlite::SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(5));

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connection_options)
        .await?;

    let sqlite_user_repo = SqliteUserRepository::new(pool.clone());
    sqlite_user_repo.init_db().await?;

    let sqlite_category_repo = SqliteCategoryRepository::new(pool.clone());
    sqlite_category_repo.init_db().await?;

    let sqlite_task_repo = SqliteTaskRepository::new(pool);
    sqlite_task_repo.init_db().await?;
    info!("Tabel database SQLite berhasil diverifikasi/diinisialisasi");

    // 4. Security Adapters & Validasi JWT_SECRET
    let jwt_secret = match std::env::var("JWT_SECRET") {
        Ok(secret) if !secret.trim().is_empty() => {
            if secret.len() < 32 {
                warn!(
                    "JWT_SECRET relatif pendek ({} karakter). Disarankan minimal 32 karakter untuk algoritma HS256.",
                    secret.len()
                );
            }
            secret
        }
        _ => {
            let is_production = std::env::var("ENVIRONMENT")
                .or_else(|_| std::env::var("APP_ENV"))
                .map(|v| v.eq_ignore_ascii_case("production"))
                .unwrap_or(false);

            if is_production {
                panic!(
                    "FATAL: Environment variable JWT_SECRET wajib diatur pada environment produksi!"
                );
            }

            warn!(
                "PERINGATAN: JWT_SECRET tidak diatur di environment. Menggunakan kunci acak dinamis untuk sesi ini."
            );
            format!(
                "{}{}",
                uuid::Uuid::new_v4().simple(),
                uuid::Uuid::new_v4().simple()
            )
        }
    };

    let jwt_expiration_hours = std::env::var("JWT_EXPIRATION_HOURS")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|&h| h > 0)
        .unwrap_or(24);

    let token_service = Arc::new(JwtTokenService::new(jwt_secret, jwt_expiration_hours));
    let password_hasher = Arc::new(Argon2PasswordHasher::new());

    // 5. Dependency Injection: Repositories & Services
    let user_repository = Arc::new(sqlite_user_repo);
    let category_repository = Arc::new(sqlite_category_repo);
    let task_repository = Arc::new(sqlite_task_repo);

    let auth_service = Arc::new(AuthService::new(
        user_repository,
        password_hasher,
        token_service.clone(),
    ));

    let category_service = Arc::new(CategoryService::new(
        category_repository.clone(),
        task_repository.clone(),
    ));
    let task_service = Arc::new(TaskService::new(task_repository, category_repository));

    // 6. Setup Presentation State dan Router Axum
    let app_state = AppState::new(task_service, category_service, auth_service, token_service);
    let app = create_router(app_state);

    // 7. Binding TCP listener & Menjalankan HTTP Server
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{host}:{port}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Server aktif dan berjalan di http://{}", addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    info!("Server berhasil dimatikan dengan aman.");

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Gagal menginstal handler sinyal Ctrl+C");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Gagal menginstal handler sinyal SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Sinyal shutdown diterima. Menghentikan server secara graceful...");
}
