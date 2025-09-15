use std::sync::Arc;

use actix_web::Result;
use actix_web::error::ErrorUnauthorized;
use actix_web::{HttpRequest, HttpResponse, Responder, web};

use crate::api::model::user::UpdateUserRequest;
use crate::db::model::user::User as UserModel;
use crate::db::model::user_permission::{PermissionType, UserPermission};
use crate::{
    api::{
        AppServices,
        model::user::{
            AccessTokenResponse, CreateUserRequest, ListUsersRequest, LoginRequest, User,
        },
    },
    service::user_management::user_management_proto::user_management_server::UserManagement,
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
            // Match on the tonic::Status code and message to handle specific errors
            match status.code() {
                tonic::Code::NotFound => HttpResponse::NotFound().body("Incorrect username"),
                tonic::Code::Unauthenticated => {
                    HttpResponse::Unauthorized().body("Incorrect password")
                }
                _ => {
                    // Fallback for all other errors
                    crate::api::map_grpc_error_to_http_response(status)
                }
            }
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
    path = "/api/user/create",
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

    let request = payload.into_inner();

    let mut conn = super::get_database_connection(
        services
            .archive_management_service
            .database_connection_pool
            .clone(),
    )?;

    // Validate JWT
    let jwt_token = services
        .user_management_service
        .jwt
        .decode_jwt_token(&authorization_header)?;

    if !jwt_token
        .claims
        .permissions
        .contains(&PermissionType::ViewUsers.to_string())
    {
        return Err(ErrorUnauthorized("Insufficient permissions to list users"));
    }

    // Fetch filtered users
    let users: Vec<User> = UserModel::get_filtered_users(
        &mut conn,
        request.username.as_deref(),
        request.name_regex.as_deref(),
        request.cellphone_number.as_deref(),
        request.email_address.as_deref(),
        request.identification_number.as_deref(),
    )?
    .iter()
    .map(|user| User::from((user, vec![])))
    .collect();

    Ok(HttpResponse::Ok().json(users))
}

/// Get user details and permissions
///
/// Returns the user info along with their permissions.
/// Authorization: bearer token.
#[utoipa::path(
    get,
    path = "/api/user/{user_id}",
    responses(
        (status = 200, description = "User details retrieved successfully", body = User),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found")
    ),
    params(
        ("user_id" = i32, Path, description = "ID of the user to fetch details for")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "User Management"
)]
pub async fn get_user_details(
    services: web::Data<Arc<AppServices>>,
    http_request: HttpRequest,
    path: web::Path<i32>,
) -> Result<impl Responder> {
    let user_id = path.into_inner();
    let authorization_header = crate::api::extract_authorization_header(&http_request)?;

    let mut conn = super::get_database_connection(
        services
            .archive_management_service
            .database_connection_pool
            .clone(),
    )?;

    // Validate JWT
    let jwt_token = services
        .user_management_service
        .jwt
        .decode_jwt_token(&authorization_header)?;

    if !jwt_token
        .claims
        .permissions
        .contains(&PermissionType::ViewUsers.to_string())
    {
        return Err(ErrorUnauthorized("Insufficient permissions to View Users"));
    }

    let user = UserModel::get_user_by_user_id(&mut conn, user_id)?;

    let user_permissions = UserPermission::get_all_user_permissions(&mut conn, user_id)?;

    Ok(HttpResponse::Ok().json(User::from((&user, user_permissions))))
}

/// Update user details and permissions
///
/// Returns the user info along with their permissions.
/// Authorization: bearer token.
#[utoipa::path(
    put,
    path = "/api/user/{user_id}",
    responses(
        (status = 200, description = "User details updated successfully", body = User),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found")
    ),
    params(
        ("user_id" = i32, Path, description = "ID of the user to update details for")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "User Management"
)]
pub async fn update_user_details(
    services: web::Data<Arc<AppServices>>,
    http_request: HttpRequest,
    path: web::Path<i32>,
    payload: web::Json<UpdateUserRequest>,
) -> Result<impl Responder> {
    let user_id = path.into_inner();
    let authorization_header = crate::api::extract_authorization_header(&http_request)?;

    let update_user_request: UpdateUserRequest = payload.into_inner();

    let mut conn = super::get_database_connection(
        services
            .archive_management_service
            .database_connection_pool
            .clone(),
    )?;

    // Validate JWT
    let jwt_token = services
        .user_management_service
        .jwt
        .decode_jwt_token(&authorization_header)?;

    if !jwt_token
        .claims
        .permissions
        .contains(&PermissionType::ModifyUsers.to_string())
    {
        return Err(ErrorUnauthorized(
            "Insufficient permissions to Modify Users",
        ));
    }

    let updated_user = UserModel::update_user(
        &mut conn,
        update_user_request.user_id,
        update_user_request.user_name.as_deref(),
        update_user_request.name.as_deref(),
        update_user_request
            .cellphone_number
            .as_ref()
            .map(Option::as_deref),
        update_user_request
            .email_address
            .as_ref()
            .map(Option::as_deref),
        update_user_request
            .identification_number
            .as_ref()
            .map(Option::as_deref),
        update_user_request.password.as_deref(),
        update_user_request.is_employee,
    )?;

    let user_permissions = if let Some(permissions) = update_user_request.permissions {
        UserPermission::set_user_permission(&mut conn, user_id, &permissions)?
    } else {
        UserPermission::get_all_user_permissions(&mut conn, user_id)?
    };

    Ok(HttpResponse::Ok().json(User::from((&updated_user, user_permissions))))
}

