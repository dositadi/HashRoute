use envconfig::Envconfig;

#[derive(Clone, Debug, Envconfig)]
pub struct DatabaseCfg {
    #[envconfig(from = "PG_SCHEME", default = "postgres")]
    pub scheme: String,
    #[envconfig(from = "PG_PASSWORD")]
    pub password: String,
    #[envconfig(from = "PG_USER", default = "divine")]
    pub user: String,
    #[envconfig(from = "PG_DB", default = "hash_db")]
    pub db: String,
    #[envconfig(from = "PG_HOST", default = "localhost")]
    pub host: String,
    #[envconfig(from = "PG_PORT", default = "5431")]
    pub port: u16,
}

impl DatabaseCfg {
    pub fn init() -> Self {
        DatabaseCfg::init_from_env().expect("Failed to load database data from environment.")
    }

    pub fn to_dsn(&self) -> String {
        
        format!(
            "{}://{}:{}@{}:{}/{}",
            self.scheme,
            self.user,
            self.password,
            self.host,
            self.port,
            self.db
        )
    }
}
