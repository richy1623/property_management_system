use argon2::{Argon2, PasswordHasher};
use diesel::{insert_into, prelude::*, result::Error};
use password_hash::SaltString;
use serde::{Deserialize, Serialize};

use crate::schema::user;

// TODO use name as well
pub const STATIC_PASSWORD_HASH_SALT: &str = "123e4567-e89b-12d3-a456-426614174000";

#[derive(
    Queryable, Selectable, Insertable, Debug, PartialEq, Eq, Clone, Serialize, Deserialize,
)]
#[diesel(table_name = user)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub user_id: i32,
    pub user_name: String,
    pub name: String,
    pub cellphone_number: Option<String>,
    pub email_address: Option<String>,
    pub nrc: Option<String>,
    pub password_hash: String,
}

impl User {
    pub fn create_user(
        conn: &mut PgConnection,
        user_name: String,
        name: String,
        cellphone_number: Option<String>,
        email_address: Option<String>,
        nrc: Option<String>,
        password: String,
    ) -> QueryResult<User> {
        insert_into(user::table)
            .values((
                user::name.eq(name),
                user::user_name.eq(user_name),
                user::cellphone_number.eq(cellphone_number),
                user::email_address.eq(email_address),
                user::nrc.eq(nrc),
                user::password_hash.eq(User::hash_password(&password).expect("TODO")),
            ))
            .get_result(conn)
    }

    pub fn validate_login(
        conn: &mut PgConnection,
        user_name: &str,
        password: &str,
    ) -> Result<bool, Error> {
        let user: User = user::table
            .filter(user::user_name.eq(user_name))
            .select(User::as_select())
            .first(conn)?;
        let password_hash_computed = User::hash_password(password).expect("TODO");
        return Ok(user.password_hash == password_hash_computed);
    }

    pub fn hash_password(password: &str) -> Result<String, password_hash::Error> {
        let salt = SaltString::from_b64(STATIC_PASSWORD_HASH_SALT)?;
        let argon2 = Argon2::default();

        let hashed = argon2.hash_password(password.as_bytes(), &salt)?;
        Ok(hashed.to_string())
    }

    pub fn get_user_by_user_name(conn: &mut PgConnection, user_name: &str) -> Result<User, Error> {
        user::table
            .filter(user::user_name.eq(user_name))
            .select(User::as_select())
            .first(conn)
    }

    pub fn get_user_by_user_id(conn: &mut PgConnection, user_id: i32) -> Result<User, Error> {
        user::table
            .find(user_id)
            .select(User::as_select())
            .first(conn)
    }
}
