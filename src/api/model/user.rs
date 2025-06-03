use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Login credentials used to authenticate a user.
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    /// The username of the user.
    #[schema(example = "john_doe")]
    pub username: String,

    /// The user's password.
    #[schema(example = "secret123", format = "password")]
    pub password: String,
}

/// Represents the response containing the JWT access token issued after a successful login.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AccessTokenResponse {
    /// The JWT access token string to be used for authorization in subsequent requests.
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXCVI9...")]
    pub access_token: String,
}

/// Payload to create a new user account.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    /// Unique username for the user.
    #[schema(example = "john_doe")]
    pub user_name: String,

    /// Full name of the user.
    #[schema(example = "John Doe")]
    pub name: String,

    /// Optional cellphone number.
    #[schema(example = "+1234567890")]
    pub cellphone_number: Option<String>,

    /// Optional email address.
    #[schema(example = "john@example.com", format = "email")]
    pub email_address: Option<String>,

    /// Optional national ID or identification number.
    #[schema(example = "ID123456789")]
    pub identification_number: Option<String>,

    /// Password for the account.
    #[schema(example = "secret123", format = "password")]
    pub password: String,
}

/// A fully populated user as stored in the database.
#[derive(Debug, Serialize, ToSchema)]
pub struct User {
    /// Unique user ID.
    #[schema(example = 101)]
    pub user_id: i32,

    /// The username.
    #[schema(example = "john_doe")]
    pub username: String,

    /// Full name.
    #[schema(example = "John Doe")]
    pub name: String,

    /// Optional cellphone number.
    #[schema(example = "+1234567890")]
    pub cellphone_number: Option<String>,

    /// Optional email address.
    #[schema(example = "john@example.com", format = "email")]
    pub email_address: Option<String>,

    /// Optional ID number.
    #[schema(example = "ID123456789")]
    pub identification_number: Option<String>,
}

/// Parameters to filter users in list query.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ListUsersRequest {
    /// Filter users by exact username
    #[schema(example = "john_doe")]
    pub username: Option<String>,

    /// Filter users by name regex pattern
    #[schema(example = "Jo*")]
    pub name_regex: Option<String>,

    /// Filter users by cellphone number
    #[schema(example = "+1234567890")]
    pub cellphone_number: Option<String>,

    /// Filter users by email address
    #[schema(format = "email", example = "john@example.com")]
    pub email_address: Option<String>,

    /// Filter users by identification number
    #[schema(example = "ID123456789")]
    pub identification_number: Option<String>,
}
