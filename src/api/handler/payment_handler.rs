use std::sync::Arc;

use crate::{
    api::{
        AppServices,
        model::payment::{
            GetOwnershipPaymentsRequest, Payment, PaymentCreationRequest, PaymentList,
            PaymentsInStateQuery, UpdatePaymentStateRequest,
        },
    },
    db::model::{
        artifact::Artifact as ArtifactModel,
        ownership::Ownership as OwnershipModel,
        payment::{Payment as PaymentModel, PaymentState},
        user::User as UserModel,
        user_permission::PermissionType,
    },
    service::payment_management::payment_management_proto::{
        PaymentStateUpdateRequest as ProtoUpdatePaymentStateRequest,
        payment_management_server::PaymentManagement,
    },
};
use actix_web::{HttpRequest, HttpResponse, Responder, error::ErrorUnauthorized, web};
use actix_web::{Result, error::ErrorInternalServerError};
use chrono::Utc;
use uuid::Uuid;

/// Retrieve payments for a given ownership
///
/// Accepts a list of ownership IDs and returns all payments associated with it.
/// Authorization: bearer token.
#[utoipa::path(
    post,
    path = "/api/payment/ownership",
    tag = "Payment Management",
    request_body(content = GetOwnershipPaymentsRequest, description = "Ownership IDs to filter payments by"),
    security(
        ("bearerAuth" = [])
    ),
    responses(
        (status = 200, description = "List of payments for ownership", body = PaymentList),
        (status = 400, description = "Missing or invalid input"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_payments_for_ownerships(
    services: web::Data<Arc<AppServices>>,
    http_request: HttpRequest,
    payload: web::Json<GetOwnershipPaymentsRequest>,
) -> Result<impl Responder> {
    let authorization_header = crate::api::extract_authorization_header(&http_request)?;

    let get_payments_request = payload.into_inner();

    let mut conn = super::get_database_connection(
        services
            .archive_management_service
            .database_connection_pool
            .clone(),
    )?;

    // Validate JWT
    let jwt_token = services
        .payment_management_service
        .jwt
        .decode_jwt_token(&authorization_header)?;

    let user = UserModel::get_user_by_user_name(&mut conn, &jwt_token.claims.sub)?;

    if !jwt_token
        .claims
        .permissions
        .contains(&PermissionType::ViewPayments.to_string())
        && !OwnershipModel::check_user_owns_ownerships(
            &mut conn,
            &get_payments_request.ownership_ids,
            user.user_id,
        )?
    {
        return Err(ErrorUnauthorized(
            "Insufficient permissions to view payments",
        ));
    }

    let payments = PaymentModel::get_all_payments_for_ownerships(
        &mut conn,
        &get_payments_request.ownership_ids,
    )?;

    Ok(HttpResponse::Ok().json(PaymentList {
        payments: payments
            .into_iter()
            .map(|payment| {
                Payment::from((
                    &payment,
                    payment
                        .proof_of_payment_artifact
                        .map(|artifact_id| ArtifactModel::get_artifact(&mut conn, artifact_id).ok())
                        .unwrap_or(None)
                        .as_ref(),
                ))
            })
            .collect(),
    }))
}

/// Retrieve payments filtered by state
///
/// Accepts a payment state and returns matching payments, with optional pagination using offset and limit.
/// Authorization: bearer token.
#[utoipa::path(
    get,
    path = "/api/payment/state",
    tag = "Payment Management",
    params(
        ("state" = PaymentState, Query, description = "State of payments to filter by"),
        ("offset" = Option<i32>, Query, description = "Pagination offset"),
        ("limit" = Option<i32>, Query, description = "Pagination limit")
    ),
    security(
        ("bearerAuth" = [])
    ),
    responses(
        (status = 200, description = "List of payments filtered by state", body = PaymentList),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_payments_in_state(
    services: web::Data<Arc<AppServices>>,
    http_request: HttpRequest,
    query: web::Query<PaymentsInStateQuery>,
) -> Result<impl Responder> {
    let authorization_header = crate::api::extract_authorization_header(&http_request)?;

    let get_payments_request = query.into_inner();

    let mut conn = super::get_database_connection(
        services
            .archive_management_service
            .database_connection_pool
            .clone(),
    )?;

    // Validate JWT
    let jwt_token = services
        .payment_management_service
        .jwt
        .decode_jwt_token(&authorization_header)?;

    if !jwt_token
        .claims
        .permissions
        .contains(&PermissionType::ViewPayments.to_string())
    {
        return Err(ErrorUnauthorized(
            "Insufficient permissions to view payments",
        ));
    }

    let payments = PaymentModel::get_all_payments_in_state(
        &mut conn,
        get_payments_request.state,
        get_payments_request.offset.unwrap_or(0) as i64,
        get_payments_request.limit.unwrap_or(10) as i64,
    )?;

    Ok(HttpResponse::Ok().json(PaymentList {
        payments: payments
            .into_iter()
            .map(|payment| {
                Payment::from((
                    &payment,
                    payment
                        .proof_of_payment_artifact
                        .map(|artifact_id| ArtifactModel::get_artifact(&mut conn, artifact_id).ok())
                        .unwrap_or(None)
                        .as_ref(),
                ))
            })
            .collect(),
    }))
}

/// Update the state and comment of a payment
///
/// Accepts the payment ID, new state, and an optional comment,
/// and updates the corresponding payment record in the system.
/// Authorization: bearer token.
#[utoipa::path(
    post,
    path = "/api/payment/update",
    tag = "Payment Management",
    request_body(content = UpdatePaymentStateRequest, description = "Details of the payment update"),
    security(
        ("bearerAuth" = [])
    ),
    responses(
        (status = 200, description = "Updated payment", body = Payment),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn update_payment_state(
    services: web::Data<Arc<AppServices>>,
    http_request: HttpRequest,
    payload: web::Json<UpdatePaymentStateRequest>,
) -> Result<impl Responder> {
    let authorization_header = crate::api::extract_authorization_header(&http_request)?;

    let request = tonic::Request::new(ProtoUpdatePaymentStateRequest::from((
        payload.into_inner(),
        authorization_header,
    )));

    match services
        .payment_management_service
        .update_payment_state(request)
        .await
    {
        Ok(response) => {
            // Convert the successful gRPC response into the HTTP response
            Ok(HttpResponse::Ok().json(Payment::from(response.into_inner())))
        }
        Err(status) => {
            // Map the gRPC error status to an appropriate HTTP response with logging
            Ok(crate::api::map_grpc_error_to_http_response(status))
        }
    }
}

/// Create a new payment record
///
/// Creates a new payment for a specific ownership. The authenticated user must either own the
/// ownership or have the `ModifyPayments` permission.
/// Authorization: bearer token.
#[utoipa::path(
    post,
    path = "/api/payment/",
    tag = "Payment Management",
    request_body(content = PaymentCreationRequest, description = "Details of the payment update"),
    security(
        ("bearerAuth" = [])
    ),
    responses(
        (status = 200, description = "Updated payment", body = Payment),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn create_payment(
    services: web::Data<Arc<AppServices>>,
    http_request: HttpRequest,
    payload: web::Json<PaymentCreationRequest>,
) -> Result<impl Responder> {
    let authorization_header = crate::api::extract_authorization_header(&http_request)?;

    let payment_creation_request = payload.into_inner();

    let jwt_token = services
        .archive_management_service
        .jwt
        .decode_jwt_token(&authorization_header)?;

    let mut conn = match services
        .archive_management_service
        .clone()
        .database_connection_pool
        .clone()
        .get()
    {
        Ok(conn) => conn,
        Err(error) => {
            tracing::error!(
                "Internal server error. Failed to establish connection to database: {:?}",
                error
            );
            return Err(ErrorInternalServerError(
                "Failed to establish connection to database",
            ));
        }
    };

    // TODO don't early exit
    let user_making_request = UserModel::get_user_by_user_name(&mut conn, &jwt_token.claims.sub)?;
    let ownership =
        OwnershipModel::get_ownership(&mut conn, payment_creation_request.ownership_id)?;

    if ownership.user_id != user_making_request.user_id
        && !jwt_token
            .claims
            .permissions
            .contains(&PermissionType::ModifyPayments.to_string())
    {
        return Err(ErrorUnauthorized(
            "Insufficient permissions to create payment",
        ));
    }

    let artifact =
        if let Some(proof_of_payment_file) = payment_creation_request.proof_of_payment_file {
            // Store the artifact
            let file_name = Uuid::new_v4().to_string();
            let stored_file_ref = services
                .archive_management_service
                .secure_storage
                .store_file(
                    &file_name,
                    &proof_of_payment_file.file_content,
                    Utc::now().date_naive(),
                )
                .await?;

            Some(ArtifactModel::create_artifact(
                &mut conn,
                &proof_of_payment_file.file_name,
                stored_file_ref.to_str().unwrap(),
                user_making_request.user_id,
                true,
            )?)
        } else {
            None
        };

    let payment = PaymentModel::create_user_payment(
        &mut conn,
        ownership.ownership_id,
        payment_creation_request.date,
        payment_creation_request.amount,
        &payment_creation_request.reference,
        user_making_request.user_id,
        artifact.as_ref().map(|artifact| artifact.artifact_id),
    )?;

    Ok(HttpResponse::Ok().json(Payment::from((&payment, artifact.as_ref()))))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{
        db::model::{payment::Payment as PaymentModel, user_permission::PermissionType},
        jwt::{ClaimsBuilder, Jwt},
        secure_storage::SecureStorage,
    };

    use super::*;

    use actix_web::{App, http::StatusCode, test};
    use chrono::NaiveDate;
    use diesel::{
        PgConnection,
        r2d2::{ConnectionManager, Pool},
    };
    use rstest::{fixture, rstest};

    #[fixture]
    #[once]
    pub fn test_services() -> (
        AppServices,
        Jwt,
        SecureStorage,
        Pool<ConnectionManager<PgConnection>>,
    ) {
        crate::api::tests::create_test_app_service("test_payment_handler")
    }

    #[rstest]
    #[actix_web::test]
    async fn test_get_payments_for_ownerships(
        test_services: &(
            AppServices,
            Jwt,
            SecureStorage,
            Pool<ConnectionManager<PgConnection>>,
        ),
    ) {
        let (services, jwt, _secure_storage, pool) = test_services.clone();
        let mut conn = pool.get().unwrap();

        let app_services = Arc::new(services);
        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(app_services.clone()))
                .route(
                    "/api/payment/ownership",
                    web::post().to(get_payments_for_ownerships),
                ),
        )
        .await;

        // Create database entries
        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let user = crate::db::model::tests::create_test_user(&mut conn);
        let ownership_1 = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);
        let ownership_2 = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);
        let ownership_3 = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);

        let payment_1 =
            crate::db::model::tests::create_test_payment(&mut conn, &ownership_1, &user);
        let payment_2 =
            crate::db::model::tests::create_test_payment(&mut conn, &ownership_1, &user);
        let payment_3 =
            crate::db::model::tests::create_test_payment(&mut conn, &ownership_2, &user);
        let payment_4 =
            crate::db::model::tests::create_test_payment(&mut conn, &ownership_3, &user);

        // Act: Send http request
        let jwt_token = jwt.encode_jwt_token(
            ClaimsBuilder::new("admin")
                .permission(PermissionType::ViewPayments.to_string())
                .build(),
        );

        // Act: Send http request
        let request_payload = fs::read_to_string(
            "tests/resources/api/payload/payment_management/request/get_payments_in_ownerships.json",
        )
        .unwrap();

        let request_payload = request_payload.replace(
            "\"{{ownership_ids}}\"",
            &format!(
                "[{},{}]",
                ownership_1.ownership_id, ownership_2.ownership_id
            ),
        );

        let req = test::TestRequest::post()
            .uri("/api/payment/ownership")
            .insert_header(("Authorization", format!("Bearer {jwt_token}")))
            .insert_header(("Content-Type", "application/json"))
            .set_payload(request_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert: Check HTTP status code and returned payload
        assert_eq!(resp.status(), StatusCode::OK);
        let response_json: PaymentList = test::read_body_json(resp).await;
        let response_payment_ids: Vec<i32> = response_json
            .payments
            .iter()
            .map(|payment| payment.payment_id)
            .collect();
        assert!(response_payment_ids.contains(&payment_1.payment_id));
        assert!(response_payment_ids.contains(&payment_2.payment_id));
        assert!(response_payment_ids.contains(&payment_3.payment_id));
        assert!(!response_payment_ids.contains(&payment_4.payment_id));
    }

    #[rstest]
    #[actix_web::test]
    async fn test_get_payments_in_state(
        test_services: &(
            AppServices,
            Jwt,
            SecureStorage,
            Pool<ConnectionManager<PgConnection>>,
        ),
    ) {
        let (services, jwt, _secure_storage, pool) = test_services.clone();
        let mut conn = pool.get().unwrap();

        let app_services = Arc::new(services);
        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(app_services.clone()))
                .route("/api/payment/state", web::get().to(get_payments_in_state)),
        )
        .await;

        // Create database entries
        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let user = crate::db::model::tests::create_test_user(&mut conn);
        let ownership_1 = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);
        let ownership_2 = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);
        let payment_1 = PaymentModel::create_payment(
            &mut conn,
            ownership_1.ownership_id,
            NaiveDate::from_ymd_opt(2025, 09, 05).unwrap(),
            10,
            PaymentState::Approved,
            "comment",
            "01",
            1,
            None,
        )
        .unwrap();
        let payment_2 = PaymentModel::create_payment(
            &mut conn,
            ownership_2.ownership_id,
            NaiveDate::from_ymd_opt(2025, 09, 05).unwrap(),
            10,
            PaymentState::Approved,
            "comment",
            "02",
            1,
            None,
        )
        .unwrap();
        let payment_3 = PaymentModel::create_payment(
            &mut conn,
            ownership_1.ownership_id,
            NaiveDate::from_ymd_opt(2025, 09, 05).unwrap(),
            10,
            PaymentState::Rejected,
            "comment",
            "03",
            1,
            None,
        )
        .unwrap();

        // Act: Send http request
        let jwt_token = jwt.encode_jwt_token(
            ClaimsBuilder::new("admin")
                .permission(PermissionType::ViewPayments.to_string())
                .build(),
        );

        let req = test::TestRequest::get()
            .uri("/api/payment/state?state=Approved")
            .insert_header(("Authorization", format!("Bearer {jwt_token}")))
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert: Check HTTP status code and returned payload
        assert_eq!(resp.status(), StatusCode::OK);
        let response_json: PaymentList = test::read_body_json(resp).await;
        let response_payment_ids: Vec<i32> = response_json
            .payments
            .iter()
            .map(|payment| payment.payment_id)
            .collect();
        assert!(response_payment_ids.contains(&payment_1.payment_id));
        assert!(response_payment_ids.contains(&payment_2.payment_id));
        assert!(!response_payment_ids.contains(&payment_3.payment_id));
    }

    #[rstest]
    #[actix_web::test]
    async fn test_update_payment_state(
        test_services: &(
            AppServices,
            Jwt,
            SecureStorage,
            Pool<ConnectionManager<PgConnection>>,
        ),
    ) {
        let (services, jwt, _secure_storage, pool) = test_services.clone();
        let mut conn = pool.get().unwrap();

        let app_services = Arc::new(services);
        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(app_services.clone()))
                .route("/api/payment/update", web::post().to(update_payment_state)),
        )
        .await;

        // Create database entries
        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let user = crate::db::model::tests::create_test_user(&mut conn);
        let ownership = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);
        let payment = PaymentModel::create_payment(
            &mut conn,
            ownership.ownership_id,
            NaiveDate::from_ymd_opt(2025, 09, 05).unwrap(),
            10,
            PaymentState::PendingApproval,
            "comment",
            "01",
            user.user_id,
            None,
        )
        .unwrap();

        // Act: Send http request
        let request_payload = fs::read_to_string(
            "tests/resources/api/payload/payment_management/request/update_payment_state.json",
        )
        .unwrap();

        let request_payload =
            request_payload.replace("\"{{payment_id}}\"", &payment.payment_id.to_string());

        let jwt_token = jwt.encode_jwt_token(
            ClaimsBuilder::new("admin")
                .permission(PermissionType::ModifyPayments.to_string())
                .build(),
        );

        let req = test::TestRequest::post()
            .uri("/api/payment/update")
            .insert_header(("Authorization", format!("Bearer {jwt_token}")))
            .insert_header(("Content-Type", "application/json"))
            .set_payload(request_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert: Check HTTP status code and returned payload
        assert_eq!(resp.status(), StatusCode::OK);
        let response_json: Payment = test::read_body_json(resp).await;
        assert_eq!(response_json.payment_id, payment.payment_id);
        assert_eq!(response_json.state, PaymentState::Approved);
        assert_eq!(
            response_json.comment,
            "Marked as completed after verification"
        );
        assert_eq!(response_json.last_changed_by_user, 1);
        // Check DB
        let updated_payment = PaymentModel::get_payment(&mut conn, payment.payment_id).unwrap();
        assert_eq!(updated_payment.state, PaymentState::Approved);
        assert_eq!(
            updated_payment.comment,
            "Marked as completed after verification"
        );
        assert_eq!(updated_payment.last_changed_by_user, 1);
    }

    #[rstest]
    #[actix_web::test]
    async fn test_create_payment(
        test_services: &(
            AppServices,
            Jwt,
            SecureStorage,
            Pool<ConnectionManager<PgConnection>>,
        ),
    ) {
        let (services, jwt, _secure_storage, pool) = test_services.clone();
        let mut conn = pool.get().unwrap();

        let app_services = Arc::new(services);
        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(app_services.clone()))
                .route("/api/payment/", web::post().to(create_payment)),
        )
        .await;

        // --- Arrange test data ---
        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let user = crate::db::model::tests::create_test_user(&mut conn);
        let ownership = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);

        // JWT with default user (owner, no special permissions)
        let jwt_token = jwt.encode_jwt_token(ClaimsBuilder::new(&user.user_name).build());

        let json_payload = fs::read_to_string(
            "tests/resources/api/payload/payment_management/request/create_payment.json",
        )
        .expect("Failed to read JSON file");
        let json_payload =
            json_payload.replace("\"{{ownership_id}}\"", &ownership.ownership_id.to_string());
        let json_payload = json_payload.replace("\"{{user_id}}\"", &user.user_id.to_string());

        // --- Act ---
        let req = test::TestRequest::post()
            .uri("/api/payment/")
            .insert_header(("Authorization", format!("Bearer {jwt_token}")))
            .insert_header(("Content-Type", "application/json"))
            .set_payload(json_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // --- Assert ---
        assert_eq!(resp.status(), StatusCode::OK);
        let payment: Payment = test::read_body_json(resp).await;

        assert_eq!(payment.ownership_id, ownership.ownership_id);
        assert_eq!(payment.amount, 12000);
        assert_eq!(payment.reference, "BANK-123");
        assert_eq!(payment.last_changed_by_user, user.user_id);
        assert_eq!(payment.state, PaymentState::PendingApproval);
        assert_eq!(payment.date, NaiveDate::from_ymd_opt(2025, 06, 11).unwrap());
    }

    #[rstest]
    #[actix_web::test]
    async fn test_create_payment_unauthorized(
        test_services: &(
            AppServices,
            Jwt,
            SecureStorage,
            Pool<ConnectionManager<PgConnection>>,
        ),
    ) {
        let (services, jwt, _secure_storage, pool) = test_services.clone();
        let mut conn = pool.get().unwrap();

        let app_services = Arc::new(services);
        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(app_services.clone()))
                .route("/api/payment/", web::post().to(create_payment)),
        )
        .await;

        // --- Arrange test data ---
        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let user = crate::db::model::tests::create_test_user(&mut conn);
        let user_other = crate::db::model::tests::create_test_user(&mut conn);
        let ownership = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);

        // JWT with default user (owner, no special permissions)
        let jwt_token = jwt.encode_jwt_token(ClaimsBuilder::new(&user_other.user_name).build());

        let json_payload = fs::read_to_string(
            "tests/resources/api/payload/payment_management/request/create_payment.json",
        )
        .expect("Failed to read JSON file");
        let json_payload =
            json_payload.replace("\"{{ownership_id}}\"", &ownership.ownership_id.to_string());
        let json_payload = json_payload.replace("\"{{user_id}}\"", &user.user_id.to_string());

        // --- Act ---
        let req = test::TestRequest::post()
            .uri("/api/payment/")
            .insert_header(("Authorization", format!("Bearer {jwt_token}")))
            .insert_header(("Content-Type", "application/json"))
            .set_payload(json_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // --- Assert ---
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
