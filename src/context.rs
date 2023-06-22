use std::collections::HashMap;

use flate2::read::ZlibDecoder;
use once_cell::sync::Lazy;

use compact_calendar::CompactCalendar;

use crate::localize::NoLocation;

/// An array of sorted holidays for each known region
pub static REGION_HOLIDAYS: Lazy<HashMap<&str, CompactCalendar>> = Lazy::new(|| {
    let mut reader = ZlibDecoder::new(include_bytes!(env!("HOLIDAYS_FILE")) as &[_]);

    env!("HOLIDAYS_REGIONS")
        .split(',')
        .map(|region| {
            let calendar =
                CompactCalendar::deserialize(&mut reader).expect("unable to parse holiday data");

            (region, calendar)
        })
        .collect()
});

const EMPTY_CALENDAR: &CompactCalendar = &CompactCalendar::new();

/// TODO: doc
#[derive(Clone, Debug)]
pub(crate) struct Context<L = NoLocation> {
    /// The sorted list of holidays
    pub(crate) holidays: &'static CompactCalendar,
    /// Localisation infos
    pub(crate) localize: L,
}

impl Default for Context<NoLocation> {
    fn default() -> Self {
        Self {
            holidays: EMPTY_CALENDAR,
            localize: Default::default(),
        }
    }
}
