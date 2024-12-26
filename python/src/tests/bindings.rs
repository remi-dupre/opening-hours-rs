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
fn load_country_code() {
    run_python(
        r#"
        from datetime import datetime
        from opening_hours import OpeningHours

        oh = OpeningHours("24/7 ; PH off", country="FR")
        assert oh.is_closed(datetime(2024, 7, 14, 12, 0))
        "#,
    )
}
