#[derive(Debug)]
pub enum RustyVbanError {
    PlayStreamError(cpal::PlayStreamError),
    PauseStreamError(cpal::PauseStreamError),
    AnyhowError(anyhow::Error),
}

impl From<cpal::PlayStreamError> for RustyVbanError {
    fn from(error: cpal::PlayStreamError) -> Self {
        RustyVbanError::PlayStreamError(error)
    }
}

impl From<cpal::PauseStreamError> for RustyVbanError {
    fn from(error: cpal::PauseStreamError) -> Self {
        RustyVbanError::PauseStreamError(error)
    }
}

impl From<anyhow::Error> for RustyVbanError {
    fn from(error: anyhow::Error) -> Self {
        RustyVbanError::AnyhowError(error)
    }
}

impl std::fmt::Display for RustyVbanError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RustyVbanError::PlayStreamError(error) => write!(f, "PlayStreamError: {}", error),
            RustyVbanError::PauseStreamError(error) => write!(f, "PauseStreamError: {}", error),
            RustyVbanError::AnyhowError(error) => write!(f, "AnyhowError: {}", error),
        }
    }
}

impl std::error::Error for RustyVbanError {}
