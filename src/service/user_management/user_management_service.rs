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
