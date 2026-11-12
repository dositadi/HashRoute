
use envconfig::Envconfig;

#[derive(Debug, Clone,  envconfig::Envconfig)]
pub struct CORSConfig {
    #[envconfig(from = "ALLOW_ORIGINS")]
    pub allow_origins: String,
    #[envconfig(from= "ALLOW_HEADERS")]
    pub allow_headers: String,
    #[envconfig(from = "ALLOW_METHODS")]
    pub allow_methods: String,
}

impl CORSConfig {
    pub fn new() -> Self {
        Self::init_from_env().expect("ALLOW_ORIGINS, ALLOW_HEADERS, ALLOW_METHODS are required")
    }
}