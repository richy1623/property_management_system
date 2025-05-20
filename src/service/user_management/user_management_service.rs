use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use diesel_migrations::MigrationHarness;
use tonic::{Request, Response, Status};

use crate::{
    MIGRATIONS,
    jwt::{ClaimsBuilder, Jwt},
    model::user::User,
};

use super::user_management_proto::{
    UserCreationRequest, UserCreationResponse, UserLoginRequest, UserLoginResponse,
    user_management_server::UserManagement,
};

pub struct UserManagementService {
    database_connection_pool: Pool<ConnectionManager<PgConnection>>,
    jwt: Jwt,
}

impl UserManagementService {
    pub fn new(database_connection_url: &str) -> Self {
        let database_connection_pool = crate::get_connection_pool(database_connection_url);
        database_connection_pool
            .clone()
            .get()
            .expect("Failed getting db pool")
            .run_pending_migrations(MIGRATIONS)
            .expect("Failed running migrations");

        UserManagementService {
            database_connection_pool: database_connection_pool,
            jwt: Jwt::new("a-string-secret-at-least-256-bits-long"),
        }
    }
}

#[tonic::async_trait]
impl UserManagement for UserManagementService {
    async fn create_user(
        &self,
        request: Request<UserCreationRequest>,
    ) -> Result<Response<UserCreationResponse>, Status> {
        println!("{:?}", request);
        let mut conn = match self.database_connection_pool.clone().get() {
            Ok(conn) => conn,
            Err(e) => return Err(Status::internal(e.to_string())),
        };

        let request = request.get_ref();
        let user = User::create_user(
            &mut conn,
            &request.user_name,
            &request.name,
            request.cellphone_number.as_deref(),
            request.email_address.as_deref(),
            request.identification_number.as_deref(),
            &request.password,
        )?;

        let reply = UserCreationResponse {
            message: format!("User '{}' created successfully!", user.user_name),
        };

        Ok(Response::new(reply))
    }

    async fn login_user(
        &self,
        request: Request<UserLoginRequest>,
    ) -> Result<Response<UserLoginResponse>, Status> {
        println!("{:?}", request);
        let mut conn = match self.database_connection_pool.clone().get() {
            Ok(conn) => conn,
            Err(e) => return Err(Status::internal(e.to_string())),
        };

        let request: &UserLoginRequest = request.get_ref();

        let user = User::validate_login(&mut conn, &request.user_name, &request.password)?;

        Ok(Response::new(UserLoginResponse {
            jwt_token: self
                .jwt
                .encode_jwt_token(ClaimsBuilder::new(&user.user_name).build()),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{jwt::Jwt, model::user::User};
    use rstest::rstest;
    use tonic::Request;

    #[rstest]
    #[tokio::test]
    async fn test_create_user_success() {
        let pool = crate::tests::get_test_db_connection_pool(
            "user_management_service_test_create_user_success",
        );
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

        let response = service.create_user(Request::new(request)).await;
        assert!(response.is_ok());

        let msg = response.unwrap().into_inner().message;
        assert!(msg.contains("alice"));
    }

    #[rstest]
    #[tokio::test]
    async fn test_login_user_success() {
        let pool = crate::tests::get_test_db_connection_pool(
            "user_management_service_test_login_user_success",
        );
        let mut conn = pool.get().unwrap();

        // Create user directly in DB to login later
        let user = User::create_user(
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
    async fn test_login_user_invalid_password() {
        let pool = crate::tests::get_test_db_connection_pool(
            "user_management_service_test_login_user_invalid_password",
        );
        let mut conn = pool.get().unwrap();

        let user = User::create_user(
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
}
