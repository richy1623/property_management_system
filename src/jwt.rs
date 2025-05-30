use std::time::Duration;

use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::model::Error;

const DEFAULT_ISSUER: &str = "my-app";
const DEFAULT_EXPIRY: Duration = Duration::from_secs(30 * 60); // 60 minutes

static JWT_VALIDATION: Lazy<Validation> = Lazy::new(|| {
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256); // adjust to match your algorithm
    validation.validate_exp = true;
    validation.set_issuer(&[DEFAULT_ISSUER]);
    validation
});

/// Represents the payload (claims) of a JWT token.
///
/// This struct is typically used with libraries like `jsonwebtoken` for encoding
/// and decoding JWTs. It contains standard and custom fields relevant to
/// authentication and authorization.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Claims {
    //TODO make this rather be a user_id
    /// Subject of the token (username).
    pub sub: String,

    /// Expiration time (as a UTC UNIX timestamp in seconds).
    ///
    /// Required by the JWT spec. Tokens are considered invalid after this time.
    pub exp: u64,

    /// Issued at time (as a UTC UNIX timestamp in seconds).
    ///
    /// Indicates when the token was generated.
    pub iat: u64,

    /// Issuer of the token (e.g., your application or authentication service).
    ///
    /// Useful for validating the origin of the token.
    pub iss: String,

    /// A list of permissions or roles granted to the subject.
    ///
    /// Can be used for access control within the application.
    pub permissions: Vec<String>,
}

pub struct ClaimsBuilder {
    expiry_duration: Duration,
    iss: String,
    sub: String,
    permissions: Vec<String>,
}

impl ClaimsBuilder {
    pub fn new(sub: &str) -> Self {
        ClaimsBuilder {
            expiry_duration: DEFAULT_EXPIRY,
            iss: DEFAULT_ISSUER.to_string(),
            sub: sub.to_string(),
            permissions: Vec::new(),
        }
    }

    /// Optional override of the expiry duration
    pub fn expiry(mut self, duration: Duration) -> Self {
        self.expiry_duration = duration;
        self
    }

    /// Optional override of the issuer (default is "my-app")
    pub fn iss<S: Into<String>>(mut self, iss: S) -> Self {
        self.iss = iss.into();
        self
    }

    pub fn permission<S: Into<String>>(mut self, permission: S) -> Self {
        self.permissions.push(permission.into());
        self
    }

    pub fn permissions<I, S>(mut self, permissions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.permissions = permissions.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn build(self) -> Claims {
        let now = jsonwebtoken::get_current_timestamp();

        let exp = now + self.expiry_duration.as_secs();
        let iat = now;

        Claims {
            exp,
            iat,
            iss: self.iss,
            sub: self.sub,
            permissions: self.permissions,
        }
    }
}

#[derive(Clone)]
pub struct Jwt {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl Jwt {
    pub fn new(secret: &str) -> Self {
        Jwt {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    pub fn encode_jwt_token(&self, claims: Claims) -> String {
        jsonwebtoken::encode(&Header::default(), &claims, &self.encoding_key).expect(&format!(
            "Failed to create JWT token for claims {:?}",
            claims
        ))
    }

    pub fn decode_jwt_token(&self, token: &str) -> Result<TokenData<Claims>, Error> {
        jsonwebtoken::decode::<Claims>(&token, &self.decoding_key, &JWT_VALIDATION)
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::Jwt;
    use crate::{
        jwt::{ClaimsBuilder, DEFAULT_ISSUER},
        model::Error,
    };

    #[test]
    pub fn test_encode() {
        let jwt = Jwt::new("a-string-secret-at-least-256-bits-long");

        let jwt_token =
            jwt.encode_jwt_token(ClaimsBuilder::new("user_1").permission("test").build());

        let token_data = jwt.decode_jwt_token(&jwt_token).unwrap();

        let now = jsonwebtoken::get_current_timestamp();

        assert_eq!(token_data.claims.iss, DEFAULT_ISSUER);
        assert_eq!(token_data.claims.sub, "user_1");
        assert_eq!(token_data.claims.permissions, vec!["test"]);
        assert!(token_data.claims.iat <= now);
        assert!(token_data.claims.exp > now);
    }

    #[test]
    pub fn test_decode() {
        let jwt = Jwt::new("a-string-secret-at-least-256-bits-long");

        let token_data = jwt
            .decode_jwt_token("eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ1c2VyXzEiLCJleHAiOjUwMDAwMDAwMDAsImlhdCI6MTc0NjU2MzcyNSwiaXNzIjoibXktYXBwIiwicGVybWlzc2lvbnMiOlsidGVzdCIsIm90aGVyIl19.RjM8WdiB8rbLsDKCaFMSMxGoSNAFNIZLFhq-MN6wVtE")
            .unwrap();

        assert_eq!(token_data.claims.iss, "my-app");
        assert_eq!(token_data.claims.sub, "user_1");
        assert_eq!(token_data.claims.permissions, vec!["test", "other"]);
        assert_eq!(token_data.claims.iat, 1746563725);
        assert_eq!(token_data.claims.exp, 5000000000);
    }

    #[test]
    pub fn test_expired_token() {
        let jwt = Jwt::new("a-string-secret-at-least-256-bits-long");

        let err = jwt
        .decode_jwt_token("eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ1c2VyXzEiLCJleHAiOjAsImlhdCI6MCwiaXNzIjoibXktYXBwIiwicGVybWlzc2lvbnMiOlsidGVzdCJdfQ.0VA-tSe1cOS3rCdFZWXBgddCCtXjzXTwbTh49WFDE8g")
        .unwrap_err();

        match err {
            Error::AuthorizationError(jwt_err) => {
                assert_eq!(
                    jwt_err.kind(),
                    &jsonwebtoken::errors::ErrorKind::ExpiredSignature
                );
            }
            _ => panic!("Expected Jwt error"),
        }
    }
}
