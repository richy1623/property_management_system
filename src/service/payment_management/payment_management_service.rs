use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use diesel_migrations::MigrationHarness;
use tonic::{Request, Response, Status};

use crate::model::{ownership::Ownership, user::User as UserModel};
use crate::{
    MIGRATIONS,
    jwt::Jwt,
    model::{
        ownership::Ownership as OwnershipModel,
        payment::{Payment as PaymentModel, PaymentState},
        user::User,
        user_permission::PermissionType,
    },
};

use super::payment_management_proto::{
    GetPaymentsForOwnershipRequest, GetPaymentsInStateRequest, Payment, PaymentCreationRequest,
    PaymentList, PaymentStateUpdateRequest, payment_management_server::PaymentManagement,
};

pub struct PaymentManagementService {
    database_connection_pool: Pool<ConnectionManager<PgConnection>>,
    jwt: Jwt,
}

impl PaymentManagementService {
    pub fn new(database_connection_url: &str, jwt_secret_key: &str) -> Self {
        let database_connection_pool = crate::get_connection_pool(database_connection_url);
        database_connection_pool
            .clone()
            .get()
            .expect("Failed getting db pool")
            .run_pending_migrations(MIGRATIONS)
            .expect("Failed running migrations");

        PaymentManagementService {
            database_connection_pool: database_connection_pool,
            jwt: Jwt::new(jwt_secret_key),
        }
    }
}

#[tonic::async_trait]
impl PaymentManagement for PaymentManagementService {
    async fn create_payment(
        &self,
        request: Request<PaymentCreationRequest>,
    ) -> Result<Response<Payment>, Status> {
        let mut conn = match self.database_connection_pool.clone().get() {
            Ok(conn) => conn,
            Err(e) => return Err(Status::internal(e.to_string())),
        };
        let payment_creation_request = request.get_ref();

        // Validate perms
        let jwt_token = self
            .jwt
            .decode_jwt_token(&payment_creation_request.jwt_token)?;

        if jwt_token.claims.sub != payment_creation_request.username
            && !jwt_token
                .claims
                .permissions
                .contains(&PermissionType::ModifyPayments.to_string())
        {
            return Err(Status::permission_denied(
                "Insufficient permissions to make payment",
            ));
        }

        let user = User::get_user_by_user_name(&mut conn, &payment_creation_request.username)?;

        let ownership = Ownership::get_ownership(&mut conn, payment_creation_request.ownership_id)?;

        let payment = PaymentModel::create_payment(
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

        Ok(Response::new((&payment).into()))
    }

    async fn update_payment_state(
        &self,
        request: Request<PaymentStateUpdateRequest>,
    ) -> Result<Response<Payment>, Status> {
        let mut conn = match self.database_connection_pool.clone().get() {
            Ok(conn) => conn,
            Err(e) => return Err(Status::internal(e.to_string())),
        };
        let payment_state_update_request = request.get_ref();

        // Validate perms
        let jwt_token = self
            .jwt
            .decode_jwt_token(&payment_state_update_request.jwt_token)?;

        if !jwt_token
            .claims
            .permissions
            .contains(&PermissionType::ModifyPayments.to_string())
        {
            return Err(Status::permission_denied(
                "Insufficient permissions to make payment",
            ));
        }

        let user_making_update =
            UserModel::get_user_by_user_name(&mut conn, &jwt_token.claims.sub)?;

        let payment = PaymentModel::update_payment_state(
            &mut conn,
            payment_state_update_request.payment_id,
            PaymentState::try_from(payment_state_update_request.state).unwrap(),
            user_making_update.user_id,
            &payment_state_update_request.comment,
        )?;

        Ok(Response::new((&payment).into()))
    }

    async fn get_payments_for_ownership(
        &self,
        request: Request<GetPaymentsForOwnershipRequest>,
    ) -> Result<Response<PaymentList>, Status> {
        let get_payments_request = request.into_inner();

        let mut conn = match self.database_connection_pool.clone().get() {
            Ok(conn) => conn,
            Err(e) => return Err(Status::internal(e.to_string())),
        };

        // Validate JWT
        let jwt_token = self.jwt.decode_jwt_token(&get_payments_request.jwt_token)?;

        let ownership =
            OwnershipModel::get_ownership(&mut conn, get_payments_request.ownership_id)?;
        let user = UserModel::get_user_by_user_id(&mut conn, ownership.user_id)?;

        if jwt_token.claims.sub != user.user_name
            && !jwt_token
                .claims
                .permissions
                .contains(&PermissionType::ViewPayments.to_string())
        {
            return Err(Status::permission_denied(
                "Insufficient permissions to view payments",
            ));
        }

        let payments = PaymentModel::get_all_payments_for_ownership_id(
            &mut conn,
            get_payments_request.ownership_id,
        )?;

        let payments = payments
            .into_iter()
            .map(|payment| (&payment).into())
            .collect();

        Ok(Response::new(PaymentList { payments: payments }))
    }

    async fn get_payments_in_state(
        &self,
        request: Request<GetPaymentsInStateRequest>,
    ) -> Result<Response<PaymentList>, Status> {
        let req = request.into_inner();

        let mut conn = match self.database_connection_pool.clone().get() {
            Ok(conn) => conn,
            Err(e) => return Err(Status::internal(e.to_string())),
        };

        // Validate JWT
        let jwt_token = self.jwt.decode_jwt_token(&req.jwt_token)?;

        if !jwt_token
            .claims
            .permissions
            .contains(&PermissionType::ViewPayments.to_string())
        {
            return Err(Status::permission_denied(
                "Insufficient permissions to view payments",
            ));
        }

        let payments = PaymentModel::get_all_payments_in_state(
            &mut conn,
            PaymentState::try_from(req.state)
                .map_err(|e| Status::invalid_argument(e.to_string()))?,
            req.offset as i64,
            req.limit as i64,
        )?;

        let payments = payments
            .into_iter()
            .map(|payment| (&payment).into())
            .collect();

        Ok(Response::new(PaymentList { payments: payments }))
    }
}

#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};

