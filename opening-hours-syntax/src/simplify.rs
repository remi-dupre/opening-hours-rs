use std::ops::Range;

use crate::rubik::{Paving, Paving5D, PavingSelector};
use crate::rules::day::{DaySelector, MonthdayRange, WeekDayRange, WeekRange, YearRange};
use crate::rules::time::{Time, TimeSelector, TimeSpan};
use crate::rules::RuleSequence;
use crate::ExtendedTime;

#[derive(Default)]
struct CanonicalDaySelector {
    year: Vec<Range<u16>>,
    month: Vec<Range<u8>>,   // 1..=12
    week: Vec<Range<u8>>,    // 0..=5
    weekday: Vec<Range<u8>>, // 0..=6
    time: Vec<Range<ExtendedTime>>,
}

impl CanonicalDaySelector {
    const TIME_BOUNDS: Range<ExtendedTime> =
        ExtendedTime::new(0, 0).unwrap()..ExtendedTime::new(48, 0).unwrap();

    #[allow(clippy::single_range_in_vec_init)]
    fn try_from_rule_sequence(rs: &RuleSequence) -> Option<Self> {
        let mut result = CanonicalDaySelector::default();
        let ds = &rs.day_selector;
        let ts = &rs.time_selector;

        for year in &ds.year {
            if year.step != 1 {
                return None;
            }

            let start = *year.range.start();
            let end = *year.range.end() + 1;
            result.year.push(start..end);
        }

        for monthday in &ds.monthday {
            match monthday {
                MonthdayRange::Month { range, year: None } => {
                    let start = *range.start() as u8;
                    let end = *range.end() as u8 + 1;
                    result.month.push(start..end);
                }
                _ => return None,
            }
        }

        for week in &ds.week {
            if week.step != 1 {
                return None;
            }

            let start = *week.range.start();
            let end = *week.range.end() + 1;
            result.week.push(start..end)
        }

        for weekday in &ds.weekday {
            match weekday {
                WeekDayRange::Fixed {
                    range,
                    offset: 0,
                    nth_from_start: [false, false, false, false, false], // TODO: could be canonical
                    nth_from_end: [false, false, false, false, false],   // TODO: could be canonical
                } => {
                    let start = *range.start() as u8;
                    let end = *range.end() as u8 + 1;
                    result.weekday.push(start..end);
                }
                _ => return None,
            }
        }

        for time in &ts.time {
            match time {
                TimeSpan { range, open_end: false, repeats: None } => {
                    let Time::Fixed(start) = range.start else {
                        return None;
                    };

                    let Time::Fixed(end) = range.end else {
                        return None;
                    };

                    result.time.push(start..end);
                }
                _ => return None,
            }
        }

        if result.year.is_empty() {
            result.year = vec![u16::MIN..u16::MAX];
        }

        if result.month.is_empty() {
            result.month = vec![1..13];
        }

        if result.week.is_empty() {
            result.week = vec![0..6];
        }

        if result.weekday.is_empty() {
            result.weekday = vec![0..7];
        }

        if result.time.is_empty() {
            result.time = vec![Self::TIME_BOUNDS.clone()];
        }

        Some(result)
    }

    fn into_day_selector(self) -> DaySelector {
        let mut result_ds = DaySelector::default();
        let mut result_ts = TimeSelector::default();

        for year_rg in self.year {
            if year_rg == (u16::MIN..u16::MAX) {
                result_ds.year.clear();
                break;
            }

            result_ds
                .year
                .push(YearRange { range: year_rg.start..=year_rg.end - 1, step: 1 })
        }

        for month_rg in self.month {
            if month_rg == (1..13) {
                result_ds.monthday.clear();
                break;
            }

            result_ds.monthday.push(MonthdayRange::Month {
                range: month_rg.start.try_into().expect("invalid starting month")
                    ..=(month_rg.end - 1).try_into().expect("invalid ending month"),
                year: None,
            })
        }

        for week_rg in self.week {
            if week_rg == (0..6) {
                result_ds.week.clear();
                break;
            }

            result_ds
                .week
                .push(WeekRange { range: week_rg.start..=week_rg.end - 1, step: 1 })
        }

        for weekday_rg in self.weekday {
            if weekday_rg == (0..7) {
                result_ds.weekday.clear();
                break;
            }

            result_ds.weekday.push(WeekDayRange::Fixed {
                range: (weekday_rg.start).try_into().expect("invalid starting day")
                    ..=(weekday_rg.end - 1).try_into().expect("invalid ending day"),
                offset: 0,
                nth_from_start: [false; 5],
                nth_from_end: [false; 5],
            })
        }

        for time_rg in self.time {
            result_ts.time.push(TimeSpan {
                range: Time::Fixed(time_rg.start)..Time::Fixed(time_rg.end),
                open_end: false,
                repeats: None,
            });
        }

        result_ds
    }

    fn as_paving(&self) -> Paving5D<ExtendedTime, u8, u8, u8, u16> {
        let mut res = Paving5D::default();

        for year in &self.year {
            for month in &self.month {
                for week in &self.week {
                    for weekday in &self.weekday {
                        for time in &self.time {
                            let selector = PavingSelector::empty()
                                .dim(year.start..year.end)
                                .dim(month.start..month.end)
                                .dim(week.start..week.end)
                                .dim(weekday.start..weekday.end)
                                .dim(time.start..time.end);

                            res.set(&selector, true);
                        }
                    }
                }
            }
        }

        res
    }

    // #[allow(clippy::single_range_in_vec_init)]
    // fn from_paving(mut paving: Paving5D<ExtendedTime, u8, u8, u8, u16>) -> Vec<Self> {
    //     let mut result = Vec::new();
    //
    //     while let Some(selector) = paving.pop_selector() {
    //         let (start_time, end_time, selector) = selector.unpack();
    //         let (start_weekday, end_weekday, selector) = selector.unpack();
    //         let (start_week, end_week, selector) = selector.unpack();
    //         let (start_month, end_month, selector) = selector.unpack();
    //         let (start_year, end_year, _) = selector.unpack();
    //
    //         result.push(Self {
    //             year: vec![*start_year..*end_year],
    //             month: vec![*start_month..*end_month],
    //             week: vec![*start_week..*end_week],
    //             weekday: vec![*start_weekday..*end_weekday],
    //             time: vec![*start_time..*end_time],
    //         })
    //     }
    //
    //     result
    // }
}
