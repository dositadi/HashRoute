use envconfig::Envconfig;

#[derive(Debug, Clone, Envconfig)]
pub struct JWTConfig {
    #[envconfig(from = "JWT_SECRET")]
    pub secrets: String,
    #[envconfig(from = "ACCESS_EXP")]
    pub access_exp: usize,
    #[envconfig(from = "REFRESH_TOKEN")]
    pub refresh_token_exp: usize,
}

impl JWTConfig {
    pub fn init() -> Self {
        JWTConfig::init_from_env().expect("failed to load env variables")
    }
}
