//! TODO: doc

use std::io;

use chrono::{Datelike, NaiveDate};

/// TODO: doc
pub struct CompactCalendar {
    first_year: i32,
    calendar: Vec<CompactYear>,
}

impl CompactCalendar {
    /// TODO: doc
    pub fn new(first_year: i32, last_year: i32) -> Self {
        assert!(first_year <= last_year);

        let length: usize = (last_year - first_year + 1)
            .try_into()
            .expect("compact calendar is too large to be initialized");

        Self {
            first_year,
            calendar: vec![CompactYear::new(); length],
        }
    }

    /// TODO: doc
    pub const fn empty() -> Self {
        Self { first_year: 0, calendar: Vec::new() }
    }

    /// TODO: doc
    pub fn year_for(&self, date: NaiveDate) -> Option<&CompactYear> {
        let year0 = {
            if let Ok(x) = usize::try_from(date.year() - self.first_year) {
                x
            } else {
                return None;
            }
        };

        self.calendar.get(year0)
    }

    /// TODO: doc
    pub fn year_for_mut(&mut self, date: NaiveDate) -> Option<&mut CompactYear> {
        let year0 = {
            if let Ok(x) = usize::try_from(date.year() - self.first_year) {
                x
            } else {
                return None;
            }
        };

        self.calendar.get_mut(year0)
    }

    /// TODO: doc
    pub fn insert(&mut self, date: NaiveDate) -> bool {
        if let Some(year) = self.year_for_mut(date) {
            year.insert(date.month(), date.day());
            true
        } else {
            false
        }
    }

    /// TODO: doc
    pub fn contains(&self, date: NaiveDate) -> bool {
        if let Some(year) = self.year_for(date) {
            year.contains(date.month(), date.day())
        } else {
            false
        }
    }

    /// TODO: doc
    pub fn first_after(&self, date: NaiveDate) -> Option<NaiveDate> {
        if let Some(year) = self.year_for(date) {
            let from_first_year = year
                .first_after(date.month(), date.day())
                .map(|(month, day)| NaiveDate::from_ymd(date.year(), month, day));

            from_first_year.or_else(|| {
                let year0 = {
                    if let Ok(x) = usize::try_from(date.year() - self.first_year) {
                        x
                    } else {
                        return None;
                    }
                };

                self.calendar[year0 + 1..]
                    .iter()
                    .enumerate()
                    .find_map(|(i, year)| {
                        year.first().map(|(month, day)| {
                            let res_year = self.first_year + (year0 + i + 1) as i32;
                            NaiveDate::from_ymd(res_year, month, day)
                        })
                    })
            })
        } else {
            None
        }
    }

    /// TODO: doc
    pub fn count(&self) -> u32 {
        self.calendar.iter().map(CompactYear::count).sum()
    }

    /// TODO: doc
    pub fn serialize(self, mut writer: impl io::Write) -> io::Result<()> {
        writer.write_all(&self.first_year.to_ne_bytes())?;
        writer.write_all(&self.calendar.len().to_ne_bytes())?;

        for year in &self.calendar {
            year.serialize(&mut writer)?;
        }

        Ok(())
    }

    /// TODO: doc
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

/// TODO: doc
#[derive(Clone, Copy)]
pub struct CompactYear([CompactMonth; 12]);

impl CompactYear {
    /// TODO: doc
    pub const fn new() -> Self {
        Self([CompactMonth::new(); 12])
    }

    /// TODO: doc
    pub fn insert(&mut self, month: u32, day: u32) {
        assert!((1..=12).contains(&month));
        assert!((1..=31).contains(&day));
        self.0[(month - 1) as usize].insert(day)
    }

    /// TODO: doc
    pub fn contains(&self, month: u32, day: u32) -> bool {
        assert!((1..=12).contains(&month));
        assert!((1..=31).contains(&day));
        self.0[(month - 1) as usize].contains(day)
    }

    /// TODO: doc
    pub fn first(&self) -> Option<(u32, u32)> {
        self.0.iter().enumerate().find_map(|(i, month)| {
            let res_month = (i + 1) as u32;
            Some((res_month, month.first()?))
        })
    }

    /// TODO: doc
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
                    let res_month = (i + month0 + 3) as u32;
                    Some((res_month, month.first()?))
                })
        }
    }

    /// TODO: doc
    pub fn count(&self) -> u32 {
        self.0.iter().copied().map(CompactMonth::count).sum()
    }

    /// TODO: doc
    pub fn serialize(self, mut writer: impl io::Write) -> io::Result<()> {
        for month in self.0 {
            month.serialize(&mut writer)?;
        }

        Ok(())
    }

    /// TODO: doc
    pub fn deserialize(mut reader: impl io::Read) -> io::Result<Self> {
        let mut res = Self::new();

        for month in &mut res.0 {
            *month = CompactMonth::deserialize(&mut reader)?;
        }

        Ok(res)
    }
}

/// TODO: doc
#[derive(Clone, Copy)]
pub struct CompactMonth(u32);

impl CompactMonth {
    /// TODO: doc
    pub const fn new() -> Self {
        Self(0)
    }

    /// TODO: doc
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
    /// assert_eq!(month.count(), 0);
    ///
    /// month.insert(26);
    /// month.insert(3);
    /// assert_eq!(month.count(), 2);
    pub fn count(self) -> u32 {
        self.0.count_ones()
    }

    /// TODO: doc
    pub fn serialize(self, mut writer: impl io::Write) -> io::Result<()> {
        writer.write_all(&self.0.to_ne_bytes())
    }

    /// TODO: doc
    pub fn deserialize(mut reader: impl io::Read) -> io::Result<Self> {
        let mut buf = [0; std::mem::size_of::<u32>()];
        reader.read_exact(&mut buf)?;
        Ok(Self(u32::from_ne_bytes(buf)))
    }
}
