use std::error::Error;

#[derive(Debug, PartialEq, Clone)]
pub enum RemoteHelperError {
    Invalid {
        what: String,
        value: String,
    },
    Missing {
        what: String,
    },
    Failure {
        action: String,
        details: Option<String>,
    },
}

impl Error for RemoteHelperError {}

impl std::fmt::Display for RemoteHelperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid { what, value } => write!(f, "invalid {}: {}", what, value,),
            Self::Missing { what } => write!(f, "missing: {}", what),
            Self::Failure { action, details } => write!(
                f,
                "{} failed: {}",
                action,
                details
                    .clone()
                    .unwrap_or("details not provided".to_string())
            ),
        }
    }
}
