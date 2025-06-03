use diesel::{insert_into, prelude::*};
use diesel_derive_enum::DbEnum;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use strum::Display;
use utoipa::ToSchema;

use crate::db::schema::payment::{self};

use crate::error::Error;

static USER_SUBMITTED_PAYMENT_COMMENT: &str = "User submitted payment request";

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    DbEnum,
    Serialize,
    Deserialize,
    TryFromPrimitive,
    IntoPrimitive,
    ToSchema,
    Display,
)]
#[db_enum(
    existing_type_path = "crate::db::schema::sql_types::PaymentState",
    value_style = "SCREAMING_SNAKE_CASE"
)]
#[repr(i32)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentState {
    PendingApproval = 0,
    Approved = 1,
    Rejected = 2,
}

#[derive(
    Queryable, Selectable, Insertable, Debug, PartialEq, Eq, Clone, Serialize, Deserialize,
)]
#[diesel(table_name = payment)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Payment {
    pub payment_id: i32,
    pub ownership_id: i32,
    pub date: chrono::NaiveDate,
    pub amount: i32,
    pub state: PaymentState,
    pub comment: String,
    pub reference: String,
    pub last_changed_by_user: i32,
    pub proof_of_payment_path: String,
}

impl Payment {
    pub fn create_payment(
        conn: &mut PgConnection,
        ownership_id: i32,
        date: chrono::NaiveDate,
        amount: i32,
        state: PaymentState,
        comment: &str,
        reference: &str,
        last_changed_by_user: i32,
        proof_of_payment_path: &str,
    ) -> Result<Payment, Error> {
        insert_into(payment::table)
            .values((
                payment::ownership_id.eq(ownership_id),
                payment::date.eq(date),
                payment::amount.eq(amount),
                payment::state.eq(state),
                payment::comment.eq(comment),
                payment::reference.eq(reference),
                payment::last_changed_by_user.eq(last_changed_by_user),
                payment::proof_of_payment_path.eq(proof_of_payment_path),
            ))
            .get_result(conn)
            .map_err(Into::into)
    }

    pub fn create_user_payment(
        conn: &mut PgConnection,
        ownership_id: i32,
        date: chrono::NaiveDate,
        amount: i32,
        reference: &str,
        last_changed_by_user: i32,
        proof_of_payment_path: &str,
    ) -> Result<Payment, Error> {
        Payment::create_payment(
            conn,
            ownership_id,
            date,
            amount,
            PaymentState::PendingApproval,
            USER_SUBMITTED_PAYMENT_COMMENT,
            reference,
            last_changed_by_user,
            proof_of_payment_path,
        )
    }

    pub fn get_payment(conn: &mut PgConnection, payment_id: i32) -> Result<Payment, Error> {
        payment::table
            .find(payment_id)
            .select(Payment::as_select())
            .first(conn)
            .map_err(Into::into)
    }

    pub fn get_all_payments_for_ownership_id(
        conn: &mut PgConnection,
        ownership_id: i32,
    ) -> Result<Vec<Payment>, Error> {
        payment::table
            .filter(payment::ownership_id.eq(ownership_id))
            .select(Payment::as_select())
            .get_results(conn)
            .map_err(Into::into)
    }

