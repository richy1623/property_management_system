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
    #[schema(example = "101")]
    pub user_id: i32,
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

mod converter {
    use super::*;
    use crate::service::user_management::user_management_proto::{
        ListUsersRequest as ProtoListUsersRequest, User as ProtoUser, UserCreationRequest,
        UserLoginRequest, UserLoginResponse,
    };

    impl From<LoginRequest> for UserLoginRequest {
        fn from(login_request: LoginRequest) -> Self {
            UserLoginRequest {
                user_name: login_request.username,
                password: login_request.password,
            }
        }
    }

    impl From<UserLoginResponse> for AccessTokenResponse {
        fn from(user_login_response: UserLoginResponse) -> Self {
            Self {
                access_token: user_login_response.jwt_token,
                user_id: user_login_response.user_id,
            }
        }
    }

    impl From<ProtoUser> for User {
        fn from(user: ProtoUser) -> Self {
            Self {
                user_id: user.user_id,
                username: user.username,
                name: user.name,
                cellphone_number: user.cellphone_number,
                email_address: user.email_address,
                identification_number: user.identification_number,
            }
        }
    }

    impl From<CreateUserRequest> for UserCreationRequest {
        fn from(create_user_request: CreateUserRequest) -> Self {
            Self {
                user_name: create_user_request.user_name,
                name: create_user_request.name,
                cellphone_number: create_user_request.cellphone_number,
                email_address: create_user_request.email_address,
                identification_number: create_user_request.identification_number,
                password: create_user_request.password,
            }
        }
    }

    impl From<ProtoListUsersRequest> for ListUsersRequest {
        fn from(list_user_request: ProtoListUsersRequest) -> Self {
            Self {
                username: list_user_request.username,
                name_regex: list_user_request.name_regex,
                cellphone_number: list_user_request.cellphone_number,
                email_address: list_user_request.email_address,
                identification_number: list_user_request.identification_number,
            }
        }
    }

    impl From<UserLoginRequest> for LoginRequest {
        fn from(login_request: UserLoginRequest) -> Self {
            Self {
                username: login_request.user_name,
                password: login_request.password,
            }
        }
    }

    impl From<AccessTokenResponse> for UserLoginResponse {
        fn from(access_token_response: AccessTokenResponse) -> Self {
            Self {
                jwt_token: access_token_response.access_token,
                user_id: access_token_response.user_id,
            }
        }
    }

    impl From<User> for ProtoUser {
        fn from(user: User) -> Self {
            Self {
                user_id: user.user_id,
                username: user.username,
                name: user.name,
                cellphone_number: user.cellphone_number,
                email_address: user.email_address,
                identification_number: user.identification_number,
            }
        }
    }
    impl From<UserCreationRequest> for CreateUserRequest {
        fn from(grpc_request: UserCreationRequest) -> Self {
            Self {
                user_name: grpc_request.user_name,
                name: grpc_request.name,
                cellphone_number: grpc_request.cellphone_number,
                email_address: grpc_request.email_address,
                identification_number: grpc_request.identification_number,
                password: grpc_request.password,
            }
        }
    }

    impl From<(ListUsersRequest, String)> for ProtoListUsersRequest {
        fn from((api_request, jwt_token): (ListUsersRequest, String)) -> Self {
            Self {
                username: api_request.username,
                name_regex: api_request.name_regex,
                cellphone_number: api_request.cellphone_number,
                email_address: api_request.email_address,
                identification_number: api_request.identification_number,
                jwt_token,
            }
        }
    }
}
