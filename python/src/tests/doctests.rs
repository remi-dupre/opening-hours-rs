use crate::tests::run_python;

#[test]
fn run_doctests() {
    run_python(
        r#"
        import doctest
        import opening_hours
        from opening_hours import OpeningHours


        opening_hours.__test__ = dict(OpeningHours.__dict__) | {
            "OpeningHours": OpeningHours
        }

        globs = {
            "opening_hours": opening_hours,
            "OpeningHours": OpeningHours,
        }

        doctest.testmod(
            opening_hours, globs=globs, optionflags=doctest.ELLIPSIS, verbose=True
        )
        "#,
    );
}
