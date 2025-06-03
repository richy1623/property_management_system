use crate::api::model::estate::{Estate, EstateCreationRequest, EstateList};
use actix_web::{HttpResponse, Responder, web};

/// Create a new estate
///
/// Accepts estate details such as name, location, district, and landmark,
/// and creates a new estate record in the system.
/// Authorization: bearer token.
#[utoipa::path(
    post,
    path = "/estate/create",
    request_body(content = EstateCreationRequest, description = "Estate creation data"),
    responses(
        (status = 201, description = "Estate created successfully", body = Estate),
        (status = 400, description = "Missing or invalid input"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Property Management"
)]
pub async fn create_estate(_payload: web::Json<EstateCreationRequest>) -> impl Responder {
    HttpResponse::NotImplemented()
}

/// Retrieve all estates in the system
///
/// Returns a list of all estates stored in the system.
/// Authorization: bearer token.
#[utoipa::path(
    get,
    path = "/estate/list",
    responses(
        (status = 200, description = "List of estates", body = EstateList),
        (status = 400, description = "Missing or invalid input"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Property Management"
)]
pub async fn list_estates() -> impl Responder {
    HttpResponse::NotImplemented()
}
