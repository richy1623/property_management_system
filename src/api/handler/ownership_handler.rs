use std::sync::Arc;

use crate::{
    api::{
        AppServices,
        model::ownership::{Ownership, OwnershipCreationRequest, OwnershipUpdateRequest},
    },
    service::property_management::property_management_proto::property_management_server::PropertyManagement,
};
use actix_web::Result;
use actix_web::{HttpRequest, HttpResponse, Responder, web};

use crate::service::property_management::property_management_proto::{
    OwnershipCreationRequest as ProtoOwnershipCreationRequest,
    OwnershipUpdateRequest as ProtoOwnershipUpdateRequest,
};

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
        (status = 400, description = "Missing or invalid input"),
        (status = 401, description = "Unauthorized")
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

    let request = tonic::Request::new(ProtoOwnershipCreationRequest::from((
        payload.into_inner(),
        authorization_header,
    )));

    match services
        .property_management_service
        .create_ownership(request)
        .await
    {
        Ok(response) => {
            // Convert the successful gRPC response into the HTTP response
            Ok(HttpResponse::Ok().json(Ownership::from(response.into_inner())))
        }
        Err(status) => {
            // Map the gRPC error status to an appropriate HTTP response with logging
            Ok(crate::api::map_grpc_error_to_http_response(status))
        }
    }
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
        (status = 401, description = "Unauthorized")
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

    let request = tonic::Request::new(ProtoOwnershipUpdateRequest::from((
        payload.into_inner(),
        authorization_header,
    )));

    match services
        .property_management_service
        .update_ownership(request)
        .await
    {
        Ok(response) => {
            // Convert the successful gRPC response into the HTTP response
            Ok(HttpResponse::Ok().json(Ownership::from(response.into_inner())))
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
        db::model::{
            ownership::{Ownership as OwnershipModel, OwnershipState},
            user_permission::PermissionType,
        },
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
            .insert_header(("Authorization", jwt_token))
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
                .route("/api/ownership/update", web::post().to(update_ownership)),
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

        let req = test::TestRequest::post()
            .uri("/api/ownership/update")
            .insert_header(("Authorization", jwt_token))
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
        // Check DB
        let updated_ownership =
            OwnershipModel::get_ownership(&mut conn, response_json.ownership_id).unwrap();
        assert_eq!(updated_ownership.state, OwnershipState::ActivePayment);
        assert_eq!(updated_ownership.total_sale_value, 5000000);
        assert_eq!(updated_ownership.monthly_repayment_amount, 50000);
    }
}
