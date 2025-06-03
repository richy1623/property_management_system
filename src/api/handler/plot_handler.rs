use crate::api::model::plot::{GetPlotsRequest, Plot, PlotCreationRequest, PlotList};
use actix_web::{HttpResponse, Responder, web};

/// Create a new plot within an estate
///
/// Accepts plot details like estate ID, plot number, phase, dimensions,
/// and creates a new plot record under the specified estate.
/// Authorization: bearer token.
#[utoipa::path(
    post,
    path = "/plot/create",
    request_body(content = PlotCreationRequest, description = "Plot creation data"),
    responses(
        (status = 201, description = "Plot created successfully", body = Plot),
        (status = 400, description = "Missing or invalid input"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Property Management"
)]
pub async fn create_plot(_payload: web::Json<PlotCreationRequest>) -> impl Responder {
    HttpResponse::NotImplemented()
}

/// Retrieve plots filtered by optional criteria
///
/// Accepts filter parameters like estate ID, plot number, phase, and ownership states,
/// and returns matching plot records.
/// Authorization: bearer token.
#[utoipa::path(
    post,
    path = "/plot/list",
    request_body(content = GetPlotsRequest, description = "Optional filters for plots"),
    responses(
        (status = 200, description = "List of plots", body = PlotList),
        (status = 400, description = "Missing or invalid input"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Property Management"
)]
pub async fn list_plots(_payload: web::Json<GetPlotsRequest>) -> impl Responder {
    HttpResponse::NotImplemented()
}
