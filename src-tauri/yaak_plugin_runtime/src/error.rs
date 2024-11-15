use thiserror::Error;
use tokio::io;
use tokio::sync::mpsc::error::SendError;
use crate::server::plugin_runtime::EventStreamEvent;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    IoErr(#[from] io::Error),

    #[error("Tauri error: {0}")]
    TauriErr(#[from] tauri::Error),

    #[error("Tauri shell error: {0}")]
    TauriShellErr(#[from] tauri_plugin_shell::Error),

    #[error("Grpc transport error: {0}")]
    GrpcTransportErr(#[from] tonic::transport::Error),

    #[error("Grpc send error: {0}")]
    GrpcSendErr(#[from] SendError<tonic::Result<EventStreamEvent>>),

    #[error("JSON error: {0}")]
    JsonErr(#[from] serde_json::Error),

    #[error("Plugin not found: {0}")]
    PluginNotFoundErr(String),

    #[error("Plugin error: {0}")]
    PluginErr(String),

    #[error("Client not initialized error")]
    ClientNotInitializedErr,

    #[error("Unknown event received")]
    UnknownEventErr,
}

impl Into<String> for Error {
    fn into(self) -> String {
        todo!()
    }
}

pub type Result<T> = std::result::Result<T, Error>;
