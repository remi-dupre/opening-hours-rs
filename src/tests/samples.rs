use super::sample;
use crate::OpeningHours;

// Here is a tough one:
// "check website http://www.senat.fr/visite/jardin/horaires.html"; Mar Su[-1]-Sep 30 07:30-19:15+ open "check closing time on website http://www.senat.fr/visite/jardin/horaires.html"; (sunset-00:10)-(sunrise-00:50) closed; 21:30-07:30 closed
#[test]
fn parse_sample() {
    for raw_oh in sample() {
        OpeningHours::parse(raw_oh)
            .unwrap_or_else(|err| panic!("Failed parsing `{}`:\n{}", raw_oh, err));
    }
}
