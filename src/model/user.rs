use argon2::{
    Argon2, Params,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use diesel::{insert_into, prelude::*, result::Error};
use once_cell::sync::Lazy;
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

use crate::schema::user;

// Create the static Argon2 instance
static ARGON2: Lazy<Argon2> = Lazy::new(|| {
    let params = Params::new(256, 4, Params::MIN_P_COST, None).expect("Invalid Argon2 params");

    Argon2::new(
        argon2::Algorithm::default(),
        argon2::Version::default(),
        params,
    )
});

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
    pub identification_number: Option<String>,
    pub password_hash: String,
}

impl User {
    pub fn create_user(
        conn: &mut PgConnection,
        user_name: &str,
        name: &str,
        cellphone_number: Option<&str>,
        email_address: Option<&str>,
        identification_number: Option<&str>,
        password: &str,
    ) -> Result<User, Error> {
        insert_into(user::table)
            .values((
                user::name.eq(name),
                user::user_name.eq(user_name),
                user::cellphone_number.eq(cellphone_number),
                user::email_address.eq(email_address),
                user::identification_number.eq(identification_number),
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
        match ARGON2.verify_password(
            password.as_bytes(),
            &PasswordHash::new(&user.password_hash).expect("TODO"),
        ) {
            Ok(_) => Ok(true),
            Err(password_hash::Error::Password) => Ok(false),
            _ => Err(crate::model::user::Error::BrokenTransactionManager),
        }
    }

    fn hash_password(password: &str) -> Result<String, password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);

        let hashed = ARGON2.hash_password(password.as_bytes(), &salt)?;
        Ok(hashed.serialize().to_string())
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

#[cfg(test)]
mod tests {
    use diesel::{
        PgConnection,
        r2d2::{ConnectionManager, Pool},
    };

    use rstest::{fixture, rstest};

    use crate::model::user::User;

    #[fixture]
    #[once]
    pub fn pool() -> Pool<ConnectionManager<PgConnection>> {
        crate::tests::get_test_db_connection_pool("test_user")
    }

    #[rstest]
    fn test_create_user(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let user = User::create_user(
            &mut conn,
            "user_1",
            "name",
            Some("0721234567"),
            Some("email@noreply.com"),
            Some("id_number"),
            "password",
        )
        .unwrap();

        assert_eq!(user.name, "name");
        assert_eq!(user.user_name, "user_1");
        assert_eq!(user.cellphone_number, Some("0721234567".to_string()));
        assert_eq!(user.email_address, Some("email@noreply.com".to_string()));
        assert_eq!(user.identification_number, Some("id_number".to_string()));

        let user_duplicate = User::create_user(
            &mut conn,
            "user_1",
            "name",
            Some("0721234567"),
            Some("email@noreply.com"),
            Some("id_number"),
            "password",
        );

        assert!(user_duplicate.is_err())
    }

    #[rstest]
    fn test_find_user_by_id(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let user = User::create_user(
            &mut conn,
            "user_2",
            "name",
            Some("0721234567"),
            Some("email@noreply.com"),
            Some("id_number"),
            "password",
        )
        .unwrap();

        assert_eq!(
            user,
            User::get_user_by_user_id(&mut conn, user.user_id).unwrap()
        );

        assert!(User::get_user_by_user_id(&mut conn, -1).is_err())
    }

    #[rstest]
    fn test_find_user_by_user_name(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let user = User::create_user(
            &mut conn,
            "user_3",
            "name",
            Some("0721234567"),
            Some("email@noreply.com"),
            Some("id_number"),
            "password",
        )
        .unwrap();

        assert_eq!(
            user,
            User::get_user_by_user_name(&mut conn, &user.user_name).unwrap()
        );

        assert!(User::get_user_by_user_name(&mut conn, "no_such_user").is_err())
    }

    #[rstest]
    fn test_validate_login(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let user = User::create_user(
            &mut conn,
            "user_4",
            "name",
            Some("0721234567"),
            Some("email@noreply.com"),
            Some("id_number"),
            "password",
        )
        .unwrap();

        assert_eq!(
            User::validate_login(&mut conn, &user.user_name, "password"),
            Ok(true),
        );

        assert_eq!(
            User::validate_login(&mut conn, &user.user_name, "password2"),
            Ok(false),
        );

        assert!(User::validate_login(&mut conn, "no_such_user", "password").is_err());
    }
}
