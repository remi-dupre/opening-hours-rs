use crate::tests::run_python;

#[test]
fn basic_state_24_7() {
    run_python(
        r#"
        from opening_hours import OpeningHours

        oh = OpeningHours("24/7")
        assert oh.is_open()
        assert not oh.is_closed()
        assert not oh.is_unknown()
        "#,
    );
}

#[test]
fn explicit_country_code() {
    run_python(
        r#"
        from datetime import datetime
        from opening_hours import OpeningHours

        dt = datetime.fromisoformat("2024-07-14 12:00")
        oh = OpeningHours("24/7 ; PH off", country="FR")
        assert oh.is_closed(dt)
        "#,
    )
}

#[test]
fn explicit_timezone() {
    run_python(
        r#"
        from datetime import datetime
        from zoneinfo import ZoneInfo
        from opening_hours import OpeningHours

        tz = ZoneInfo("Europe/Paris")
        dt = datetime.fromisoformat("2024-12-12 11:30")
        oh = OpeningHours("10:00-12:00", timezone=tz)

        assert oh.is_open(dt)
        assert oh.is_open(dt.replace(tzinfo=tz))
        assert oh.is_closed(dt.replace(tzinfo=ZoneInfo("UTC")))

        # Soon supported : https://github.com/PyO3/pyo3/issues/3266
        # assert oh.next_change().tzinfo == tz
        assert oh.next_change(dt) == oh.next_change(dt.replace(tzinfo=tz))
        "#,
    )
}

#[test]
fn auto_from_coord() {
    run_python(
        r#"
        from datetime import datetime
        from zoneinfo import ZoneInfo
        from opening_hours import OpeningHours

        dt = datetime.fromisoformat("2024-12-12 11:30")
        oh = OpeningHours("10:00-12:00 ; PH off", coords=(48.8535, 2.34839))

        assert oh.is_closed(datetime.fromisoformat("2024-07-14 11:30"))
        assert oh.is_open(dt)
        assert oh.is_open(dt.replace(tzinfo=ZoneInfo("Europe/Paris")))
        assert oh.is_closed(dt.replace(tzinfo=ZoneInfo("UTC")))
        "#,
    )
}
