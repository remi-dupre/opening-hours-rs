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
