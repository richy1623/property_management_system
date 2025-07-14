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
    path = "/api/plot/create",
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
            Ok(HttpResponse::Created().json(Plot::from(response.into_inner())))
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
    path = "/api/plot/list",
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

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{
        db::model::{plot::Plot as PlotModel, user_permission::PermissionType},
        jwt::{ClaimsBuilder, Jwt},
        secure_storage::SecureStorage,
    };

    use super::*;

    use actix_web::{App, http::StatusCode, test};
    use diesel::{
        PgConnection,
        r2d2::{ConnectionManager, Pool},
    };
    use rstest::{fixture, rstest};

    #[fixture]
    #[once]
    pub fn test_services() -> (
        AppServices,
        Jwt,
        SecureStorage,
        Pool<ConnectionManager<PgConnection>>,
    ) {
        crate::api::tests::create_test_app_service("test_plot_handler")
    }

    #[rstest]
    #[actix_web::test]
    async fn test_create_plot(
        test_services: &(
            AppServices,
            Jwt,
            SecureStorage,
            Pool<ConnectionManager<PgConnection>>,
        ),
    ) {
        let (services, jwt, _secure_storage, pool) = test_services.clone();
        let mut conn = pool.get().unwrap();

        let app_services = Arc::new(services);
        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(app_services.clone()))
                .route("/api/plot/create", web::post().to(create_plot)),
        )
        .await;

        // Create database entries
        let estate: crate::db::model::estate::Estate =
            crate::db::model::tests::create_test_estate(&mut conn);

        // Act: Send a login request with known test user credentials
        let json_payload = fs::read_to_string(
            "tests/resources/api/payload/property_management/request/create_plot.json",
        )
        .expect("Failed to read JSON file");

        let json_payload = json_payload.replace("\"{{estate_id}}\"", &estate.estate_id.to_string());

        let jwt_token = jwt.encode_jwt_token(
            ClaimsBuilder::new("admin")
                .permission(PermissionType::ModifyProperties.to_string())
                .build(),
        );

        let req = test::TestRequest::post()
            .uri("/api/plot/create")
            .insert_header(("Content-Type", "application/json"))
            .insert_header(("Authorization", format!("Bearer {jwt_token}")))
            .set_payload(json_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert: Check HTTP status code and returned payload
        assert_eq!(resp.status(), StatusCode::CREATED);
        let response_json: Plot = test::read_body_json(resp).await;

        assert_eq!(response_json.plot_number, "P-101");
        assert_eq!(response_json.phase, "Phase 1");
        assert_eq!(response_json.length, 40);
        assert_eq!(response_json.width, 30);
        assert_eq!(response_json.estate_id, estate.estate_id);

        assert!(PlotModel::get_plot(&mut conn, response_json.plot_id).is_ok());
    }

    #[rstest]
    #[actix_web::test]
    async fn test_list_plots(
        test_services: &(
            AppServices,
            Jwt,
            SecureStorage,
            Pool<ConnectionManager<PgConnection>>,
        ),
    ) {
        let (services, jwt, _secure_storage, pool) = test_services.clone();
        let mut conn = pool.get().unwrap();

        let app_services = Arc::new(services);
        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(app_services.clone()))
                .route("/api/plot/list", web::post().to(list_plots)),
        )
        .await;

        // Create database entries
        let estate = crate::db::model::tests::create_test_estate(&mut conn);

        let plot_1 =
            PlotModel::create_plot(&mut conn, estate.estate_id, "PN-1", "Phase 1", 20, 30).unwrap();
        let plot_2 =
            PlotModel::create_plot(&mut conn, estate.estate_id, "PN-2", "Phase 1", 25, 35).unwrap();
        let _plot_3 =
            PlotModel::create_plot(&mut conn, estate.estate_id, "PN-3", "Phase 1", 25, 35).unwrap();

        let user = crate::db::model::tests::create_test_user(&mut conn);
        crate::db::model::tests::create_test_ownership(&mut conn, &plot_1, &user);
        crate::db::model::tests::create_test_ownership(&mut conn, &plot_2, &user);

        // Act: Send a login request with known test user credentials
        let json_payload = fs::read_to_string(
            "tests/resources/api/payload/property_management/request/list_plots.json",
        )
        .expect("Failed to read JSON file");

        dbg!(&json_payload);

        let jwt_token = jwt.encode_jwt_token(
            ClaimsBuilder::new("admin")
                .permission(PermissionType::ViewProperties.to_string())
                .build(),
        );

        let req = test::TestRequest::post()
            .uri("/api/plot/list")
            .insert_header(("Content-Type", "application/json"))
            .insert_header(("Authorization", format!("Bearer {jwt_token}")))
            .set_payload(json_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert: Check HTTP status code and returned payload
        assert_eq!(resp.status(), StatusCode::OK);
        let response_json: PlotList = test::read_body_json(resp).await;
        assert_eq!(response_json.plots.len(), 2);
        assert_eq!(
            response_json
                .plots
                .iter()
                .map(|plot| plot.plot_number.clone())
                .collect::<Vec<String>>(),
            vec!["PN-1", "PN-2"]
        );
    }
}
