use std::sync::Arc;

use crate::{
    api::{
        AppServices,
        model::estate::{Estate, EstateCreationRequest, EstateList},
    },
    service::property_management::property_management_proto::{
        Empty, property_management_server::PropertyManagement,
    },
};
use actix_web::Result;
use actix_web::{HttpRequest, HttpResponse, Responder, web};

use crate::service::property_management::property_management_proto::EstateCreationRequest as ProtoEstateCreationRequest;

/// Create a new estate
///
/// Accepts estate details such as name, location, district, and landmark,
/// and creates a new estate record in the system.
/// Authorization: bearer token.
#[utoipa::path(
    post,
    path = "/api/estate/create",
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
pub async fn create_estate(
    services: web::Data<Arc<AppServices>>,
    http_request: HttpRequest,
    payload: web::Json<EstateCreationRequest>,
) -> Result<impl Responder> {
    let authorization_header = crate::api::extract_authorization_header(&http_request)?;

    let request = tonic::Request::new(ProtoEstateCreationRequest::from((
        payload.into_inner(),
        authorization_header,
    )));

    match services
        .property_management_service
        .create_estate(request)
        .await
    {
        Ok(response) => {
            // Convert the successful gRPC response into the HTTP response
            Ok(HttpResponse::Ok().json(Estate::from(response.into_inner())))
        }
        Err(status) => {
            // Map the gRPC error status to an appropriate HTTP response with logging
            Ok(crate::api::map_grpc_error_to_http_response(status))
        }
    }
}

/// Retrieve all estates in the system
///
/// Returns a list of all estates stored in the system.
/// Authorization: bearer token.
#[utoipa::path(
    get,
    path = "/api/estate/list",
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
pub async fn list_estates(services: web::Data<Arc<AppServices>>) -> Result<impl Responder> {
    match services
        .property_management_service
        .get_all_estates(tonic::Request::new(Empty {}))
        .await
    {
        Ok(response) => {
            // Convert the successful gRPC response into the HTTP response
            Ok(HttpResponse::Ok().json(EstateList::from(response.into_inner())))
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
        db::model::{estate::Estate as EstateModel, user_permission::PermissionType},
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
        crate::api::tests::create_test_app_service("test_estate_handler")
    }

    #[rstest]
    #[actix_web::test]
    async fn test_create_estate(
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
                .route("/api/estate/create", web::post().to(create_estate)),
        )
        .await;

        // Create database entries
        let user = crate::db::model::tests::create_test_user(&mut conn);

        // Act: Send http request
        let json_payload = fs::read_to_string(
            "tests/resources/api/payload/property_management/request/create_estate.json",
        )
        .expect("Failed to read JSON file");

        let jwt_token = jwt.encode_jwt_token(
            ClaimsBuilder::new(&user.user_name)
                .permission(PermissionType::ModifyProperties.to_string())
                .build(),
        );

        let req = test::TestRequest::post()
            .uri("/api/estate/create")
            .insert_header(("Authorization", jwt_token))
            .insert_header(("Content-Type", "application/json"))
            .set_payload(json_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert: Check HTTP status code and returned payload
        assert_eq!(resp.status(), StatusCode::OK);
        let response_json: Estate = test::read_body_json(resp).await;
        // Check DB
        assert!(EstateModel::get_estate(&mut conn, response_json.estate_id).is_ok());
    }

    #[rstest]
    #[actix_web::test]
    async fn test_list_estates(
        test_services: &(
            AppServices,
            Jwt,
            SecureStorage,
            Pool<ConnectionManager<PgConnection>>,
        ),
    ) {
        let (services, _jwt, _secure_storage, pool) = test_services.clone();
        let mut conn = pool.get().unwrap();

        let app_services = Arc::new(services);
        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(app_services.clone()))
                .route("/api/estate/list", web::get().to(list_estates)),
        )
        .await;

        // Create database entries
        let estate_1 = crate::db::model::tests::create_test_estate(&mut conn);
        let estate_2 = crate::db::model::tests::create_test_estate(&mut conn);

        // Act: Send http request
        let req = test::TestRequest::get()
            .uri("/api/estate/list")
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert: Check HTTP status code and returned payload
        assert_eq!(resp.status(), StatusCode::OK);
        let response_json: EstateList = test::read_body_json(resp).await;
        let estate_ids = response_json
            .estates
            .iter()
            .map(|estate| estate.estate_id)
            .collect::<Vec<i32>>();
        assert!(estate_ids.contains(&estate_1.estate_id));
        assert!(estate_ids.contains(&estate_2.estate_id));
    }
}
