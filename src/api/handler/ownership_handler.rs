use std::sync::Arc;

use crate::{
    api::{
        AppServices,
        model::ownership::{
            Ownership, OwnershipCreationRequest, OwnershipFilter, OwnershipUpdateRequest,
        },
    },
    db::model::{
        ownership::{Ownership as OwnershipModel, OwnershipState},
        user::User as UserModel,
        user_permission::PermissionType,
    },
    error::Error,
};
use actix_web::{
    HttpRequest, HttpResponse, Responder,
    error::{ErrorBadRequest, ErrorInternalServerError},
    web,
};
use actix_web::{Result, error::ErrorUnauthorized};

/// Create a new ownership record
///
/// Accepts user ID, plot ID, ownership state, total sale value, and repayment details.
/// Creates a new ownership link between a user and a plot in the system.
/// Authorization: bearer token.
#[utoipa::path(
    post,
    path = "/api/ownership/create",
    request_body(content = OwnershipCreationRequest, description = "Ownership creation data"),
    responses(
        (status = 201, description = "Ownership created successfully", body = Ownership),
        (status = 400, description = "Missing or invalid input", body = String),
        (status = 401, description = "Unauthorized", body = String)
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Property Management"
)]
pub async fn create_ownership(
    services: web::Data<Arc<AppServices>>,
    http_request: HttpRequest,
    payload: web::Json<OwnershipCreationRequest>,
) -> Result<impl Responder> {
    let authorization_header = crate::api::extract_authorization_header(&http_request)?;

    let request = payload.into_inner();

    // Validate JWT token
    let jwt = services
        .property_management_service
        .jwt
        .decode_jwt_token(&authorization_header)?;

    // Check permission
    if !jwt
        .claims
        .permissions
        .contains(&PermissionType::ModifyOwnerships.to_string())
    {
        return Err(ErrorUnauthorized(
            "Insufficient permissions to Modify Ownerships",
        ));
    }

    // Check that a contract start date is provided for any ActivePayment contracts
    if request.state == OwnershipState::ActivePayment && request.contract_start_date.is_none() {
        return Err(ErrorBadRequest(
            "Any contract in ActivePayment requires a contract start date",
        ));
    }

    // Initialize DB connection
    let mut conn = super::get_database_connection(
        services
            .property_management_service
            .database_connection_pool
            .clone(),
    )?;

    // Only allow a single non Defaulted ownership
    if request.state != OwnershipState::Defaulted {
        let ownerships_for_plot =
            OwnershipModel::get_ownerships_filtered(&mut conn, None, Some(request.plot_id), None)?;
        if ownerships_for_plot
            .iter()
            .any(|ownership| ownership.state != OwnershipState::Defaulted)
        {
            return Err(ErrorBadRequest(
                "Attempting to create multiple active Ownership records",
            ));
        }
    }

    // Create ownership in DB
    let ownership = OwnershipModel::create_ownership(
        &mut conn,
        request.user_id,
        request.plot_id,
        request
            .state
            .clone()
            .try_into()
            .map_err(|_e| ErrorBadRequest(format!("Invalid state: {:?}", request.state)))?,
        request.total_sale_value,
        request.monthly_repayment_amount,
        request.contract_start_date,
    )?;

    // Fetch the user to get username
    let user = UserModel::get_user_by_user_id(&mut conn, request.user_id)?;

    Ok(HttpResponse::Ok().json(Ownership::from((ownership, user.user_name))))
}

