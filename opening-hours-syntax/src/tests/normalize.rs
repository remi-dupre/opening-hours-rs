use alloc::string::ToString;

use rstest::rstest;

use crate::rules::OpeningHoursExpression;

#[rstest]
#[case("Sa; 24/7", "24/7")]
#[case("06:00+;24/7", "06:00+; 24/7")]
#[case("06:00-24:00;24/7", "24/7")]
#[case("Tu-Mo", "24/7")]
#[case("2022;Fr", "2022, Fr")]
#[case("Mo,Th open; Tu,Fr-Su open", "Mo-Tu,Th-Su")]
#[case("Mo-Fr 10:00-14:00 ; We-Su 10:00-14:00", "10:00-14:00")]
#[case("Mo,Tu,We,Th,Fr,Sa,Su 10:00-21:00", "10:00-21:00")]
#[case("5554Mo;5555", "5554-5555 Mo; 5555 Tu-Su")]
#[case("4405-4500,4400-4450", "4400-4500")]
#[case("Jun24:00+", "Jun 24:00+")]
#[case("24/7 ; Su closed", "Mo-Sa")]
#[case("Tu off; off ; Jun", "Jun")]
#[case("off ; Jun unknown", "Jun unknown")]
#[case("Mo-Fr open; We unknown", "Mo-Tu,Th-Fr; We unknown")]
#[case("Mo unknown ; Tu open ; We closed", "Tu; Mo unknown")]
#[case("unknown|| Th|| We", "24/7 unknown || Th || We")]
#[case("dusk-dusk", "dusk-dusk")]
#[case("dusk-48:00+", "dusk-48:00+")]
#[case("Sep24:00-04:20", "Sep 24:00-04:20")]
#[case("Sa;Su;2490-2490/8", "2490, Sa-Su")]
#[case("Mo 10:00-21:00; Tu,We,Th,Fr,Sa,Su 10:00-21:00", "10:00-21:00")]
#[case("week2Mo;Jun;Fr", "Jun, week02 Mo,Fr, Fr")]
#[case(
    "10:00-12:00 open; 14:00-16:00 closed \"on demand\"",
    "10:00-12:00, Mo-Su 14:00-16:00 closed \"on demand\""
)]
#[case(
    "10:00-16:00, We 15:00-20:00 unknown",
    "Mo-Tu,Th-Su 10:00-16:00; We 10:00-15:00, We 15:00-20:00 unknown"
)]
#[case(
    "Nov-Mar Mo-Fr 10:00-16:00; Apr-Nov Mo-Fr 08:00-18:00",
    "Mo-Fr 10:00-16:00, Apr-Nov Mo-Fr 08:00-18:00"
)]
#[case(
    "Apr-Oct Mo-Fr 08:00-18:00; Mo-Fr 10:00-16:00 open",
    "Mo-Fr 10:00-16:00"
)]
#[case(
    "Mo-Fr 10:00-16:00 open; Apr-Oct Mo-Fr 08:00-18:00",
    "Mo-Fr 10:00-16:00, Apr-Oct Mo-Fr 08:00-18:00"
)]
#[case(
        "Mo-Su 00:00-01:00, 10:30-24:00; PH off; 2021 Apr 10 00:00-01:00; 2021 Apr 11-16 off; 2021 Apr 17 10:30-24:00",
        "00:00-01:00,10:30-24:00; PH closed; 2021 Apr 10 00:00-01:00; 2021 Apr 11-2021 Apr 16 closed; 2021 Apr 17 10:30-24:00",
    )]
#[case(
    "week04 Mo; Jul; Jun 5; Sep Fr; 04:00-04:20",
    "Jul, week04 Mo; Jun 5; Sep Fr; 04:00-04:20"
)]
fn normalize(#[case] example: OpeningHoursExpression, #[case] normalized_expected: &str) {
    let normalized = example.normalize_v1();

    assert_eq!(
        normalized.to_string(),
        normalized_expected,
        "normalized expression differs from expected",
    );

    assert_eq!(
        normalized.to_string(),
        normalized.normalize_v1().to_string(),
        "normalize is not idempotent",
    );
}

