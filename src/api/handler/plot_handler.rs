use std::sync::Arc;

use crate::{
    api::{
        AppServices,
        model::plot::{GetPlotsRequest, Plot, PlotCreationRequest, PlotList},
    },
    service::property_management::property_management_proto::{
        GetPlotsRequest as ProtoGetPlotsRequest, PlotCreationRequest as ProtoPlotCreationRequest,
        property_management_server::PropertyManagement,
    },
};
use actix_web::Result;
use actix_web::{HttpRequest, HttpResponse, Responder, web};

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
pub async fn create_plot(
    services: web::Data<Arc<AppServices>>,
    http_request: HttpRequest,
    payload: web::Json<PlotCreationRequest>,
) -> Result<impl Responder> {
    let authorization_header = crate::api::extract_authorization_header(&http_request)?;

    let request = tonic::Request::new(ProtoPlotCreationRequest::from((
        payload.into_inner(),
        authorization_header,
    )));

    match services
        .property_management_service
        .create_plot(request)
        .await
    {
        Ok(response) => {
            // Convert the successful gRPC response into the HTTP response
            Ok(HttpResponse::Ok().json(Plot::from(response.into_inner())))
        }
        Err(status) => {
            // Map the gRPC error status to an appropriate HTTP response with logging
            Ok(crate::api::map_grpc_error_to_http_response(status))
        }
    }
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
pub async fn list_plots(
    services: web::Data<Arc<AppServices>>,
    http_request: HttpRequest,
    payload: web::Json<GetPlotsRequest>,
) -> Result<impl Responder> {
    let authorization_header = crate::api::extract_authorization_header(&http_request)?;

    let request = tonic::Request::new(ProtoGetPlotsRequest::from((
        payload.into_inner(),
        authorization_header,
    )));

    match services
        .property_management_service
        .get_plots(request)
        .await
    {
        Ok(response) => {
            // Convert the successful gRPC response into the HTTP response
            Ok(HttpResponse::Ok().json(PlotList::from(response.into_inner())))
        }
        Err(status) => {
            // Map the gRPC error status to an appropriate HTTP response with logging
            Ok(crate::api::map_grpc_error_to_http_response(status))
        }
    }
}
