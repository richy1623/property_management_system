use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use diesel_migrations::MigrationHarness;
use tonic::{Request, Response, Status};

use crate::model::{user::User as UserModel, user_permission::PermissionType};
use crate::{
    MIGRATIONS,
    jwt::{ClaimsBuilder, Jwt},
};

use super::user_management_proto::{
    ListUsersRequest, ListUsersResponse, User, UserCreationRequest, UserLoginRequest,
    UserLoginResponse, user_management_server::UserManagement,
};

pub struct UserManagementService {
    database_connection_pool: Pool<ConnectionManager<PgConnection>>,
    jwt: Jwt,
}

impl UserManagementService {
    pub fn new(database_connection_url: &str, jwt_secret_key: &str) -> Self {
        let database_connection_pool = crate::get_connection_pool(database_connection_url);
        database_connection_pool
            .clone()
            .get()
            .expect("Failed getting db pool")
            .run_pending_migrations(MIGRATIONS)
            .expect("Failed running migrations");

        UserManagementService {
            database_connection_pool: database_connection_pool,
            jwt: Jwt::new(jwt_secret_key),
        }
    }
}

#[tonic::async_trait]
impl UserManagement for UserManagementService {
    async fn create_user(
        &self,
        request: Request<UserCreationRequest>,
    ) -> Result<Response<User>, Status> {
        let mut conn = match self.database_connection_pool.clone().get() {
            Ok(conn) => conn,
            Err(e) => return Err(Status::internal(e.to_string())),
        };

        let request = request.get_ref();
        let user = UserModel::create_user(
            &mut conn,
            &request.user_name,
            &request.name,
            request.cellphone_number.as_deref(),
            request.email_address.as_deref(),
            request.identification_number.as_deref(),
            &request.password,
        )?;

        Ok(Response::new((&user).into()))
    }

    async fn login_user(
        &self,
        request: Request<UserLoginRequest>,
    ) -> Result<Response<UserLoginResponse>, Status> {
        let mut conn = match self.database_connection_pool.clone().get() {
            Ok(conn) => conn,
            Err(e) => return Err(Status::internal(e.to_string())),
        };

        let request: &UserLoginRequest = request.get_ref();

        let user = UserModel::validate_login(&mut conn, &request.user_name, &request.password)?;

        Ok(Response::new(UserLoginResponse {
            jwt_token: __self
                .jwt
                .encode_jwt_token(ClaimsBuilder::new(&user.user_name).build()),
            user_id: user.user_id,
        }))
    }

