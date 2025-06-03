use crate::{
    api::model::payment::{Payment, PaymentList, UpdatePaymentStateRequest},
    db::model::payment::PaymentState,
};
use actix_web::{HttpResponse, Responder, web};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct OwnershipPaymentsQuery {
    ownership_id: i32,
}

#[derive(Deserialize)]
pub struct PaymentsInStateQuery {
    state: PaymentState,
    offset: Option<i32>,
    limit: Option<i32>,
}

/// Retrieve payments for a given ownership
///
/// Accepts an ownership ID and returns all payments associated with it.  
/// Authorization: bearer token.
#[utoipa::path(
    get,
    path = "/payment/ownership",
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
    _query: web::Query<OwnershipPaymentsQuery>,
) -> impl Responder {
    HttpResponse::NotImplemented()
}

/// Retrieve payments filtered by state
///
/// Accepts a payment state and returns matching payments, with optional pagination using offset and limit.  
/// Authorization: bearer token.
#[utoipa::path(
    get,
    path = "/payment/state",
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
pub async fn get_payments_in_state(_query: web::Query<PaymentsInStateQuery>) -> impl Responder {
    HttpResponse::NotImplemented()
}

/// Update the state and comment of a payment
///
/// Accepts the payment ID, new state, and an optional comment,  
/// and updates the corresponding payment record in the system.  
/// Authorization: bearer token.
#[utoipa::path(
    post,
    path = "/payment/update",
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
    _payload: web::Json<UpdatePaymentStateRequest>,
) -> impl Responder {
    HttpResponse::NotImplemented()
}
