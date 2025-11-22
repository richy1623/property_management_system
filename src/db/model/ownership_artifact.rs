use diesel::{insert_into, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{db::schema::ownership_artifact, error::Error};

#[derive(
    Queryable, Selectable, Insertable, Debug, PartialEq, Eq, Clone, Serialize, Deserialize,
)]
#[diesel(table_name = ownership_artifact)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OwnershipArtifact {
    pub ownership_id: i32,
    pub artifact_id: i32,
}

impl OwnershipArtifact {
    pub fn get_all_artifact_ids_for_ownership(
        conn: &mut PgConnection,
        ownership_id: i32,
    ) -> Result<Vec<i32>, Error> {
        ownership_artifact::table
            .filter(ownership_artifact::ownership_id.eq(ownership_id))
            .select(ownership_artifact::artifact_id)
            .get_results(conn)
            .map_err(Into::into)
    }

    pub fn assign_artifact_to_ownership(
        conn: &mut PgConnection,
        ownership_id: i32,
        artifact_id: i32,
    ) -> Result<OwnershipArtifact, Error> {
        insert_into(ownership_artifact::table)
            .values((
                ownership_artifact::ownership_id.eq(ownership_id),
                ownership_artifact::artifact_id.eq(artifact_id),
            ))
            .get_result(conn)
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

    use super::*;

    #[fixture]
    #[once]
    pub fn pool() -> Pool<ConnectionManager<PgConnection>> {
        crate::tests::get_test_db_connection_pool("test_ownership_artifact")
    }

    #[rstest]
    pub fn test_insert_artifact_for_ownership(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let user = crate::db::model::tests::create_test_user(&mut conn);
        let ownership = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);
        let artifact = crate::db::model::tests::create_test_artifact(&mut conn, &user, "file_path");

        let ownership_artifact = OwnershipArtifact::assign_artifact_to_ownership(
            &mut conn,
            ownership.ownership_id,
            artifact.artifact_id,
        )
        .unwrap();

        assert_eq!(ownership_artifact.ownership_id, ownership.ownership_id);
        assert_eq!(ownership_artifact.artifact_id, artifact.artifact_id);

        let inserted_ownership_artifacts = ownership_artifact::table
            .filter(ownership_artifact::ownership_id.eq(ownership.ownership_id))
            .filter(ownership_artifact::artifact_id.eq(artifact.artifact_id))
            .select(OwnershipArtifact::as_select())
            .get_results(&mut conn)
            .unwrap();

        assert_eq!(inserted_ownership_artifacts, vec![ownership_artifact]);
    }

    #[rstest]
    pub fn test_get_all_artifacts_for_ownership(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let user = crate::db::model::tests::create_test_user(&mut conn);
        let ownership_1 = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);
        let ownership_2 = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);
        let artifact_1 =
            crate::db::model::tests::create_test_artifact(&mut conn, &user, "file_path");
        let artifact_2 =
            crate::db::model::tests::create_test_artifact(&mut conn, &user, "file_path");
        let artifact_3 =
            crate::db::model::tests::create_test_artifact(&mut conn, &user, "file_path");

        OwnershipArtifact::assign_artifact_to_ownership(
            &mut conn,
            ownership_1.ownership_id,
            artifact_1.artifact_id,
        )
        .unwrap();
        OwnershipArtifact::assign_artifact_to_ownership(
            &mut conn,
            ownership_1.ownership_id,
            artifact_2.artifact_id,
        )
        .unwrap();
        OwnershipArtifact::assign_artifact_to_ownership(
            &mut conn,
            ownership_2.ownership_id,
            artifact_3.artifact_id,
        )
        .unwrap();

        assert_eq!(
            OwnershipArtifact::get_all_artifact_ids_for_ownership(
                &mut conn,
                ownership_1.ownership_id
            )
            .unwrap(),
            vec![artifact_1.artifact_id, artifact_2.artifact_id]
        );
        assert_eq!(
            OwnershipArtifact::get_all_artifact_ids_for_ownership(
                &mut conn,
                ownership_2.ownership_id
            )
            .unwrap(),
            vec![artifact_3.artifact_id]
        );
        assert_eq!(
            OwnershipArtifact::get_all_artifact_ids_for_ownership(&mut conn, -1).unwrap(),
            Vec::<i32>::new()
        );
    }
}
