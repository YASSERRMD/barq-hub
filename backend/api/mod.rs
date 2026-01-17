//! REST API implementation

mod handlers;
mod routes;
mod state;
mod middleware;
mod workflow_handlers;
mod knowledge_handlers;
mod governance_handlers;
mod agent_handlers;
mod provider_handlers;
mod admin_handlers;
mod voice_handlers;

pub use routes::create_router;
pub use state::AppState;


