use std::sync::Arc;
use std::time::Duration;

use crate::application::ports::TokenService;
use crate::application::{AuthService, CategoryService, TaskService};
use crate::presentation::middleware::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub task_service: Arc<TaskService>,
    pub category_service: Arc<CategoryService>,
    pub auth_service: Arc<AuthService>,
    pub token_service: Arc<dyn TokenService>,
    pub auth_rate_limiter: Arc<RateLimiter>,
}

impl AppState {
    pub fn new(
        task_service: Arc<TaskService>,
        category_service: Arc<CategoryService>,
        auth_service: Arc<AuthService>,
        token_service: Arc<dyn TokenService>,
    ) -> Self {
        Self {
            task_service,
            category_service,
            auth_service,
            token_service,
            auth_rate_limiter: Arc::new(RateLimiter::new(10, Duration::from_secs(60))),
        }
    }

    pub fn with_rate_limiter(mut self, rate_limiter: RateLimiter) -> Self {
        self.auth_rate_limiter = Arc::new(rate_limiter);
        self
    }
}
