use crate::internal::config::{ app_config::RetryConfig, database_cfg::DatabaseCfg };

#[derive(Clone, Debug)]
pub struct AppState {
    pub retry: RetryConfig,
    pub database: DatabaseCfg,
}

impl AppState {
    pub fn init() -> Self {
        AppState { retry: RetryConfig::default(), database: DatabaseCfg::init() }
    }
    
}
