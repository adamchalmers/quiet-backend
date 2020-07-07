use actix_web::http::StatusCode;
use std::fmt;

/// Used to create HTTP responses with the given text and status code.
#[derive(Debug)]
pub struct ExternalError {
    /// A user-facing explanation of what caused the error.
    pub cause: Cause,
    /// Error text that will describe the problem to the user.
    pub text: &'static str,
}

/// A user-facing explanation of what caused the error.
#[derive(Debug, Clone, Copy)]
pub enum Cause {
    ServerError,
    UserActionInvalid,
    UserBadAuth,
    UserConflict,
    UserInvalidField,
    NotFound,
}

impl fmt::Display for Cause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        // Make fmt::Display the same as fmt::Debug, i.e. each variant's name.
        write!(f, "{:?}", self)
    }
}

impl Into<StatusCode> for Cause {
    /// Causes can be mapped to HTTP status codes. ExternalError doesn't use status codes directly,
    /// because some components (e.g. the Datastore) shouldn't need to know about HTTP codes.
    fn into(self) -> StatusCode {
        match self {
            Self::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UserActionInvalid => StatusCode::BAD_REQUEST,
            Self::UserInvalidField => StatusCode::BAD_REQUEST,
            Self::UserBadAuth => StatusCode::UNAUTHORIZED,
            Self::UserConflict => StatusCode::CONFLICT,
            Self::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

impl fmt::Display for ExternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}: {}", self.cause, self.text)
    }
}

impl Default for ExternalError {
    // Default to ServerError and a very vague generic message.
    fn default() -> Self {
        Self {
            cause: Cause::ServerError,
            text: "Internal server error",
        }
    }
}
