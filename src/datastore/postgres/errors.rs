use crate::twoface::{ExternalError, Fallible, TfError};
use actix_web::error::BlockingError;
use anyhow::anyhow;
use diesel::result::Error as DieselError;

type DbPoolErr = BlockingError<DieselError>;
pub type DbPoolResult<T> = Result<T, DbPoolErr>;

/// Convenience extension used to extract errors from `web::block`.
pub trait BlockingResp<T> {
    /// Convert the return from a web::block into a normal `Fallible<T>`.
    fn to_resp(self) -> Fallible<T>;
}

impl<T, I: std::fmt::Debug + Into<TfError>> BlockingResp<T> for Result<T, BlockingError<I>> {
    fn to_resp(self) -> Fallible<T> {
        match self {
            Ok(t) => Ok(t),
            Err(BlockingError::Error(err)) => Err(err.into()),
            Err(BlockingError::Canceled) => Err(TfError {
                internal: anyhow!("DB operation cancelled"),
                external: ExternalError::default(),
            }),
        }
    }
}