    pub fn get_all_payments_in_state(
        conn: &mut PgConnection,
        payment_state: PaymentState,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Payment>, Error> {
        payment::table
            .filter(payment::state.eq(payment_state))
            .order(payment::payment_id)
            .limit(limit)
            .offset(offset)
            .select(Payment::as_select())
            .get_results(conn)
            .map_err(Into::into)
    }

    pub fn update_payment_state(
        conn: &mut PgConnection,
        payment_id: i32,
        payment_state: PaymentState,
        user_updating_state_id: i32,
        comment: &str,
    ) -> Result<Payment, Error> {
        diesel::update(payment::table.filter(payment::payment_id.eq(payment_id)))
            .set((
                payment::state.eq(payment_state),
                payment::last_changed_by_user.eq(user_updating_state_id),
                payment::comment.eq(comment),
            ))
            .get_result(conn)
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, Utc};
    use diesel::{
        PgConnection,
        r2d2::{ConnectionManager, Pool},
    };

    use rstest::{fixture, rstest};

    use crate::db::model::payment_state_audit::PaymentStateAudit;

    use super::*;

    #[fixture]
    #[once]
    pub fn pool() -> Pool<ConnectionManager<PgConnection>> {
        crate::tests::get_test_db_connection_pool("test_payment")
    }

    #[rstest]
    pub fn test_create_payment(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let user = crate::db::model::tests::create_test_user(&mut conn);
        let ownership = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);

        let date = NaiveDate::from_ymd_opt(2025, 09, 05).unwrap();

        let payment = Payment::create_payment(
            &mut conn,
            ownership.ownership_id,
            date,
            100,
            PaymentState::Rejected,
            "comment",
            "reference",
            user.user_id,
            "path/to/payment",
        )
        .unwrap();

        assert_eq!(payment.ownership_id, ownership.ownership_id);
        assert_eq!(payment.date, date);
        assert_eq!(payment.amount, 100);
        assert_eq!(payment.state, PaymentState::Rejected);
        assert_eq!(payment.comment, "comment");
        assert_eq!(payment.reference, "reference");
        assert_eq!(payment.last_changed_by_user, user.user_id);
        assert_eq!(payment.proof_of_payment_path, "path/to/payment");

        assert_eq!(
            payment,
            Payment::get_payment(&mut conn, payment.payment_id).unwrap()
        );
    }

    #[rstest]
    pub fn test_update_payment_state(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let user_1 = crate::db::model::tests::create_test_user(&mut conn);
        let user_2 = crate::db::model::tests::create_test_user(&mut conn);
        let ownership = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user_1);

        let payment_1 =
            crate::db::model::tests::create_test_payment(&mut conn, &ownership, &user_1);
        let payment_2 =
            crate::db::model::tests::create_test_payment(&mut conn, &ownership, &user_1);

        let updated_payment = Payment::update_payment_state(
            &mut conn,
            payment_1.payment_id,
            PaymentState::Rejected,
            user_2.user_id,
            "new comment",
        )
        .unwrap();

        // Check payment changed
        assert_eq!(updated_payment.state, PaymentState::Rejected);
        assert_eq!(updated_payment.last_changed_by_user, user_2.user_id);
        assert_eq!(updated_payment.comment, "new comment");

        // Check updated in db
        assert_eq!(
            updated_payment,
            Payment::get_payment(&mut conn, payment_1.payment_id).unwrap()
        );
        assert_ne!(
            payment_1,
            Payment::get_payment(&mut conn, payment_1.payment_id).unwrap()
        );
        // Check only effected expected entry
        assert_eq!(
            payment_2,
            Payment::get_payment(&mut conn, payment_2.payment_id).unwrap()
        );

