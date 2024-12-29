use std::collections::HashSet;

use crate::country::Country;

#[test]
fn test_parse_bijective() {
    for country in &Country::ALL {
        assert_eq!(*country, country.iso_code().parse().unwrap());
    }
}

#[test]
fn test_name_unique() {
    let names: HashSet<_> = Country::ALL.iter().map(|c| c.name()).collect();
    assert_eq!(names.len(), Country::ALL.len());
}

#[test]
fn test_parse_invalid() {
    assert!("France".parse::<Country>().is_err());
    assert!("fr".parse::<Country>().is_err());
}
