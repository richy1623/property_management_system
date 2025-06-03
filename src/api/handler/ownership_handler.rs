use crate::api::model::ownership::{Ownership, OwnershipCreationRequest};
use actix_web::{HttpResponse, Responder, web};

/// Create a new ownership record
///
/// Accepts user ID, plot ID, ownership state, total sale value, and repayment details.
/// Creates a new ownership link between a user and a plot in the system.
/// Authorization: bearer token.
#[utoipa::path(
    post,
    path = "/ownership/create",
    request_body(content = OwnershipCreationRequest, description = "Ownership creation data"),
    responses(
        (status = 201, description = "Ownership created successfully", body = Ownership),
        (status = 400, description = "Missing or invalid input"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Property Management"
)]
pub async fn create_ownership(_payload: web::Json<OwnershipCreationRequest>) -> impl Responder {
    HttpResponse::NotImplemented()
}
