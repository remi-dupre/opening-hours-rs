use crate::localization::{Coordinates, TzLocation};
use crate::{datetime, Context, OpeningHours};

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
    let tz = chrono_tz::Europe::Paris;
    let ctx = Context::default().with_locale(TzLocation::new(tz));

    let oh = OpeningHours::parse("10:00-18:00")
        .unwrap()
        .with_context(ctx);

    assert_eq!(
        oh.next_change(datetime!("2024-12-23 14:44", tz)).unwrap(),
        datetime!("2024-12-23 18:00", tz),
    );
}

// In France, time skipped from 02:00 to 03:00 on 31/03/2024
// See https://www.service-public.fr/particuliers/actualites/A15539
#[test]
fn ends_at_invalid_time() {
    let tz = chrono_tz::Europe::Paris;
    let ctx = Context::default().with_locale(TzLocation::new(tz));

    let oh = OpeningHours::parse("10:00-26:30")
        .unwrap()
        .with_context(ctx);

    assert_eq!(
        oh.next_change(datetime!("2024-03-30 14:44", tz)).unwrap(),
        datetime!("2024-03-31 03:00", tz),
    );
}

// In France, the clock jumped back to 02:00 on 27/10/2024 03:00
// See https://www.service-public.fr/particuliers/actualites/A15263
#[test]
fn ends_at_ambiguous_time() {
    let tz = chrono_tz::Europe::Paris;
    let ctx = Context::default().with_locale(TzLocation::new(tz));

    let oh = OpeningHours::parse("10:00-26:30")
        .unwrap()
        .with_context(ctx);

    assert_eq!(
        oh.next_change(datetime!("2024-10-27 14:44", tz)).unwrap(),
        datetime!("2024-10-28 02:30", tz),
    );
}

#[cfg(feature = "auto-timezone")]
#[test]
fn infer_tz() {
    let tz = chrono_tz::Europe::Paris; // will be infered for context
    let ctx = Context::default().with_locale(TzLocation::from_coords(COORDS_PARIS));

    let oh = OpeningHours::parse("sunrise-sunset")
        .unwrap()
        .with_context(ctx);

    assert_eq!(
        oh.next_change(datetime!("2024-12-23 14:44", tz)).unwrap(),
        datetime!("2024-12-23 16:57", tz),
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
    let tz = chrono_tz::Europe::Paris; // Will be infered for context
    let ctx = Context::from_coords(COORDS_PARIS);

    let oh = OpeningHours::parse("sunrise-sunset; PH off")
        .unwrap()
        .with_context(ctx);

    assert_eq!(
        oh.next_change(datetime!("2024-12-23 14:44", tz)).unwrap(),
        datetime!("2024-12-23 16:57", tz),
    );

    // 14th of July is french national day
    assert_eq!(
        oh.next_change(datetime!("2024-07-14 14:44", tz)).unwrap(),
        datetime!("2024-07-15 06:03", tz),
    );
}
