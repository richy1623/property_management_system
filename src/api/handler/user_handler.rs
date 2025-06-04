use std::sync::Arc;

use actix_web::{HttpResponse, Responder, web};

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
    payload: web::Json<ListUsersRequest>,
) -> impl Responder {
    let request = tonic::Request::new(ProtoListUsersRequest::from((
        payload.into_inner(),
        "".to_string(),
    )));

    match services.user_management_service.list_users(request).await {
        Ok(response) => {
            // Convert the successful gRPC response into the HTTP response
            HttpResponse::Created().json(
                response
                    .into_inner()
                    .users
                    .into_iter()
                    .map(|user| User::from(user))
                    .collect::<Vec<User>>(),
            )
        }
        Err(status) => {
            // Map the gRPC error status to an appropriate HTTP response with logging
            crate::api::map_grpc_error_to_http_response(status)
        }
    }
}
