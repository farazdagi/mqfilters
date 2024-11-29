/// Query filter error.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum QueryFilterError {
    /// Some error occurred.
    #[error("Some error occurred.")]
    Other(String),
}

/// Query filter result.
pub type QueryFilterResult<T> = Result<T, QueryFilterError>;
