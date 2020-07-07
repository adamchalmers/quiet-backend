//! `twoface::Error` wraps a Rust error type with a user-facing description. This stops users from
//! seeing your internal errors, which might contain sensitive implementation details that should be
//! kept private.

mod extensions;
pub mod externalerror;
mod integrations;

pub use extensions::*;
pub use externalerror::{Cause, ExternalError};
use std::fmt;
use std::fmt::{Display, Formatter};

/// Wraps a Rust error type with a user-facing description. This stops users from seeing your internal
/// errors, which might contain sensitive implementation details that should be kept private.
#[derive(Debug)]
pub struct TfError {
    /// The underlying error, from some function. May contain sensitive information, so it should
    /// not be shown to users.
    pub internal: anyhow::Error,
    /// A user-friendly error that doesn't contain any sensitive information.
    pub external: ExternalError,
}

/// Displaying a twoface::Error will only display the external section. The internal error remains
/// private.
impl Display for TfError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        write!(f, "{}", self.external)
    }
}

/// Return type of a function that could fail. If it fails, it includes a twoface error (an error with
/// both internal- and external-facing values).
pub type Fallible<T> = Result<T, TfError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_only_external_part_is_shown() {
        let io_err = std::fs::read("secret-filename-do-not-leak-to-user").unwrap_err();
        let err = io_err.describe(ExternalError {
            cause: Cause::ServerError,
            text: "An IO error occurred",
        });
        assert_eq!(err.to_string(), "ServerError: An IO error occurred");
    }
}
