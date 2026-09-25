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
