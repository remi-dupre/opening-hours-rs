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

/// Find Easter date for given year using.
///
/// See https://en.wikipedia.org/wiki/Date_of_Easter#Anonymous_Gregorian_algorithm
pub(crate) fn easter(year: i32) -> Option<NaiveDate> {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let n = (h + l - 7 * m + 114) / 31;
    let o = (h + l - 7 * m + 114) % 31;

    NaiveDate::from_ymd_opt(
        year,
        n.try_into().expect("month cannot be negative"),
        (o + 1).try_into().expect("day cannot be negative"),
    )
}

#[cfg(test)]
mod test {
    use super::easter;
    use crate::date;

    #[test]
    fn test_easter() {
        assert_eq!(easter(i32::MIN), None);
        assert_eq!(easter(i32::MAX), None);
        assert_eq!(easter(1901), Some(date!("1901-04-07")));
        assert_eq!(easter(1961), Some(date!("1961-04-02")));
        assert_eq!(easter(2024), Some(date!("2024-03-31")));
        assert_eq!(easter(2025), Some(date!("2025-04-20")));
        assert_eq!(easter(2050), Some(date!("2050-04-10")));
        assert_eq!(easter(2106), Some(date!("2106-04-18")));
        assert_eq!(easter(2200), Some(date!("2200-04-06")));
        assert_eq!(easter(3000), Some(date!("3000-04-13")));
    }
}
