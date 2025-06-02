use std::{env, path::PathBuf};

use dotenvy::dotenv;
use tonic::transport::Server;

use property_management_system::{
    secure_storage::SecureStorage,
    service::archive_management::{
        archive_management_proto::artifact_management_server::ArtifactManagementServer,
        archive_management_service::ArtifactManagementService,
    },
};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret_key = env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set");
    let file_storage_path = env::var("FILE_STORAGE_PATH").unwrap_or(".".to_string());

    let addr = "[::1]:50054".parse()?;

    let secure_storage = SecureStorage::new(PathBuf::from(file_storage_path), &[0u8; 32])
        .expect("Invalid file path supplied");

    let service = ArtifactManagementService::new(&database_url, &jwt_secret_key, secure_storage);

    Server::builder()
        .add_service(ArtifactManagementServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
