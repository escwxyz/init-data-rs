use thiserror::Error;

#[derive(Error, Debug)]
pub enum InitDataError {
    #[error("auth_date is missing")]
    AuthDateMissing,

    #[error("hash is missing")]
    HashMissing,

    #[error("hash is invalid")]
    HashInvalid,

    #[error("init data has unexpected format: {0}")]
    UnexpectedFormat(String),

    #[error("init data is expired")]
    Expired,

    #[error("internal library's error occurred: {0}")]
    Internal(String),
}
