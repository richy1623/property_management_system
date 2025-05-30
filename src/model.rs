use tonic::{Code, Status};

pub mod artifact;
pub mod estate;
pub mod ownership;
pub mod payment;
pub mod payment_state_audit;
pub mod plot;
pub mod user;
pub mod user_permission;

#[derive(Debug, PartialEq)]
pub enum Error {
    DatabaseError(diesel::result::Error),
    CryptographyError(password_hash::Error),
    AuthorizationError(jsonwebtoken::errors::Error),
    InternalError(String),
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        Error::DatabaseError(err)
    }
}

impl From<password_hash::Error> for Error {
    fn from(err: password_hash::Error) -> Self {
        Error::CryptographyError(err)
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        Error::AuthorizationError(err)
    }
}

impl From<aes_gcm::Error> for Error {
    fn from(err: aes_gcm::Error) -> Self {
        Error::InternalError(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::InternalError(err.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        Error::InternalError(err.to_string())
    }
}

impl From<Error> for Status {
    fn from(err: Error) -> Self {
        match err {
            Error::DatabaseError(diesel_err) => match diesel_err {
                diesel::result::Error::NotFound => {
                    Status::new(Code::NotFound, "Resource not found")
                }
                diesel::result::Error::DatabaseError(kind, info) => match kind {
                    diesel::result::DatabaseErrorKind::UniqueViolation => {
                        Status::new(Code::AlreadyExists, "Duplicate entry")
                    }
                    diesel::result::DatabaseErrorKind::UnableToSendCommand
                    | diesel::result::DatabaseErrorKind::ClosedConnection => {
                        Status::new(Code::Unavailable, "Database connection issue")
                    }
                    _ => Status::new(
                        Code::Internal,
                        format!("Database error: {}", info.message()),
                    ),
                },
                diesel::result::Error::BrokenTransactionManager => Status::new(
                    Code::Unavailable,
                    "Broken Transaction Manager, likely due to database connection issue",
                ),
                _ => Status::new(
                    Code::Internal,
                    format!("Unhandled database error: {}", diesel_err),
                ),
            },
            Error::CryptographyError(crypto_err) => match crypto_err {
                password_hash::Error::Password => {
                    Status::new(Code::Unauthenticated, "Invalid password")
                }
                _ => Status::new(Code::Internal, "Internal cryptographic error"),
            },
            Error::AuthorizationError(jwt_error) => {
                Status::new(Code::Unauthenticated, jwt_error.to_string())
            }
            Error::InternalError(message) => Status::new(Code::Internal, message),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use chrono::NaiveDate;
    use diesel::PgConnection;
    use uuid::Uuid;

    use super::{
        artifact::Artifact,
        estate::Estate,
        ownership::Ownership,
        payment::{Payment, PaymentState},
        plot::Plot,
        user::User,
        user_permission::{PermissionType, UserPermission},
    };

    pub fn create_test_user(conn: &mut PgConnection) -> User {
        User::create_user(
            conn,
            &Uuid::new_v4().to_string(),
            "name",
            Some("0721234567"),
            Some("email@noreply.com"),
            Some("id_number"),
            "password",
        )
        .unwrap()
    }

    pub fn create_test_estate(conn: &mut PgConnection) -> Estate {
        Estate::create_estate(
            conn,
            &Uuid::new_v4().to_string(),
            "location",
            "district",
            "landmark",
        )
        .unwrap()
    }

    pub fn create_test_plot(conn: &mut PgConnection, estate: &Estate) -> Plot {
        Plot::create_plot(
            conn,
            estate.estate_id,
            &Uuid::new_v4().to_string(),
            "phase 1",
            30,
            40,
        )
        .unwrap()
    }

    pub fn create_test_ownership(conn: &mut PgConnection, plot: &Plot, user: &User) -> Ownership {
        Ownership::create_ownership(
            conn,
            user.user_id,
            plot.plot_id,
            super::ownership::OwnershipState::PendingContract,
            1000,
            100,
        )
        .unwrap()
    }

    pub fn create_test_payment(
        conn: &mut PgConnection,
        ownership: &Ownership,
        user: &User,
    ) -> Payment {
        Payment::create_payment(
            conn,
            ownership.ownership_id,
            NaiveDate::from_ymd_opt(2025, 09, 05).unwrap(),
            100,
            PaymentState::PendingApproval,
            "comment",
            "reference",
            user.user_id,
            "path/to/payment",
        )
        .unwrap()
    }

    pub fn create_test_artifact(conn: &mut PgConnection, user: &User, file_path: &str) -> Artifact {
        Artifact::create_artifact(
            conn,
            &Uuid::new_v4().to_string(),
            file_path,
            user.user_id,
            true,
        )
        .unwrap()
    }

    pub fn grant_user_all_permissions(conn: &mut PgConnection, user: &User) -> Vec<PermissionType> {
        UserPermission::set_user_permission(
            conn,
            user.user_id,
            vec![
                PermissionType::Employee,
                PermissionType::ViewPayments,
                PermissionType::ModifyPayments,
                PermissionType::ViewUsers,
                PermissionType::ModifyUsers,
                PermissionType::ViewProperties,
                PermissionType::ModifyProperties,
                PermissionType::ViewOwnerships,
                PermissionType::ModifyOwnerships,
                PermissionType::ViewReports,
            ],
        )
        .unwrap()
    }
}
