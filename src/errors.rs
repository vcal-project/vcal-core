//! errors.rs — lightweight error type for VCAL-core
//!
//! * No external crates (`thiserror`) to keep MSRV 1.56.

use core::fmt;

pub type Result<T, E = VcalError> = core::result::Result<T, E>;

#[derive(Debug, Clone)]
pub enum VcalError {
    DimensionMismatch { expected: usize, found: usize },
    EmptyIndex,
    #[cfg(feature = "serde")]
    Serialize(String),
}

impl fmt::Display for VcalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VcalError::DimensionMismatch { expected, found } => {
                write!(f, "dimension mismatch: expected {}, got {}", expected, found)
            }
            VcalError::EmptyIndex => write!(f, "index is empty"),
            #[cfg(feature = "serde")]
            VcalError::Serialize(msg) => write!(f, "serialization error: {}", msg),
        }
    }
}

impl std::error::Error for VcalError {}
