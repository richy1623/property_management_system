use actix_web::{App, HttpServer, web};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(property_management_system::api::management_server::hello)
            .service(property_management_system::api::management_server::echo)
            .route(
                "/hey",
                web::get().to(property_management_system::api::management_server::manual_hello),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
