use alloc::vec::Vec;
use core::convert::{TryFrom, TryInto};
use core::fmt::Display;
use core::ops::{Deref, DerefMut, RangeInclusive};

use chrono::prelude::Datelike;
use chrono::{Duration, NaiveDate};

// Reexport Weekday from chrono as part of the public type.
pub use chrono::Weekday;

use crate::display::{write_days_offset, write_selector};

// --
// -- Helpers: Display
// --

fn wday_str(wday: Weekday) -> &'static str {
    match wday {
        Weekday::Mon => "Mo",
        Weekday::Tue => "Tu",
        Weekday::Wed => "We",
        Weekday::Thu => "Th",
        Weekday::Fri => "Fr",
        Weekday::Sat => "Sa",
        Weekday::Sun => "Su",
    }
}

// --
// -- Errors
// --

#[derive(Clone, Debug)]
pub struct InvalidMonth;

// --
// -- Struct: DaySelector
// --

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct DaySelector {
    pub year: Vec<YearRange>,
    pub monthday: Vec<MonthdayRange>,
    pub week: Vec<WeekRange>,
    pub weekday: Vec<WeekDayRange>,
}

impl DaySelector {
    /// Return `true` if there is no date filter in this expression.
    pub fn is_empty(&self) -> bool {
        self.year.is_empty()
            && self.monthday.is_empty()
            && self.week.is_empty()
            && self.weekday.is_empty()
    }

    /// Format this day selector into given formatter.
    ///
    /// If `force` is set to true, this is guaranteed to yield a non-empty string by adding "Mo-Su"
    /// as fallback.
    pub(crate) fn display(
        &self,
        f: &mut core::fmt::Formatter<'_>,
        force: bool,
    ) -> core::fmt::Result {
        if force && self.is_empty() {
            return write!(f, "Mo-Su");
        }

        if !(self.year.is_empty() && self.monthday.is_empty() && self.week.is_empty()) {
            write_selector(f, &self.year)?;
            write_selector(f, &self.monthday)?;

            if !self.week.is_empty() {
                if !self.year.is_empty() || !self.monthday.is_empty() {
                    write!(f, " ")?;
                }

                write!(f, "week")?;
                write_selector(f, &self.week)?;
            }

            if !self.weekday.is_empty() {
                write!(f, " ")?;
            }
        }

        write_selector(f, &self.weekday)
    }
}

impl Display for DaySelector {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.display(f, false)
    }
}

// --
// -- Struct: Year (newtype)
// --

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Year(pub i32);

impl Deref for Year {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Year {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// YearRange

/// A year range that ensures that bounds are always in the right order
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct YearRange {
    range: RangeInclusive<Year>,
    step: u16,
}

impl YearRange {
    /// Create a new `YearRange`. Return `None` if the bounds are in the wrong order.
    pub fn new(range: RangeInclusive<Year>, mut step: u16) -> Option<Self> {
        if range.start() > range.end() {
            return None;
        }

        if range.start().abs_diff(**range.end()) < step.into() {
            step = 1;
        }

        Some(Self { range, step })
    }

    /// Extract range and step from this object.
    pub fn into_parts(&self) -> (RangeInclusive<Year>, u16) {
        (self.range.clone(), self.step)
    }
}

impl Display for YearRange {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let start = **self.range.start();
        let end = **self.range.end();

        write!(f, "{start}")?;

        if start != end {
            write!(f, "-{end}")?;

            if self.step != 1 {
                write!(f, "/{}", self.step)?;
            }
        }

        Ok(())
    }
}

// --
// -- Enum: MonthdayRange
// --

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum MonthdayRange {
    Month {
        range: RangeInclusive<Month>,
        year: Option<Year>,
    },
    Date {
        start: (Date, DateOffset),
        end: (Date, DateOffset),
    },
}

impl Display for MonthdayRange {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Month { range, year } => {
                if let Some(year) = year {
                    write!(f, "{}", **year)?;
                }

                write!(f, "{}", range.start())?;

                if range.start() != range.end() {
                    write!(f, "-{}", range.end())?;
                }
            }
            Self::Date { start, end } => {
                write!(f, "{}{}", start.0, start.1)?;

                if start != end {
                    write!(f, "-{}{}", end.0, end.1)?;
                }
            }
        }

        Ok(())
    }
}

// --
// -- Enum: Date
// --

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Date {
    Fixed {
        year: Option<Year>,
        month: Month,
        day: u8,
    },
    Weekday {
        year: Option<Year>,
        month: Month,
        wday: Weekday,
        nth: i8,
    },
    Easter {
        year: Option<Year>,
    },
}

