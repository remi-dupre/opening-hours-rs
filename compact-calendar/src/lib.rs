#![doc = include_str!("../README.md")]

use std::{fmt, io};

use chrono::{Datelike, NaiveDate};

/// A compact representation of included days in a range of years, using u32-based bit arrays.
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct CompactCalendar {
    first_year: i32,
    calendar: Vec<CompactYear>,
}

impl CompactCalendar {
    /// Create a new year that does not include any day but has the capacity to add any new day in
    /// the range of `[first_year, last_year]`.
    ///
    /// ```
    /// use compact_calendar::CompactCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let day1 = NaiveDate::from_ymd_opt(2013, 11, 3).unwrap();
    /// let day2 = NaiveDate::from_ymd_opt(2022, 3, 5).unwrap();
    /// let day3 = NaiveDate::from_ymd_opt(2055, 7, 5).unwrap();
    ///
    /// let mut cal = CompactCalendar::new(2020, 2050);
    /// assert!(!cal.insert(day1));
    /// assert!(cal.insert(day2));
    /// assert!(!cal.insert(day3));
    /// assert_eq!(cal.count(), 1);
    /// ```
    pub fn new(first_year: i32, last_year: i32) -> Self {
        assert!(
            first_year <= last_year,
            "use CompactCalendar::empty() if you need to init a calendar with no capacity",
        );

        let length: usize = (last_year - first_year + 1)
            .try_into()
            .expect("compact calendar is too large to be initialized");

        Self {
            first_year,
            calendar: vec![CompactYear::new(); length],
        }
    }

    /// Create a new calendar year that does not include any day and does not have the capacity to
    /// add any.
    ///
    /// ```
    /// use compact_calendar::CompactCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let mut cal = CompactCalendar::empty();
    /// assert!(!cal.insert(NaiveDate::from_ymd_opt(2013, 11, 3).unwrap()));
    /// assert_eq!(cal.count(), 0);
    /// ```
    pub const fn empty() -> Self {
        Self { first_year: 0, calendar: Vec::new() }
    }

    /// Get a reference to the year containing give date.
    ///
    /// ```
    /// use compact_calendar::CompactCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let day1 = NaiveDate::from_ymd_opt(2013, 11, 3).unwrap();
    /// let day2 = NaiveDate::from_ymd_opt(2055, 3, 5).unwrap();
    ///
    /// let mut cal = CompactCalendar::new(2000, 2050);
    /// cal.insert(day1);
    ///
    /// assert!(cal.year_for(day1).unwrap().contains(11, 3));
    /// assert!(cal.year_for(day2).is_none());
    /// ```
    pub fn year_for(&self, date: NaiveDate) -> Option<&CompactYear> {
        let year0 = usize::try_from(date.year() - self.first_year).ok()?;
        self.calendar.get(year0)
    }

    /// Get a mutable reference to the year containing give date.
    ///
    /// ```
    /// use compact_calendar::CompactCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let day1 = NaiveDate::from_ymd_opt(2013, 11, 3).unwrap();
    /// let day2 = NaiveDate::from_ymd_opt(2022, 3, 5).unwrap();
    /// let day3 = NaiveDate::from_ymd_opt(2055, 7, 5).unwrap();
    ///
    /// let mut cal = CompactCalendar::new(2000, 2050);
    /// assert!(cal.year_for_mut(day3).is_none());
    ///
    /// cal.insert(day1);
    /// assert!(cal.year_for_mut(day1).unwrap().contains(11, 3));
    ///
    /// cal.year_for_mut(day2).unwrap().insert(3, 5);
    /// assert!(cal.contains(day2));
    /// ```
    pub fn year_for_mut(&mut self, date: NaiveDate) -> Option<&mut CompactYear> {
        let year0 = usize::try_from(date.year() - self.first_year).ok()?;
        self.calendar.get_mut(year0)
    }

