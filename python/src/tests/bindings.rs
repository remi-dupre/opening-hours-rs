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
