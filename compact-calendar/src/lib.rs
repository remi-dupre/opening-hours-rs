#![doc = include_str!("../README.md")]

use std::collections::VecDeque;
use std::{fmt, io};

use chrono::{Datelike, NaiveDate};

/// A compact representation of included days in a range of years, using u32-based bit arrays.
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct CompactCalendar {
    first_year: i32,
    calendar: VecDeque<CompactYear>,
}

impl CompactCalendar {
    /// Get a reference to the year containing give date.
    ///
    /// ```
    /// use compact_calendar::CompactCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let day1 = NaiveDate::from_ymd_opt(2013, 11, 3).unwrap();
    /// let day2 = NaiveDate::from_ymd_opt(2055, 3, 5).unwrap();
    ///
    /// let mut cal = CompactCalendar::default();
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
    /// let mut cal = CompactCalendar::default();
    /// assert!(cal.year_for_mut(day3).is_none());
    ///
    /// cal.insert(day1);
    /// assert!(cal.year_for_mut(day1).unwrap().contains(11, 3));
    /// ```
    pub fn year_for_mut(&mut self, date: NaiveDate) -> Option<&mut CompactYear> {
        let year0 = usize::try_from(date.year() - self.first_year).ok()?;
        self.calendar.get_mut(year0)
    }

    /// Include a day in this calendar. Return `false` if the day already
    /// belonged to the calendar.
    ///
    /// ```
    /// use compact_calendar::CompactCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let day1 = NaiveDate::from_ymd_opt(2013, 11, 3).unwrap();
    /// let day2 = NaiveDate::from_ymd_opt(2022, 3, 5).unwrap();
    /// let day3 = NaiveDate::from_ymd_opt(2055, 9, 7).unwrap();
    ///
    /// let mut cal = CompactCalendar::default();
    /// assert!(cal.insert(day1));
    /// assert!(cal.insert(day2));
    /// assert!(cal.insert(day3));
    /// assert!(!cal.insert(day1));
    /// assert_eq!(cal.count(), 3);
    /// ```
    pub fn insert(&mut self, date: NaiveDate) -> bool {
        let year = {
            if let Some(year) = self.year_for_mut(date) {
                year
            } else if self.calendar.is_empty() {
                self.first_year = date.year();
                self.calendar.push_back(CompactYear::default());
                self.calendar.back_mut().unwrap() // just pushed
            } else if date.year() < self.first_year {
                for _ in date.year()..self.first_year {
                    eprintln!("Push front");
                    self.calendar.push_front(CompactYear::default());
                }

                self.first_year = date.year();
                self.calendar.front_mut().unwrap() // just pushed
            } else {
                let last_year = self.first_year
                    + i32::try_from(self.calendar.len()).expect("calendar is too large")
                    - 1;

                for _ in last_year..date.year() {
                    self.calendar.push_back(CompactYear::default());
                }

                self.calendar.back_mut().unwrap() // just pushed
            }
        };

        year.insert(date.month(), date.day())
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
    /// let mut cal = CompactCalendar::default();
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
    /// let mut cal = CompactCalendar::default();
    /// cal.insert(day3);
    /// cal.insert(day1);
    /// cal.insert(day2);
    ///
    /// let days: Vec<_> = cal.iter().collect();
    /// assert_eq!(days, [day1, day2, day3])
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = NaiveDate> + Send + Sync + '_ {
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
    /// let mut cal = CompactCalendar::default();
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
                    .zip(self.calendar.iter().skip(year0 + 1))
                    .find_map(|(year_i, year)| {
                        let (month, day) = year.first()?;
                        Some(
                            NaiveDate::from_ymd_opt(year_i, month, day)
                                .expect("invalid date loaded from calendar"),
                        )
                    })
            })
        } else {
            self.iter().next()
        }
    }

    /// Count number of days included for this calendar.
    ///
    /// ```
    /// use compact_calendar::CompactCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let mut cal = CompactCalendar::default();
    /// cal.insert(NaiveDate::from_ymd_opt(2022, 8, 12).unwrap());
    /// cal.insert(NaiveDate::from_ymd_opt(2022, 3, 5).unwrap());
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
    /// let mut cal = CompactCalendar::default();
    ///
    /// let mut buf1 = Vec::new();
    /// cal.insert(NaiveDate::from_ymd_opt(2022, 8, 12).unwrap());
    /// cal.serialize(&mut buf1).unwrap();
    ///
    /// let mut buf2 = Vec::new();
    /// cal.insert(NaiveDate::from_ymd_opt(2022, 3, 5).unwrap());
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
    /// let mut cal1 = CompactCalendar::default();
    /// cal1.insert(NaiveDate::from_ymd_opt(2022, 8, 12).unwrap());
    /// cal1.insert(NaiveDate::from_ymd_opt(2022, 3, 5).unwrap());
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

impl Default for CompactCalendar {
    /// Create a new year that does not include any day.
    ///
    /// ```
    /// use compact_calendar::CompactCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let mut cal = CompactCalendar::default();
    /// assert_eq!(cal.count(), 0);
    /// ```
    fn default() -> Self {
        Self { first_year: 0, calendar: VecDeque::default() }
    }
}

