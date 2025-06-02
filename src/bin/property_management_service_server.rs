use std::env;

use dotenvy::dotenv;
use property_management_system::service::property_management::{
    property_management_proto::property_management_server::PropertyManagementServer,
    property_management_service::PropertyManagementService,
};
use tonic::transport::Server;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret_key = env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set");

    let addr = "[::1]:50053".parse()?;
    let greeter = PropertyManagementService::new(&database_url, &jwt_secret_key);

    Server::builder()
        .add_service(PropertyManagementServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
