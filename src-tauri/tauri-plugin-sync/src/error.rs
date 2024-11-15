use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("SQL error: {0}")]
    SqlError(#[from] rusqlite::Error),

    #[error("JSON error: {0}")]
    JsonErr(#[from] serde_json::Error),

    #[error("Yaak model error: {0}")]
    ModelErr(#[from] yaak_models::error::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
