pub(crate) fn timestamp_string() -> String {
    jiff::ZonedDateTime::now().to_string()
}
