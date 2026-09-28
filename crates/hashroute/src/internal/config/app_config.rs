use std::time::Duration;

use envconfig::Envconfig;

#[derive(Debug, Clone, Envconfig)]
pub struct AppConfig {
    #[envconfig(nested)]
    pub server: ServerConfig,
}

impl AppConfig {
    pub fn init() -> Self {
        Self::init_from_env().expect(
            "Failed to init environment variables. Check the environment variables if they are set."
        )
    }
}

#[derive(Debug, Clone, Envconfig)]
pub struct ServerConfig {
    #[envconfig(from = "HOST")]
    pub host: String,
    #[envconfig(from = "PORT")]
    pub port: u32,
}

impl ServerConfig {
    pub fn to_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u8,
    pub max_retries: Duration,
    pub min_retries: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            max_attempts: 5,
            max_retries: Duration::from_secs(2),
            min_retries: Duration::from_millis(200),
        }
    }
}
