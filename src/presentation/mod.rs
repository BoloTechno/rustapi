pub mod dto;
pub mod errors;
pub mod extractors;
pub mod handlers;
pub mod routes;
pub mod state;

pub use dto::{PaginatedResponse, PaginationMeta};
pub use errors::ApiError;
pub use extractors::AuthenticatedUser;
pub use routes::create_router;
pub use state::AppState;
