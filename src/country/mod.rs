mod generated;

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use compact_calendar::CompactCalendar;
use flate2::bufread::DeflateDecoder;

use crate::context::ContextHolidays;

pub use generated::*;

impl Country {
    /// Load holidays for this country from a compact embeded database.
    ///
    /// ```
    /// use chrono::NaiveDate;
    /// use opening_hours::country::Country;
    ///
    /// let holidays_fr = Country::FR.holidays();
    /// let date = NaiveDate::from_ymd_opt(2024, 7, 14).unwrap(); // french national day
    /// assert!(holidays_fr.public.contains(date));
    /// ```
    pub fn holidays(self) -> ContextHolidays {
        fn decode_holidays_db(
            countries: &'static str,
            encoded_data: &'static [u8],
        ) -> HashMap<Country, Arc<CompactCalendar>> {
            let mut reader = DeflateDecoder::new(encoded_data);

            countries
                .split(',')
                .filter_map(|region| {
                    let calendar = CompactCalendar::deserialize(&mut reader)
                        .expect("unable to parse holiday data");

                    let Ok(country) = region.parse() else {
                        log::warn!("Unknown initialized country code {region}");
                        return None;
                    };

                    Some((country, Arc::new(calendar)))
                })
                .collect()
        }

        static DB_PUBLIC: LazyLock<HashMap<Country, Arc<CompactCalendar>>> = LazyLock::new(|| {
            decode_holidays_db(
                env!("HOLIDAYS_PUBLIC_REGIONS"),
                include_bytes!(env!("HOLIDAYS_PUBLIC_FILE")),
            )
        });

        static DB_SCHOOL: LazyLock<HashMap<Country, Arc<CompactCalendar>>> = LazyLock::new(|| {
            decode_holidays_db(
                env!("HOLIDAYS_SCHOOL_REGIONS"),
                include_bytes!(env!("HOLIDAYS_SCHOOL_FILE")),
            )
        });

        ContextHolidays {
            public: DB_PUBLIC.get(&self).cloned().unwrap_or_default(),
            school: DB_SCHOOL.get(&self).cloned().unwrap_or_default(),
        }
    }
}
