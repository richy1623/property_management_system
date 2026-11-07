use std::sync::Arc;

use crate::{api::AppServices, error::Error};
use actix_web::Result;
use actix_web::{HttpResponse, Responder, web};
use diesel::r2d2::R2D2Connection;

/// Health check endpoint
///
/// Returns the health status of the service and its database.
#[utoipa::path(
    get,
    path = "/api/health",
    tag = "Health Check",
    responses(
        (status = 200, description = "Service is healthy"),
        (status = 503, description = "Service or database is unavailable")
    )
)]
pub async fn health_check(services: web::Data<Arc<AppServices>>) -> Result<impl Responder> {
    let mut conn = super::get_database_connection(
        services
            .archive_management_service
            .database_connection_pool
            .clone(),
    )?;
    conn.ping().map_err(Into::<Error>::into)?;

    Ok(HttpResponse::Ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, http::StatusCode, test, web};

    #[actix_web::test]
    async fn test_health_check_ok() {
        let (app_services, _, _, _) =
            crate::api::tests::create_test_app_service("test_health_check_ok_handler");

        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(Arc::new(app_services)))
                .route("/api/health", web::get().to(health_check)),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/health").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }
}