impl Date {
    #[inline]
    pub fn ymd(day: u8, month: Month, year: Year) -> Self {
        Self::Fixed { day, month, year: Some(year) }
    }

    #[inline]
    pub fn md(day: u8, month: Month) -> Self {
        Self::Fixed { day, month, year: None }
    }

    #[inline]
    pub fn year(&self) -> Option<Year> {
        match *self {
            Self::Fixed { year, .. } | Self::Weekday { year, .. } | Self::Easter { year } => year,
        }
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Fixed { year, month, day } => {
                if let Some(year) = year {
                    write!(f, "{} ", **year)?;
                }

                write!(f, "{month} {day}")
            }
            Self::Weekday { year, month, wday: weekday, nth } => {
                if let Some(year) = year {
                    write!(f, "{} ", **year)?;
                }

                write!(f, "{month} {}", wday_str(*weekday))?;

                if *nth != 0 {
                    write!(f, "[{nth}]")?;
                }

                Ok(())
            }
            Self::Easter { year } => {
                if let Some(year) = year {
                    write!(f, "{} ", **year)?;
                }

                write!(f, "easter")
            }
        }
    }
}

// --
// -- Struct: DateOffset
// --

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub struct DateOffset {
    pub wday_offset: WeekDayOffset,
    pub day_offset: i16,
}

impl DateOffset {
    #[inline]
    pub fn apply(&self, mut date: NaiveDate) -> NaiveDate {
        date += Duration::days(self.day_offset.into());

        match self.wday_offset {
            WeekDayOffset::None => {}
            WeekDayOffset::Prev(target) => {
                let diff = (7 + date.weekday().days_since(Weekday::Mon)
                    - target.days_since(Weekday::Mon))
                    % 7;

                date -= Duration::days(diff.into());
                debug_assert_eq!(date.weekday(), target);
            }
            WeekDayOffset::Next(target) => {
                let diff = (7 + target.days_since(Weekday::Mon)
                    - date.weekday().days_since(Weekday::Mon))
                    % 7;

                date += Duration::days(diff.into());
                debug_assert_eq!(date.weekday(), target);
            }
        }

        date
    }
}

impl Display for DateOffset {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.wday_offset)?;
        write_days_offset(f, self.day_offset)?;
        Ok(())
    }
}

// WeekDayOffset

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum WeekDayOffset {
    None,
    Next(Weekday),
    Prev(Weekday),
}

impl Default for WeekDayOffset {
    #[inline]
    fn default() -> Self {
        Self::None
    }
}

impl Display for WeekDayOffset {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::None => {}
            Self::Next(wday) => write!(f, "+{}", wday_str(*wday))?,
            Self::Prev(wday) => write!(f, "-{}", wday_str(*wday))?,
        }

        Ok(())
    }
}

// --
// -- Enum: WeekDayRange
// --

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum WeekDayRange {
    Fixed {
        range: RangeInclusive<Weekday>,
        offset: i16,
        nth_from_start: [bool; 5],
        nth_from_end: [bool; 5],
    },
    Holiday {
        kind: HolidayKind,
        offset: i16,
    },
}

impl Display for WeekDayRange {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Fixed { range, offset, nth_from_start, nth_from_end } => {
                write!(f, "{}", wday_str(*range.start()))?;

                if range.start() != range.end() {
                    write!(f, "-{}", wday_str(*range.end()))?;
                }

                if nth_from_start.contains(&false) || nth_from_end.contains(&false) {
                    let pos_weeknum_iter = nth_from_start
                        .iter()
                        .enumerate()
                        .filter(|(_, x)| **x)
                        .map(|(idx, _)| (idx + 1) as isize);

                    let neg_weeknum_iter = nth_from_end
                        .iter()
                        .enumerate()
                        .filter(|(_, x)| **x)
                        .map(|(idx, _)| -(idx as isize) - 1);

                    let mut weeknum_iter = pos_weeknum_iter.chain(neg_weeknum_iter);

                    if let Some(first_num) = weeknum_iter.next() {
                        write!(f, "[{first_num}")?;

                        for num in weeknum_iter {
                            write!(f, ",{num}")?;
                        }

                        write!(f, "]")?;
                    }
                }

                write_days_offset(f, *offset)?;
            }
            Self::Holiday { kind, offset } => {
                write!(f, "{kind}")?;

                if *offset != 0 {
                    write!(f, " {offset}")?;
                }
            }
        }

        Ok(())
    }
}

