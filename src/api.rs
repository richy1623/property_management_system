use std::sync::Arc;

use actix_web::HttpResponse;

use crate::{
    jwt::Jwt,
    secure_storage::SecureStorage,
    service::{
        artifact_management::artifact_management_service::ArtifactManagementService,
        payment_management::payment_management_service::PaymentManagementService,
        property_management::property_management_service::PropertyManagementService,
        user_management::user_management_service::UserManagementService,
    },
};

pub mod config;
pub mod gateway;
pub mod handler;
pub mod model;

#[derive(Clone)]
pub struct AppServices {
    pub user_management_service: Arc<UserManagementService>,
    pub payment_management_service: Arc<PaymentManagementService>,
    pub property_management_service: Arc<PropertyManagementService>,
    pub archive_management_service: Arc<ArtifactManagementService>,
}

impl AppServices {
    pub fn new(database_url: &str, jwt_secret: &str, secure_storage: SecureStorage) -> Self {
        let db_pool = crate::get_connection_pool(database_url);

        AppServices {
            user_management_service: Arc::new(UserManagementService {
                database_connection_pool: db_pool.clone(),
                jwt: Jwt::new(jwt_secret),
            }),
            payment_management_service: Arc::new(PaymentManagementService {
                database_connection_pool: db_pool.clone(),
                jwt: Jwt::new(jwt_secret),
            }),
            property_management_service: Arc::new(PropertyManagementService {
                database_connection_pool: db_pool.clone(),
                jwt: Jwt::new(jwt_secret),
            }),
            archive_management_service: Arc::new(ArtifactManagementService {
                database_connection_pool: db_pool,
                jwt: Jwt::new(jwt_secret),
                secure_storage,
            }),
        }
    }
}

pub fn map_grpc_error_to_http_response(status: tonic::Status) -> HttpResponse {
    match status.code() {
        tonic::Code::InvalidArgument | tonic::Code::FailedPrecondition => {
            tracing::warn!("Client error: {:?}", status);
            HttpResponse::BadRequest().body("Invalid input provided")
        }
        tonic::Code::Unauthenticated | tonic::Code::PermissionDenied => {
            tracing::warn!("Unauthorized access attempt: {:?}", status);
            HttpResponse::Unauthorized().body("Unauthorized access")
        }
        tonic::Code::NotFound => {
            tracing::info!("Resource not found: {:?}", status);
            HttpResponse::BadRequest().body("Resource not found")
        }
        tonic::Code::AlreadyExists => {
            tracing::info!("Conflict error: {:?}", status);
            HttpResponse::Conflict().body("Resource already exists")
        }
        tonic::Code::Unavailable | tonic::Code::DeadlineExceeded => {
            tracing::error!("Service unavailable or timeout: {:?}", status);
            HttpResponse::ServiceUnavailable().body("Service temporarily unavailable")
        }
        _ => {
            tracing::error!("Internal server error: {:?}", status);
            HttpResponse::InternalServerError().body("Internal server error")
        }
    }
}