    /// Include a day in this calendar. Return false if the given day was not inserted because it
    /// does not belong to the year range of this calendar.
    ///
    /// ```
    /// use compact_calendar::CompactCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let day1 = NaiveDate::from_ymd_opt(2013, 11, 3).unwrap();
    /// let day2 = NaiveDate::from_ymd_opt(2022, 3, 5).unwrap();
    /// let day3 = NaiveDate::from_ymd_opt(2055, 9, 7).unwrap();
    ///
    /// let mut cal = CompactCalendar::new(2000, 2050);
    /// assert!(cal.insert(day1));
    /// assert!(cal.insert(day1));
    /// assert!(cal.insert(day2));
    /// assert_eq!(cal.count(), 2);
    ///
    /// assert!(!cal.insert(day3));
    /// assert_eq!(cal.count(), 2);
    /// ```
    pub fn insert(&mut self, date: NaiveDate) -> bool {
        if let Some(year) = self.year_for_mut(date) {
            year.insert(date.month(), date.day());
            true
        } else {
            false
        }
    }

    /// Check if this calendar includes the given day.
    ///
    /// ```
    /// use compact_calendar::CompactCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let day1 = NaiveDate::from_ymd_opt(2013, 11, 3).unwrap();
    /// let day2 = NaiveDate::from_ymd_opt(2022, 3, 5).unwrap();
    /// let day3 = NaiveDate::from_ymd_opt(2022, 8, 12).unwrap();
    ///
    /// let mut cal = CompactCalendar::new(2000, 2050);
    /// cal.insert(day1);
    /// cal.insert(day2);
    ///
    /// assert!(cal.contains(day1));
    /// assert!(cal.contains(day2));
    /// assert!(!cal.contains(day3));
    /// ```
    pub fn contains(&self, date: NaiveDate) -> bool {
        if let Some(year) = self.year_for(date) {
            year.contains(date.month(), date.day())
        } else {
            false
        }
    }

