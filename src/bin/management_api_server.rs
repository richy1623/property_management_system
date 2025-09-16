use std::{env, net::Ipv4Addr, sync::Arc};

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger};
use dotenvy::dotenv;
use property_management_system::{api::AppServices, secure_storage::SecureStorage};
use tracing::Level;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret_key = env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set");
    let file_storage_path = env::var("FILE_STORAGE_PATH").unwrap_or(".".to_string());
    let secure_storage_encryption_key_bytes = env::var("SECURE_STORAGE_ENCRYPTION_KEY")
        .expect("SECURE_STORAGE_ENCRYPTION_KEY must be set")
        .into_bytes();

    let secure_storage_encryption_key = secure_storage_encryption_key_bytes
        .as_slice()
        .try_into()
        .expect("key must be 32 bytes");

    let secure_storage = match SecureStorage::new(file_storage_path, secure_storage_encryption_key)
    {
        Ok(secure_storage) => secure_storage,
        Err(e) => panic!("Failed to initialize SecureStorage module: {:?}", e),
    };

    let app_services: Arc<AppServices> = Arc::new(AppServices::new(
        &database_url,
        &jwt_secret_key,
        secure_storage,
    ));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(
                // TODO remove for PROD
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .service(
                property_management_system::api::server::management::api_routes(
                    app_services.clone(),
                ),
            )
    })
    .bind((Ipv4Addr::UNSPECIFIED, 8080))?
    .run()
    .await
}
