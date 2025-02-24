use super::sample;
use crate::OpeningHours;

#[test]
fn parse_24_7() {
    assert!(OpeningHours::parse("24/7").is_ok());
}

#[test]
fn parse_open_ended() {
    assert!(OpeningHours::parse("12:00+").is_ok());
    assert!(OpeningHours::parse("dusk-dusk+").is_ok());
}

#[test]
fn parse_invalid() {
    assert!(OpeningHours::parse("this is not a valid expression").is_err());
    assert!(OpeningHours::parse("10:00-100:00").is_err());
    assert!(OpeningHours::parse("10:00-12:00 tomorrow").is_err());
}

#[test]
fn parse_sample() {
    for raw_oh in sample() {
        assert!(OpeningHours::parse(raw_oh).is_ok());
    }
}

#[test]
fn parse_relaxed() {
    assert!(OpeningHours::parse("4:00-8:00").is_ok());
    assert!(OpeningHours::parse("04:00 - 08:00").is_ok());
    assert!(OpeningHours::parse("4:00 - 8:00").is_ok());
    assert!(OpeningHours::parse("Mo-Fr 10:00-18:00;Sa-Su 10:00-12:00").is_ok());
}

#[test]
fn parse_reformated_sample() {
    for raw_oh in sample() {
        let oh: OpeningHours = raw_oh.parse().unwrap();
        assert!(OpeningHours::parse(&oh.to_string()).is_ok());
    }
}

#[test]
fn parse_reformated() {
    let format_and_parse =
        |oh: &str| OpeningHours::parse(&OpeningHours::parse(oh).unwrap().to_string());

    assert!(format_and_parse("Oct").is_ok());
    assert!(format_and_parse("dawn-dawn+").is_ok());
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

#[test]

fn comments() {
    assert!(OpeningHours::parse(r#"Mo-Fr open "ring the bell""#).is_ok());
    assert!(OpeningHours::parse(r#"Mo-Fr open "ring "the bell"""#).is_err());
}
