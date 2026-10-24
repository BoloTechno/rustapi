use axum::{
    Router,
    routing::{get, patch, post},
};

use crate::presentation::handlers;
use crate::presentation::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let api_v1 = Router::new()
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
        .route("/tasks/{id}/reopen", patch(handlers::reopen_task));

    Router::new().nest("/api/v1", api_v1).with_state(state)
}
