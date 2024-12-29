use chrono::{Datelike, Months, NaiveDate};

pub(crate) fn count_days_in_month(date: NaiveDate) -> u8 {
    let Some(date_next_month) = date.checked_add_months(Months::new(1)) else {
        // December of last supported year
        return 31;
    };

    let first_this_month = date
        .with_day(1)
        .expect("first of the month should always exist");

    let first_next_month = date_next_month
        .with_day(1)
        .expect("first of the month should always exist");

    (first_next_month - first_this_month)
        .num_days()
        .try_into()
        .expect("time not monotonic while comparing dates")
}
