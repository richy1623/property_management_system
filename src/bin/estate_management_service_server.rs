use std::env;

use dotenvy::dotenv;
use property_management_system::service::estate_management::{
    estate_management_proto::estate_management_server::EstateManagementServer,
    estate_management_service::EstateManagementService,
};
use tonic::transport::Server;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let addr = "[::1]:50053".parse()?;
    let greeter = EstateManagementService::new(&database_url);

    Server::builder()
        .add_service(EstateManagementServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