// Edge case is open & unknown in the same day
// Mo-Su 08:00-14:00 open, Mo-Sun 10:00-18:00 unknown; Mo-Su 18:00-20:00 closed "may vary"
//
// Other edge case is variable time

#[rstest]
#[case("Sa; 24/7", "24/7")]
#[case("06:00+;24/7", "24/7")]
#[case("06:00-24:00;24/7", "24/7")]
#[case("Tu-Mo", "24/7")]
#[case("2022;Fr", "2022; Fr")]
#[case("Mo,Th open; Tu,Fr-Su open", "Mo-Tu,Th-Su")]
#[case("Mo-Fr 10:00-14:00 ; We-Su 10:00-14:00", "10:00-14:00")]
#[case("Mo,Tu,We,Th,Fr,Sa,Su 10:00-21:00", "10:00-21:00")]
#[case("5554Mo;5555", "5554-5555 Mo; 5555 Tu-Su")]
#[case("4405-4500,4400-4450", "4400-4500")]
#[case("Jun24:00+", "Jun 24:00+")]
#[case("24/7 ; Su closed", "Mo-Sa")]
#[case("Tu off; off ; Jun", "Jun")]
#[case("off ; Jun unknown", "Jun unknown")]
#[case("Mo-Fr open; We unknown", "Mo-Tu,Th-Fr; We unknown")]
#[case("Mo unknown ; Tu open ; We closed", "Tu; Mo unknown")]
#[case("unknown|| Th|| We", "24/7 unknown || Th || We")]
#[case("dusk-dusk", "dusk-dusk")]
#[case("dusk-48:00+", "dusk-48:00+")]
#[case("Sep24:00-04:20", "Sep 24:00-04:20")]
#[case("Sa;Su;2490-2490/8", "2490; Sa-Su")]
#[case("Mo 10:00-21:00; Tu,We,Th,Fr,Sa,Su 10:00-21:00", "10:00-21:00")]
#[case("week2Mo;Jun;Fr", "Jun; week02 Mo,Fr; Fr")]
#[case(
    "10:00-12:00 open; 14:00-16:00 closed \"on demand\"",
    "10:00-12:00, Mo-Su 14:00-16:00 closed \"on demand\""
)]
#[case(
    "10:00-16:00, We 15:00-20:00 unknown",
    "Mo-Tu,Th-Su 10:00-16:00; We 10:00-15:00, We 15:00-20:00 unknown"
)]
#[case(
    "Nov-Mar Mo-Fr 10:00-16:00; Apr-Nov Mo-Fr 08:00-18:00",
    "Jan-Mar,Dec Mo-Fr 10:00-16:00; Apr-Nov Mo-Fr 08:00-18:00"
)]
#[case(
    "Apr-Oct Mo-Fr 08:00-18:00; Mo-Fr 10:00-16:00 open",
    "Mo-Fr 10:00-16:00"
)]
#[case(
    "Mo-Fr 10:00-16:00 open; Apr-Oct Mo-Fr 08:00-18:00",
    "Jan-Mar,Nov-Dec Mo-Fr 10:00-16:00; Apr-Oct Mo-Fr 08:00-18:00"
)]
#[case(
        "Mo-Su 00:00-01:00, 10:30-24:00; PH off; 2021 Apr 10 00:00-01:00; 2021 Apr 11-16 off; 2021 Apr 17 10:30-24:00",
        "00:00-01:00,10:30-24:00; PH closed; 2021 Apr 10 00:00-01:00; 2021 Apr 11-2021 Apr 16 closed; 2021 Apr 17 10:30-24:00",
    )]
#[case(
    "week04 Mo; Jul; Jun 5; Sep Fr; 04:00-04:20",
    "Jul; week04 Mo; Jun 5; Sep Fr; 04:00-04:20"
)]
fn normalize_v2(#[case] example: OpeningHoursExpression, #[case] normalized_expected: &str) {
    let normalized = example.normalize();

    assert_eq!(
        normalized.to_string(),
        normalized_expected,
        "normalized expression differs from expected",
    );

    assert_eq!(
        normalized.to_string(),
        normalized.normalize().to_string(),
        "normalize is not idempotent",
    );
}
