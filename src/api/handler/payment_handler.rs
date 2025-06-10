use std::sync::Arc;

use crate::{
    api::{
        AppServices,
        model::payment::{
            OwnershipPaymentsQuery, Payment, PaymentList, PaymentsInStateQuery,
            UpdatePaymentStateRequest,
        },
    },
    db::model::payment::PaymentState,
    service::payment_management::payment_management_proto::{
        GetPaymentsForOwnershipRequest as ProtoGetPaymentsForOwnershipRequest,
        GetPaymentsInStateRequest as ProtoGetPaymentsInStateRequest,
        PaymentStateUpdateRequest as ProtoUpdatePaymentStateRequest,
        payment_management_server::PaymentManagement,
    },
};
use actix_web::Result;
use actix_web::{HttpRequest, HttpResponse, Responder, web};

/// Retrieve payments for a given ownership
///
/// Accepts an ownership ID and returns all payments associated with it.  
/// Authorization: bearer token.
#[utoipa::path(
    get,
    path = "/api/payment/ownership",
    tag = "Payment Management",
    summary = "Retrieve payments for a given ownership",
    description = "Accepts an ownership ID and returns all payments associated with it.  
                   Authorization: bearer token.",
    params(
        ("ownership_id" = i32, Query, description = "Ownership ID to filter payments by")
    ),
    responses(
        (status = 200, description = "List of payments for ownership", body = PaymentList),
        (status = 400, description = "Missing or invalid input"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_payments_for_ownership(
    services: web::Data<Arc<AppServices>>,
    http_request: HttpRequest,
    query: web::Query<OwnershipPaymentsQuery>,
) -> Result<impl Responder> {
    let authorization_header = crate::api::extract_authorization_header(&http_request)?;

    let request = tonic::Request::new(ProtoGetPaymentsForOwnershipRequest::from((
        query.into_inner(),
        authorization_header,
    )));

    match services
        .payment_management_service
        .get_payments_for_ownership(request)
        .await
    {
        Ok(response) => {
            // Convert the successful gRPC response into the HTTP response
            Ok(HttpResponse::Ok().json(PaymentList::from(response.into_inner())))
        }
        Err(status) => {
            // Map the gRPC error status to an appropriate HTTP response with logging
            Ok(crate::api::map_grpc_error_to_http_response(status))
        }
    }
}

/// Retrieve payments filtered by state
///
/// Accepts a payment state and returns matching payments, with optional pagination using offset and limit.  
/// Authorization: bearer token.
#[utoipa::path(
    get,
    path = "/api/payment/state",
    tag = "Payment Management",
    summary = "Retrieve payments filtered by state",
    description = "Accepts a payment state and returns matching payments,  
                   with optional pagination using offset and limit.  
                   Authorization: bearer token.",
    params(
        ("state" = PaymentState, Query, description = "State of payments to filter by"),
        ("offset" = Option<i32>, Query, description = "Pagination offset"),
        ("limit" = Option<i32>, Query, description = "Pagination limit")
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
    tracing::debug!("query: {:?}", query);
    let authorization_header = crate::api::extract_authorization_header(&http_request)?;

    let request = tonic::Request::new(ProtoGetPaymentsInStateRequest::from((
        query.into_inner(),
        authorization_header,
    )));

    match services
        .payment_management_service
        .get_payments_in_state(request)
        .await
    {
        Ok(response) => {
            // Convert the successful gRPC response into the HTTP response
            Ok(HttpResponse::Ok().json(PaymentList::from(response.into_inner())))
        }
        Err(status) => {
            // Map the gRPC error status to an appropriate HTTP response with logging
            Ok(crate::api::map_grpc_error_to_http_response(status))
        }
    }
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
    summary = "Update the state and comment of a payment",
    description = "Accepts the payment ID, new state, and an optional comment,  
                   and updates the corresponding payment record in the system.  
                   Authorization: bearer token.",
    request_body(content = UpdatePaymentStateRequest, description = "Details of the payment update"),
    responses(
        (status = 200, description = "Updated payment", body = Payment),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn update_payment_state(
    services: web::Data<Arc<AppServices>>,
    http_request: HttpRequest,
    query: web::Json<UpdatePaymentStateRequest>,
) -> Result<impl Responder> {
    let authorization_header = crate::api::extract_authorization_header(&http_request)?;

    let request = tonic::Request::new(ProtoUpdatePaymentStateRequest::from((
        query.into_inner(),
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
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
        crate::api::tests::create_test_app_service("test_payment_handler")
    }

    #[rstest]
    #[actix_web::test]
    async fn test_get_payments_for_ownership(
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
                    web::get().to(get_payments_for_ownership),
                ),
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
            "path",
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
            "path",
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
            "path",
        )
        .unwrap();

        // Act: Send http request
        let jwt_token = jwt.encode_jwt_token(
            ClaimsBuilder::new("admin")
                .permission(PermissionType::ViewPayments.to_string())
                .build(),
        );

        let req = test::TestRequest::get()
            .uri(&format!(
                "/api/payment/ownership?ownership_id={}",
                ownership_1.ownership_id.to_string()
            ))
            .insert_header(("Authorization", jwt_token))
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
        assert!(!response_payment_ids.contains(&payment_2.payment_id));
        assert!(response_payment_ids.contains(&payment_3.payment_id));
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
            "path",
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
            "path",
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
            "path",
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
            .insert_header(("Authorization", jwt_token))
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
            "path",
        )
        .unwrap();

        // Act: Send http request
        let request_payload = fs::read_to_string(
            "tests/resources/api/payload/payment_management/request/update_payment_state_request.json",
        ).unwrap();

        let request_payload = request_payload.replace(
            "\"{{payment_id}}\"",
            &format!("{}", payment.payment_id.to_string()),
        );

        let jwt_token = jwt.encode_jwt_token(
            ClaimsBuilder::new("admin")
                .permission(PermissionType::ModifyPayments.to_string())
                .build(),
        );

        let req = test::TestRequest::post()
            .uri("/api/payment/update")
            .insert_header(("Authorization", jwt_token))
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
}
