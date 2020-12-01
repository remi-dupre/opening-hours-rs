use crate::parse;

#[test]
fn parse_24_7() {
    assert!(parse("24/7").is_ok());
}

#[test]
fn parse_invalid() {
    assert!(parse("this is not a valid expression").is_err());
    assert!(parse("10:00-100:00").is_err());
    assert!(parse("10:00-12:00 tomorrow").is_err());
}
