use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppError {
    InvalidId,
}

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
