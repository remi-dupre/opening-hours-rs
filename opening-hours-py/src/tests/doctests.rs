use crate::tests::run_python;

#[test]
fn run_doctests() {
    run_python(
        r#"
        import doctest
        import opening_hours
        from datetime import datetime
        from opening_hours import OpeningHours, State


        opening_hours.__test__ = dict(OpeningHours.__dict__) | {
            "OpeningHours": OpeningHours,
            "State": State,
        }

        globs = {
            "datetime": datetime,
            "opening_hours": opening_hours,
            "OpeningHours": OpeningHours,
        }

        result = doctest.testmod(
            opening_hours,
            globs=globs,
            optionflags=doctest.ELLIPSIS,
            verbose=True
        )

        assert result.failed == 0
        "#,
    );
}
