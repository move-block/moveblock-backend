use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::time::Duration;

pub type PostgresPool = Pool<Postgres>;

pub fn new_postgres_pool(env_var: &str) -> PostgresPool {
    let url_from_env =
        std::env::var(env_var).unwrap_or_else(|_| panic!("env var {} not found", env_var));

    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy(&url_from_env)
        .unwrap()
}
