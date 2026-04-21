use core::fmt;
use std::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppError {
    InvalidId,
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidId => formatter.write_str("invalid id"),
        }
    }
}

impl Error for AppError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Id(String);

impl Id {
    pub fn new(value: &str) -> Result<Self, AppError> {
        if value.is_empty() {
            return Err(AppError::InvalidId);
        }

        Ok(Self(value.to_owned()))
    }
}

impl fmt::Display for Id {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