/// Update an existing ownership record
///
/// Accepts the ownership ID along with optional updates to state, total sale value,
/// and monthly repayment amount. Requires a valid bearer token for authorization.
/// Returns the updated ownership record if successful.
///
/// Authorization: bearer token.
#[utoipa::path(
    put,
    path = "/api/ownership/update",
    request_body(content = OwnershipUpdateRequest, description = "Ownership update data"),
    responses(
        (status = 200, description = "Ownership updated successfully", body = Ownership),
        (status = 400, description = "Missing or invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Invalid OwnershipUpdateRequest payload received"),
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Property Management"
)]
pub async fn update_ownership(
    services: web::Data<Arc<AppServices>>,
    http_request: HttpRequest,
    payload: web::Json<OwnershipUpdateRequest>,
) -> Result<impl Responder> {
    let authorization_header = crate::api::extract_authorization_header(&http_request)?;

    let request = payload.into_inner();

    // Decode and validate JWT
    let jwt = services
        .property_management_service
        .jwt
        .decode_jwt_token(&authorization_header)?;
    if !jwt
        .claims
        .permissions
        .contains(&PermissionType::ModifyOwnerships.to_string())
    {
        return Err(ErrorUnauthorized(
            "Insufficient permissions to Modify Ownerships",
        ));
    }

    // Check that a contract start date is provided for any ActivePayment contracts
    if request.state.is_some()
        && request.state.clone().unwrap() == OwnershipState::ActivePayment
        && request.contract_start_date.is_none()
    {
        return Err(ErrorBadRequest(
            "Any contract in ActivePayment requires a contract start date",
        ));
    }

    // Initialize DB connection
    let mut conn = super::get_database_connection(
        services
            .property_management_service
            .database_connection_pool
            .clone(),
    )?;

    // Get user making update
    let user_making_update = UserModel::get_user_by_user_name(&mut conn, &jwt.claims.sub)?;

    // Update ownership
    let updated_ownership = OwnershipModel::update_ownership(
        &mut conn,
        request.ownership_id,
        request.state,
        request.total_sale_value,
        request.monthly_repayment_amount,
        request.contract_start_date,
        user_making_update.user_id,
    )?;

    Ok(HttpResponse::Ok().json(Ownership::from((
        updated_ownership,
        user_making_update.user_name,
    ))))
}

/// Get an existing ownership record by ID
///
/// Returns the ownership record for the specified ownership ID.
/// Requires a valid bearer token for authorization.
///
/// Authorization: bearer token.
#[utoipa::path(
    get,
    path = "/api/ownership/{ownership_id}",
    params(
        ("ownership_id" = i32, Path, description = "The ID of the ownership record to retrieve",
        example = 42)
    ),
    responses(
        (status = 200, description = "Ownership retrieved successfully", body = Ownership),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Ownership not found")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Property Management"
)]
pub async fn get_ownership(
    services: web::Data<Arc<AppServices>>,
    http_request: HttpRequest,
    path: web::Path<i32>,
) -> Result<impl Responder> {
    let authorization_header = crate::api::extract_authorization_header(&http_request)?;

    let ownership_id = path.into_inner();

    // Get DB connection
    let mut conn = match services
        .archive_management_service
        .clone()
        .database_connection_pool
        .clone()
        .get()
    {
        Ok(conn) => conn,
        Err(error) => {
            tracing::error!(
                "Internal server error. Failed to establish connection to database: {:?}",
                error
            );
            return Err(ErrorInternalServerError(
                "Failed to establish connection to database",
            ));
        }
    };

    // Decode and validate JWT
    let jwt_token = services
        .archive_management_service
        .clone()
        .jwt
        .decode_jwt_token(&authorization_header)?;

    // TODO don't early exit with information
    let ownership = OwnershipModel::get_ownership(&mut conn, ownership_id)?;
    let user_making_request = UserModel::get_user_by_user_name(&mut conn, &jwt_token.claims.sub)?;

    if !jwt_token
        .claims
        .permissions
        .contains(&PermissionType::ViewOwnerships.to_string())
        && ownership.user_id != user_making_request.user_id
    {
        return Err(ErrorUnauthorized(
            "Insufficient permissions to view ownership",
        ));
    }

    let ownership_user = UserModel::get_user_by_user_id(&mut conn, ownership.user_id)?;

    Ok(HttpResponse::Ok().json(Ownership::from((ownership, ownership_user.user_name))))
}