        // Check updated the payment audit log
        let payment_state_audits = PaymentStateAudit::get_all_payment_state_audits_for_payment_id(
            &mut conn,
            payment_1.payment_id,
        )
        .unwrap();
        assert_eq!(payment_state_audits.len(), 1);
        let payment_state_audit = &payment_state_audits[0];
        assert_eq!(payment_state_audit.changed_by_user, user_2.user_id);
        assert_eq!(
            payment_state_audit.previous_state,
            PaymentState::PendingApproval
        );
        assert_eq!(payment_state_audit.new_state, PaymentState::Rejected);
        assert_eq!(
            payment_state_audit.change_timestamp.date_naive(),
            Utc::now().date_naive()
        );
        assert_eq!(payment_state_audit.comment, "new comment");
    }

    #[rstest]
    pub fn test_get_all_ownerships_for_user_id(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let estate = crate::db::model::tests::create_test_estate(&mut conn);

        let plot_1 = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let plot_2 = crate::db::model::tests::create_test_plot(&mut conn, &estate);

        let user_1 = crate::db::model::tests::create_test_user(&mut conn);
        let user_2 = crate::db::model::tests::create_test_user(&mut conn);

        let ownership_1 =
            crate::db::model::tests::create_test_ownership(&mut conn, &plot_1, &user_1);
        let ownership_2 =
            crate::db::model::tests::create_test_ownership(&mut conn, &plot_2, &user_2);

        let payment_1 =
            crate::db::model::tests::create_test_payment(&mut conn, &ownership_1, &user_1);
        let payment_2 =
            crate::db::model::tests::create_test_payment(&mut conn, &ownership_1, &user_1);
        let payment_3 =
            crate::db::model::tests::create_test_payment(&mut conn, &ownership_2, &user_2);

        assert_eq!(
            Payment::get_all_payments_for_ownership_id(&mut conn, ownership_1.ownership_id)
                .unwrap(),
            vec![payment_1, payment_2]
        );
        assert_eq!(
            Payment::get_all_payments_for_ownership_id(&mut conn, ownership_2.ownership_id)
                .unwrap(),
            vec![payment_3]
        );
    }

    #[rstest]
    pub fn test_get_all_payments_in_state(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let estate = crate::db::model::tests::create_test_estate(&mut conn);

        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);

        let user = crate::db::model::tests::create_test_user(&mut conn);

        let ownership = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);

        let payment_1 = crate::db::model::tests::create_test_payment(&mut conn, &ownership, &user);
        let payment_1 = Payment::update_payment_state(
            &mut conn,
            payment_1.payment_id,
            PaymentState::Approved,
            user.user_id,
            "comment",
        )
        .unwrap();
        let payment_2 = crate::db::model::tests::create_test_payment(&mut conn, &ownership, &user);
        let payment_2 = Payment::update_payment_state(
            &mut conn,
            payment_2.payment_id,
            PaymentState::Approved,
            user.user_id,
            "comment",
        )
        .unwrap();
        let payment_3 = crate::db::model::tests::create_test_payment(&mut conn, &ownership, &user);
        let payment_3 = Payment::update_payment_state(
            &mut conn,
            payment_3.payment_id,
            PaymentState::Rejected,
            user.user_id,
            "comment",
        )
        .unwrap();

        let payments =
            Payment::get_all_payments_in_state(&mut conn, PaymentState::Approved, 0, 10).unwrap();
        assert!(payments.contains(&payment_1));
        assert!(payments.contains(&payment_2));
        assert!(!payments.contains(&payment_3));

        let payments =
            Payment::get_all_payments_in_state(&mut conn, PaymentState::Approved, 0, 1).unwrap();
        assert!(
            payments.contains(&payment_1),
            "Expected: {}. Contains: {:?}",
            payment_1.payment_id,
            payments
        );
        assert!(!payments.contains(&payment_2));
        assert!(!payments.contains(&payment_3));

        let payments =
            Payment::get_all_payments_in_state(&mut conn, PaymentState::Approved, 1, 1).unwrap();
        assert!(!payments.contains(&payment_1));
        assert!(payments.contains(&payment_2));
        assert!(!payments.contains(&payment_3));

        assert_eq!(
            Payment::get_all_payments_in_state(&mut conn, PaymentState::Approved, 1000, 1).unwrap(),
            vec![]
        );

        let payments =
            Payment::get_all_payments_in_state(&mut conn, PaymentState::Rejected, 0, 10).unwrap();
        assert!(!payments.contains(&payment_1));
        assert!(!payments.contains(&payment_2));
        assert!(payments.contains(&payment_3));
    }

    #[rstest]
    pub fn test_create_user_payment(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let user = crate::db::model::tests::create_test_user(&mut conn);
        let ownership = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);

        let date = NaiveDate::from_ymd_opt(2025, 09, 05).unwrap();

        let payment = Payment::create_user_payment(
            &mut conn,
            ownership.ownership_id,
            date,
            100,
            "reference",
            user.user_id,
            "path",
        )
        .unwrap();

        assert_eq!(payment.ownership_id, ownership.ownership_id);
        assert_eq!(payment.date, date);
        assert_eq!(payment.amount, 100);
        assert_eq!(payment.state, PaymentState::PendingApproval);
        assert_eq!(payment.comment, USER_SUBMITTED_PAYMENT_COMMENT);
        assert_eq!(payment.reference, "reference");
        assert_eq!(payment.last_changed_by_user, user.user_id);
        assert_eq!(payment.proof_of_payment_path, "path");
    }
}