impl FromIterator<NaiveDate> for CompactCalendar {
    /// Create a calendar from a list of dates.
    ///
    /// ```
    /// use compact_calendar::CompactCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let dates = [
    ///     NaiveDate::from_ymd_opt(2013, 11, 3).unwrap(),
    ///     NaiveDate::from_ymd_opt(2022, 3, 5).unwrap(),
    ///     NaiveDate::from_ymd_opt(2055, 7, 5).unwrap(),
    ///     NaiveDate::from_ymd_opt(2013, 11, 3).unwrap(),
    /// ];
    ///
    /// let cal: CompactCalendar = dates.iter().copied().collect();
    /// assert_eq!(cal.count(), 3);
    /// assert!(cal.contains(dates[0]));
    /// ```
    fn from_iter<T: IntoIterator<Item = NaiveDate>>(iter: T) -> Self {
        let mut calendar = CompactCalendar::default();

        for date in iter {
            calendar.insert(date);
        }

        calendar.calendar.make_contiguous();
        calendar.calendar.shrink_to_fit();
        calendar
    }
}

impl fmt::Debug for CompactCalendar {
    /// ```
    /// use compact_calendar::CompactCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let mut cal = CompactCalendar::default();
    /// cal.insert(NaiveDate::from_ymd_opt(2022, 8, 12).unwrap());
    /// cal.insert(NaiveDate::from_ymd_opt(2022, 3, 5).unwrap());
    /// assert_eq!(format!("{cal:?}"), "CompactCalendar({2022-03-05, 2022-08-12})");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugCalendar<'a>(&'a CompactCalendar);

        impl fmt::Debug for DebugCalendar<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_set().entries(self.0.iter()).finish()
            }
        }

        f.debug_tuple("CompactCalendar")
            .field(&DebugCalendar(self))
            .finish()
    }
}

/// A compact representation of included days for a year, using a collection of u32-based bit
/// array.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct CompactYear([CompactMonth; 12]);

impl CompactYear {
    /// Include a day in this year. Return `false` if the day was already
    /// included.
    ///
    /// ```
    /// use compact_calendar::CompactYear;
    ///
    /// let mut year = CompactYear::default();
    /// year.insert(11, 3);
    /// year.insert(11, 3);
    /// year.insert(1, 25);
    /// assert_eq!(year.count(), 2);
    /// ```
    pub fn insert(&mut self, month: u32, day: u32) -> bool {
        assert!((1..=12).contains(&month));
        assert!((1..=31).contains(&day));
        self.0[(month - 1) as usize].insert(day)
    }

    /// Check if this year includes the given day.
    ///
    /// ```
    /// use compact_calendar::CompactYear;
    ///
    /// let mut year = CompactYear::default();
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
    /// let mut year = CompactYear::default();
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
    /// let mut year = CompactYear::default();
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
    /// let mut year = CompactYear::default();
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
    /// let mut year = CompactYear::default();
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
    /// let mut year = CompactYear::default();
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
    /// let mut year1 = CompactYear::default();
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
        let mut res = Self::default();

        for month in &mut res.0 {
            *month = CompactMonth::deserialize(&mut reader)?;
        }

        Ok(res)
    }
}

impl Default for CompactYear {
    /// Create a new year that does not include any day.
    ///
    /// ```
    /// use compact_calendar::CompactYear;
    ///
    /// let year = CompactYear::default();
    /// assert_eq!(year.count(), 0);
    /// ```
    fn default() -> Self {
        Self([CompactMonth::default(); 12])
    }
}

impl fmt::Debug for CompactYear {
    /// ```
    /// use compact_calendar::CompactYear;
    ///
    /// let mut year = CompactYear::default();
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
    /// Include a day in this month. Return `false` if it was already included.
    ///
    /// ```
    /// use compact_calendar::CompactMonth;
    ///
    /// let mut month = CompactMonth::default();
    /// month.insert(2);
    /// month.insert(2);
    /// month.insert(19);
    /// assert_eq!(month.count(), 2);
    /// ```
    pub fn insert(&mut self, day: u32) -> bool {
        assert!((1..=31).contains(&day));

        if self.contains(day) {
            false
        } else {
            self.0 |= 1 << (day - 1);
            true
        }
    }

    /// Check if this month includes the given day.
    ///
    /// ```
    /// use compact_calendar::CompactMonth;
    ///
    /// let mut month = CompactMonth::default();
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
    /// let mut month = CompactMonth::default();
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
    /// let mut month = CompactMonth::default();
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
    /// let mut month = CompactMonth::default();
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
    /// let mut month = CompactMonth::default();
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
    /// let mut month = CompactMonth::default();
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
    /// let mut month1 = CompactMonth::default();
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

impl Default for CompactMonth {
    /// Create a new month that does not include any day.
    ///
    /// ```
    /// use compact_calendar::CompactMonth;
    ///
    /// let month = CompactMonth::default();
    /// assert_eq!(month.count(), 0);
    /// ```
    fn default() -> Self {
        Self(0)
    }
}

impl fmt::Debug for CompactMonth {
    /// ```
    /// use compact_calendar::CompactMonth;
    ///
    /// let mut month = CompactMonth::default();
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