    use super::*;
    use crate::{
        jwt::{ClaimsBuilder, Jwt},
        service::payment_management::payment_management_proto::Date,
    };

    #[fixture]
    #[once]
    pub fn pool() -> Pool<ConnectionManager<PgConnection>> {
        crate::tests::get_test_db_connection_pool("payment_management_service")
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_payment_success(pool: &Pool<ConnectionManager<PgConnection>>) {
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
                .permission(PermissionType::ModifyPayments.to_string())
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

        let payment = service
            .create_payment(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        let payment_inserted: Payment =
            PaymentModel::get_all_payments_for_ownership_id(&mut conn, ownership.ownership_id)
                .unwrap()
                .get(0)
                .unwrap()
                .into();

        assert_eq!(payment, payment_inserted);
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_payment_permission_denied(pool: &Pool<ConnectionManager<PgConnection>>) {
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

    #[rstest]
    #[tokio::test]
    async fn test_update_payment_state_with_permission(
        pool: &Pool<ConnectionManager<PgConnection>>,
    ) {
        let mut conn = pool.get().unwrap();

        let service = PaymentManagementService {
            database_connection_pool: pool.clone(),
            jwt: Jwt::new("a-string-secret-at-least-256-bits-long"),
        };

        // Setup test data
        let user = crate::model::tests::create_test_user(&mut conn);
        let estate = crate::model::tests::create_test_estate(&mut conn);
        let plot = crate::model::tests::create_test_plot(&mut conn, &estate);
        let ownership = crate::model::tests::create_test_ownership(&mut conn, &plot, &user);
        let payment = crate::model::tests::create_test_payment(&mut conn, &ownership, &user);

        // Encode token with permission
        let jwt_token = service.jwt.encode_jwt_token(
            ClaimsBuilder::new(&user.user_name)
                .permission(PermissionType::ModifyPayments.to_string())
                .build(),
        );

        // Call service
        let request = PaymentStateUpdateRequest {
            jwt_token,
            payment_id: payment.payment_id,
            state: PaymentState::Approved.into(),
            comment: "Approved by admin".into(),
        };

        let response = service
            .update_payment_state(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        // Assert gRPC response
        assert_eq!(response.payment_id, payment.payment_id);
        assert_eq!(response.state, Into::<i32>::into(PaymentState::Approved));
        assert_eq!(response.comment, "Approved by admin");
        assert_eq!(response.last_changed_by_user, user.user_id);

        // --- DB verification ---
        let updated_payment = PaymentModel::get_payment(&mut conn, payment.payment_id).unwrap();
        assert_eq!(updated_payment.state, PaymentState::Approved);
        assert_eq!(updated_payment.comment, "Approved by admin".to_string());
        assert_eq!(updated_payment.last_changed_by_user, user.user_id);
    }

    #[rstest]
    #[tokio::test]
    async fn test_update_payment_state_without_permission(
        pool: &Pool<ConnectionManager<PgConnection>>,
    ) {
        let mut conn = pool.get().unwrap();

        let service = PaymentManagementService {
            database_connection_pool: pool.clone(),
            jwt: Jwt::new("a-string-secret-at-least-256-bits-long"),
        };

        let user = crate::model::tests::create_test_user(&mut conn);
        let estate = crate::model::tests::create_test_estate(&mut conn);
        let plot = crate::model::tests::create_test_plot(&mut conn, &estate);
        let ownership = crate::model::tests::create_test_ownership(&mut conn, &plot, &user);
        let payment = crate::model::tests::create_test_payment(&mut conn, &ownership, &user);

        let jwt_token = service
            .jwt
            .encode_jwt_token(ClaimsBuilder::new(&user.user_name).build());

        let request = PaymentStateUpdateRequest {
            jwt_token,
            payment_id: payment.payment_id,
            state: PaymentState::Rejected.into(),
            comment: "No reason".into(),
        };

        let response = service.update_payment_state(Request::new(request)).await;

        assert!(response.is_err());
        assert_eq!(response.unwrap_err().code(), tonic::Code::PermissionDenied);
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_payments_for_ownership(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.get().unwrap();

        let service = PaymentManagementService {
            database_connection_pool: pool.clone(),
            jwt: Jwt::new("a-string-secret-at-least-256-bits-long"),
        };

        let user = crate::model::tests::create_test_user(&mut conn);
        let estate = crate::model::tests::create_test_estate(&mut conn);
        let plot = crate::model::tests::create_test_plot(&mut conn, &estate);
        let ownership = crate::model::tests::create_test_ownership(&mut conn, &plot, &user);
        let payment_1 = crate::model::tests::create_test_payment(&mut conn, &ownership, &user);
        let payment_2 = crate::model::tests::create_test_payment(&mut conn, &ownership, &user);

        let expected_payments = vec![(&payment_1).into(), (&payment_2).into()];

        let jwt_token = service.jwt.encode_jwt_token(
            ClaimsBuilder::new(&user.user_name)
                .permission(PermissionType::ViewPayments.to_string())
                .build(),
        );

        let request = GetPaymentsForOwnershipRequest {
            jwt_token,
            ownership_id: ownership.ownership_id,
        };

        let response = service
            .get_payments_for_ownership(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(response.payments, expected_payments);
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_payments_in_state(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.get().unwrap();

        let service = PaymentManagementService {
            database_connection_pool: pool.clone(),
            jwt: Jwt::new("a-string-secret-at-least-256-bits-long"),
        };

        let user = crate::model::tests::create_test_user(&mut conn);
        let estate = crate::model::tests::create_test_estate(&mut conn);
        let plot = crate::model::tests::create_test_plot(&mut conn, &estate);
        let ownership = crate::model::tests::create_test_ownership(&mut conn, &plot, &user);
        let payment_1 = crate::model::tests::create_test_payment(&mut conn, &ownership, &user);
        let payment_2 = crate::model::tests::create_test_payment(&mut conn, &ownership, &user);

        let jwt_token = service.jwt.encode_jwt_token(
            ClaimsBuilder::new(&user.user_name)
                .permission(PermissionType::ViewPayments.to_string())
                .build(),
        );

        let request = GetPaymentsInStateRequest {
            jwt_token,
            state: PaymentState::PendingApproval as i32,
            offset: 0,
            limit: 100,
        };

        let response = service
            .get_payments_in_state(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert!(response.payments.contains(&Payment::from(&payment_1)));
        assert!(response.payments.contains(&Payment::from(&payment_2)));
        assert!(
            response
                .payments
                .iter()
                .all(|payment| payment.state == PaymentState::PendingApproval as i32)
        )
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_payments_for_ownership_permission_denied(
        pool: &Pool<ConnectionManager<PgConnection>>,
    ) {
        let mut conn = pool.get().unwrap();

        let service = PaymentManagementService {
            database_connection_pool: pool.clone(),
            jwt: Jwt::new("a-string-secret-at-least-256-bits-long"),
        };

        let user_valid = crate::model::tests::create_test_user(&mut conn);
        let user_invalid = crate::model::tests::create_test_user(&mut conn);
        let estate = crate::model::tests::create_test_estate(&mut conn);
        let plot = crate::model::tests::create_test_plot(&mut conn, &estate);
        let ownership = crate::model::tests::create_test_ownership(&mut conn, &plot, &user_valid);

        let jwt_token = service.jwt.encode_jwt_token(
            ClaimsBuilder::new(&user_invalid.user_name).build(), // No permissions
        );

        let request = GetPaymentsForOwnershipRequest {
            jwt_token,
            ownership_id: ownership.ownership_id,
        };

        let result = service
            .get_payments_for_ownership(Request::new(request))
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::PermissionDenied);
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_payments_in_state_permission_denied(
        pool: &Pool<ConnectionManager<PgConnection>>,
    ) {
        let mut conn = pool.get().unwrap();

        let service = PaymentManagementService {
            database_connection_pool: pool.clone(),
            jwt: Jwt::new("a-string-secret-at-least-256-bits-long"),
        };

        let user = crate::model::tests::create_test_user(&mut conn);

        let jwt_token = service.jwt.encode_jwt_token(
            ClaimsBuilder::new(&user.user_name).build(), // No permissions
        );

        let request = GetPaymentsInStateRequest {
            jwt_token,
            state: PaymentState::PendingApproval as i32,
            offset: 0,
            limit: 10,
        };

        let result = service.get_payments_in_state(Request::new(request)).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::PermissionDenied);
    }
}
