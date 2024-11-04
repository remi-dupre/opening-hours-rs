use super::sample;
use crate::OpeningHours;

#[test]
fn parse_24_7() {
    assert!("24/7".parse::<OpeningHours>().is_ok());
}

#[test]
fn parse_invalid() {
    assert!("this is not a valid expression"
        .parse::<OpeningHours>()
        .is_err());
    assert!("10:00-100:00".parse::<OpeningHours>().is_err());
    assert!("10:00-12:00 tomorrow".parse::<OpeningHours>().is_err());
}

// Here is a tough one:
// "check website http://www.senat.fr/visite/jardin/horaires.html"; Mar Su[-1]-Sep 30 07:30-19:15+ open "check closing time on website http://www.senat.fr/visite/jardin/horaires.html"; (sunset-00:10)-(sunrise-00:50) closed; 21:30-07:30 closed
#[test]
fn parse_sample() {
    for raw_oh in sample() {
        assert!(raw_oh.parse::<OpeningHours>().is_ok());
    }
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
