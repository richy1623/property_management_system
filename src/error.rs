use tonic::{Code, Status};

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
