use envconfig::Envconfig;

/// Comma-separated CORS settings. Origins need a scheme and no trailing slash,
/// e.g. `http://localhost:5173,https://app.example.com`. `*` allows any origin.
#[derive(Debug, Clone, Envconfig)]
pub struct CORSConfig {
    #[envconfig(from = "ALLOW_ORIGINS", default = "http://localhost:5173,http://localhost:3000")]
    pub allow_origins: String,
    #[envconfig(from = "ALLOW_HEADERS", default = "authorization,content-type,x-refresh")]
    pub allow_headers: String,
    #[envconfig(from = "ALLOW_METHODS", default = "GET,POST,OPTIONS")]
    pub allow_methods: String,
}