/// Get ownership records with optional filters
///
/// Returns a list of ownership records matching the provided filters.
/// Requires a valid bearer token for authorization.
///
/// Authorization: bearer token.
///
/// ### Example Query
/// `/api/ownerships?user_id=10&plot_id=5&state=ACTIVE`
#[utoipa::path(
    get,
    path = "/api/ownerships",
    params(
        OwnershipFilter
    ),
    responses(
        (status = 200, description = "Ownerships retrieved successfully", body = [Ownership]),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Property Management"
)]
pub async fn get_ownerships(
    services: web::Data<Arc<AppServices>>,
    http_request: HttpRequest,
    query: web::Query<OwnershipFilter>,
) -> Result<impl Responder, actix_web::Error> {
    let authorization_header = crate::api::extract_authorization_header(&http_request)?;
    let jwt_token = services
        .archive_management_service
        .jwt
        .decode_jwt_token(&authorization_header)?;

    let mut conn = match services
        .archive_management_service
        .clone()
        .database_connection_pool
        .clone()
        .get()
    {
        Ok(conn) => conn,
        Err(error) => {
            tracing::error!(
                "Internal server error. Failed to establish connection to database: {:?}",
                error
            );
            return Err(ErrorInternalServerError(
                "Failed to establish connection to database",
            ));
        }
    };

    let user_making_request =
        match UserModel::get_user_by_user_name(&mut conn, &jwt_token.claims.sub) {
            Ok(user) => user,
            Err(_) => return Err(ErrorUnauthorized("Invalid JWT user (subject) claim")),
        };

    if !jwt_token
        .claims
        .permissions
        .contains(&PermissionType::ViewOwnerships.to_string())
        && query
            .user_id
            .filter(|user_id| user_id == &user_making_request.user_id)
            .is_none()
    {
        return Err(ErrorUnauthorized(
            "Insufficient permissions to view ownership",
        ));
    }

    let ownerships = OwnershipModel::get_ownerships_filtered(
        &mut conn,
        query.user_id,
        query.plot_id,
        query.state.clone(),
    )?;

    let results: Vec<Ownership> = ownerships
        .into_iter()
        .map(|ownership| {
            let username = UserModel::get_user_by_user_id(&mut conn, ownership.user_id)?.user_name;
            Ok(Ownership::from((ownership, username)))
        })
        .collect::<Result<Vec<Ownership>, Error>>()?;

    Ok(HttpResponse::Ok().json(results))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{
        db::model::{ownership::OwnershipState, user_permission::PermissionType},
        jwt::{ClaimsBuilder, Jwt},
        secure_storage::SecureStorage,
    };

    use super::*;

    use actix_web::{App, http::StatusCode, test};
    use chrono::NaiveDate;
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
        crate::api::tests::create_test_app_service("test_ownership_handler")
    }

    #[rstest]
    #[actix_web::test]
    async fn test_create_ownership(
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
                .route("/api/ownership/create", web::post().to(create_ownership)),
        )
        .await;

        // Create database entries
        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let user = crate::db::model::tests::create_test_user(&mut conn);

        // Act: Send http request
        let json_payload = fs::read_to_string(
            "tests/resources/api/payload/property_management/request/create_ownership.json",
        )
        .expect("Failed to read JSON file");

        let json_payload = json_payload.replace("\"{{user_id}}\"", &user.user_id.to_string());
        let json_payload = json_payload.replace("\"{{plot_id}}\"", &plot.plot_id.to_string());

        let jwt_token = jwt.encode_jwt_token(
            ClaimsBuilder::new(&user.user_name)
                .permission(PermissionType::ModifyOwnerships.to_string())
                .build(),
        );

        let req = test::TestRequest::post()
            .uri("/api/ownership/create")
            .insert_header(("Authorization", format!("Bearer {jwt_token}")))
            .insert_header(("Content-Type", "application/json"))
            .set_payload(json_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert: Check HTTP status code and returned payload
        assert_eq!(resp.status(), StatusCode::OK);
        let response_json: Ownership = test::read_body_json(resp).await;
        assert_eq!(response_json.plot_id, plot.plot_id);
        assert_eq!(response_json.user_id, user.user_id);
        assert_eq!(response_json.user_name, user.user_name);
        assert_eq!(response_json.state, OwnershipState::PendingContract);
        assert_eq!(response_json.total_sale_value, 5000000);
        assert_eq!(response_json.monthly_repayment_amount, 50000);
        assert_eq!(
            response_json.contract_start_date,
            Some(NaiveDate::from_ymd_opt(2025, 08, 14).unwrap())
        );
        // Check DB
        assert!(OwnershipModel::get_ownership(&mut conn, response_json.ownership_id).is_ok());
    }

    #[rstest]
    #[actix_web::test]
    async fn test_update_ownership(
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
                .route("/api/ownership/update", web::put().to(update_ownership)),
        )
        .await;

        // Create database entries
        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let user = crate::db::model::tests::create_test_user(&mut conn);
        let ownership = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);

        // Act: Send http request
        let json_payload = fs::read_to_string(
            "tests/resources/api/payload/property_management/request/update_ownership.json",
        )
        .expect("Failed to read JSON file");

        let json_payload =
            json_payload.replace("\"{{ownership_id}}\"", &ownership.ownership_id.to_string());

        let jwt_token = jwt.encode_jwt_token(
            ClaimsBuilder::new(&user.user_name)
                .permission(PermissionType::ModifyOwnerships.to_string())
                .build(),
        );

        let req = test::TestRequest::put()
            .uri("/api/ownership/update")
            .insert_header(("Authorization", format!("Bearer {jwt_token}")))
            .insert_header(("Content-Type", "application/json"))
            .set_payload(json_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert: Check HTTP status code and returned payload
        assert_eq!(resp.status(), StatusCode::OK);
        let response_json: Ownership = test::read_body_json(resp).await;
        assert_eq!(response_json.plot_id, plot.plot_id);
        assert_eq!(response_json.user_id, user.user_id);
        assert_eq!(response_json.user_name, user.user_name);
        assert_eq!(response_json.state, OwnershipState::ActivePayment);
        assert_eq!(response_json.total_sale_value, 5000000);
        assert_eq!(response_json.monthly_repayment_amount, 50000);
        assert_eq!(
            response_json.contract_start_date,
            Some(NaiveDate::from_ymd_opt(2025, 08, 14).unwrap())
        );
        // Check DB
        let updated_ownership =
            OwnershipModel::get_ownership(&mut conn, response_json.ownership_id).unwrap();
        assert_eq!(updated_ownership.state, OwnershipState::ActivePayment);
        assert_eq!(updated_ownership.total_sale_value, 5000000);
        assert_eq!(updated_ownership.monthly_repayment_amount, 50000);
        assert_eq!(
            updated_ownership.contract_start_date,
            Some(NaiveDate::from_ymd_opt(2025, 08, 14).unwrap())
        );
    }

    #[rstest]
    #[actix_web::test]
    async fn test_get_ownership(
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
                .route(
                    "/api/ownership/{ownership_id}",
                    web::get().to(get_ownership),
                ),
        )
        .await;

        let user_making_request = crate::db::model::tests::create_test_user(&mut conn);

        // Create database entries
        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let user = crate::db::model::tests::create_test_user(&mut conn);
        let ownership = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);

        // Act: Send http request
        let jwt_token = jwt.encode_jwt_token(
            ClaimsBuilder::new(&user_making_request.user_name)
                .permission(PermissionType::ViewOwnerships.to_string())
                .build(),
        );

        let req = test::TestRequest::get()
            .uri(&format!("/api/ownership/{}", ownership.ownership_id))
            .insert_header(("Authorization", format!("Bearer {jwt_token}")))
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert: Check HTTP status code and returned payload
        assert_eq!(resp.status(), StatusCode::OK);
        let response_json: Ownership = test::read_body_json(resp).await;
        assert_eq!(response_json.plot_id, ownership.plot_id);
        assert_eq!(response_json.user_id, ownership.user_id);
        assert_eq!(response_json.user_name, user.user_name);
        assert_eq!(response_json.state, ownership.state);
        assert_eq!(response_json.total_sale_value, ownership.total_sale_value);
        assert_eq!(
            response_json.monthly_repayment_amount,
            ownership.monthly_repayment_amount
        );
    }

    #[rstest]
    #[actix_web::test]
    async fn test_get_ownership_unauthorized(
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
                .route(
                    "/api/ownership/{ownership_id}",
                    web::get().to(get_ownership),
                ),
        )
        .await;

        // Create database entries
        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let user = crate::db::model::tests::create_test_user(&mut conn);
        let user_other = crate::db::model::tests::create_test_user(&mut conn);
        let ownership = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);

        // Act: Send http request
        let jwt_token = jwt.encode_jwt_token(ClaimsBuilder::new(&user_other.user_name).build());

        let req = test::TestRequest::get()
            .uri(&format!("/api/ownership/{}", ownership.ownership_id))
            .insert_header(("Authorization", format!("Bearer {jwt_token}")))
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert: Check HTTP status code and returned payload
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[rstest]
    #[actix_web::test]
    async fn test_get_ownerships_filtered(
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
                .route("/api/ownerships", web::get().to(get_ownerships)),
        )
        .await;

        // --- Arrange test data ---
        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let user = crate::db::model::tests::create_test_user(&mut conn);
        let ownership = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);

        // --- Create JWT with permission ---
        let jwt_token = jwt.encode_jwt_token(
            ClaimsBuilder::new(&user.user_name)
                .permission(PermissionType::ViewOwnerships.to_string())
                .build(),
        );

        // --- Act: call GET /api/ownerships with filters ---
        let uri = format!(
            "/api/ownerships?user_id={}&plot_id={}&state={:?}",
            ownership.user_id,
            ownership.plot_id,
            OwnershipState::PendingContract
        );

        let req = test::TestRequest::get()
            .uri(&uri)
            .insert_header(("Authorization", format!("Bearer {jwt_token}")))
            .to_request();

        let resp = test::call_service(&app, req).await;

        // --- Assert ---
        assert_eq!(resp.status(), StatusCode::OK);
        let response_json: Vec<Ownership> = test::read_body_json(resp).await;
        assert!(response_json.contains(&Ownership::from((ownership, user.user_name.to_string()))));
    }

    #[rstest]
    #[actix_web::test]
    async fn test_get_ownerships_filtered_unauthorized(
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
                .route("/api/ownerships", web::get().to(get_ownerships)),
        )
        .await;

        // --- Arrange test data ---
        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let user = crate::db::model::tests::create_test_user(&mut conn);
        let user_other = crate::db::model::tests::create_test_user(&mut conn);
        let ownership = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);

        // --- Create JWT with permission ---
        let jwt_token = jwt.encode_jwt_token(ClaimsBuilder::new(&user_other.user_name).build());

        // --- Act: call GET /api/ownerships with filters ---
        let uri = format!(
            "/api/ownerships?user_id={}&plot_id={}&state={:?}",
            ownership.user_id,
            ownership.plot_id,
            OwnershipState::PendingContract
        );

        let req = test::TestRequest::get()
            .uri(&uri)
            .insert_header(("Authorization", format!("Bearer {jwt_token}")))
            .to_request();

        let resp = test::call_service(&app, req).await;

        // --- Assert ---
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
