use std::{env, net::Ipv4Addr, path::Path, sync::Arc};

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger};
use dotenvy::dotenv;
use property_management_system::{api::AppServices, secure_storage::SecureStorage};
use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};
use tracing::Level;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    setup_logging();

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

    let rustls_config = load_rustls_config();

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
    .bind_rustls_0_23((Ipv4Addr::UNSPECIFIED, 8443), rustls_config)?
    .run()
    .await
}

fn load_rustls_config() -> rustls::ServerConfig {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    // load TLS key/cert files
    let cert_chain = CertificateDer::pem_file_iter(Path::new("certs").join("cert.pem"))
        .unwrap()
        .flatten()
        .collect();

    let key_der = PrivateKeyDer::from_pem_file(Path::new("certs").join("key.pem"))
        .expect("Could not locate PKCS 8 private keys.");

    ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key_der)
        .unwrap()
}

fn setup_logging() {
    // Default: INFO for everything
    let mut filter = tracing_subscriber::EnvFilter::new("info");

    // Enable DEBUG logs
    filter = filter.add_directive("property_management_system=debug".parse().unwrap());
    filter = filter.add_directive("actix_web=debug".parse().unwrap());
    filter = filter.add_directive("actix_server=debug".parse().unwrap());

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_env_filter(filter)
        .init();
}
