use diesel::{insert_into, prelude::*};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

use crate::schema::user_permission;

use super::Error;

#[derive(Debug, Clone, PartialEq, Eq, DbEnum, Serialize, Deserialize)]
#[db_enum(
    existing_type_path = "crate::schema::sql_types::PermissionType",
    value_style = "SCREAMING_SNAKE_CASE"
)]
pub enum PermissionType {
    ViewPayments,
    EditPayments,
    ManageUsers,
    ViewReports,
}

#[derive(
    Queryable, Selectable, Insertable, Debug, PartialEq, Eq, Clone, Serialize, Deserialize,
)]
#[diesel(table_name = user_permission)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserPermission {
    pub user_id: i32,
    pub permission: PermissionType,
}

impl UserPermission {
    pub fn set_user_permission(
        conn: &mut PgConnection,
        user_id: i32,
        permissions: Vec<PermissionType>,
    ) -> Result<Vec<PermissionType>, Error> {
        UserPermission::remove_all_user_permission(conn, user_id)?;

        insert_into(user_permission::table)
            .values(
                permissions
                    .iter()
                    .map(|permission| {
                        (
                            user_permission::user_id.eq(user_id),
                            user_permission::permission.eq(permission),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .get_results(conn)
            .map(|permissions: Vec<UserPermission>| {
                permissions
                    .into_iter()
                    .map(|user_permission| user_permission.permission)
                    .collect()
            })
            .map_err(Into::into)
    }

    pub fn remove_all_user_permission(
        conn: &mut PgConnection,
        user_id: i32,
    ) -> Result<usize, Error> {
        diesel::delete(user_permission::table)
            .filter(user_permission::user_id.eq(user_id))
            .execute(conn)
            .map_err(Into::into)
    }

    pub fn get_all_user_permissions(
        conn: &mut PgConnection,
        user_id: i32,
    ) -> Result<Vec<PermissionType>, Error> {
        user_permission::table
            .filter(user_permission::user_id.eq(user_id))
            .select(user_permission::permission)
            .get_results(conn)
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use diesel::{
        PgConnection,
        r2d2::{ConnectionManager, Pool},
    };

    use rstest::{fixture, rstest};

    use crate::model::user_permission::{PermissionType, UserPermission};

    #[fixture]
    #[once]
    pub fn pool() -> Pool<ConnectionManager<PgConnection>> {
        crate::tests::get_test_db_connection_pool("test_user_permission")
    }

    #[rstest]
    pub fn test_create_permissions(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let user = crate::model::tests::create_test_user(&mut conn);

        let user_permission = UserPermission::set_user_permission(
            &mut conn,
            user.user_id,
            vec![PermissionType::ViewPayments, PermissionType::EditPayments],
        )
        .unwrap();

        assert_eq!(
            user_permission,
            vec![PermissionType::ViewPayments, PermissionType::EditPayments]
        );

        let user_permission = UserPermission::set_user_permission(
            &mut conn,
            user.user_id,
            vec![PermissionType::ViewPayments, PermissionType::ManageUsers],
        )
        .unwrap();
        assert_eq!(
            user_permission,
            vec![PermissionType::ViewPayments, PermissionType::ManageUsers]
        );
    }

    #[rstest]
    pub fn get_all_user_permissions(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let user = crate::model::tests::create_test_user(&mut conn);
        let permissions = crate::model::tests::grant_user_all_permissions(&mut conn, &user);

        assert_eq!(
            UserPermission::get_all_user_permissions(&mut conn, user.user_id).unwrap(),
            permissions
        );
    }
}