// /// Update a user’s permissions
// ///
// /// Replaces the existing permissions for the given user with the new set.
// /// Authorization: bearer token.
// #[utoipa::path(
//     put,
//     path = "/api/user/{user_id}/permissions",
//     request_body(content = Vec<PermissionType>, description = "New permissions for the user"),
//     responses(
//         (status = 200, description = "User permissions updated successfully"),
//         (status = 400, description = "Invalid input"),
//         (status = 401, description = "Unauthorized"),
//         (status = 404, description = "User not found")
//     ),
//     params(
//         ("user_id" = i32, Path, description = "ID of the user to update permissions for")
//     ),
//     security(
//         ("bearerAuth" = [])
//     ),
//     tag = "User Management"
// )]
// pub async fn update_user_permissions(
//     services: web::Data<Arc<AppServices>>,
//     http_request: HttpRequest,
//     path: web::Path<i32>,
//     payload: web::Json<Vec<PermissionType>>,
// ) -> Result<impl Responder> {
//     let user_id = path.into_inner();
//     let new_permissions = &payload.into_inner();

//     let authorization_header = crate::api::extract_authorization_header(&http_request)?;

//     let mut conn = super::get_database_connection(
//         services
//             .archive_management_service
//             .database_connection_pool
//             .clone(),
//     )?;

//     let user = UserModel::get_user_by_user_id(&mut conn, user_id)?;
//     let _user_permissions = UserPermission::get_all_user_permissions(&mut conn, user_id)?;

//     // Validate JWT
//     let jwt_token = services
//         .user_management_service
//         .jwt
//         .decode_jwt_token(&authorization_header)?;

//     // TODO add more logic around who can do what
//     if !jwt_token
//         .claims
//         .permissions
//         .contains(&PermissionType::ModifyUsers.to_string())
//     {
//         return Err(ErrorUnauthorized(
//             "Insufficient permissions to Modify Users",
//         ));
//     }

//     let new_permissions = UserPermission::set_user_permission(&mut conn, user_id, new_permissions)?;

