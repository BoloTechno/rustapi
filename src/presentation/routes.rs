use axum::{
    Router,
    routing::{get, patch, post},
};

use crate::presentation::handlers;
use crate::presentation::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let auth_routes = Router::new()
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::presentation::middleware::rate_limit_auth,
        ));

    let api_v1 = Router::new()
        .route("/health", get(handlers::health_check))
        .nest("/auth", auth_routes)
        .route("/auth/me", get(handlers::get_me))
        .route(
            "/tasks",
            post(handlers::create_task).get(handlers::list_tasks),
        )
        .route(
            "/tasks/{id}",
            get(handlers::get_task)
                .patch(handlers::update_task)
                .delete(handlers::delete_task),
        )
        .route("/tasks/{id}/complete", patch(handlers::complete_task))
        .route("/tasks/{id}/reopen", patch(handlers::reopen_task))
        .route(
            "/categories",
            post(handlers::create_category).get(handlers::list_categories),
        )
        .route(
            "/categories/{id}",
            get(handlers::get_category)
                .patch(handlers::update_category)
                .delete(handlers::delete_category),
        );

    Router::new()
        .route("/health", get(handlers::health_check))
        .nest("/api/v1", api_v1)
        .with_state(state)
}
