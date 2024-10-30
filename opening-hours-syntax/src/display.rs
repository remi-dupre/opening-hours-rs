use std::fmt::Display;

pub(crate) fn write_days_offset(f: &mut std::fmt::Formatter<'_>, offset: i64) -> std::fmt::Result {
    if offset == 0 {
        return Ok(());
    }

    write!(f, " ")?;

    if offset > 0 {
        write!(f, "+")?;
    }

    write!(f, "{offset} day")?;

    if offset.abs() > 1 {
        write!(f, "s")?;
    }

    Ok(())
}

pub(crate) fn write_selector(
    f: &mut std::fmt::Formatter<'_>,
    seq: &[impl Display],
) -> std::fmt::Result {
    let Some(first) = seq.first() else {
        return Ok(());
    };

    write!(f, "{first}")?;

    for elem in &seq[1..] {
        write!(f, ",{elem}")?;
    }

    Ok(())
}
