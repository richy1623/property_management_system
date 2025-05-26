use crate::schema::artifact;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use super::Error;

#[derive(
    Queryable, Selectable, Insertable, Debug, PartialEq, Eq, Clone, Serialize, Deserialize,
)]
#[diesel(table_name = artifact)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Artifact {
    pub artifact_id: i32,
    pub file_display_name: String,
    pub file_path: String,
    pub upload_date: chrono::NaiveDate,
    pub uploaded_by: i32,
}

impl Artifact {
    pub fn create_artifact(
        conn: &mut PgConnection,
        file_display_name: &str,
        file_path: &str,
        upload_date: chrono::NaiveDate,
        uploaded_by: i32,
    ) -> Result<Artifact, Error> {
        diesel::insert_into(artifact::table)
            .values((
                artifact::file_display_name.eq(file_display_name),
                artifact::file_path.eq(file_path),
                artifact::upload_date.eq(upload_date),
                artifact::uploaded_by.eq(uploaded_by),
            ))
            .get_result(conn)
            .map_err(Into::into)
    }

    pub fn get_artifact(conn: &mut PgConnection, artifact_id: i32) -> Result<Artifact, Error> {
        artifact::table
            .find(artifact_id)
            .select(Artifact::as_select())
            .first(conn)
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use diesel::{
        PgConnection,
        r2d2::{ConnectionManager, Pool},
    };
    use rstest::{fixture, rstest};

    use crate::model::artifact::Artifact;

    #[fixture]
    #[once]
    pub fn pool() -> Pool<ConnectionManager<PgConnection>> {
        crate::tests::get_test_db_connection_pool("test_artifact")
    }

    #[rstest]
    pub fn test_create_artifact(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let file_display_name = "test_document.pdf";
        let file_path = "/tmp/test_document.pdf";
        let upload_date = NaiveDate::from_ymd_opt(2025, 5, 26).unwrap();

        let user = crate::model::tests::create_test_user(&mut conn);

        let artifact = Artifact::create_artifact(
            &mut conn,
            file_display_name,
            file_path,
            upload_date,
            user.user_id,
        )
        .unwrap();

        assert_eq!(artifact.file_display_name, file_display_name);
        assert_eq!(artifact.file_path, file_path);
        assert_eq!(artifact.upload_date, upload_date);
        assert_eq!(artifact.uploaded_by, user.user_id);

        assert_eq!(
            artifact,
            Artifact::get_artifact(&mut conn, artifact.artifact_id).unwrap()
        );
    }

    #[rstest]
    fn test_get_artifact(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let user = crate::model::tests::create_test_user(&mut conn);
        let artifact = crate::model::tests::create_test_artifact(&mut conn, &user, "file_path");

        assert_eq!(
            artifact,
            Artifact::get_artifact(&mut conn, artifact.artifact_id).unwrap()
        );

        assert!(Artifact::get_artifact(&mut conn, -1).is_err())
    }
}
