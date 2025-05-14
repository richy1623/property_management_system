use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use diesel_migrations::MigrationHarness;
use tonic::{Request, Response, Status};

use crate::{
    MIGRATIONS,
    jwt::Jwt,
    model::{
        ownership::Ownership,
        payment::{Payment, PaymentState},
        user::User,
        user_permission::PermissionType,
    },
};

use super::payment_management_proto::{
    PaymentCreationRequest, PaymentCreationResponse, payment_management_server::PaymentManagement,
};

pub struct PaymentManagementService {
    database_connection_pool: Pool<ConnectionManager<PgConnection>>,
    jwt: Jwt,
}

impl PaymentManagementService {
    pub fn new(database_connection_url: &str) -> Self {
        let database_connection_pool = crate::get_connection_pool(database_connection_url);
        database_connection_pool
            .clone()
            .get()
            .expect("Failed getting db pool")
            .run_pending_migrations(MIGRATIONS)
            .expect("Failed running migrations");

        PaymentManagementService {
            database_connection_pool: database_connection_pool,
            // TODO use a secret
            jwt: Jwt::new("a-string-secret-at-least-256-bits-long"),
        }
    }
}

#[tonic::async_trait]
impl PaymentManagement for PaymentManagementService {
    async fn create_payment(
        &self,
        request: Request<PaymentCreationRequest>,
    ) -> Result<Response<PaymentCreationResponse>, Status> {
        let mut conn = match self.database_connection_pool.clone().get() {
            Ok(conn) => conn,
            Err(e) => return Err(Status::internal(e.to_string())),
        };
        let payment_creation_request = request.get_ref();

        // Validate perms
        let jwt_token = match self
            .jwt
            .decode_jwt_token(&payment_creation_request.jwt_token)
        {
            Ok(token) => token,
            Err(_) => return Err(Status::unauthenticated("Invalid JWT token")),
        };

        if jwt_token.claims.sub != payment_creation_request.username
            && !jwt_token
                .claims
                .permissions
                .contains(&PermissionType::CreatePayments.to_string())
        {
            return Err(Status::permission_denied(
                "Insufficient permissions to make payment",
            ));
        }

        let user = User::get_user_by_user_name(&mut conn, &payment_creation_request.username)?;

        let ownership = Ownership::get_ownership(&mut conn, payment_creation_request.ownership_id)?;

        let payment = Payment::create_payment(
            &mut conn,
            ownership.ownership_id,
            payment_creation_request.date.unwrap().into(),
            payment_creation_request.amount,
            PaymentState::PendingApproval,
            "comment",
            &payment_creation_request.reference,
            user.user_id,
            "payment_creation_request.proof_of_payment_path",
        )?;

        Ok(Response::new(PaymentCreationResponse {
            message: format!("Created payment: {:?}", payment),
        }))
    }
}
