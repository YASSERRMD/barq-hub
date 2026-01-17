//! Configuration management for SYNAPSE Brain

use serde::Deserialize;
use std::env;

/// Main configuration structure
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
    /// Database configuration
    pub database: DatabaseConfig,
    /// Redis configuration
    pub redis: RedisConfig,
    /// Provider configurations
    pub providers: ProvidersConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProvidersConfig {
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub mistral_api_key: Option<String>,
    pub groq_api_key: Option<String>,
    pub together_api_key: Option<String>,
    pub cohere_api_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl Config {
    /// Load configuration from environment variables and config files
    pub fn load() -> Result<Self, config::ConfigError> {
        // Load .env file if present
        dotenvy::dotenv().ok();

        let config = config::Config::builder()
            // Set defaults
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 3000)?
            .set_default("server.cors_origins", vec!["*"])?
            .set_default("database.max_connections", 20)?
            .set_default("database.min_connections", 5)?
            .set_default("logging.level", "info")?
            .set_default("logging.format", "json")?
            // Load from config file if exists
            .add_source(
                config::File::with_name("config")
                    .required(false)
            )
            // Override with environment variables
            .add_source(
                config::Environment::with_prefix("SYNAPSE")
                    .separator("__")
            )
            .build()?;

        // Build config, filling in from env vars where needed
        Ok(Config {
            server: ServerConfig {
                host: config.get("server.host").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: config.get("server.port").unwrap_or(3000),
                cors_origins: config
                    .get("server.cors_origins")
                    .unwrap_or_else(|_| vec!["*".to_string()]),
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgresql://localhost/synapse".to_string()),
                max_connections: config.get("database.max_connections").unwrap_or(20),
                min_connections: config.get("database.min_connections").unwrap_or(5),
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            },
            providers: ProvidersConfig {
                openai_api_key: env::var("OPENAI_API_KEY").ok(),
                anthropic_api_key: env::var("ANTHROPIC_API_KEY").ok(),
                mistral_api_key: env::var("MISTRAL_API_KEY").ok(),
                groq_api_key: env::var("GROQ_API_KEY").ok(),
                together_api_key: env::var("TOGETHER_API_KEY").ok(),
                cohere_api_key: env::var("COHERE_API_KEY").ok(),
            },
            logging: LoggingConfig {
                level: config.get("logging.level").unwrap_or_else(|_| "info".to_string()),
                format: config.get("logging.format").unwrap_or_else(|_| "json".to_string()),
            },
        })
    }

    /// Get server address string
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        // Test that config loading doesn't panic with defaults
        let config = Config::load();
        assert!(config.is_ok());
    }
}
