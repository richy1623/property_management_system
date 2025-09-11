pub mod artifact;
pub mod estate;
pub mod ownership;
pub mod payment;
pub mod payment_state_audit;
pub mod plot;
pub mod user;
pub mod user_permission;

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
            false,
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
            None,
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
            &vec![
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
