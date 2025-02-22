use super::sample;
use crate::OpeningHours;

#[test]
fn parse_24_7() {
    assert!("24/7".parse::<OpeningHours>().is_ok());
}

#[test]
fn parse_open_ended() {
    assert_eq!(
        "12:00+".parse::<OpeningHours>().is_ok(),
        "12:00-24:00".parse::<OpeningHours>().is_ok(),
    );
}

#[test]
fn parse_invalid() {
    assert!(
        "this is not a valid expression"
            .parse::<OpeningHours>()
            .is_err()
    );
    assert!("10:00-100:00".parse::<OpeningHours>().is_err());
    assert!("10:00-12:00 tomorrow".parse::<OpeningHours>().is_err());
}

#[test]
fn parse_sample() {
    for raw_oh in sample() {
        assert!(raw_oh.parse::<OpeningHours>().is_ok());
    }
}

#[test]
fn parse_relaxed() {
    assert!("4:00-8:00".parse::<OpeningHours>().is_ok());
    assert!("04:00 - 08:00".parse::<OpeningHours>().is_ok());
    assert!("4:00 - 8:00".parse::<OpeningHours>().is_ok());

    assert!(
        "Mo-Fr 10:00-18:00;Sa-Su 10:00-12:00"
            .parse::<OpeningHours>()
            .is_ok()
    );
}

#[test]
fn parse_reformated_sample() {
    for raw_oh in sample() {
        let oh = raw_oh.parse::<OpeningHours>().unwrap();
        assert!(&oh.to_string().parse::<OpeningHours>().is_ok());
    }
}

#[test]
fn parse_reformated() {
    let format_and_parse = |oh: &str| {
        oh.parse::<OpeningHours>()
            .unwrap()
            .to_string()
            .parse::<OpeningHours>()
    };

    assert!(format_and_parse("Oct").is_ok());
}

#[test]
fn monthday_00_invalid() {
    assert!(OpeningHours::parse("Jan 0").is_err());
    assert!(OpeningHours::parse("Jan 00").is_err());
    assert!(OpeningHours::parse("Jan 00-15").is_err());
}

#[test]
fn copy_start_offset() {
    let raw_oh = "Jun 7+Tu";
    let oh = OpeningHours::parse(raw_oh).unwrap();
    assert_eq!(raw_oh, &oh.to_string());
}

#[test]
fn no_extended_time_as_begining() {
    assert!(OpeningHours::parse("27:43").is_err());
    assert!(OpeningHours::parse("24:11").is_err());
    assert!(OpeningHours::parse("27:43+").is_err());
    assert!(OpeningHours::parse("24:11+").is_err());
    assert!(OpeningHours::parse("27:43-28:00").is_err());
    assert!(OpeningHours::parse("24:11-28:00").is_err());
}

#[test]
fn with_24_00() {
    assert!(OpeningHours::parse("Jun24:00+").is_ok())
}
