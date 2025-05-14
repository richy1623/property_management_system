tonic::include_proto!("property_management_system.payment_management");

use chrono::NaiveDate;

impl From<Date> for NaiveDate {
    fn from(d: Date) -> Self {
        NaiveDate::from_ymd_opt(d.year, d.month as u32, d.day as u32)
            .expect("Invalid date from protobuf")
    }
}
