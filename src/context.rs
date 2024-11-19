use std::sync::Arc;

use compact_calendar::CompactCalendar;

#[derive(Clone, Default, Debug, Hash, PartialEq, Eq)]
pub struct ContextHolidays {
    pub public: Arc<CompactCalendar>,
    pub school: Arc<CompactCalendar>,
}

/// All the context attached to a parsed OpeningHours expression and that can
/// alter its evaluation semantics.
#[derive(Clone, Default, Debug, Hash, PartialEq, Eq)]
pub struct Context {
    pub(crate) holidays: ContextHolidays,
}

impl Context {
    /// TODO: doc
    pub fn with_holidays(mut self, holidays: ContextHolidays) -> Self {
        self.holidays = holidays;
        self
    }
}
