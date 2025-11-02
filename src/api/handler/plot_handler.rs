use std::sync::Arc;

use crate::{
    api::{
        AppServices,
        model::plot::{GetPlotsRequest, Plot, PlotCreationRequest, PlotList},
    },
    db::model::{
        ownership::Ownership as OwnershipModel, plot::Plot as PlotModel, user::User as UserModel,
        user_permission::PermissionType,
    },
    service::property_management::property_management_proto::{
        GetPlotsRequest as ProtoGetPlotsRequest, PlotCreationRequest as ProtoPlotCreationRequest,
        property_management_server::PropertyManagement,
    },
};
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use actix_web::{Result, error::ErrorUnauthorized};

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

/// Fetch a plot with a given plot ID
///
/// Returns the plot matching the plot ID.
/// Authorization: bearer token.
#[utoipa::path(
    get,
    path = "/api/plot/{plot_id}",
    params(
        ("plot_id" = i32, Path, description = "The ID of the plot record to retrieve",
        example = 42)
    ),
    responses(
        (status = 200, description = "The desired Plot", body = Plot),
        (status = 400, description = "Missing or invalid input"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Property Management"
)]
pub async fn get_plot(
    services: web::Data<Arc<AppServices>>,
    http_request: HttpRequest,
    path: web::Path<i32>,
) -> Result<impl Responder> {
    let authorization_header = crate::api::extract_authorization_header(&http_request)?;

    let plot_id = path.into_inner();

    // Decode and validate JWT
    let jwt_token = services
        .archive_management_service
        .clone()
        .jwt
        .decode_jwt_token(&authorization_header)?;

    let mut conn = super::get_database_connection(
        services
            .archive_management_service
            .database_connection_pool
            .clone(),
    )?;

    // Get Plot
    let plot = PlotModel::get_plot(&mut conn, plot_id)?;

    let user_making_request = UserModel::get_user_by_user_name(&mut conn, &jwt_token.claims.sub)?;
    let ownerships =
        OwnershipModel::get_all_ownerships_for_user_id(&mut conn, user_making_request.user_id)?;

    if !jwt_token
        .claims
        .permissions
        .contains(&PermissionType::ViewProperties.to_string())
        && ownerships
            .iter()
            .find(|ownership| ownership.plot_id == plot_id)
            .is_none()
    {
        return Err(ErrorUnauthorized("Insufficient permissions to view plots"));
    }

    Ok(HttpResponse::Ok().json(Plot::from(plot)))
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
            PlotModel::create_plot(&mut conn, estate.estate_id, "LS-PN-1", "Phase LS", 20, 30)
                .unwrap();
        let plot_2 =
            PlotModel::create_plot(&mut conn, estate.estate_id, "LS-PN-2", "Phase LS", 25, 35)
                .unwrap();
        let _plot_3 =
            PlotModel::create_plot(&mut conn, estate.estate_id, "LS-PN-3", "Phase LS", 25, 35)
                .unwrap();

        let user = crate::db::model::tests::create_test_user(&mut conn);
        crate::db::model::tests::create_test_ownership(&mut conn, &plot_1, &user);
        crate::db::model::tests::create_test_ownership(&mut conn, &plot_2, &user);

        // Act: Send a login request with known test user credentials
        let json_payload = fs::read_to_string(
            "tests/resources/api/payload/property_management/request/list_plots.json",
        )
        .expect("Failed to read JSON file");

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
            vec!["LS-PN-1", "LS-PN-2"]
        );
    }

    #[rstest]
    #[actix_web::test]
    async fn test_get_plot(
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
                .route("/api/plot/{plot_id}", web::get().to(get_plot)),
        )
        .await;

        // Create database entries
        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot_1 =
            PlotModel::create_plot(&mut conn, estate.estate_id, "PN-1", "Phase 1", 20, 30).unwrap();
        let user = crate::db::model::tests::create_test_user(&mut conn);
        crate::db::model::tests::create_test_ownership(&mut conn, &plot_1, &user);

        let jwt_token = jwt.encode_jwt_token(ClaimsBuilder::new(&user.user_name).build());

        let req = test::TestRequest::get()
            .uri(&format!("/api/plot/{}", plot_1.plot_id))
            .insert_header(("Content-Type", "application/json"))
            .insert_header(("Authorization", format!("Bearer {jwt_token}")))
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert: Check HTTP status code and returned payload
        assert_eq!(resp.status(), StatusCode::OK);
        let response_json: Plot = test::read_body_json(resp).await;
        assert_eq!(response_json, plot_1.into());

        // Check no such plot
        let req = test::TestRequest::get()
            .uri(&format!("/api/plot/{}", 99999999))
            .insert_header(("Content-Type", "application/json"))
            .insert_header(("Authorization", format!("Bearer {jwt_token}")))
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert: Check HTTP status code and returned payload
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[rstest]
    #[actix_web::test]
    async fn test_get_plot_unauthorized(
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
                .route("/api/plot/{plot_id}", web::get().to(get_plot)),
        )
        .await;

        // Create database entries
        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot_1 =
            PlotModel::create_plot(&mut conn, estate.estate_id, "PN-1", "Phase 1", 20, 30).unwrap();
        let user = crate::db::model::tests::create_test_user(&mut conn);
        let user_2 = crate::db::model::tests::create_test_user(&mut conn);
        crate::db::model::tests::create_test_ownership(&mut conn, &plot_1, &user_2);

        let jwt_token = jwt.encode_jwt_token(ClaimsBuilder::new(&user.user_name).build());

        let req = test::TestRequest::get()
            .uri(&format!("/api/plot/{}", plot_1.plot_id))
            .insert_header(("Content-Type", "application/json"))
            .insert_header(("Authorization", format!("Bearer {jwt_token}")))
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert: Check HTTP status code and returned payload
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
