use std::error::Error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum ReplayError {
    BadSelector,
    Sqs(Box<dyn std::error::Error>),
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ReplayError::BadSelector => write!(f, "bad selector"),
            ReplayError::Sqs(e) => write!(f, "{}", e),
        }
    }
}

impl Error for ReplayError {}

unsafe impl Send for ReplayError {}

impl From<sqs::Error> for ReplayError {
    fn from(err: sqs::Error) -> ReplayError {
        ReplayError::Sqs(Box::new(err))
    }
}
