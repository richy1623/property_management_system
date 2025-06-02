use std::env;

use dotenvy::dotenv;
use property_management_system::service::payment_management::{
    payment_management_proto::payment_management_server::PaymentManagementServer,
    payment_management_service::PaymentManagementService,
};
use tonic::transport::Server;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret_key = env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set");

    let addr = "[::1]:50052".parse()?;
    let greeter = PaymentManagementService::new(&database_url, &jwt_secret_key);

    Server::builder()
        .add_service(PaymentManagementServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
