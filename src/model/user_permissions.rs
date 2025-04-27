use diesel::{insert_into, prelude::*};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

use crate::schema::user_permissions;

#[derive(Debug, Clone, PartialEq, Eq, DbEnum, Serialize, Deserialize)]
#[db_enum(existing_type_path = "crate::schema::sql_types::PermissionType")]
pub enum PermissionType {
    ViewPayments,
    EditPayments,
    ManageUsers,
    ViewReports,
}

#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct UserPermission {
    pub user_id: i32,
    pub permission: PermissionType,
}

impl UserPermission {
    pub fn set_user_permissions(
        conn: &mut PgConnection,
        user_id: i32,
        permissions: Vec<PermissionType>,
    ) -> Result<Vec<UserPermission>, diesel::result::Error> {
        UserPermission::remove_all_user_permissions(conn, user_id)?;

        insert_into(user_permissions::table)
            .values(
                permissions
                    .iter()
                    .map(|permission| {
                        (
                            user_permissions::user_id.eq(user_id),
                            user_permissions::permission.eq(permission),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .get_results(conn)
    }

    pub fn remove_all_user_permissions(
        conn: &mut PgConnection,
        user_id: i32,
    ) -> Result<usize, diesel::result::Error> {
        diesel::delete(user_permissions::table)
            .filter(user_permissions::user_id.eq(user_id))
            .execute(conn)
    }
}
