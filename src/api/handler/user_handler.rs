use actix_web::{HttpResponse, Responder, web};

use crate::api::model::user::{
    AccessTokenResponse, CreateUserRequest, ListUsersRequest, LoginRequest, User,
};

/// Authenticate a user and return an access token.
///
/// Accepts a username and password, verifies credentials, and returns a JWT access token
/// for use in subsequent authenticated requests.
#[utoipa::path(
    post,
    path = "/user/login",
    request_body(content = LoginRequest, description = "User login credentials"),
    responses(
        (status = 200, description = "Login successful", body = AccessTokenResponse),
        (status = 400, description = "Missing or invalid input"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "User Management"
)]
pub async fn login_user(_payload: web::Json<LoginRequest>) -> impl Responder {
    HttpResponse::NotImplemented()
}

/// Register a new user in the system
///
/// Accepts user details, validates them, and creates a new user account if the username
/// is not already taken.
/// Authorization: bearer token.
#[utoipa::path(
    post,
    path = "/user/create",
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
pub async fn create_user(_payload: web::Json<CreateUserRequest>) -> impl Responder {
    HttpResponse::NotImplemented()
}

/// Return a list of users, filtered by optional parameters
///
/// Accepts optional filter parameters to query registered users by username, name regex,
/// cellphone, email, or identification number.
/// Authorization: bearer token.
#[utoipa::path(
    post,
    path = "/user/list",
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
pub async fn list_users(_payload: web::Json<ListUsersRequest>) -> impl Responder {
    HttpResponse::NotImplemented()
}
