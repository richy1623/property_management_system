tonic::include_proto!("property_management_system.payment_management");

use chrono::{Datelike, NaiveDate};

use crate::db::model::payment::Payment as PaymentModel;

impl From<NaiveDate> for Date {
    fn from(d: NaiveDate) -> Self {
        Date {
            day: d.day().try_into().unwrap(),
            year: d.year_ce().1.try_into().unwrap(),
            month: <u32 as TryInto<i32>>::try_into(d.month0()).unwrap() + 1,
        }
    }
}
impl From<Date> for NaiveDate {
    fn from(d: Date) -> Self {
        NaiveDate::from_ymd_opt(d.year, d.month as u32, d.day as u32)
            .expect("Invalid date from protobuf")
    }
}

impl From<&PaymentModel> for Payment {
    fn from(payment: &PaymentModel) -> Self {
        Payment {
            payment_id: payment.payment_id,
            ownership_id: payment.ownership_id,
            date: Some(payment.date.into()),
            amount: payment.amount,
            state: payment.state.clone().into(),
            comment: payment.comment.clone(),
            reference: payment.reference.clone(),
            last_changed_by_user: payment.last_changed_by_user,
        }
    }
}