    /// Iterate over the days included in this calendar.
    ///
    /// ```
    /// use compact_calendar::CompactCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let day1 = NaiveDate::from_ymd_opt(2013, 11, 3).unwrap();
    /// let day2 = NaiveDate::from_ymd_opt(2022, 3, 5).unwrap();
    /// let day3 = NaiveDate::from_ymd_opt(2022, 8, 12).unwrap();
    ///
    /// let mut cal = CompactCalendar::new(2000, 2050);
    /// cal.insert(day3);
    /// cal.insert(day1);
    /// cal.insert(day2);
    ///
    /// let days: Vec<_> = cal.iter().collect();
    /// assert_eq!(days, [day1, day2, day3])
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = NaiveDate> + '_ {
        (self.first_year..)
            .zip(self.calendar.iter())
            .flat_map(|(year_i, year)| {
                year.iter().map(move |(month, day)| {
                    NaiveDate::from_ymd_opt(year_i, month, day)
                        .expect("invalid date loaded from calendar")
                })
            })
    }

    /// Get the first day included in this calendar that follows the input day, if such a day
    /// exists.
    ///
    /// ```
    /// use compact_calendar::CompactCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let day0 = NaiveDate::from_ymd_opt(2010, 1, 1).unwrap();
    /// let day1 = NaiveDate::from_ymd_opt(2013, 11, 3).unwrap();
    /// let day2 = NaiveDate::from_ymd_opt(2022, 3, 5).unwrap();
    /// let day3 = NaiveDate::from_ymd_opt(2022, 8, 12).unwrap();
    ///
    /// let mut cal = CompactCalendar::new(2000, 2050);
    /// cal.insert(day1);
    /// cal.insert(day2);
    /// cal.insert(day3);
    ///
    /// assert_eq!(cal.first_after(day0), Some(day1));
    /// assert_eq!(cal.first_after(day2), Some(day3));
    /// assert_eq!(cal.first_after(day3), None);
    /// ```
    pub fn first_after(&self, date: NaiveDate) -> Option<NaiveDate> {
        if let Some(year) = self.year_for(date) {
            let from_first_year = year
                .first_after(date.month(), date.day())
                .map(|(month, day)| {
                    NaiveDate::from_ymd_opt(date.year(), month, day)
                        .expect("invalid date loaded from calendar")
                });

            from_first_year.or_else(|| {
                let year0 = usize::try_from(date.year() - self.first_year).ok()?;

                (date.year() + 1..)
                    .zip(&self.calendar[year0 + 1..])
                    .find_map(|(year_i, year)| {
                        let (month, day) = year.first()?;
                        Some(
                            NaiveDate::from_ymd_opt(year_i, month, day)
                                .expect("invalid date loaded from calendar"),
                        )
                    })
            })
        } else {
            None
        }
    }

    /// Count number of days included for this calendar.
    ///
    /// ```
    /// use compact_calendar::CompactCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let mut cal = CompactCalendar::new(2000, 2050);
    /// cal.insert(NaiveDate::from_ymd(2022, 8, 12));
    /// cal.insert(NaiveDate::from_ymd(2022, 3, 5));
    /// assert_eq!(cal.count(), 2);
    /// ```
    pub fn count(&self) -> u32 {
        self.calendar.iter().map(CompactYear::count).sum()
    }

    /// Serialize this calendar into a writer.
    ///
    /// ```
    /// use compact_calendar::CompactCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let mut cal = CompactCalendar::new(2000, 2050);
    ///
    /// let mut buf1 = Vec::new();
    /// cal.insert(NaiveDate::from_ymd(2022, 8, 12));
    /// cal.serialize(&mut buf1).unwrap();
    ///
    /// let mut buf2 = Vec::new();
    /// cal.insert(NaiveDate::from_ymd(2022, 3, 5));
    /// cal.serialize(&mut buf2).unwrap();
    ///
    /// assert_ne!(buf1, buf2);
    /// ```
    pub fn serialize(&self, mut writer: impl io::Write) -> io::Result<()> {
        writer.write_all(&self.first_year.to_ne_bytes())?;
        writer.write_all(&self.calendar.len().to_ne_bytes())?;

        for year in &self.calendar {
            year.serialize(&mut writer)?;
        }

        Ok(())
    }

    /// Deserialize a calendar from a reader.
    ///
    /// ```
    /// use compact_calendar::CompactCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let mut cal1 = CompactCalendar::new(2000, 2050);
    /// cal1.insert(NaiveDate::from_ymd(2022, 8, 12));
    /// cal1.insert(NaiveDate::from_ymd(2022, 3, 5));
    ///
    /// let mut buf = Vec::new();
    /// cal1.serialize(&mut buf).unwrap();
    ///
    /// let cal2 = CompactCalendar::deserialize(buf.as_slice()).unwrap();
    /// assert_eq!(cal1, cal2);
    /// ```
    pub fn deserialize(mut reader: impl io::Read) -> io::Result<Self> {
        let first_year = {
            let mut buf = [0; std::mem::size_of::<i32>()];
            reader.read_exact(&mut buf)?;
            i32::from_ne_bytes(buf)
        };

        let length = {
            let mut buf = [0; std::mem::size_of::<usize>()];
            reader.read_exact(&mut buf)?;
            usize::from_ne_bytes(buf)
        };

        let calendar = (0..length)
            .map(|_| CompactYear::deserialize(&mut reader))
            .collect::<Result<_, _>>()?;

        Ok(Self { first_year, calendar })
    }
}

impl fmt::Debug for CompactCalendar {
    /// ```
    /// use compact_calendar::CompactCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let mut cal = CompactCalendar::new(2000, 2050);
    /// cal.insert(NaiveDate::from_ymd(2022, 8, 12));
    /// cal.insert(NaiveDate::from_ymd(2022, 3, 5));
    ///
    /// assert_eq!(
    ///     format!("{cal:?}"),
    ///     "CompactCalendar { first_year: 2000, last_year: 2050, calendar: {2022-03-05, 2022-08-12} }",
    /// );
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugCalendar<'a>(&'a CompactCalendar);

        impl<'a> fmt::Debug for DebugCalendar<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_set().entries(self.0.iter()).finish()
            }
        }

        let last_year = self.first_year + self.calendar.len() as i32 - 1;

        f.debug_struct("CompactCalendar")
            .field("first_year", &self.first_year)
            .field("last_year", &last_year)
            .field("calendar", &DebugCalendar(self))
            .finish()
    }
}

