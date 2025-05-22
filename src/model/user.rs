use argon2::{
    Argon2, Params,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use diesel::{insert_into, prelude::*};
use once_cell::sync::Lazy;
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

use crate::schema::user;

use super::Error;

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
            .map_err(Into::into)
    }

    pub fn validate_login(
        conn: &mut PgConnection,
        user_name: &str,
        password: &str,
    ) -> Result<User, Error> {
        let user: User = user::table
            .filter(user::user_name.eq(user_name))
            .select(User::as_select())
            .first(conn)?;
        ARGON2.verify_password(
            password.as_bytes(),
            &PasswordHash::new(&user.password_hash)?,
        )?;
        Ok(user)
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
            .map_err(Into::into)
    }

    pub fn get_user_by_user_id(conn: &mut PgConnection, user_id: i32) -> Result<User, Error> {
        user::table
            .find(user_id)
            .select(User::as_select())
            .first(conn)
            .map_err(Into::into)
    }

    pub fn get_filtered_users(
        conn: &mut PgConnection,
        username: Option<&str>,
        name_regex: Option<&str>,
        cellphone_number: Option<&str>,
        email_address: Option<&str>,
        identification_number: Option<&str>,
    ) -> Result<Vec<User>, Error> {
        let mut query = user::table.into_boxed();

        if let Some(username) = username {
            query = query.filter(user::user_name.eq(username));
        }

        if let Some(number) = cellphone_number {
            query = query.filter(user::cellphone_number.eq(number));
        }

        if let Some(email) = email_address {
            query = query.filter(user::email_address.eq(email));
        }

        if let Some(id) = identification_number {
            query = query.filter(user::identification_number.eq(id));
        }

        if let Some(regex) = name_regex {
            query = query.filter(user::name.ilike(format!("%{}%", regex)));
        }

        query
            .select(User::as_select())
            .get_results(conn)
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use diesel::{
        PgConnection,
        r2d2::{ConnectionManager, Pool},
    };

    use rstest::{fixture, rstest};

    use crate::model::{Error, user::User};

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

        let user = crate::model::tests::create_test_user(&mut conn);

        assert_eq!(
            user,
            User::get_user_by_user_id(&mut conn, user.user_id).unwrap()
        );

        assert!(User::get_user_by_user_id(&mut conn, -1).is_err())
    }

    #[rstest]
    fn test_find_user_by_user_name(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let user = crate::model::tests::create_test_user(&mut conn);

        assert_eq!(
            user,
            User::get_user_by_user_name(&mut conn, &user.user_name).unwrap()
        );

        assert!(User::get_user_by_user_name(&mut conn, "no_such_user").is_err())
    }

    #[rstest]
    fn test_validate_login(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let user = crate::model::tests::create_test_user(&mut conn);

        assert!(User::validate_login(&mut conn, &user.user_name, "password").is_ok());

        assert_eq!(
            User::validate_login(&mut conn, &user.user_name, "password2").unwrap_err(),
            Error::CryptographyError(password_hash::Error::Password),
        );

        assert!(User::validate_login(&mut conn, "no_such_user", "password").is_err());
    }

    #[rstest]
    pub fn test_get_filtered_users(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        // Seed users
        let user1 = User::create_user(
            &mut conn,
            "alice123",
            "Alice DxD",
            Some("123456789"),
            Some("alice@example.com"),
            Some("ID001"),
            "password",
        )
        .unwrap();

        let user2 = User::create_user(
            &mut conn,
            "bob456",
            "Bob DxD",
            Some("987654321"),
            Some("bob@example.com"),
            Some("ID002"),
            "password",
        )
        .unwrap();

        let user3 = User::create_user(
            &mut conn,
            "carol789",
            "Carol ZxZ",
            Some("123123123"),
            Some("carol@example.com"),
            Some("ID003"),
            "password",
        )
        .unwrap();

        // --- Filter by username ---
        let result =
            User::get_filtered_users(&mut conn, Some("alice123"), None, None, None, None).unwrap();
        assert_eq!(result, vec![user1.clone()]);

        // --- Filter by name regex ---
        let result =
            User::get_filtered_users(&mut conn, None, Some("Ali"), None, None, None).unwrap();
        assert_eq!(result, vec![user1.clone()]);

        // --- Filter by name regex 2---
        let result =
            User::get_filtered_users(&mut conn, None, Some("DxD"), None, None, None).unwrap();
        assert_eq!(result, vec![user1.clone(), user2.clone()]);

        // --- Filter by name regex 3---
        let result =
            User::get_filtered_users(&mut conn, None, Some("x"), None, None, None).unwrap();
        assert_eq!(result, vec![user1.clone(), user2.clone(), user3.clone()]);

        // --- Filter by cellphone number ---
        let result =
            User::get_filtered_users(&mut conn, None, None, Some("987654321"), None, None).unwrap();
        assert_eq!(result, vec![user2.clone()]);

        // --- Filter by email address ---
        let result =
            User::get_filtered_users(&mut conn, None, None, None, Some("carol@example.com"), None)
                .unwrap();
        assert_eq!(result, vec![user3.clone()]);

        // --- Filter by identification number ---
        let result =
            User::get_filtered_users(&mut conn, None, None, None, None, Some("ID002")).unwrap();
        assert_eq!(result, vec![user2.clone()]);

        // --- Multiple filters ---
        let result =
            User::get_filtered_users(&mut conn, None, Some("Bob"), Some("987654321"), None, None)
                .unwrap();
        assert_eq!(result, vec![user2.clone()]);
    }
}
