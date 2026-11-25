use thiserror::Error;

#[derive(Debug, Error)]
pub enum DBError {
    #[error("not found")]
    NotFound,
    #[error("email already exists")]
    EmailConflict,
    #[error("something wrong happened")]
    Internal,
    #[error("timeout")]
    TimeOut,
    #[error("connection failure")] Io(String),
}