/// A compact representation of included days for a year, using a collection of u32-based bit
/// array.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct CompactYear([CompactMonth; 12]);

impl CompactYear {
    /// Create a new year that does not include any day.
    ///
    /// ```
    /// use compact_calendar::CompactYear;
    ///
    /// let year = CompactYear::new();
    /// assert_eq!(year.count(), 0);
    /// ```
    pub const fn new() -> Self {
        Self([CompactMonth::new(); 12])
    }

    /// Include a day in this year.
    ///
    /// ```
    /// use compact_calendar::CompactYear;
    ///
    /// let mut year = CompactYear::new();
    /// year.insert(11, 3);
    /// year.insert(11, 3);
    /// year.insert(1, 25);
    /// assert_eq!(year.count(), 2);
    /// ```
    pub fn insert(&mut self, month: u32, day: u32) {
        assert!((1..=12).contains(&month));
        assert!((1..=31).contains(&day));
        self.0[(month - 1) as usize].insert(day)
    }

    /// Check if this year includes the given day.
    ///
    /// ```
    /// use compact_calendar::CompactYear;
    ///
    /// let mut year = CompactYear::new();
    /// year.insert(3, 1);
    /// year.insert(9, 5);
    ///
    /// assert!(year.contains(3, 1));
    /// assert!(year.contains(9, 5));
    /// assert!(!year.contains(7, 14));
    /// ```
    pub fn contains(&self, month: u32, day: u32) -> bool {
        assert!((1..=12).contains(&month));
        assert!((1..=31).contains(&day));
        self.0[(month - 1) as usize].contains(day)
    }

    /// Iterate over the days included in this year.
    ///
    /// ```
    /// use compact_calendar::CompactYear;
    ///
    /// let mut year = CompactYear::new();
    /// year.insert(9, 5);
    /// year.insert(3, 1);
    ///
    /// let days: Vec<_> = year.iter().collect();
    /// assert_eq!(days, [(3, 1), (9, 5)])
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        ((1..=12).zip(&self.0))
            .flat_map(|(month_i, month)| month.iter().map(move |day| (month_i, day)))
    }

    /// Get the first day included in this year if it is not empty.
    ///
    /// ```
    /// use compact_calendar::CompactYear;
    ///
    /// let mut year = CompactYear::new();
    /// assert_eq!(year.first(), None);
    ///
    /// year.insert(12, 31);
    /// assert_eq!(year.first(), Some((12, 31)));
    ///
    /// year.insert(5, 8);
    /// assert_eq!(year.first(), Some((5, 8)));
    /// ```
    pub fn first(&self) -> Option<(u32, u32)> {
        self.0.iter().enumerate().find_map(|(i, month)| {
            let res_month = (i + 1) as u32;
            Some((res_month, month.first()?))
        })
    }

    /// Get the first day included in this year that follows the input day, if such a day exists.
    ///
    /// ```
    /// use compact_calendar::CompactYear;
    ///
    /// let mut year = CompactYear::new();
    /// year.insert(3, 15);
    /// year.insert(10, 9);
    /// year.insert(2, 7);
    ///
    /// assert_eq!(year.first_after(2, 2), Some((2, 7)));
    /// assert_eq!(year.first_after(2, 7), Some((3, 15)));
    /// assert_eq!(year.first_after(11, 1), None);
    /// ```
    pub fn first_after(&self, month: u32, day: u32) -> Option<(u32, u32)> {
        assert!((1..=12).contains(&month));
        assert!((1..=31).contains(&day));
        let month0: usize = (month - 1) as usize;

        if let Some(res) = self.0[month0].first_after(day) {
            Some((month, res))
        } else {
            self.0[month0 + 1..]
                .iter()
                .enumerate()
                .find_map(|(i, month)| {
                    let res_month = (i + month0 + 2) as u32;
                    Some((res_month, month.first()?))
                })
        }
    }

    /// Count number of days included for this year.
    ///
    /// ```
    /// use compact_calendar::CompactYear;
    ///
    /// let mut year = CompactYear::new();
    /// year.insert(11, 3);
    /// year.insert(4, 28);
    /// assert_eq!(year.count(), 2);
    /// ```
    pub fn count(&self) -> u32 {
        self.0.iter().copied().map(CompactMonth::count).sum()
    }

    /// Serialize this year into a writer.
    ///
    /// ```
    /// use compact_calendar::CompactYear;
    ///
    /// let mut year = CompactYear::new();
    ///
    /// let mut buf1 = Vec::new();
    /// year.insert(11, 3);
    /// year.serialize(&mut buf1).unwrap();
    ///
    /// let mut buf2 = Vec::new();
    /// year.insert(4, 28);
    /// year.serialize(&mut buf2).unwrap();
    ///
    /// assert_ne!(buf1, buf2);
    /// ```
    pub fn serialize(&self, mut writer: impl io::Write) -> io::Result<()> {
        for month in self.0 {
            month.serialize(&mut writer)?;
        }

        Ok(())
    }

    /// Deserialize a year from a reader.
    ///
    /// ```
    /// use compact_calendar::CompactYear;
    ///
    /// let mut year1 = CompactYear::new();
    /// year1.insert(11, 3);
    /// year1.insert(4, 28);
    ///
    /// let mut buf = Vec::new();
    /// year1.serialize(&mut buf).unwrap();
    ///
    /// let year2 = CompactYear::deserialize(buf.as_slice()).unwrap();
    /// assert_eq!(year1, year2);
    /// ```
    pub fn deserialize(mut reader: impl io::Read) -> io::Result<Self> {
        // NOTE: could use `try_from_fn` when stabilized:
        //       https://doc.rust-lang.org/std/array/fn.try_from_fn.html
        let mut res = Self::new();

        for month in &mut res.0 {
            *month = CompactMonth::deserialize(&mut reader)?;
        }

        Ok(res)
    }
}