    async fn list_users(
        &self,
        request: tonic::Request<ListUsersRequest>,
    ) -> Result<tonic::Response<ListUsersResponse>, tonic::Status> {
        let mut conn = self
            .database_connection_pool
            .clone()
            .get()
            .map_err(|e| Status::internal(e.to_string()))?;

        let request = request.into_inner();

        // Validate JWT
        let jwt_token = self.jwt.decode_jwt_token(&request.jwt_token)?;
        if !jwt_token
            .claims
            .permissions
            .contains(&PermissionType::ViewUsers.to_string())
        {
            return Err(Status::permission_denied(
                "Insufficient permissions to list users",
            ));
        }

        // Fetch filtered users
        let users = UserModel::get_filtered_users(
            &mut conn,
            request.username.as_deref(),
            request.name_regex.as_deref(),
            request.cellphone_number.as_deref(),
            request.email_address.as_deref(),
            request.identification_number.as_deref(),
        )?;

        let proto_users = users.into_iter().map(|user| (&user).into()).collect();

        Ok(Response::new(ListUsersResponse { users: proto_users }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{jwt::Jwt, model::user_permission::PermissionType};
    use rstest::{fixture, rstest};
    use tonic::Request;

    #[fixture]
    #[once]
    pub fn pool() -> Pool<ConnectionManager<PgConnection>> {
        crate::tests::get_test_db_connection_pool("user_management_service")
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_user_success(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.get().unwrap();
        let service = UserManagementService {
            database_connection_pool: pool.clone(),
            jwt: Jwt::new("a-string-secret-at-least-256-bits-long"),
        };

        let request = UserCreationRequest {
            user_name: "alice".to_string(),
            name: "Alice Smith".to_string(),
            cellphone_number: Some("1234567890".to_string()),
            email_address: Some("alice@example.com".to_string()),
            identification_number: Some("ID123456".to_string()),
            password: "password".to_string(),
        };

        let response: User = service
            .create_user(Request::new(request.clone()))
            .await
            .unwrap()
            .into_inner();

        let user_inserted: User =
            (&UserModel::get_user_by_user_name(&mut conn, &request.user_name).unwrap()).into();

        assert_eq!(user_inserted, response);
    }

    #[rstest]
    #[tokio::test]
    async fn test_login_user_success(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.get().unwrap();

        // Create user directly in DB to login later
        let user = UserModel::create_user(
            &mut conn,
            "bob",
            "Bob Marley",
            Some("0987654321"),
            Some("bob@example.com"),
            Some("ID654321"),
            "my_password",
        )
        .unwrap();

        let service = UserManagementService {
            database_connection_pool: pool.clone(),
            jwt: Jwt::new("a-string-secret-at-least-256-bits-long"),
        };

        let request = UserLoginRequest {
            user_name: user.user_name.clone(),
            password: "my_password".to_string(),
        };

        let response = service.login_user(Request::new(request)).await;
        assert!(response.is_ok());

        let login_response = response.unwrap().into_inner();
        assert!(!login_response.jwt_token.is_empty());
    }

    #[rstest]
    #[tokio::test]
    async fn test_login_user_invalid_password(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.get().unwrap();

        let user = UserModel::create_user(
            &mut conn,
            "charlie",
            "Charlie Brown",
            None,
            None,
            None,
            "right_password",
        )
        .unwrap();

        let service = UserManagementService {
            database_connection_pool: pool.clone(),
            jwt: Jwt::new("a-string-secret-at-least-256-bits-long"),
        };

        let request = UserLoginRequest {
            user_name: user.user_name.clone(),
            password: "wrong_password".to_string(),
        };

        let result = service.login_user(Request::new(request)).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
    }

    #[rstest]
    #[tokio::test]
    async fn test_list_users_with_permission(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.get().unwrap();

        let service = UserManagementService {
            database_connection_pool: pool.clone(),
            jwt: Jwt::new("a-string-secret-at-least-256-bits-long"),
        };

        // Setup test users
        let user_1: User = (&UserModel::create_user(
            &mut conn,
            "alpha",
            "Alice Eve",
            Some("1234"),
            Some("alice@mail.com"),
            Some("ID123"),
            "password",
        )
        .unwrap())
            .into();
        let user_2: User = (&UserModel::create_user(
            &mut conn,
            "beta",
            "Bob Eve",
            Some("5678"),
            Some("bob@mail.com"),
            Some("ID456"),
            "password",
        )
        .unwrap())
            .into();

        let request = ListUsersRequest {
            jwt_token: service.jwt.encode_jwt_token(
                ClaimsBuilder::new("user")
                    .permission(PermissionType::ViewUsers.to_string())
                    .build(),
            ),
            username: None,
            name_regex: Some("Eve".to_string()),
            cellphone_number: None,
            email_address: None,
            identification_number: None,
        };

        let response = service
            .list_users(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(response.users, vec![user_1, user_2]);
    }

    #[rstest]
    #[tokio::test]
    async fn test_list_users_without_permission(pool: &Pool<ConnectionManager<PgConnection>>) {
        let service = UserManagementService {
            database_connection_pool: pool.clone(),
            jwt: Jwt::new("a-string-secret-at-least-256-bits-long"),
        };

        // Request with username filter
        let request = ListUsersRequest {
            jwt_token: service
                .jwt
                .encode_jwt_token(ClaimsBuilder::new("user").build()),
            username: None,
            name_regex: None,
            cellphone_number: None,
            email_address: None,
            identification_number: None,
        };

        let response = service.list_users(Request::new(request)).await;

        assert!(response.is_err());
        assert_eq!(response.unwrap_err().code(), tonic::Code::PermissionDenied);
    }
}
