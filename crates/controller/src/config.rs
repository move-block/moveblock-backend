use crate::service;

use database::db::{new_postgres_pool, PostgresPool};
use middleware::AptosAuth;

use actix_web::web::{self, Data};

pub struct ApiContext {
    pub app_db: PostgresPool,
    pub aptos_full_index_db: PostgresPool,
    pub function_index_db: PostgresPool,
}

impl ApiContext {
    pub fn new(
        app_db: PostgresPool,
        aptos_full_index_db: PostgresPool,
        function_index_db: PostgresPool,
    ) -> Self {
        ApiContext {
            app_db,
            aptos_full_index_db,
            function_index_db,
        }
    }
}

pub fn config_port() -> u16 {
    std::env::var("PORT")
        .expect("env PORT not found")
        .parse()
        .unwrap()
}

pub fn config_cors() -> actix_cors::Cors {
    actix_cors::Cors::default()
        .allow_any_origin()
        .allow_any_method()
        .allow_any_header()
}

pub fn config_service(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(
                service::block_stack::routers(web::scope("/block-stacks")).wrap(AptosAuth::build()),
            )
            .service(service::account::routers(web::scope("/accounts")))
            .service(service::function::routers(web::scope("/functions"))),
    );
}

pub fn config_app_data(cfg: &mut web::ServiceConfig) {
    let app_db = new_postgres_pool("DATABASE_URL");
    let function_indexer_db = new_postgres_pool("FUNCTION_INDEXER_URL");
    let aptos_full_indexer_db = new_postgres_pool("FULL_INDEXER_URL");

    cfg.app_data(Data::new(ApiContext::new(
        app_db,
        aptos_full_indexer_db,
        function_indexer_db,
    )));
}