//     Ok(HttpResponse::Ok().json(User::from((user, new_permissions))))
// }

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{
        db::{
            self,
            model::{user::User as UserModel, user_permission::PermissionType},
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

        let test_user = UserModel::create_user(
            &mut conn,
            "john_doe_test_login",
            "name",
            None,
            None,
            None,
            "secret123",
            false,
        )
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
    async fn test_update_user_details_success(
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

        // Initialize Actix app with the update_user_details route
        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(app_services.clone()))
                .route("/api/user/{user_id}", web::put().to(update_user_details)),
        )
        .await;

        // Create calling user (with permission to update)
        let calling_user = db::model::tests::create_test_user(&mut conn);

        // Create test user to update
        let test_user = UserModel::create_user(
            &mut conn,
            "john_doe",
            "John Doe",
            None,
            Some("john@example.com"),
            None,
            "secret123",
            false,
        )
        .unwrap();

        // Build JWT for the calling user
        let jwt_token = jwt.encode_jwt_token(
            ClaimsBuilder::new(&calling_user.user_name)
                .permission(PermissionType::ModifyUsers.to_string())
                .build(),
        );

        // Load update payload from test resources
        let json_payload = fs::read_to_string(
            "tests/resources/api/payload/user_management/request/update_user.json",
        )
        .expect("Failed to read JSON file");
        let json_payload = json_payload.replace("\"{{user_id}}\"", &test_user.user_id.to_string());

        // Act: send PUT request
        let req = test::TestRequest::put()
            .uri(&format!("/api/user/{}", test_user.user_id))
            .insert_header(("Authorization", format!("Bearer {jwt_token}")))
            .insert_header(("Content-Type", "application/json"))
            .set_payload(json_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert
        assert_eq!(resp.status(), 200);
        let response_json: User = test::read_body_json(resp).await;
        assert_eq!(response_json.user_id, test_user.user_id);
        assert_eq!(response_json.username, "john_doe_update");
        assert_eq!(response_json.name, "John Doe Update");
        assert_eq!(
            response_json.email_address,
            Some("john_update@example.com".to_string())
        );
        assert!(response_json.is_employee);

        assert_eq!(
            UserPermission::get_all_user_permissions(&mut conn, test_user.user_id).unwrap(),
            vec![
                PermissionType::ViewPayments,
                PermissionType::ModifyOwnerships
            ]
        );
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
            false,
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
            false,
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
            .insert_header(("Authorization", format!("Bearer {jwt_token}")))
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

    #[rstest]
    #[actix_web::test]
    async fn test_get_user_details_success(
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

        // Initialize Actix app with the route
        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(app_services.clone()))
                .route("/api/user/{user_id}", web::get().to(get_user_details)),
        )
        .await;

        // Create a calling user in DB
        let user_making_request = db::model::tests::create_test_user(&mut conn);
        // Create a test user in DB
        let test_user = db::model::tests::create_test_user(&mut conn);

        // Add some permissions to the user
        let permissions = vec![
            PermissionType::ViewPayments,
            PermissionType::ModifyOwnerships,
        ];
        UserPermission::set_user_permission(&mut conn, test_user.user_id, &permissions).unwrap();

        // Send GET request
        let jwt_token = jwt.encode_jwt_token(
            ClaimsBuilder::new(&user_making_request.user_name)
                .permission(PermissionType::ViewUsers.to_string())
                .build(),
        );

        let req = test::TestRequest::get()
            .uri(&format!("/api/user/{}", test_user.user_id))
            .insert_header(("Authorization", format!("Bearer {jwt_token}")))
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert
        assert_eq!(resp.status(), 200);
        let response_json: User = test::read_body_json(resp).await;
        assert_eq!(response_json.user_id, test_user.user_id);
        assert_eq!(response_json.username, test_user.user_name);
        assert_eq!(response_json.email_address, test_user.email_address);
        assert_eq!(response_json.cellphone_number, test_user.cellphone_number);
        assert_eq!(
            response_json.identification_number,
            test_user.identification_number
        );
        assert_eq!(response_json.name, test_user.name);
        assert_eq!(response_json.permissions, permissions);
    }

    // #[rstest]
    // #[actix_web::test]
    // async fn test_update_user_permissions_success(
    //     test_services: &(
    //         AppServices,
    //         Jwt,
    //         SecureStorage,
    //         Pool<ConnectionManager<PgConnection>>,
    //     ),
    // ) {
    //     let (services, jwt, _secure_storage, pool) = test_services.clone();
    //     let mut conn = pool.get().unwrap();
    //     let app_services = Arc::new(services);

    //     // Initialize Actix app with the route
    //     let app = test::init_service(
    //         App::new()
    //             .app_data(actix_web::web::Data::new(app_services.clone()))
    //             .route(
    //                 "/api/user/{user_id}/permissions",
    //                 web::put().to(update_user_permissions),
    //             ),
    //     )
    //     .await;

    //     // Create a calling user in DB
    //     let user_making_request = db::model::tests::create_test_user(&mut conn);

    //     // Create a test user in DB
    //     let test_user = db::model::tests::create_test_user(&mut conn);

    //     // Initial permissions (optional, could be empty)
    //     let initial_permissions = vec![PermissionType::ViewUsers];
    //     UserPermission::set_user_permission(&mut conn, test_user.user_id, &initial_permissions)
    //         .unwrap();

    //     // New permissions to update
    //     let new_permissions = vec![
    //         PermissionType::ViewPayments,
    //         PermissionType::ModifyOwnerships,
    //     ];

    //     // Encode JWT for the calling user
    //     let jwt_token = jwt.encode_jwt_token(
    //         ClaimsBuilder::new(&user_making_request.user_name)
    //             .permission(PermissionType::ModifyUsers.to_string()) // assume this allows updates
    //             .build(),
    //     );

    //     // Send PUT request to update permissions
    //     let req = test::TestRequest::put()
    //         .uri(&format!("/api/user/{}/permissions", test_user.user_id))
    //         .insert_header(("Authorization", format!("Bearer {jwt_token}")))
    //         .set_json(&new_permissions)
    //         .to_request();

    //     let resp = test::call_service(&app, req).await;

    //     // Assert HTTP status
    //     assert_eq!(resp.status(), 200);

    //     // Verify in DB that permissions were updated
    //     let db_permissions =
    //         UserPermission::get_all_user_permissions(&mut conn, test_user.user_id).unwrap();
    //     assert_eq!(db_permissions, new_permissions);
    // }
}
