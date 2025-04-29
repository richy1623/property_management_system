use std::env;

use dotenvy::dotenv;
use property_management_system::service::user_management::{
    user_management_proto::user_management_server::UserManagementServer,
    user_management_service::UserManagementService,
};
use tonic::transport::Server;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let addr = "[::1]:50051".parse()?;
    let greeter = UserManagementService::new(&database_url);

    Server::builder()
        .add_service(UserManagementServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
