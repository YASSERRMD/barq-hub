//! BARQ HUB - AI Management Console
//!
//! A complete AI orchestration platform with:
//! - Multi-provider LLM gateway (Phase 1)
//! - Workflow orchestration engine (Phase 2)
//! - Knowledge and RAG system (Phase 3)
//! - Enterprise governance (Phase 4)

pub mod types;
pub mod providers;
pub mod router;
pub mod api;
pub mod cost;
pub mod config;
pub mod error;
pub mod workflow;
pub mod knowledge;
pub mod governance;
pub mod agents;
pub mod db;

pub use types::*;
pub use error::*;
pub use config::Config;

