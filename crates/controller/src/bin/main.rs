use controller::config;

use actix_web::middleware::{Compress, Logger, NormalizePath};
use actix_web::{App, HttpServer};
use env_logger::Env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(NormalizePath::trim())
            .wrap(Compress::default())
            .wrap(config::config_cors())
            .configure(config::config_service)
            .configure(config::config_app_data)
    })
    .bind(("0.0.0.0", config::config_port()))?
    .run()
    .await
}
