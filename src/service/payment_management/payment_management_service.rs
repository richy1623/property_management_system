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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        jwt::{ClaimsBuilder, Jwt},
        service::payment_management::payment_management_proto::Date,
    };
    use rstest::rstest;

    #[rstest]
    #[tokio::test]
    async fn test_create_payment_success() {
        let pool = crate::tests::get_test_db_connection_pool(
            "payment_management_service_test_create_payment_success",
        );
        let jwt = Jwt::new("a-string-secret-at-least-256-bits-long");
        let service = PaymentManagementService {
            database_connection_pool: pool.clone(),
            jwt: jwt.clone(),
        };

        let mut conn = pool.get().unwrap();

        let user = crate::model::tests::create_test_user(&mut conn);
        let estate = crate::model::tests::create_test_estate(&mut conn);
        let plot = &crate::model::tests::create_test_plot(&mut conn, &estate);
        let ownership = crate::model::tests::create_test_ownership(&mut conn, &plot, &user);

        let token = jwt.encode_jwt_token(
            ClaimsBuilder::new(&user.user_name)
                .permission(PermissionType::CreatePayments.to_string())
                .build(),
        );

        let request = PaymentCreationRequest {
            jwt_token: token,
            username: user.user_name.clone(),
            ownership_id: ownership.ownership_id,
            date: Some(Date {
                year: 2024,
                month: 5,
                day: 18,
            }),
            amount: 10000,
            reference: "REF123".to_string(),
            proof_of_payment_path: "/path/to/proof.pdf".to_string(),
        };

        let result = service.create_payment(Request::new(request)).await;

        assert!(result.is_ok());
        assert!(
            result
                .as_ref()
                .unwrap()
                .get_ref()
                .message
                .contains("Created payment")
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_payment_permission_denied() {
        let pool = crate::tests::get_test_db_connection_pool(
            "payment_management_service_test_create_payment_permission_denied",
        );
        let jwt = Jwt::new("a-string-secret-at-least-256-bits-long");
        let service = PaymentManagementService {
            database_connection_pool: pool.clone(),
            jwt: jwt.clone(),
        };

        let mut conn = pool.get().unwrap();

        let user = crate::model::tests::create_test_user(&mut conn);
        let estate = crate::model::tests::create_test_estate(&mut conn);
        let plot = &crate::model::tests::create_test_plot(&mut conn, &estate);
        let ownership = crate::model::tests::create_test_ownership(&mut conn, &plot, &user);

        // No permissions included in token
        let token = jwt.encode_jwt_token(ClaimsBuilder::new("another_user").build());

        let request = PaymentCreationRequest {
            jwt_token: token,
            username: user.user_name.clone(),
            ownership_id: ownership.ownership_id,
            date: Some(Date {
                year: 2024,
                month: 5,
                day: 18,
            }),
            amount: 10000,
            reference: "REF123".to_string(),
            proof_of_payment_path: "/path/to/proof.pdf".to_string(),
        };

        let result = service.create_payment(Request::new(request)).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::PermissionDenied);
    }
}