impl fmt::Debug for CompactYear {
    /// ```
    /// use compact_calendar::CompactYear;
    ///
    /// let mut year = CompactYear::new();
    /// year.insert(11, 3);
    /// year.insert(4, 28);
    /// assert_eq!(format!("{year:?}"), "{04-28, 11-03}");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugMonthDay {
            month: u32,
            day: u32,
        }

        impl fmt::Debug for DebugMonthDay {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:02}-{:02}", self.month, self.day)
            }
        }

        f.debug_set()
            .entries(self.iter().map(|(month, day)| DebugMonthDay { month, day }))
            .finish()
    }
}

/// A compact representation of included days for a month, using a u32-based bit array.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct CompactMonth(u32);

impl CompactMonth {
    /// Create a new month that does not include any day.
    ///
    /// ```
    /// use compact_calendar::CompactMonth;
    ///
    /// let month = CompactMonth::new();
    /// assert_eq!(month.count(), 0);
    /// ```
    pub const fn new() -> Self {
        Self(0)
    }

    /// Include a day in this month.
    ///
    /// ```
    /// use compact_calendar::CompactMonth;
    ///
    /// let mut month = CompactMonth::new();
    /// month.insert(2);
    /// month.insert(2);
    /// month.insert(19);
    /// assert_eq!(month.count(), 2);
    /// ```
    pub fn insert(&mut self, day: u32) {
        assert!((1..=31).contains(&day));
        self.0 |= 1 << (day - 1)
    }

    /// Check if this month includes the given day.
    ///
    /// ```
    /// use compact_calendar::CompactMonth;
    ///
    /// let mut month = CompactMonth::new();
    /// month.insert(1);
    /// month.insert(18);
    ///
    /// assert!(month.contains(1));
    /// assert!(month.contains(18));
    /// assert!(!month.contains(5));
    /// ```
    pub fn contains(self, day: u32) -> bool {
        assert!((1..=31).contains(&day));
        self.0 & (1 << (day - 1)) != 0
    }