// --
// -- Enum: HolidayKind
// --

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum HolidayKind {
    Public,
    School,
}

impl Display for HolidayKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Public => write!(f, "PH"),
            Self::School => write!(f, "SH"),
        }
    }
}

// --
// -- Struct: WeekNum (newtype)
// --

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct WeekNum(pub u8);

impl Deref for WeekNum {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WeekNum {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// --
// -- Struct: WeekRange
// --

// TODO: ensure there can't be wrapping with step
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WeekRange {
    range: RangeInclusive<WeekNum>,
    step: u8,
}

impl WeekRange {
    /// Create a new `WeekRange`. Return `None` if the bounds are in the wrong order.
    pub fn new(range: RangeInclusive<WeekNum>, mut step: u8) -> Option<Self> {
        if range.start() > range.end() {
            return None;
        }

        if range.start().abs_diff(**range.end()) < step {
            step = 1;
        }

        Some(Self { range, step })
    }

    /// Extract range and step from this object.
    pub fn into_parts(&self) -> (RangeInclusive<WeekNum>, u8) {
        (self.range.clone(), self.step)
    }
}

impl Display for WeekRange {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.range.start() == self.range.end() && self.step == 1 {
            return write!(f, "{:02}", **self.range.start());
        }

        write!(f, "{:02}", **self.range.start())?;

        if self.range.start() != self.range.end() {
            write!(f, "-{:02}", **self.range.end())?;

            if self.step != 1 {
                write!(f, "/{}", self.step)?;
            }
        }

        Ok(())
    }
}

// Enum: Month

#[derive(Copy, Clone, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub enum Month {
    January = 1,
    February = 2,
    March = 3,
    April = 4,
    May = 5,
    June = 6,
    July = 7,
    August = 8,
    September = 9,
    October = 10,
    November = 11,
    December = 12,
}

impl Month {
    #[inline]
    pub fn next(self) -> Self {
        let num = self as u8;
        #[allow(clippy::unwrap_used)] // (x % 12) + 1 is guaranteed to be in [1, 12]
        ((num % 12) + 1).try_into().unwrap()
    }

    #[inline]
    pub fn prev(self) -> Self {
        let num = self as u8;
        #[allow(clippy::unwrap_used)] // (x % 12) + 1 is guaranteed to be in [1, 12]
        (((num + 10) % 12) + 1).try_into().unwrap()
    }

    /// Extract a month from a [`chrono::Datelike`].
    #[inline]
    pub fn from_date(date: impl Datelike) -> Self {
        match date.month() {
            1 => Self::January,
            2 => Self::February,
            3 => Self::March,
            4 => Self::April,
            5 => Self::May,
            6 => Self::June,
            7 => Self::July,
            8 => Self::August,
            9 => Self::September,
            10 => Self::October,
            11 => Self::November,
            12 => Self::December,
            other => unreachable!("Unexpected month for date `{other}`"),
        }
    }

    /// Stringify the month (`"January"`, `"February"`, ...).
    #[inline]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::January => "January",
            Self::February => "February",
            Self::March => "March",
            Self::April => "April",
            Self::May => "May",
            Self::June => "June",
            Self::July => "July",
            Self::August => "August",
            Self::September => "September",
            Self::October => "October",
            Self::November => "November",
            Self::December => "December",
        }
    }
}

impl Display for Month {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", &self.as_str()[..3])
    }
}

macro_rules! impl_convert_for_month {
    ( $from_type: ty ) => {
        impl TryFrom<$from_type> for Month {
            type Error = InvalidMonth;

            #[inline]
            fn try_from(value: $from_type) -> Result<Self, Self::Error> {
                let value: u8 = value.try_into().map_err(|_| InvalidMonth)?;

                Ok(match value {
                    1 => Self::January,
                    2 => Self::February,
                    3 => Self::March,
                    4 => Self::April,
                    5 => Self::May,
                    6 => Self::June,
                    7 => Self::July,
                    8 => Self::August,
                    9 => Self::September,
                    10 => Self::October,
                    11 => Self::November,
                    12 => Self::December,
                    _ => return Err(InvalidMonth),
                })
            }
        }

        impl From<Month> for $from_type {
            fn from(val: Month) -> Self {
                val as _
            }
        }
    };
    ( $from_type: ty, $( $tail: tt )+ ) => {
        impl_convert_for_month!($from_type);
        impl_convert_for_month!($($tail)+);
    };
}

impl_convert_for_month!(
    u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize
);
