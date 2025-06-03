use utiles_core::UtilesCoreError;

#[derive(Debug)]
pub struct UtilesCoverError {
    pub message: String,
}

impl UtilesCoverError {
    #[must_use]
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for UtilesCoverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UtilesCoverError: {}", self.message)
    }
}

impl std::error::Error for UtilesCoverError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<UtilesCoreError> for UtilesCoverError {
    fn from(err: UtilesCoreError) -> Self {
        Self::new(&err.to_string())
    }
}
