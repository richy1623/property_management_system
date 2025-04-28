pub mod estate;
pub mod ownership;
pub mod payment;
pub mod payment_audit;
pub mod plot;
pub mod user;
pub mod user_permissions;

#[cfg(test)]
mod tests {
    use diesel::PgConnection;
    use uuid::Uuid;

    use super::{estate::Estate, ownership::Ownership, plot::Plot, user::User};

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
            10,
        )
        .unwrap()
    }
}
