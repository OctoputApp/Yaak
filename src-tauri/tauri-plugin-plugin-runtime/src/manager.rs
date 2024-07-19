use log::{debug, info};
use tauri::{AppHandle, Manager, Runtime};
use tokio::sync::Mutex;
use tonic::transport::Channel;

use crate::nodejs::node_start;
use crate::plugin_runtime::plugin_runtime_client::PluginRuntimeClient;
use crate::plugin_runtime::{HookExportRequest, HookImportRequest, HookResponse, HookResponseFilterRequest};

pub struct PluginManager {
    client: Mutex<PluginRuntimeClient<Channel>>,
}

impl PluginManager {
    pub async fn new<R: Runtime>(app_handle: &AppHandle<R>) -> PluginManager {
        let temp_dir = app_handle.path().temp_dir().unwrap();

        let addr = node_start(app_handle, &temp_dir).await;
        info!("Connecting to gRPC client at {addr}");

        let client = match PluginRuntimeClient::connect(addr.clone()).await {
            Ok(v) => v,
            Err(err) => {
                panic!("{}", err.to_string());
            }
        };

        PluginManager {
            client: Mutex::new(client),
        }
    }

    pub async fn run_import(&self, data: &str) -> Result<HookResponse, String> {
        let response = self
            .client
            .lock()
            .await
            .hook_import(tonic::Request::new(HookImportRequest {
                data: data.to_string(),
            }))
            .await
            .map_err(|e| e.message().to_string())?;

        Ok(response.into_inner())
    }

    pub async fn run_export_curl(&self, request: &str) -> Result<HookResponse, String> {
        let response = self
            .client
            .lock()
            .await
            .hook_export(tonic::Request::new(HookExportRequest {
                request: request.to_string(),
            }))
            .await
            .map_err(|e| e.message().to_string())?;

        Ok(response.into_inner())
    }

    pub async fn run_response_filter(
        &self,
        filter: &str,
        body: &str,
        content_type: &str,
    ) -> Result<HookResponse, String> {
        debug!("Running plugin filter");
        let response = self
            .client
            .lock()
            .await
            .hook_response_filter(tonic::Request::new(HookResponseFilterRequest {
                filter: filter.to_string(),
                body: body.to_string(),
                content_type: content_type.to_string(),
            }))
            .await
            .map_err(|e| e.message().to_string())?;

        let result = response.into_inner();
        debug!("Ran plugin response filter {}", result.data);
        Ok(result)
    }
}