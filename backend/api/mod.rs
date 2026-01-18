//! REST API implementation

mod handlers;
mod routes;
mod state;
mod middleware;
mod governance_handlers;

mod provider_handlers;
mod admin_handlers;
mod settings_handlers;
mod voice_handlers;

pub use routes::create_router;
pub use state::AppState;


