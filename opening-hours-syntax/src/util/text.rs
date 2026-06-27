pub(crate) fn is_capitalized(s: &str) -> bool {
    let mut chars = s.chars();

    let Some(first_char) = chars.next() else {
        return true;
    };

    (first_char.is_uppercase() || !first_char.is_alphanumeric())
        && chars.all(|c| c.is_lowercase() || !c.is_alphanumeric())
}

pub(crate) fn is_lowercase(s: &str) -> bool {
    s.chars().all(|c| c.is_lowercase() || !c.is_alphanumeric())
}
