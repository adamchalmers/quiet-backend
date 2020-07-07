//! Convenience methods to turn any error (from any library) into twoface errors.
use crate::twoface::{ExternalError, TfError};

pub trait Describe {
    /// Convert an error into a twoface::Error by describing it to your users.
    fn describe(self, external: ExternalError) -> TfError;
}

impl<Internal: Into<anyhow::Error>> Describe for Internal {
    fn describe(self, external: ExternalError) -> TfError {
        TfError {
            internal: self.into(),
            external,
        }
    }
}

/// Any regular internal error can be turned into a twoface Error, using the default external error.
/// If you want to give an internal error a custom external error, use `internal.describe(ExternalError)`
impl<Internal: Into<anyhow::Error>> From<Internal> for TfError {
    fn from(internal: Internal) -> TfError {
        internal.describe(Default::default())
    }
}

pub trait DescribeErr<T> {
    /// Convert a result's error into a twoface::Error by describing it to your users.
    /// ```rust
    //  // These two are equivalent:
    /// let result = Result<i32, &'static str> = Err("some private internal error").map_err(|e| e.describe(external))
    /// let result = Result<i32, &'static str> = Err("some private internal error").describe_err(external)
    /// ```
    fn describe_err(self, external: ExternalError) -> Result<T, TfError>;
}

impl<T, E> DescribeErr<T> for Result<T, E>
where
    E: Into<anyhow::Error>,
{
    fn describe_err(self, external: ExternalError) -> Result<T, TfError> {
        self.map_err(|e| e.describe(external))
    }
}
