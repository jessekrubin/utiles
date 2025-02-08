pub(crate) fn timestamp_string() -> String {
    jiff::Zoned::now().to_string()
}
