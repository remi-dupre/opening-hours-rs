use std::str::FromStr;

use crate::localization::{Coordinates, TzLocation};
use crate::tests::utils::parse::dtz;
use crate::{Context, OpeningHours};

const TIMEZONE: chrono_tz::Tz = chrono_tz::Europe::Paris;

#[cfg(feature = "auto-timezone")]
const COORDS_PARIS: Coordinates = Coordinates::new(48.8535, 2.34839).unwrap();

#[test]
fn coords_cannot_be_nan() {
    assert_eq!(Coordinates::new(f64::NAN, 1.0), None);
    assert_eq!(Coordinates::new(1.0, f64::NAN), None);
    assert_eq!(Coordinates::new(f64::NAN, f64::NAN), None);
}

#[test]
fn ctx_with_tz() {
    let ctx = Context::default().with_locale(TzLocation::new(TIMEZONE));

    let oh = OpeningHours::from_str("10:00-18:00")
        .unwrap()
        .with_context(ctx);

    assert_eq!(
        oh.next_change(dtz("2024-12-23 14:44", TIMEZONE)).unwrap(),
        dtz("2024-12-23 18:00", TIMEZONE),
    );
}

// In France, time skipped from 02:00 to 03:00 on 31/03/2024
// See https://www.service-public.fr/particuliers/actualites/A15539
#[test]
fn ends_at_invalid_time() {
    let ctx = Context::default().with_locale(TzLocation::new(TIMEZONE));

    let oh = OpeningHours::from_str("10:00-26:30")
        .unwrap()
        .with_context(ctx);

    assert_eq!(
        oh.next_change(dtz("2024-03-30 14:44", TIMEZONE)).unwrap(),
        dtz("2024-03-31 03:00", TIMEZONE),
    );
}

// In France, the clock jumped back to 02:00 on 27/10/2024 03:00
// See https://www.service-public.fr/particuliers/actualites/A15263
#[test]
fn ends_at_ambiguous_time() {
    let ctx = Context::default().with_locale(TzLocation::new(TIMEZONE));

    let oh = OpeningHours::from_str("10:00-26:30")
        .unwrap()
        .with_context(ctx);

    assert_eq!(
        oh.next_change(dtz("2024-10-27 14:44", TIMEZONE)).unwrap(),
        dtz("2024-10-28 02:30", TIMEZONE),
    );
}

#[cfg(feature = "auto-timezone")]
#[test]
fn infer_tz() {
    let ctx = Context::default().with_locale(TzLocation::from_coords(COORDS_PARIS));
    assert_eq!(ctx.locale.get_timezone(), &TIMEZONE);

    let oh = OpeningHours::from_str("sunrise-sunset")
        .unwrap()
        .with_context(ctx);

    assert_eq!(
        oh.next_change(dtz("2024-12-23 14:44", TIMEZONE)).unwrap(),
        dtz("2024-12-23 16:57", TIMEZONE),
    );
}

#[test]
fn invalid_coord() {
    assert!(Coordinates::new(2000.0, 0.0).is_none());
    assert!(Coordinates::new(0.0, 2000.0).is_none());
}

#[cfg(feature = "auto-country")]
#[cfg(feature = "auto-timezone")]
#[test]
fn infer_all() {
    let ctx = Context::from_coords(COORDS_PARIS);
    assert_eq!(ctx.locale.get_timezone(), &TIMEZONE);

    let oh = OpeningHours::from_str("sunrise-sunset; PH off")
        .unwrap()
        .with_context(ctx);

    assert_eq!(
        oh.next_change(dtz("2024-12-23 14:44", TIMEZONE)).unwrap(),
        dtz("2024-12-23 16:57", TIMEZONE),
    );

    // 14th of July is french national day
    assert_eq!(
        oh.next_change(dtz("2024-07-14 14:44", TIMEZONE)).unwrap(),
        dtz("2024-07-15 06:03", TIMEZONE),
    );
}
