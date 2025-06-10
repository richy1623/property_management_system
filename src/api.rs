use std::sync::Arc;

use actix_web::{
    HttpRequest, HttpResponse,
    error::{Error, ErrorBadRequest, ErrorUnauthorized},
};

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

pub fn extract_authorization_header(http_request: &HttpRequest) -> Result<String, Error> {
    match http_request.headers().get("Authorization") {
        Some(value) => match value.to_str() {
            Ok(s) => Ok(s.to_string()),
            Err(_) => Err(ErrorBadRequest("Invalid Authorization header format")),
        },
        None => Err(ErrorUnauthorized("Missing Authorization header")),
    }
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

#[cfg(test)]
mod tests {
    use diesel::{
        PgConnection,
        r2d2::{ConnectionManager, Pool},
    };
    use tempfile::TempDir;

    use super::*;

    pub fn create_test_app_service(
        test_name: &str,
    ) -> (
        AppServices,
        Jwt,
        SecureStorage,
        Pool<ConnectionManager<PgConnection>>,
    ) {
        let db_pool = crate::tests::get_test_db_connection_pool(test_name);

        let temp_dir = TempDir::new().unwrap();
        let secure_storage = SecureStorage::new(temp_dir.path(), &[0u8; 32]).unwrap();

        let jwt_secret = "a-secret-at-least-256-bits-long";
        let jwt = Jwt::new(jwt_secret);

        (
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
                    database_connection_pool: db_pool.clone(),
                    jwt: Jwt::new(jwt_secret),
                    secure_storage: secure_storage.clone(),
                }),
            },
            jwt,
            secure_storage,
            db_pool.clone(),
        )
    }
}
