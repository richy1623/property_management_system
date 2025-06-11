use std::sync::Arc;

use actix_web::Result;
use actix_web::{HttpRequest, HttpResponse, Responder, web};

use crate::{
    api::{
        AppServices,
        model::user::{
            AccessTokenResponse, CreateUserRequest, ListUsersRequest, LoginRequest, User,
        },
    },
    service::user_management::user_management_proto::{
        ListUsersRequest as ProtoListUsersRequest, user_management_server::UserManagement,
    },
};

/// Authenticate a user and return an access token.
///
/// Accepts a username and password, verifies credentials, and returns a JWT access token
/// for use in subsequent authenticated requests.
#[utoipa::path(
    post,
    path = "/api/user/login",
    request_body(content = LoginRequest, description = "User login credentials"),
    responses(
        (status = 200, description = "Login successful", body = AccessTokenResponse),
        (status = 400, description = "Missing or invalid input"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "User Management"
)]
pub async fn login_user(
    services: web::Data<Arc<AppServices>>,
    payload: web::Json<LoginRequest>,
) -> impl Responder {
    let request = tonic::Request::new(payload.into_inner().into());

    match services.user_management_service.login_user(request).await {
        Ok(response) => {
            // Convert the successful gRPC response into the HTTP response
            HttpResponse::Ok().json(AccessTokenResponse::from(response.into_inner()))
        }
        Err(status) => {
            // Map the gRPC error status to an appropriate HTTP response with logging
            crate::api::map_grpc_error_to_http_response(status)
        }
    }
}

/// Register a new user in the system
///
/// Accepts user details, validates them, and creates a new user account if the username
/// is not already taken.
/// Authorization: bearer token.
#[utoipa::path(
    post,
    path = "/api/login/create",
    request_body(content = CreateUserRequest, description = "User creation details"),
    responses(
        (status = 201, description = "User created successfully", body = User),
        (status = 400, description = "Missing or invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "User already exists")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "User Management"
)]
pub async fn create_user(
    services: web::Data<Arc<AppServices>>,
    payload: web::Json<CreateUserRequest>,
) -> impl Responder {
    let request = tonic::Request::new(payload.into_inner().into());

    match services.user_management_service.create_user(request).await {
        Ok(response) => {
            // Convert the successful gRPC response into the HTTP response
            HttpResponse::Created().json(User::from(response.into_inner()))
        }
        Err(status) => {
            // Map the gRPC error status to an appropriate HTTP response with logging
            crate::api::map_grpc_error_to_http_response(status)
        }
    }
}

/// Return a list of users, filtered by optional parameters
///
/// Accepts optional filter parameters to query registered users by username, name regex,
/// cellphone, email, or identification number.
/// Authorization: bearer token.
#[utoipa::path(
    post,
    path = "/api/user/list",
    request_body(content = ListUsersRequest, description = "Filters for querying users"),
    responses(
        (status = 200, description = "List of users retrieved successfully", body = [User]),
        (status = 400, description = "Missing or invalid input"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "User Management"
)]
pub async fn list_users(
    services: web::Data<Arc<AppServices>>,
    http_request: HttpRequest,
    payload: web::Json<ListUsersRequest>,
) -> Result<impl Responder> {
    let authorization_header = crate::api::extract_authorization_header(&http_request)?;

    let request = tonic::Request::new(ProtoListUsersRequest::from((
        payload.into_inner(),
        authorization_header,
    )));

    match services.user_management_service.list_users(request).await {
        Ok(response) => {
            // Convert the successful gRPC response into the HTTP response
            Ok(HttpResponse::Ok().json(
                response
                    .into_inner()
                    .users
                    .into_iter()
                    .map(|user| User::from(user))
                    .collect::<Vec<User>>(),
            ))
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
        db::model::{user::User as UserModel, user_permission::PermissionType},
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
        crate::api::tests::create_test_app_service("test_user_handler")
    }

    #[rstest]
    #[actix_web::test]
    async fn test_login_user_success(
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
                .route("/api/user/login", web::post().to(login_user)),
        )
        .await;

        let test_user =
            UserModel::create_user(&mut conn, "john_doe", "name", None, None, None, "secret123")
                .unwrap();

        // Act: Send a login request with known test user credentials
        let json_payload =
            fs::read_to_string("tests/resources/api/payload/user_management/request/login.json")
                .expect("Failed to read JSON file");

        let req = test::TestRequest::post()
            .uri("/api/user/login")
            .insert_header(("Content-Type", "application/json"))
            .set_payload(json_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert: Check HTTP status code and returned payload
        assert_eq!(resp.status(), 200);
        let response_json: AccessTokenResponse = test::read_body_json(resp).await;
        assert_eq!(response_json.user_id, test_user.user_id);
        assert!(jwt.decode_jwt_token(&response_json.access_token).is_ok());
    }

    #[rstest]
    #[actix_web::test]
    async fn test_create_user(
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
                .route("/api/user/create", web::post().to(create_user)),
        )
        .await;

        // Act: Send a login request with known test user credentials
        let json_payload = fs::read_to_string(
            "tests/resources/api/payload/user_management/request/create_user.json",
        )
        .expect("Failed to read JSON file");

        let req = test::TestRequest::post()
            .uri("/api/user/create")
            .insert_header(("Content-Type", "application/json"))
            .set_payload(json_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert: Check HTTP status code and returned payload
        assert_eq!(resp.status(), StatusCode::CREATED);
        let response_json: User = test::read_body_json(resp).await;

        let inserted_user =
            UserModel::get_user_by_user_id(&mut conn, response_json.user_id).unwrap();
        assert_eq!(inserted_user.user_name, response_json.username);
        assert_eq!(inserted_user.name, response_json.name);
        assert_eq!(
            inserted_user.cellphone_number,
            response_json.cellphone_number
        );
        assert_eq!(
            inserted_user.identification_number,
            response_json.identification_number
        );
        assert_eq!(inserted_user.email_address, response_json.email_address);
    }

    #[rstest]
    #[actix_web::test]
    async fn test_list_users_success(
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
                .route("/api/user/list", web::post().to(list_users)),
        )
        .await;

        // Setup test users
        let user_1: UserModel = UserModel::create_user(
            &mut conn,
            "alpha",
            "Alice Eve",
            Some("1234"),
            Some("alice@mail.com"),
            Some("ID123"),
            "password",
        )
        .unwrap();
        let user_2: UserModel = UserModel::create_user(
            &mut conn,
            "beta",
            "Bob Eve",
            Some("5678"),
            Some("bob@mail.com"),
            Some("ID456"),
            "password",
        )
        .unwrap();

        // Act: Send a login request with known test user credentials
        let json_payload = fs::read_to_string(
            "tests/resources/api/payload/user_management/request/list_users.json",
        )
        .expect("Failed to read JSON file");

        let jwt_token = jwt.encode_jwt_token(
            ClaimsBuilder::new(&user_1.user_name)
                .permission(PermissionType::ViewUsers.to_string())
                .build(),
        );

        let req = test::TestRequest::post()
            .uri("/api/user/list")
            .insert_header(("Content-Type", "application/json"))
            .insert_header(("Authorization", jwt_token))
            .set_payload(json_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert: Check HTTP status code and returned payload
        assert_eq!(resp.status(), StatusCode::OK);
        let response_json: Vec<User> = test::read_body_json(resp).await;
        assert_eq!(response_json.len(), 2);
        assert_eq!(
            response_json
                .iter()
                .map(|user| user.user_id)
                .collect::<Vec<i32>>(),
            vec![user_1.user_id, user_2.user_id]
        )
    }
}