    /// Iterate over the days included in this month.
    ///
    /// ```
    /// use compact_calendar::CompactMonth;
    ///
    /// let mut month = CompactMonth::new();
    /// month.insert(18);
    /// month.insert(1);
    ///
    /// let days: Vec<u32> = month.iter().collect();
    /// assert_eq!(days, [1, 18])
    /// ```
    pub fn iter(self) -> impl Iterator<Item = u32> {
        let mut val = self.0;

        std::iter::from_fn(move || {
            if val != 0 {
                let day0 = val.trailing_zeros();
                val ^= 1 << day0;
                Some(day0 + 1)
            } else {
                None
            }
        })
    }

    /// Get the first day included in this month if it is not empty.
    ///
    /// ```
    /// use compact_calendar::CompactMonth;
    ///
    /// let mut month = CompactMonth::new();
    /// assert_eq!(month.first(), None);
    ///
    /// month.insert(31);
    /// assert_eq!(month.first(), Some(31));
    ///
    /// month.insert(8);
    /// assert_eq!(month.first(), Some(8));
    /// ```
    pub fn first(self) -> Option<u32> {
        if self.0 == 0 {
            None
        } else {
            Some(self.0.trailing_zeros() + 1)
        }
    }

    /// Get the first day included in this month that follows the input day, if such a day exists.
    ///
    /// ```
    /// use compact_calendar::CompactMonth;
    ///
    /// let mut month = CompactMonth::new();
    /// month.insert(4);
    /// month.insert(17);
    ///
    /// assert_eq!(month.first_after(2), Some(4));
    /// assert_eq!(month.first_after(4), Some(17));
    /// assert_eq!(month.first_after(17), None);
    /// ```
    pub fn first_after(self, day: u32) -> Option<u32> {
        assert!((1..=31).contains(&day));
        let shifted = self.0 >> day;

        if shifted == 0 {
            None
        } else {
            Some(day + shifted.trailing_zeros() + 1)
        }
    }

    /// Count number of days included for this month.
    ///
    /// ```
    /// use compact_calendar::CompactMonth;
    ///
    /// let mut month = CompactMonth::new();
    /// month.insert(26);
    /// month.insert(3);
    /// assert_eq!(month.count(), 2);
    /// ```
    pub fn count(self) -> u32 {
        self.0.count_ones()
    }

    /// Serialize this month into a writer.
    ///
    /// ```
    /// use compact_calendar::CompactMonth;
    ///
    /// let mut month = CompactMonth::new();
    ///
    /// let mut buf1 = Vec::new();
    /// month.insert(31);
    /// month.serialize(&mut buf1).unwrap();
    ///
    /// let mut buf2 = Vec::new();
    /// month.insert(1);
    /// month.serialize(&mut buf2).unwrap();
    ///
    /// assert_ne!(buf1, buf2);
    /// ```
    pub fn serialize(self, mut writer: impl io::Write) -> io::Result<()> {
        writer.write_all(&self.0.to_ne_bytes())
    }

    /// Deserialize a month from a reader.
    ///
    /// ```
    /// use compact_calendar::CompactMonth;
    ///
    /// let mut month1 = CompactMonth::new();
    /// month1.insert(30);
    /// month1.insert(2);
    ///
    /// let mut buf = Vec::new();
    /// month1.serialize(&mut buf).unwrap();
    ///
    /// let month2 = CompactMonth::deserialize(buf.as_slice()).unwrap();
    /// assert_eq!(month1, month2);
    /// ```
    pub fn deserialize(mut reader: impl io::Read) -> io::Result<Self> {
        let mut buf = [0; std::mem::size_of::<u32>()];
        reader.read_exact(&mut buf)?;
        Ok(Self(u32::from_ne_bytes(buf)))
    }
}

impl fmt::Debug for CompactMonth {
    /// ```
    /// use compact_calendar::CompactMonth;
    ///
    /// let mut month = CompactMonth::new();
    /// month.insert(26);
    /// month.insert(3);
    /// assert_eq!(format!("{month:?}"), "{03, 26}");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugDay(u32);

        impl fmt::Debug for DebugDay {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:02}", self.0)
            }
        }

        f.debug_set().entries(self.iter().map(DebugDay)).finish()
    }
}
