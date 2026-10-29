use std::{ time::Duration };

use sqlx::{ Connection, Executor, Result, migrate, postgres::PgPoolOptions };
use tracing::info;

use crate::internal::config::{
    app_config::RetryConfig,
    database_cfg::DatabaseCfg,
    jwt_config::JWTConfig,
};

#[derive(Clone, Debug)]
pub struct AppState {
    pub repository: Repository,
    pub jwt_secret: JWTConfig,
}

impl AppState {
    pub async fn init(cfg: &DatabaseCfg) -> Self {
        AppState {
            repository: Repository {
                retry: RetryConfig::default(),
                postgres: Postgres::new(cfg).await.unwrap(),
            },
            jwt_secret: JWTConfig::init(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Repository {
    pub retry: RetryConfig,
    pub postgres: Postgres,
}

#[derive(Debug, Clone)]
pub struct Postgres {
    pub db_pool: sqlx::postgres::PgPool,
}

impl Postgres {
    pub async fn new(cfg: &DatabaseCfg) -> Result<Self> {
        let options = PgPoolOptions::new()
            .max_connections(cfg.max_conn.into())
            .acquire_timeout(Duration::from_secs(cfg.acquire_timeout))
            .idle_timeout(Duration::from_secs(cfg.idle_timeout))
            .before_acquire(|conn, meta|
                Box::pin(async move {
                    if meta.idle_for.as_secs() > 60 {
                        conn.ping().await?;
                    }

                    info!("establishing connection");

                    Ok(true)
                })
            )
            .after_release(|conn, meta|
                Box::pin(async move {
                    // Check that connection exceeds 6hours
                    if meta.age.as_secs() < 6 * 60 * 60 {
                        return Ok(true);
                    }

                    let total_memory_usage: i16 = sqlx
                        ::query_scalar("select sum(used_bytes) from pg_backend_memory_contexts")
                        .fetch_one(conn).await?;

                    Ok(total_memory_usage <= 1 << 28)
                })
            )
            .after_connect(|conn, _met|
                Box::pin(async move {
                    conn.execute("SET application_name = 'hash_route';").await?;

                    info!("connection established");

                    Ok(())
                })
            );

        let pool = options.connect(cfg.to_dsn().as_str()).await.unwrap();

        info!("Postgres pool created succefully!");

        Ok(Self { db_pool: pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        migrate!("../../migrations").run(&self.db_pool).await?;

        Ok(())
    }
}
