use super::sample;
use crate::OpeningHours;

#[test]
fn parse_24_7() {
    assert!(OpeningHours::parse("24/7").is_ok());
}

#[test]
fn parse_invalid() {
    assert!(OpeningHours::parse("this is not a valid expression").is_err());
    assert!(OpeningHours::parse("10:00-100:00").is_err());
    assert!(OpeningHours::parse("10:00-12:00 tomorrow").is_err());
}

// Here is a tough one:
// "check website http://www.senat.fr/visite/jardin/horaires.html"; Mar Su[-1]-Sep 30 07:30-19:15+ open "check closing time on website http://www.senat.fr/visite/jardin/horaires.html"; (sunset-00:10)-(sunrise-00:50) closed; 21:30-07:30 closed
#[test]
fn parse_sample() {
    for raw_oh in sample() {
        assert!(OpeningHours::parse(raw_oh).is_ok());
    }
}

#[test]
fn parse_reformated_sample() {
    for raw_oh in sample() {
        let oh = OpeningHours::parse(raw_oh).unwrap();
        assert!(OpeningHours::parse(&oh.to_string()).is_ok());
    }
}

#[test]
fn parse_reformated() {
    let format_and_parse = |oh| OpeningHours::parse(&OpeningHours::parse(oh).unwrap().to_string());
    assert!(format_and_parse("Oct").is_ok());
}
