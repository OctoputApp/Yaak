extern crate core;
#[cfg(target_os = "macos")]
extern crate objc;

use std::collections::BTreeMap;
use std::fs::{create_dir_all, File};
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;
use std::time::Duration;
use std::{fs, panic};

use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use chrono::Utc;
use eventsource_client::{EventParser, SSE};
use fern::colors::ColoredLevelConfig;
use log::{debug, error, info, warn};
use rand::random;
use regex::Regex;
use serde::Serialize;
use serde_json::{json, Value};
#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;
use tauri::{AppHandle, Emitter, LogicalSize, RunEvent, State, WebviewUrl, WebviewWindow};
use tauri::{Listener, Runtime};
use tauri::{Manager, WindowEvent};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_log::{fern, Target, TargetKind};
use tauri_plugin_shell::ShellExt;
use tokio::fs::read_to_string;
use tokio::sync::Mutex;
use tokio::task::block_in_place;
use yaak_grpc::manager::{DynamicMessage, GrpcHandle};
use yaak_grpc::{deserialize_message, serialize_message, Code, ServiceDefinition};
use yaak_plugin_runtime::manager::PluginManager;

use crate::analytics::{AnalyticsAction, AnalyticsResource};
use crate::export_resources::{get_workspace_export_resources, WorkspaceExportResources};
use crate::grpc::metadata_to_map;
use crate::http_request::send_http_request;
use crate::notifications::YaakNotifier;
use crate::render::{render_grpc_request, render_http_request, render_json_value, render_template};
use crate::template_callback::PluginTemplateCallback;
use crate::updates::{UpdateMode, YaakUpdater};
use crate::window_menu::app_menu;
use yaak_models::models::{
    CookieJar, Environment, EnvironmentVariable, Folder, GrpcConnection, GrpcConnectionState,
    GrpcEvent, GrpcEventType, GrpcRequest, HttpRequest, HttpResponse, HttpResponseState, KeyValue,
    ModelType, Plugin, Settings, Workspace,
};
use yaak_models::queries::{
    cancel_pending_grpc_connections, cancel_pending_responses, create_default_http_response,
    delete_all_grpc_connections, delete_all_grpc_connections_for_workspace,
    delete_all_http_responses_for_request, delete_all_http_responses_for_workspace,
    delete_cookie_jar, delete_environment, delete_folder, delete_grpc_connection,
    delete_grpc_request, delete_http_request, delete_http_response, delete_plugin,
    delete_workspace, duplicate_grpc_request, duplicate_http_request, generate_id,
    generate_model_id, get_cookie_jar, get_environment, get_folder, get_grpc_connection,
    get_grpc_request, get_http_request, get_http_response, get_key_value_raw,
    get_or_create_settings, get_plugin, get_workspace, list_cookie_jars, list_environments,
    list_folders, list_grpc_connections_for_workspace, list_grpc_events, list_grpc_requests,
    list_http_requests, list_http_responses_for_request, list_http_responses_for_workspace,
    list_plugins, list_workspaces, set_key_value_raw, update_response_if_id, update_settings,
    upsert_cookie_jar, upsert_environment, upsert_folder, upsert_grpc_connection,
    upsert_grpc_event, upsert_grpc_request, upsert_http_request, upsert_plugin, upsert_workspace,
};
use yaak_plugin_runtime::events::{
    BootResponse, CallHttpRequestActionRequest, FilterResponse, FindHttpResponsesResponse,
    GetHttpRequestActionsResponse, GetHttpRequestByIdResponse, GetTemplateFunctionsResponse, Icon,
    InternalEvent, InternalEventPayload, PromptTextResponse, RenderHttpRequestResponse,
    RenderPurpose, SendHttpRequestResponse, ShowToastRequest, TemplateRenderResponse,
    WindowContext,
};
use yaak_plugin_runtime::plugin_handle::PluginHandle;
use yaak_sse::sse::ServerSentEvent;
use yaak_templates::format::format_json;
use yaak_templates::{Parser, Tokens};

mod analytics;
mod export_resources;
mod grpc;
mod http_request;
mod notifications;
mod render;
#[cfg(target_os = "macos")]
mod tauri_plugin_mac_window;
mod template_callback;
mod updates;
mod window_menu;

const DEFAULT_WINDOW_WIDTH: f64 = 1100.0;
const DEFAULT_WINDOW_HEIGHT: f64 = 600.0;

const MIN_WINDOW_WIDTH: f64 = 300.0;
const MIN_WINDOW_HEIGHT: f64 = 300.0;

const MAIN_WINDOW_PREFIX: &str = "main_";
const OTHER_WINDOW_PREFIX: &str = "other_";

#[derive(serde::Serialize)]
#[serde(default, rename_all = "camelCase")]
struct AppMetaData {
    is_dev: bool,
    version: String,
    name: String,
    app_data_dir: String,
    app_log_dir: String,
}

#[tauri::command]
async fn cmd_metadata(app_handle: AppHandle) -> Result<AppMetaData, ()> {
    let app_data_dir = app_handle.path().app_data_dir().unwrap();
    let app_log_dir = app_handle.path().app_log_dir().unwrap();
    Ok(AppMetaData {
        is_dev: is_dev(),
        version: app_handle.package_info().version.to_string(),
        name: app_handle.package_info().name.to_string(),
        app_data_dir: app_data_dir.to_string_lossy().to_string(),
        app_log_dir: app_log_dir.to_string_lossy().to_string(),
    })
}

#[tauri::command]
async fn cmd_parse_template(template: &str) -> Result<Tokens, String> {
    Ok(Parser::new(template).parse())
}

#[tauri::command]
async fn cmd_template_tokens_to_string(tokens: Tokens) -> Result<String, String> {
    Ok(tokens.to_string())
}

#[tauri::command]
async fn cmd_render_template<R: Runtime>(
    window: WebviewWindow<R>,
    app_handle: AppHandle<R>,
    template: &str,
    workspace_id: &str,
    environment_id: Option<&str>,
) -> Result<String, String> {
    let environment = match environment_id {
        Some(id) => Some(get_environment(&window, id).await.map_err(|e| e.to_string())?),
        None => None,
    };
    let workspace = get_workspace(&window, &workspace_id).await.map_err(|e| e.to_string())?;
    let rendered = render_template(
        template,
        &workspace,
        environment.as_ref(),
        &PluginTemplateCallback::new(
            &app_handle,
            &WindowContext::from_window(&window),
            RenderPurpose::Preview,
        ),
    )
    .await;
    Ok(rendered)
}

#[tauri::command]
async fn cmd_dismiss_notification<R: Runtime>(
    window: WebviewWindow<R>,
    notification_id: &str,
    yaak_notifier: State<'_, Mutex<YaakNotifier>>,
) -> Result<(), String> {
    yaak_notifier.lock().await.seen(&window, notification_id).await
}

#[tauri::command]
async fn cmd_grpc_reflect<R: Runtime>(
    request_id: &str,
    proto_files: Vec<String>,
    window: WebviewWindow<R>,
    grpc_handle: State<'_, Mutex<GrpcHandle>>,
) -> Result<Vec<ServiceDefinition>, String> {
    let req = get_grpc_request(&window, request_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Failed to find GRPC request")?;

    let uri = safe_uri(&req.url);

    grpc_handle
        .lock()
        .await
        .services(
            &req.id,
            &uri,
            &proto_files.iter().map(|p| PathBuf::from_str(p).unwrap()).collect(),
        )
        .await
}

#[tauri::command]
async fn cmd_grpc_go<R: Runtime>(
    request_id: &str,
    environment_id: Option<&str>,
    proto_files: Vec<String>,
    window: WebviewWindow<R>,
    grpc_handle: State<'_, Mutex<GrpcHandle>>,
) -> Result<String, String> {
    let environment = match environment_id {
        Some(id) => Some(get_environment(&window, id).await.map_err(|e| e.to_string())?),
        None => None,
    };
    let req = get_grpc_request(&window, request_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Failed to find GRPC request")?;
    let workspace = get_workspace(&window, &req.workspace_id).await.map_err(|e| e.to_string())?;
    let req = render_grpc_request(
        &req,
        &workspace,
        environment.as_ref(),
        &PluginTemplateCallback::new(
            window.app_handle(),
            &WindowContext::from_window(&window),
            RenderPurpose::Send,
        ),
    )
    .await;
    let mut metadata = BTreeMap::new();

    // Add the rest of metadata
    for h in req.clone().metadata {
        if h.name.is_empty() && h.value.is_empty() {
            continue;
        }

        if !h.enabled {
            continue;
        }

        metadata.insert(h.name, h.value);
    }

    if let Some(b) = &req.authentication_type {
        let req = req.clone();
        let empty_value = &serde_json::to_value("").unwrap();
        let a = req.authentication;

        if b == "basic" {
            let username = a.get("username").unwrap_or(empty_value).as_str().unwrap_or("");
            let password = a.get("password").unwrap_or(empty_value).as_str().unwrap_or("");

            let auth = format!("{username}:{password}");
            let encoded = BASE64_STANDARD.encode(auth);
            metadata.insert("Authorization".to_string(), format!("Basic {}", encoded));
        } else if b == "bearer" {
            let token = a.get("token").unwrap_or(empty_value).as_str().unwrap_or("");
            metadata.insert("Authorization".to_string(), format!("Bearer {token}"));
        }
    }

    let conn = {
        let req = req.clone();
        upsert_grpc_connection(
            &window,
            &GrpcConnection {
                workspace_id: req.workspace_id,
                request_id: req.id,
                status: -1,
                elapsed: 0,
                state: GrpcConnectionState::Initialized,
                url: req.url.clone(),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())?
    };

    let conn_id = conn.id.clone();

    let base_msg = GrpcEvent {
        workspace_id: req.clone().workspace_id,
        request_id: req.clone().id,
        connection_id: conn.clone().id,
        ..Default::default()
    };

    let (in_msg_tx, in_msg_rx) = tauri::async_runtime::channel::<DynamicMessage>(16);
    let maybe_in_msg_tx = std::sync::Mutex::new(Some(in_msg_tx.clone()));
    let (cancelled_tx, mut cancelled_rx) = tokio::sync::watch::channel(false);

    let uri = safe_uri(&req.url);

    let in_msg_stream = tokio_stream::wrappers::ReceiverStream::new(in_msg_rx);

    let (service, method) = {
        let req = req.clone();
        match (req.service, req.method) {
            (Some(service), Some(method)) => (service, method),
            _ => return Err("Service and method are required".to_string()),
        }
    };

    let start = std::time::Instant::now();
    let connection = grpc_handle
        .lock()
        .await
        .connect(
            &req.clone().id,
            uri.as_str(),
            &proto_files.iter().map(|p| PathBuf::from_str(p).unwrap()).collect(),
        )
        .await;

    let connection = match connection {
        Ok(c) => c,
        Err(err) => {
            upsert_grpc_connection(
                &window,
                &GrpcConnection {
                    elapsed: start.elapsed().as_millis() as i32,
                    error: Some(err.clone()),
                    state: GrpcConnectionState::Closed,
                    ..conn.clone()
                },
            )
            .await
            .map_err(|e| e.to_string())?;
            return Ok(conn_id);
        }
    };

    let method_desc = connection.method(&service, &method).map_err(|e| e.to_string())?;

    #[derive(serde::Deserialize)]
    enum IncomingMsg {
        Message(String),
        Cancel,
        Commit,
    }

    let cb = {
        let cancelled_rx = cancelled_rx.clone();
        let window = window.clone();
        let workspace = workspace.clone();
        let environment = environment.clone();
        let base_msg = base_msg.clone();
        let method_desc = method_desc.clone();

        move |ev: tauri::Event| {
            if *cancelled_rx.borrow() {
                // Stream is canceled
                return;
            }

            let mut maybe_in_msg_tx = maybe_in_msg_tx.lock().expect("previous holder not to panic");
            let in_msg_tx = if let Some(in_msg_tx) = maybe_in_msg_tx.as_ref() {
                in_msg_tx
            } else {
                // This would mean that the stream is already committed because
                // we have already dropped the sending half
                return;
            };

            match serde_json::from_str::<IncomingMsg>(ev.payload()) {
                Ok(IncomingMsg::Message(msg)) => {
                    let window = window.clone();
                    let base_msg = base_msg.clone();
                    let method_desc = method_desc.clone();
                    let msg = {
                        block_in_place(|| {
                            tauri::async_runtime::block_on(async {
                                render_template(
                                    msg.as_str(),
                                    &workspace,
                                    environment.as_ref(),
                                    &PluginTemplateCallback::new(
                                        window.app_handle(),
                                        &WindowContext::from_window(&window),
                                        RenderPurpose::Send,
                                    ),
                                )
                                .await
                            })
                        })
                    };
                    let d_msg: DynamicMessage = match deserialize_message(msg.as_str(), method_desc)
                    {
                        Ok(d_msg) => d_msg,
                        Err(e) => {
                            tauri::async_runtime::spawn(async move {
                                upsert_grpc_event(
                                    &window,
                                    &GrpcEvent {
                                        event_type: GrpcEventType::Error,
                                        content: e.to_string(),
                                        ..base_msg.clone()
                                    },
                                )
                                .await
                                .unwrap();
                            });
                            return;
                        }
                    };
                    in_msg_tx.try_send(d_msg).unwrap();
                    tauri::async_runtime::spawn(async move {
                        upsert_grpc_event(
                            &window,
                            &GrpcEvent {
                                content: msg,
                                event_type: GrpcEventType::ClientMessage,
                                ..base_msg.clone()
                            },
                        )
                        .await
                        .unwrap();
                    });
                }
                Ok(IncomingMsg::Commit) => {
                    maybe_in_msg_tx.take();
                }
                Ok(IncomingMsg::Cancel) => {
                    cancelled_tx.send_replace(true);
                }
                Err(e) => {
                    error!("Failed to parse gRPC message: {:?}", e);
                }
            }
        }
    };
    let event_handler = window.listen_any(format!("grpc_client_msg_{}", conn.id).as_str(), cb);

    let grpc_listen = {
        let window = window.clone();
        let base_event = base_msg.clone();
        let req = req.clone();
        let msg = if req.message.is_empty() { "{}".to_string() } else { req.message };
        let msg = render_template(
            msg.as_str(),
            &workspace.clone(),
            environment.as_ref(),
            &PluginTemplateCallback::new(
                window.app_handle(),
                &WindowContext::from_window(&window),
                RenderPurpose::Send,
            ),
        )
        .await;

        upsert_grpc_event(
            &window,
            &GrpcEvent {
                content: format!("Connecting to {}", req.url),
                event_type: GrpcEventType::ConnectionStart,
                metadata: metadata.clone(),
                ..base_event.clone()
            },
        )
        .await
        .unwrap();

        async move {
            let (maybe_stream, maybe_msg) =
                match (method_desc.is_client_streaming(), method_desc.is_server_streaming()) {
                    (true, true) => (
                        Some(
                            connection.streaming(&service, &method, in_msg_stream, metadata).await,
                        ),
                        None,
                    ),
                    (true, false) => (
                        None,
                        Some(
                            connection
                                .client_streaming(&service, &method, in_msg_stream, metadata)
                                .await,
                        ),
                    ),
                    (false, true) => (
                        Some(connection.server_streaming(&service, &method, &msg, metadata).await),
                        None,
                    ),
                    (false, false) => {
                        (None, Some(connection.unary(&service, &method, &msg, metadata).await))
                    }
                };

            if !method_desc.is_client_streaming() {
                upsert_grpc_event(
                    &window,
                    &GrpcEvent {
                        event_type: GrpcEventType::ClientMessage,
                        content: msg,
                        ..base_event.clone()
                    },
                )
                .await
                .unwrap();
            }

            match maybe_msg {
                Some(Ok(msg)) => {
                    upsert_grpc_event(
                        &window,
                        &GrpcEvent {
                            metadata: metadata_to_map(msg.metadata().clone()),
                            content: if msg.metadata().len() == 0 {
                                "Received response"
                            } else {
                                "Received response with metadata"
                            }
                            .to_string(),
                            event_type: GrpcEventType::Info,
                            ..base_event.clone()
                        },
                    )
                    .await
                    .unwrap();
                    upsert_grpc_event(
                        &window,
                        &GrpcEvent {
                            content: serialize_message(&msg.into_inner()).unwrap(),
                            event_type: GrpcEventType::ServerMessage,
                            ..base_event.clone()
                        },
                    )
                    .await
                    .unwrap();
                    upsert_grpc_event(
                        &window,
                        &GrpcEvent {
                            content: "Connection complete".to_string(),
                            event_type: GrpcEventType::ConnectionEnd,
                            status: Some(Code::Ok as i32),
                            ..base_event.clone()
                        },
                    )
                    .await
                    .unwrap();
                }
                Some(Err(e)) => {
                    upsert_grpc_event(
                        &window,
                        &(match e.status {
                            Some(s) => GrpcEvent {
                                error: Some(s.message().to_string()),
                                status: Some(s.code() as i32),
                                content: "Failed to connect".to_string(),
                                metadata: metadata_to_map(s.metadata().clone()),
                                event_type: GrpcEventType::ConnectionEnd,
                                ..base_event.clone()
                            },
                            None => GrpcEvent {
                                error: Some(e.message),
                                status: Some(Code::Unknown as i32),
                                content: "Failed to connect".to_string(),
                                event_type: GrpcEventType::ConnectionEnd,
                                ..base_event.clone()
                            },
                        }),
                    )
                    .await
                    .unwrap();
                }
                None => {
                    // Server streaming doesn't return initial message
                }
            }

            let mut stream = match maybe_stream {
                Some(Ok(stream)) => {
                    upsert_grpc_event(
                        &window,
                        &GrpcEvent {
                            metadata: metadata_to_map(stream.metadata().clone()),
                            content: if stream.metadata().len() == 0 {
                                "Received response"
                            } else {
                                "Received response with metadata"
                            }
                            .to_string(),
                            event_type: GrpcEventType::Info,
                            ..base_event.clone()
                        },
                    )
                    .await
                    .unwrap();
                    stream.into_inner()
                }
                Some(Err(e)) => {
                    warn!("GRPC stream error {e:?}");
                    upsert_grpc_event(
                        &window,
                        &(match e.status {
                            Some(s) => GrpcEvent {
                                error: Some(s.message().to_string()),
                                status: Some(s.code() as i32),
                                content: "Failed to connect".to_string(),
                                metadata: metadata_to_map(s.metadata().clone()),
                                event_type: GrpcEventType::ConnectionEnd,
                                ..base_event.clone()
                            },
                            None => GrpcEvent {
                                error: Some(e.message),
                                status: Some(Code::Unknown as i32),
                                content: "Failed to connect".to_string(),
                                event_type: GrpcEventType::ConnectionEnd,
                                ..base_event.clone()
                            },
                        }),
                    )
                    .await
                    .unwrap();
                    return;
                }
                None => return,
            };

            loop {
                match stream.message().await {
                    Ok(Some(msg)) => {
                        let message = serialize_message(&msg).unwrap();
                        upsert_grpc_event(
                            &window,
                            &GrpcEvent {
                                content: message,
                                event_type: GrpcEventType::ServerMessage,
                                ..base_event.clone()
                            },
                        )
                        .await
                        .unwrap();
                    }
                    Ok(None) => {
                        let trailers =
                            stream.trailers().await.unwrap_or_default().unwrap_or_default();
                        upsert_grpc_event(
                            &window,
                            &GrpcEvent {
                                content: "Connection complete".to_string(),
                                status: Some(Code::Ok as i32),
                                metadata: metadata_to_map(trailers),
                                event_type: GrpcEventType::ConnectionEnd,
                                ..base_event.clone()
                            },
                        )
                        .await
                        .unwrap();
                        break;
                    }
                    Err(status) => {
                        upsert_grpc_event(
                            &window,
                            &GrpcEvent {
                                content: status.to_string(),
                                status: Some(status.code() as i32),
                                metadata: metadata_to_map(status.metadata().clone()),
                                event_type: GrpcEventType::ConnectionEnd,
                                ..base_event.clone()
                            },
                        )
                        .await
                        .unwrap();
                    }
                }
            }
        }
    };

    {
        let conn_id = conn_id.clone();
        tauri::async_runtime::spawn(async move {
            let w = window.clone();
            tokio::select! {
                _ = grpc_listen => {
                    let events = list_grpc_events(&w, &conn_id)
                        .await
                        .unwrap();
                    let closed_event = events
                        .iter()
                        .find(|e| GrpcEventType::ConnectionEnd == e.event_type);
                    let closed_status = closed_event.and_then(|e| e.status).unwrap_or(Code::Unavailable as i32);
                    upsert_grpc_connection(
                        &w,
                        &GrpcConnection{
                            elapsed: start.elapsed().as_millis() as i32,
                            status: closed_status,
                            state: GrpcConnectionState::Closed,
                            ..get_grpc_connection(&w, &conn_id).await.unwrap().clone()
                        },
                    ).await.unwrap();
                },
                _ = cancelled_rx.changed() => {
                    upsert_grpc_event(
                        &w,
                        &GrpcEvent {
                            content: "Cancelled".to_string(),
                            event_type: GrpcEventType::ConnectionEnd,
                            status: Some(Code::Cancelled as i32),
                            ..base_msg.clone()
                        },
                    ).await.unwrap();
                    upsert_grpc_connection(
                        &w,
                        &GrpcConnection {
                            elapsed: start.elapsed().as_millis() as i32,
                            status: Code::Cancelled as i32,
                            state: GrpcConnectionState::Closed,
                            ..get_grpc_connection(&w, &conn_id).await.unwrap().clone()
                        },
                    )
                    .await
                    .unwrap();
                },
            }
            w.unlisten(event_handler);
        });
    };

    Ok(conn.id)
}

#[tauri::command]
async fn cmd_send_ephemeral_request(
    mut request: HttpRequest,
    environment_id: Option<&str>,
    cookie_jar_id: Option<&str>,
    window: WebviewWindow,
) -> Result<HttpResponse, String> {
    let response = HttpResponse::new();
    request.id = "".to_string();
    let environment = match environment_id {
        Some(id) => Some(get_environment(&window, id).await.expect("Failed to get environment")),
        None => None,
    };
    let cookie_jar = match cookie_jar_id {
        Some(id) => Some(get_cookie_jar(&window, id).await.expect("Failed to get cookie jar")),
        None => None,
    };

    let (cancel_tx, mut cancel_rx) = tokio::sync::watch::channel(false);
    window.listen_any(format!("cancel_http_response_{}", response.id), move |_event| {
        if let Err(e) = cancel_tx.send(true) {
            warn!("Failed to send cancel event for ephemeral request {e:?}");
        }
    });

    send_http_request(&window, &request, &response, environment, cookie_jar, &mut cancel_rx).await
}

#[tauri::command]
async fn cmd_format_json(text: &str) -> Result<String, String> {
    Ok(format_json(text, "  "))
}

#[tauri::command]
async fn cmd_filter_response<R: Runtime>(
    window: WebviewWindow<R>,
    response_id: &str,
    plugin_manager: State<'_, PluginManager>,
    filter: &str,
) -> Result<FilterResponse, String> {
    let response =
        get_http_response(&window, response_id).await.expect("Failed to get http response");

    if let None = response.body_path {
        return Err("Response body path not set".to_string());
    }

    let mut content_type = "".to_string();
    for header in response.headers.iter() {
        if header.name.to_lowercase() == "content-type" {
            content_type = header.value.to_string().to_lowercase();
            break;
        }
    }

    let body = read_to_string(response.body_path.unwrap()).await.unwrap();

    // TODO: Have plugins register their own content type (regex?)
    plugin_manager
        .filter_data(&window, filter, &body, &content_type)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_get_sse_events(file_path: &str) -> Result<Vec<ServerSentEvent>, String> {
    let body = fs::read(file_path).map_err(|e| e.to_string())?;
    let mut p = EventParser::new();
    p.process_bytes(body.into()).map_err(|e| e.to_string())?;

    let mut events = Vec::new();
    while let Some(e) = p.get_event() {
        if let SSE::Event(e) = e {
            events.push(ServerSentEvent {
                event_type: e.event_type,
                data: e.data,
                id: e.id,
                retry: e.retry,
            });
        }
    }

    Ok(events)
}

#[tauri::command]
async fn cmd_import_data<R: Runtime>(
    window: WebviewWindow<R>,
    plugin_manager: State<'_, PluginManager>,
    file_path: &str,
) -> Result<WorkspaceExportResources, String> {
    let file = read_to_string(file_path)
        .await
        .unwrap_or_else(|_| panic!("Unable to read file {}", file_path));
    let file_contents = file.as_str();
    let (import_result, plugin_name) =
        plugin_manager.import_data(&window, file_contents).await.map_err(|e| e.to_string())?;

    let mut imported_resources = WorkspaceExportResources::default();
    let mut id_map: BTreeMap<String, String> = BTreeMap::new();

    fn maybe_gen_id(id: &str, model: ModelType, ids: &mut BTreeMap<String, String>) -> String {
        if !id.starts_with("GENERATE_ID::") {
            return id.to_string();
        }

        let unique_key = id.replace("GENERATE_ID", "");
        if let Some(existing) = ids.get(unique_key.as_str()) {
            existing.to_string()
        } else {
            let new_id = generate_model_id(model);
            ids.insert(unique_key, new_id.clone());
            new_id
        }
    }

    fn maybe_gen_id_opt(
        id: Option<String>,
        model: ModelType,
        ids: &mut BTreeMap<String, String>,
    ) -> Option<String> {
        match id {
            Some(id) => Some(maybe_gen_id(id.as_str(), model, ids)),
            None => None,
        }
    }

    let resources = import_result.resources;

    for mut v in resources.workspaces {
        v.id = maybe_gen_id(v.id.as_str(), ModelType::TypeWorkspace, &mut id_map);
        let x = upsert_workspace(&window, v).await.map_err(|e| e.to_string())?;
        imported_resources.workspaces.push(x.clone());
    }
    info!("Imported {} workspaces", imported_resources.workspaces.len());

    for mut v in resources.environments {
        v.id = maybe_gen_id(v.id.as_str(), ModelType::TypeEnvironment, &mut id_map);
        v.workspace_id =
            maybe_gen_id(v.workspace_id.as_str(), ModelType::TypeWorkspace, &mut id_map);
        let x = upsert_environment(&window, v).await.map_err(|e| e.to_string())?;
        imported_resources.environments.push(x.clone());
    }
    info!("Imported {} environments", imported_resources.environments.len());

    // Folders can foreign-key to themselves, so we need to import from
    // the top of the tree to the bottom to avoid foreign key conflicts.
    // We do this by looping until we've imported them all, only importing if:
    //  - The parent folder has been imported
    //  - The folder hasn't already been imported
    // The loop exits when imported.len == to_import.len
    while imported_resources.folders.len() < resources.folders.len() {
        for mut v in resources.folders.clone() {
            v.id = maybe_gen_id(v.id.as_str(), ModelType::TypeFolder, &mut id_map);
            v.workspace_id =
                maybe_gen_id(v.workspace_id.as_str(), ModelType::TypeWorkspace, &mut id_map);
            v.folder_id = maybe_gen_id_opt(v.folder_id, ModelType::TypeFolder, &mut id_map);
            if let Some(fid) = v.folder_id.clone() {
                let imported_parent = imported_resources.folders.iter().find(|f| f.id == fid);
                if imported_parent.is_none() {
                    continue;
                }
            }
            if let Some(_) = imported_resources.folders.iter().find(|f| f.id == v.id) {
                continue;
            }
            let x = upsert_folder(&window, v).await.map_err(|e| e.to_string())?;
            imported_resources.folders.push(x.clone());
        }
    }
    info!("Imported {} folders", imported_resources.folders.len());

    for mut v in resources.http_requests {
        v.id = maybe_gen_id(v.id.as_str(), ModelType::TypeHttpRequest, &mut id_map);
        v.workspace_id =
            maybe_gen_id(v.workspace_id.as_str(), ModelType::TypeWorkspace, &mut id_map);
        v.folder_id = maybe_gen_id_opt(v.folder_id, ModelType::TypeFolder, &mut id_map);
        let x = upsert_http_request(&window, v).await.map_err(|e| e.to_string())?;
        imported_resources.http_requests.push(x.clone());
    }
    info!("Imported {} http_requests", imported_resources.http_requests.len());

    for mut v in resources.grpc_requests {
        v.id = maybe_gen_id(v.id.as_str(), ModelType::TypeGrpcRequest, &mut id_map);
        v.workspace_id =
            maybe_gen_id(v.workspace_id.as_str(), ModelType::TypeWorkspace, &mut id_map);
        v.folder_id = maybe_gen_id_opt(v.folder_id, ModelType::TypeFolder, &mut id_map);
        let x = upsert_grpc_request(&window, &v).await.map_err(|e| e.to_string())?;
        imported_resources.grpc_requests.push(x.clone());
    }
    info!("Imported {} grpc_requests", imported_resources.grpc_requests.len());

    analytics::track_event(
        &window,
        AnalyticsResource::App,
        AnalyticsAction::Import,
        Some(json!({ "plugin": plugin_name })),
    )
    .await;

    Ok(imported_resources)
}

#[tauri::command]
async fn cmd_http_request_actions<R: Runtime>(
    window: WebviewWindow<R>,
    plugin_manager: State<'_, PluginManager>,
) -> Result<Vec<GetHttpRequestActionsResponse>, String> {
    plugin_manager.get_http_request_actions(&window).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_template_functions<R: Runtime>(
    window: WebviewWindow<R>,
    plugin_manager: State<'_, PluginManager>,
) -> Result<Vec<GetTemplateFunctionsResponse>, String> {
    plugin_manager.get_template_functions(&window).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_call_http_request_action<R: Runtime>(
    window: WebviewWindow<R>,
    req: CallHttpRequestActionRequest,
    plugin_manager: State<'_, PluginManager>,
) -> Result<(), String> {
    plugin_manager.call_http_request_action(&window, req).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_curl_to_request<R: Runtime>(
    window: WebviewWindow<R>,
    command: &str,
    plugin_manager: State<'_, PluginManager>,
    workspace_id: &str,
) -> Result<HttpRequest, String> {
    let (import_result, plugin_name) =
        { plugin_manager.import_data(&window, command).await.map_err(|e| e.to_string())? };

    analytics::track_event(
        &window,
        AnalyticsResource::App,
        AnalyticsAction::Import,
        Some(json!({ "plugin": plugin_name })),
    )
    .await;

    import_result.resources.http_requests.get(0).ok_or("No curl command found".to_string()).map(
        |r| {
            let mut request = r.clone();
            request.workspace_id = workspace_id.into();
            request.id = "".to_string();
            request
        },
    )
}

#[tauri::command]
async fn cmd_export_data(
    window: WebviewWindow,
    export_path: &str,
    workspace_ids: Vec<&str>,
) -> Result<(), String> {
    let export_data = get_workspace_export_resources(&window, workspace_ids).await;
    let f = File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(export_path)
        .expect("Unable to create file");

    serde_json::to_writer_pretty(&f, &export_data)
        .map_err(|e| e.to_string())
        .expect("Failed to write");

    f.sync_all().expect("Failed to sync");

    analytics::track_event(&window, AnalyticsResource::App, AnalyticsAction::Export, None).await;

    Ok(())
}

#[tauri::command]
async fn cmd_save_response(
    window: WebviewWindow,
    response_id: &str,
    filepath: &str,
) -> Result<(), String> {
    let response = get_http_response(&window, response_id).await.map_err(|e| e.to_string())?;

    let body_path = match response.body_path {
        None => {
            return Err("Response does not have a body".to_string());
        }
        Some(p) => p,
    };

    fs::copy(body_path, filepath).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn cmd_send_http_request(
    window: WebviewWindow,
    environment_id: Option<&str>,
    cookie_jar_id: Option<&str>,
    // NOTE: We receive the entire request because to account for the race
    //   condition where the user may have just edited a field before sending
    //   that has not yet been saved in the DB.
    request: HttpRequest,
) -> Result<HttpResponse, String> {
    let response =
        create_default_http_response(&window, &request.id).await.map_err(|e| e.to_string())?;

    let (cancel_tx, mut cancel_rx) = tokio::sync::watch::channel(false);
    window.listen_any(format!("cancel_http_response_{}", response.id), move |_event| {
        if let Err(e) = cancel_tx.send(true) {
            warn!("Failed to send cancel event for request {e:?}");
        }
    });

    let environment = match environment_id {
        Some(id) => match get_environment(&window, id).await {
            Ok(env) => Some(env),
            Err(e) => {
                warn!("Failed to find environment by id {id} {}", e);
                None
            }
        },
        None => None,
    };

    let cookie_jar = match cookie_jar_id {
        Some(id) => Some(get_cookie_jar(&window, id).await.expect("Failed to get cookie jar")),
        None => None,
    };

    send_http_request(&window, &request, &response, environment, cookie_jar, &mut cancel_rx).await
}

async fn response_err<R: Runtime>(
    response: &HttpResponse,
    error: String,
    w: &WebviewWindow<R>,
) -> HttpResponse {
    warn!("Failed to send request: {error:?}");
    let mut response = response.clone();
    response.state = HttpResponseState::Closed;
    response.error = Some(error.clone());
    response = update_response_if_id(w, &response).await.expect("Failed to update response");
    response
}

#[tauri::command]
async fn cmd_track_event(
    window: WebviewWindow,
    resource: &str,
    action: &str,
    attributes: Option<Value>,
) -> Result<(), String> {
    match (AnalyticsResource::from_str(resource), AnalyticsAction::from_str(action)) {
        (Ok(resource), Ok(action)) => {
            analytics::track_event(&window, resource, action, attributes).await;
        }
        (r, a) => {
            error!(
                "Invalid action/resource for track_event: {resource}.{action} = {:?}.{:?}",
                r, a
            );
            return Err("Invalid analytics event".to_string());
        }
    };
    Ok(())
}

#[tauri::command]
async fn cmd_set_update_mode(update_mode: &str, w: WebviewWindow) -> Result<KeyValue, String> {
    cmd_set_key_value("app", "update_mode", update_mode, w).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_get_key_value(
    namespace: &str,
    key: &str,
    w: WebviewWindow,
) -> Result<Option<KeyValue>, ()> {
    let result = get_key_value_raw(&w, namespace, key).await;
    Ok(result)
}

#[tauri::command]
async fn cmd_set_key_value(
    namespace: &str,
    key: &str,
    value: &str,
    w: WebviewWindow,
) -> Result<KeyValue, String> {
    let (key_value, _created) = set_key_value_raw(&w, namespace, key, value).await;
    Ok(key_value)
}

#[tauri::command]
async fn cmd_create_workspace(name: &str, w: WebviewWindow) -> Result<Workspace, String> {
    upsert_workspace(&w, Workspace::new(name.to_string())).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_install_plugin<R: Runtime>(
    directory: &str,
    url: Option<String>,
    plugin_manager: State<'_, PluginManager>,
    window: WebviewWindow<R>,
) -> Result<Plugin, String> {
    plugin_manager
        .add_plugin_by_dir(WindowContext::from_window(&window), &directory, true)
        .await
        .map_err(|e| e.to_string())?;

    let plugin = upsert_plugin(
        &window,
        Plugin {
            directory: directory.into(),
            url,
            ..Default::default()
        },
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(plugin)
}

#[tauri::command]
async fn cmd_uninstall_plugin<R: Runtime>(
    plugin_id: &str,
    plugin_manager: State<'_, PluginManager>,
    window: WebviewWindow<R>,
) -> Result<Plugin, String> {
    let plugin = delete_plugin(&window, plugin_id).await.map_err(|e| e.to_string())?;

    plugin_manager
        .uninstall(WindowContext::from_window(&window), plugin.directory.as_str())
        .await
        .map_err(|e| e.to_string())?;

    Ok(plugin)
}

#[tauri::command]
async fn cmd_update_cookie_jar(
    cookie_jar: CookieJar,
    w: WebviewWindow,
) -> Result<CookieJar, String> {
    upsert_cookie_jar(&w, &cookie_jar).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_delete_cookie_jar(w: WebviewWindow, cookie_jar_id: &str) -> Result<CookieJar, String> {
    delete_cookie_jar(&w, cookie_jar_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_create_cookie_jar(
    workspace_id: &str,
    name: &str,
    w: WebviewWindow,
) -> Result<CookieJar, String> {
    upsert_cookie_jar(
        &w,
        &CookieJar {
            name: name.to_string(),
            workspace_id: workspace_id.to_string(),
            ..Default::default()
        },
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_create_environment(
    workspace_id: &str,
    name: &str,
    variables: Vec<EnvironmentVariable>,
    w: WebviewWindow,
) -> Result<Environment, String> {
    upsert_environment(
        &w,
        Environment {
            workspace_id: workspace_id.to_string(),
            name: name.to_string(),
            variables,
            ..Default::default()
        },
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_create_grpc_request(
    workspace_id: &str,
    name: &str,
    sort_priority: f32,
    folder_id: Option<&str>,
    w: WebviewWindow,
) -> Result<GrpcRequest, String> {
    upsert_grpc_request(
        &w,
        &GrpcRequest {
            workspace_id: workspace_id.to_string(),
            name: name.to_string(),
            folder_id: folder_id.map(|s| s.to_string()),
            sort_priority,
            ..Default::default()
        },
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_duplicate_grpc_request(id: &str, w: WebviewWindow) -> Result<GrpcRequest, String> {
    duplicate_grpc_request(&w, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_create_http_request(
    request: HttpRequest,
    w: WebviewWindow,
) -> Result<HttpRequest, String> {
    upsert_http_request(&w, request).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_duplicate_http_request(id: &str, w: WebviewWindow) -> Result<HttpRequest, String> {
    duplicate_http_request(&w, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_update_workspace(workspace: Workspace, w: WebviewWindow) -> Result<Workspace, String> {
    upsert_workspace(&w, workspace).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_update_environment(
    environment: Environment,
    w: WebviewWindow,
) -> Result<Environment, String> {
    upsert_environment(&w, environment).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_update_grpc_request(
    request: GrpcRequest,
    w: WebviewWindow,
) -> Result<GrpcRequest, String> {
    upsert_grpc_request(&w, &request).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_update_http_request(
    request: HttpRequest,
    window: WebviewWindow,
) -> Result<HttpRequest, String> {
    upsert_http_request(&window, request).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_delete_grpc_request(
    w: WebviewWindow,
    request_id: &str,
) -> Result<GrpcRequest, String> {
    delete_grpc_request(&w, request_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_delete_http_request(
    w: WebviewWindow,
    request_id: &str,
) -> Result<HttpRequest, String> {
    delete_http_request(&w, request_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_list_folders(workspace_id: &str, w: WebviewWindow) -> Result<Vec<Folder>, String> {
    list_folders(&w, workspace_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_create_folder(
    workspace_id: &str,
    name: &str,
    sort_priority: f32,
    folder_id: Option<&str>,
    w: WebviewWindow,
) -> Result<Folder, String> {
    upsert_folder(
        &w,
        Folder {
            workspace_id: workspace_id.to_string(),
            name: name.to_string(),
            folder_id: folder_id.map(|s| s.to_string()),
            sort_priority,
            ..Default::default()
        },
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_update_folder(folder: Folder, w: WebviewWindow) -> Result<Folder, String> {
    upsert_folder(&w, folder).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_write_file_dev(pathname: &str, contents: &str) -> Result<(), String> {
    if !is_dev() {
        panic!("Cannot write arbitrary files when not in dev mode");
    }

    fs::write(pathname, contents).map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_delete_folder(w: WebviewWindow, folder_id: &str) -> Result<Folder, String> {
    delete_folder(&w, folder_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_delete_environment(
    w: WebviewWindow,
    environment_id: &str,
) -> Result<Environment, String> {
    delete_environment(&w, environment_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_list_grpc_connections(
    workspace_id: &str,
    w: WebviewWindow,
) -> Result<Vec<GrpcConnection>, String> {
    list_grpc_connections_for_workspace(&w, workspace_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_list_grpc_events(
    connection_id: &str,
    w: WebviewWindow,
) -> Result<Vec<GrpcEvent>, String> {
    list_grpc_events(&w, connection_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_list_grpc_requests(
    workspace_id: &str,
    w: WebviewWindow,
) -> Result<Vec<GrpcRequest>, String> {
    list_grpc_requests(&w, workspace_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_list_http_requests(
    workspace_id: &str,
    w: WebviewWindow,
) -> Result<Vec<HttpRequest>, String> {
    list_http_requests(&w, workspace_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_list_environments(
    workspace_id: &str,
    w: WebviewWindow,
) -> Result<Vec<Environment>, String> {
    list_environments(&w, workspace_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_list_plugins(w: WebviewWindow) -> Result<Vec<Plugin>, String> {
    list_plugins(&w).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_reload_plugins<R: Runtime>(
    window: WebviewWindow<R>,
    plugin_manager: State<'_, PluginManager>,
) -> Result<(), String> {
    plugin_manager
        .initialize_all_plugins(window.app_handle(), WindowContext::from_window(&window))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn cmd_plugin_info(
    id: &str,
    w: WebviewWindow,
    plugin_manager: State<'_, PluginManager>,
) -> Result<BootResponse, String> {
    let plugin = get_plugin(&w, id).await.map_err(|e| e.to_string())?;
    Ok(plugin_manager
        .get_plugin_by_dir(plugin.directory.as_str())
        .await
        .ok_or("Failed to find plugin for info".to_string())?
        .info()
        .await)
}

#[tauri::command]
async fn cmd_get_settings(w: WebviewWindow) -> Result<Settings, ()> {
    Ok(get_or_create_settings(&w).await)
}

#[tauri::command]
async fn cmd_update_settings(settings: Settings, w: WebviewWindow) -> Result<Settings, String> {
    update_settings(&w, settings).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_get_folder(id: &str, w: WebviewWindow) -> Result<Folder, String> {
    get_folder(&w, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_get_grpc_request(id: &str, w: WebviewWindow) -> Result<Option<GrpcRequest>, String> {
    get_grpc_request(&w, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_get_http_request(id: &str, w: WebviewWindow) -> Result<Option<HttpRequest>, String> {
    get_http_request(&w, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_get_cookie_jar(id: &str, w: WebviewWindow) -> Result<CookieJar, String> {
    get_cookie_jar(&w, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_list_cookie_jars(
    workspace_id: &str,
    w: WebviewWindow,
) -> Result<Vec<CookieJar>, String> {
    let cookie_jars = list_cookie_jars(&w, workspace_id).await.expect("Failed to find cookie jars");

    if cookie_jars.is_empty() {
        let cookie_jar = upsert_cookie_jar(
            &w,
            &CookieJar {
                name: "Default".to_string(),
                workspace_id: workspace_id.to_string(),
                ..Default::default()
            },
        )
        .await
        .expect("Failed to create CookieJar");
        Ok(vec![cookie_jar])
    } else {
        Ok(cookie_jars)
    }
}

#[tauri::command]
async fn cmd_get_environment(id: &str, w: WebviewWindow) -> Result<Environment, String> {
    get_environment(&w, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_get_workspace(id: &str, w: WebviewWindow) -> Result<Workspace, String> {
    get_workspace(&w, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_list_http_responses(
    workspace_id: &str,
    limit: Option<i64>,
    w: WebviewWindow,
) -> Result<Vec<HttpResponse>, String> {
    list_http_responses_for_workspace(&w, workspace_id, limit).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_delete_http_response(id: &str, w: WebviewWindow) -> Result<HttpResponse, String> {
    delete_http_response(&w, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_delete_grpc_connection(id: &str, w: WebviewWindow) -> Result<GrpcConnection, String> {
    delete_grpc_connection(&w, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_delete_all_grpc_connections(request_id: &str, w: WebviewWindow) -> Result<(), String> {
    delete_all_grpc_connections(&w, request_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_delete_send_history(workspace_id: &str, w: WebviewWindow) -> Result<(), String> {
    delete_all_http_responses_for_workspace(&w, workspace_id).await.map_err(|e| e.to_string())?;
    delete_all_grpc_connections_for_workspace(&w, workspace_id).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn cmd_delete_all_http_responses(request_id: &str, w: WebviewWindow) -> Result<(), String> {
    delete_all_http_responses_for_request(&w, request_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_list_workspaces(w: WebviewWindow) -> Result<Vec<Workspace>, String> {
    let workspaces = list_workspaces(&w).await.expect("Failed to find workspaces");
    if workspaces.is_empty() {
        let workspace = upsert_workspace(
            &w,
            Workspace {
                name: "Yaak".to_string(),
                setting_follow_redirects: true,
                setting_validate_certificates: true,
                ..Default::default()
            },
        )
        .await
        .expect("Failed to create Workspace");
        Ok(vec![workspace])
    } else {
        Ok(workspaces)
    }
}

#[tauri::command]
async fn cmd_new_child_window(
    parent_window: WebviewWindow,
    url: &str,
    label: &str,
    title: &str,
    inner_size: (f64, f64),
) -> Result<(), String> {
    let app_handle = parent_window.app_handle();
    let label = format!("{OTHER_WINDOW_PREFIX}_{label}");
    let scale_factor = parent_window.scale_factor().unwrap();

    let current_pos = parent_window.inner_position().unwrap().to_logical::<f64>(scale_factor);
    let current_size = parent_window.inner_size().unwrap().to_logical::<f64>(scale_factor);

    // Position the new window in the middle of the parent
    let position = (
        current_pos.x + current_size.width / 2.0 - inner_size.0 / 2.0,
        current_pos.y + current_size.height / 2.0 - inner_size.1 / 2.0,
    );

    let config = CreateWindowConfig {
        label: label.as_str(),
        title,
        url,
        inner_size,
        position,
    };

    let child_window = create_window(&app_handle, config);

    // NOTE: These listeners will remain active even when the windows close. Unfortunately,
    //   there's no way to unlisten to events for now, so we just have to be defensive.

    {
        let parent_window = parent_window.clone();
        let child_window = child_window.clone();
        child_window.clone().on_window_event(move |e| match e {
            // When the new window is destroyed, bring the other up behind it
            WindowEvent::Destroyed => {
                if let Some(w) = parent_window.get_webview_window(child_window.label()) {
                    w.set_focus().unwrap();
                }
            }
            _ => {}
        });
    }

    {
        let parent_window = parent_window.clone();
        let child_window = child_window.clone();
        parent_window.clone().on_window_event(move |e| match e {
            // When the parent window is closed, close the child
            WindowEvent::CloseRequested { .. } => child_window.destroy().unwrap(),
            // When the parent window is focused, bring the child above
            WindowEvent::Focused(focus) => {
                if *focus {
                    if let Some(w) = parent_window.get_webview_window(child_window.label()) {
                        w.set_focus().unwrap();
                    };
                }
            }
            _ => {}
        });
    }

    Ok(())
}

#[tauri::command]
async fn cmd_new_main_window(app_handle: AppHandle, url: &str) -> Result<(), String> {
    create_main_window(&app_handle, url);
    Ok(())
}

#[tauri::command]
async fn cmd_delete_workspace(w: WebviewWindow, workspace_id: &str) -> Result<Workspace, String> {
    delete_workspace(&w, workspace_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cmd_check_for_updates(
    app_handle: AppHandle,
    yaak_updater: State<'_, Mutex<YaakUpdater>>,
) -> Result<bool, String> {
    let update_mode = get_update_mode(&app_handle).await;
    yaak_updater.lock().await.force_check(&app_handle, update_mode).await.map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder =
        tauri::Builder::default()
            .plugin(
                tauri_plugin_log::Builder::default()
                    .targets([
                        Target::new(TargetKind::Stdout),
                        Target::new(TargetKind::LogDir { file_name: None }),
                        Target::new(TargetKind::Webview),
                    ])
                    .level_for("plugin_runtime", log::LevelFilter::Info)
                    .level_for("cookie_store", log::LevelFilter::Info)
                    .level_for("eventsource_client::event_parser", log::LevelFilter::Info)
                    .level_for("h2", log::LevelFilter::Info)
                    .level_for("hyper", log::LevelFilter::Info)
                    .level_for("hyper_util", log::LevelFilter::Info)
                    .level_for("hyper_rustls", log::LevelFilter::Info)
                    .level_for("reqwest", log::LevelFilter::Info)
                    .level_for("sqlx", log::LevelFilter::Warn)
                    .level_for("tao", log::LevelFilter::Info)
                    .level_for("tokio_util", log::LevelFilter::Info)
                    .level_for("tonic", log::LevelFilter::Info)
                    .level_for("tower", log::LevelFilter::Info)
                    .level_for("tracing", log::LevelFilter::Warn)
                    .level_for("swc_ecma_codegen", log::LevelFilter::Off)
                    .level_for("swc_ecma_transforms_base", log::LevelFilter::Off)
                    .with_colors(ColoredLevelConfig::default())
                    .level(if is_dev() { log::LevelFilter::Trace } else { log::LevelFilter::Info })
                    .build(),
            )
            .plugin(tauri_plugin_clipboard_manager::init())
            .plugin(
                tauri_plugin_window_state::Builder::default()
                    .with_denylist(&["ignored"])
                    .map_label(|label| {
                        if label.starts_with(OTHER_WINDOW_PREFIX) {
                            "ignored"
                        } else {
                            label
                        }
                    })
                    .build(),
            )
            .plugin(tauri_plugin_shell::init())
            .plugin(tauri_plugin_updater::Builder::default().build())
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_os::init())
            .plugin(tauri_plugin_fs::init())
            .plugin(yaak_models::plugin::Builder::default().build())
            .plugin(yaak_plugin_runtime::plugin::init())
            .plugin(tauri_plugin_sync::init());

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_plugin_mac_window::init());
    }

    builder
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().unwrap();
            create_dir_all(app_data_dir.clone()).expect("Problem creating App directory!");

            // Add updater
            let yaak_updater = YaakUpdater::new();
            app.manage(Mutex::new(yaak_updater));

            // Add notifier
            let yaak_notifier = YaakNotifier::new();
            app.manage(Mutex::new(yaak_notifier));

            // Add GRPC manager
            let grpc_handle = GrpcHandle::new(&app.app_handle());
            app.manage(Mutex::new(grpc_handle));

            monitor_plugin_events(&app.app_handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd_call_http_request_action,
            cmd_check_for_updates,
            cmd_create_cookie_jar,
            cmd_create_environment,
            cmd_create_folder,
            cmd_create_grpc_request,
            cmd_create_http_request,
            cmd_create_workspace,
            cmd_curl_to_request,
            cmd_delete_all_grpc_connections,
            cmd_delete_all_http_responses,
            cmd_delete_cookie_jar,
            cmd_delete_environment,
            cmd_delete_folder,
            cmd_delete_grpc_connection,
            cmd_delete_grpc_request,
            cmd_delete_http_request,
            cmd_delete_http_response,
            cmd_delete_send_history,
            cmd_delete_workspace,
            cmd_dismiss_notification,
            cmd_duplicate_grpc_request,
            cmd_duplicate_http_request,
            cmd_export_data,
            cmd_filter_response,
            cmd_format_json,
            cmd_get_cookie_jar,
            cmd_get_environment,
            cmd_get_folder,
            cmd_get_grpc_request,
            cmd_get_http_request,
            cmd_get_key_value,
            cmd_get_settings,
            cmd_get_sse_events,
            cmd_get_workspace,
            cmd_grpc_go,
            cmd_grpc_reflect,
            cmd_http_request_actions,
            cmd_import_data,
            cmd_install_plugin,
            cmd_list_cookie_jars,
            cmd_list_environments,
            cmd_list_folders,
            cmd_list_grpc_connections,
            cmd_list_grpc_events,
            cmd_list_grpc_requests,
            cmd_list_http_requests,
            cmd_list_http_responses,
            cmd_list_plugins,
            cmd_list_workspaces,
            cmd_metadata,
            cmd_new_child_window,
            cmd_new_main_window,
            cmd_parse_template,
            cmd_plugin_info,
            cmd_reload_plugins,
            cmd_render_template,
            cmd_save_response,
            cmd_send_ephemeral_request,
            cmd_send_http_request,
            cmd_set_key_value,
            cmd_set_update_mode,
            cmd_template_functions,
            cmd_template_tokens_to_string,
            cmd_track_event,
            cmd_uninstall_plugin,
            cmd_update_cookie_jar,
            cmd_update_environment,
            cmd_update_folder,
            cmd_update_grpc_request,
            cmd_update_http_request,
            cmd_update_settings,
            cmd_update_workspace,
            cmd_write_file_dev,
        ])
        .register_uri_scheme_protocol("yaak", |_app, _req| {
            debug!("Testing yaak protocol");
            tauri::http::Response::builder().body("Success".as_bytes().to_vec()).unwrap()
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            match event {
                RunEvent::Ready => {
                    let w = create_main_window(app_handle, "/");
                    tauri::async_runtime::spawn(async move {
                        let info = analytics::track_launch_event(&w).await;
                        debug!("Launched Yaak {:?}", info);
                    });

                    // Cancel pending requests
                    let h = app_handle.clone();
                    tauri::async_runtime::block_on(async move {
                        let _ = cancel_pending_responses(&h).await;
                        let _ = cancel_pending_grpc_connections(&h).await;
                    });
                }
                RunEvent::WindowEvent {
                    event: WindowEvent::Focused(true),
                    ..
                } => {
                    let h = app_handle.clone();
                    // Run update check whenever the window is focused
                    tauri::async_runtime::spawn(async move {
                        let update_mode = get_update_mode(&h).await;
                        let updater: State<'_, Mutex<YaakUpdater>> = h.state();
                        let mut updater = updater.lock().await;
                        if let Err(e) = updater.check(&h, update_mode).await {
                            warn!("Failed to check for updates {e:?}");
                        }
                    });

                    let h = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        let windows = h.webview_windows();
                        let w = windows.values().next().unwrap();
                        tokio::time::sleep(Duration::from_millis(4000)).await;
                        let val: State<'_, Mutex<YaakNotifier>> = w.state();
                        let mut n = val.lock().await;
                        if let Err(e) = n.check(&w).await {
                            warn!("Failed to check for notifications {}", e)
                        }
                    });
                }
                _ => {}
            };
        });
}

fn is_dev() -> bool {
    #[cfg(dev)]
    {
        return true;
    }
    #[cfg(not(dev))]
    {
        return false;
    }
}

fn create_main_window(handle: &AppHandle, url: &str) -> WebviewWindow {
    let label = format!("{MAIN_WINDOW_PREFIX}{}", handle.webview_windows().len());
    let config = CreateWindowConfig {
        url,
        label: label.as_str(),
        title: "Yaak",
        inner_size: (DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT),
        position: (
            // Offset by random amount so it's easier to differentiate
            100.0 + random::<f64>() * 20.0,
            100.0 + random::<f64>() * 20.0,
        ),
    };
    create_window(handle, config)
}

struct CreateWindowConfig<'s> {
    url: &'s str,
    label: &'s str,
    title: &'s str,
    inner_size: (f64, f64),
    position: (f64, f64),
}

fn create_window(handle: &AppHandle, config: CreateWindowConfig) -> WebviewWindow {
    #[allow(unused_variables)]
    let menu = app_menu(handle).unwrap();

    // This causes the window to not be clickable (in AppImage), so disable on Linux
    #[cfg(not(target_os = "linux"))]
    handle.set_menu(menu).expect("Failed to set app menu");

    info!("Create new window label={}", config.label);

    let mut win_builder =
        tauri::WebviewWindowBuilder::new(handle, config.label, WebviewUrl::App(config.url.into()))
            .title(config.title)
            .resizable(true)
            .visible(false) // To prevent theme flashing, the frontend code calls show() immediately after configuring the theme
            .fullscreen(false)
            .disable_drag_drop_handler() // Required for frontend Dnd on windows
            .inner_size(config.inner_size.0, config.inner_size.1)
            .position(config.position.0, config.position.1)
            .min_inner_size(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT);

    // Add macOS-only things
    #[cfg(target_os = "macos")]
    {
        win_builder = win_builder.hidden_title(true).title_bar_style(TitleBarStyle::Overlay);
    }

    // Add non-MacOS things
    #[cfg(not(target_os = "macos"))]
    {
        // Doesn't seem to work from Rust, here, so we do it in main.tsx
        win_builder = win_builder.decorations(false);
    }

    if let Some(w) = handle.webview_windows().get(config.label) {
        info!("Webview with label {} already exists. Focusing existing", config.label);
        w.set_focus().unwrap();
        return w.to_owned();
    }

    let win = win_builder.build().unwrap();

    let webview_window = win.clone();
    win.on_menu_event(move |w, event| {
        if !w.is_focused().unwrap() {
            return;
        }

        let event_id = event.id().0.as_str();
        match event_id {
            "quit" => exit(0),
            "close" => w.close().unwrap(),
            "zoom_reset" => w.emit("zoom_reset", true).unwrap(),
            "zoom_in" => w.emit("zoom_in", true).unwrap(),
            "zoom_out" => w.emit("zoom_out", true).unwrap(),
            "settings" => w.emit("settings", true).unwrap(),
            "open_feedback" => {
                if let Err(e) =
                    webview_window.app_handle().shell().open("https://yaak.app/feedback", None)
                {
                    warn!("Failed to open feedback {e:?}");
                }
            }

            // Commands for development
            "dev.reset_size" => webview_window
                .set_size(LogicalSize::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT))
                .unwrap(),
            "dev.refresh" => webview_window.eval("location.reload()").unwrap(),
            "dev.generate_theme_css" => {
                w.emit("generate_theme_css", true).unwrap();
            }
            "dev.toggle_devtools" => {
                if webview_window.is_devtools_open() {
                    webview_window.close_devtools();
                } else {
                    webview_window.open_devtools();
                }
            }
            _ => {}
        }
    });

    win
}

async fn get_update_mode(h: &AppHandle) -> UpdateMode {
    let settings = get_or_create_settings(h).await;
    UpdateMode::new(settings.update_channel.as_str())
}

fn safe_uri(endpoint: &str) -> String {
    if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        endpoint.into()
    } else {
        format!("http://{}", endpoint)
    }
}

fn monitor_plugin_events<R: Runtime>(app_handle: &AppHandle<R>) {
    let app_handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let plugin_manager: State<'_, PluginManager> = app_handle.state();
        let (rx_id, mut rx) = plugin_manager.subscribe("app").await;

        while let Some(event) = rx.recv().await {
            let app_handle = app_handle.clone();
            let plugin =
                match plugin_manager.get_plugin_by_ref_id(event.plugin_ref_id.as_str()).await {
                    None => {
                        warn!("Failed to get plugin for event {:?}", event);
                        continue;
                    }
                    Some(p) => p,
                };

            // We might have recursive back-and-forth calls between app and plugin, so we don't
            // want to block here
            tauri::async_runtime::spawn(async move {
                handle_plugin_event(&app_handle, &event, &plugin).await;
            });
        }
        plugin_manager.unsubscribe(rx_id.as_str()).await;
    });
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FrontendCall<T: Serialize + Clone> {
    args: T,
    reply_id: String,
}

async fn call_frontend<T: Serialize + Clone, R: Runtime>(
    window: WebviewWindow<R>,
    event_name: &str,
    args: T,
) -> PromptTextResponse {
    let reply_id = format!("{event_name}_reply_{}", generate_id());
    let payload = FrontendCall {
        args,
        reply_id: reply_id.clone(),
    };
    window.emit_to(window.label(), event_name, payload).unwrap();
    let (tx, mut rx) = tokio::sync::watch::channel(PromptTextResponse::default());

    let event_id = window.clone().listen(reply_id, move |ev| {
        let resp: PromptTextResponse = serde_json::from_str(ev.payload()).unwrap();
        if let Err(e) = tx.send(resp) {
            warn!("Failed to prompt for text {e:?}");
        }
    });

    // When reply shows up, unlisten to events and return
    if let Err(e) = rx.changed().await {
        warn!("Failed to check channel changed {e:?}");
    }
    window.unlisten(event_id);

    let foo = rx.borrow();
    foo.clone()
}

async fn handle_plugin_event<R: Runtime>(
    app_handle: &AppHandle<R>,
    event: &InternalEvent,
    plugin_handle: &PluginHandle,
) {
    // info!("Got event to app {}", event.id);
    let window_context = event.window_context.to_owned();
    let response_event: Option<InternalEventPayload> = match event.clone().payload {
        InternalEventPayload::CopyTextRequest(req) => {
            app_handle
                .clipboard()
                .write_text(req.text.as_str())
                .expect("Failed to write text to clipboard");
            None
        }
        InternalEventPayload::ShowToastRequest(req) => {
            match window_context {
                WindowContext::Label { label } => app_handle
                    .emit_to(label, "show_toast", req)
                    .expect("Failed to emit show_toast to window"),
                _ => app_handle.emit("show_toast", req).expect("Failed to emit show_toast"),
            };
            None
        }
        InternalEventPayload::PromptTextRequest(req) => {
            let window = get_window_from_window_context(app_handle, &window_context)
                .expect("Failed to find window for render");
            let resp = call_frontend(window, "show_prompt", req).await;
            Some(InternalEventPayload::PromptTextResponse(resp))
        }
        InternalEventPayload::FindHttpResponsesRequest(req) => {
            let http_responses = list_http_responses_for_request(
                app_handle,
                req.request_id.as_str(),
                req.limit.map(|l| l as i64),
            )
            .await
            .unwrap_or_default();
            Some(InternalEventPayload::FindHttpResponsesResponse(FindHttpResponsesResponse {
                http_responses,
            }))
        }
        InternalEventPayload::GetHttpRequestByIdRequest(req) => {
            let http_request = get_http_request(app_handle, req.id.as_str()).await.unwrap();
            Some(InternalEventPayload::GetHttpRequestByIdResponse(GetHttpRequestByIdResponse {
                http_request,
            }))
        }
        InternalEventPayload::RenderHttpRequestRequest(req) => {
            let window = get_window_from_window_context(app_handle, &window_context)
                .expect("Failed to find window for render http request");

            let workspace = workspace_from_window(&window)
                .await
                .expect("Failed to get workspace_id from window URL");
            let environment = environment_from_window(&window).await;
            let cb = PluginTemplateCallback::new(app_handle, &window_context, req.purpose);
            let http_request =
                render_http_request(&req.http_request, &workspace, environment.as_ref(), &cb).await;
            Some(InternalEventPayload::RenderHttpRequestResponse(RenderHttpRequestResponse {
                http_request,
            }))
        }
        InternalEventPayload::TemplateRenderRequest(req) => {
            let window = get_window_from_window_context(app_handle, &window_context)
                .expect("Failed to find window for render");

            let workspace = workspace_from_window(&window)
                .await
                .expect("Failed to get workspace_id from window URL");
            let environment = environment_from_window(&window).await;
            let cb = PluginTemplateCallback::new(app_handle, &window_context, req.purpose);
            let data = render_json_value(req.data, &workspace, environment.as_ref(), &cb).await;
            Some(InternalEventPayload::TemplateRenderResponse(TemplateRenderResponse { data }))
        }
        InternalEventPayload::ReloadResponse => {
            let window = get_window_from_window_context(app_handle, &window_context)
                .expect("Failed to find window for plugin reload");
            let plugins = list_plugins(app_handle).await.unwrap();
            for plugin in plugins {
                if plugin.directory != plugin_handle.dir {
                    continue;
                }

                let new_plugin = Plugin {
                    updated_at: Utc::now().naive_utc(), // TODO: Add reloaded_at field to use instead
                    ..plugin
                };
                upsert_plugin(&window, new_plugin).await.unwrap();
            }
            let toast_event = plugin_handle.build_event_to_send(
                WindowContext::from_window(&window),
                &InternalEventPayload::ShowToastRequest(ShowToastRequest {
                    message: format!("Reloaded plugin {}", plugin_handle.dir),
                    icon: Some(Icon::Info),
                    ..Default::default()
                }),
                None,
            );
            Box::pin(handle_plugin_event(app_handle, &toast_event, plugin_handle)).await;
            None
        }
        InternalEventPayload::SendHttpRequestRequest(req) => {
            let window = get_window_from_window_context(app_handle, &window_context)
                .expect("Failed to find window for sending HTTP request");
            let cookie_jar = cookie_jar_from_window(&window).await;
            let environment = environment_from_window(&window).await;

            let resp =
                create_default_http_response(&window, req.http_request.id.as_str()).await.unwrap();

            let result = send_http_request(
                &window,
                &req.http_request,
                &resp,
                environment,
                cookie_jar,
                &mut tokio::sync::watch::channel(false).1, // No-op cancel channel
            )
            .await;

            let http_response = match result {
                Ok(r) => r,
                Err(_e) => return,
            };

            Some(InternalEventPayload::SendHttpRequestResponse(SendHttpRequestResponse {
                http_response,
            }))
        }
        _ => None,
    };

    if let Some(e) = response_event {
        let plugin_manager: State<'_, PluginManager> = app_handle.state();
        if let Err(e) = plugin_manager.reply(&event, &e).await {
            warn!("Failed to reply to plugin manager: {:?}", e)
        }
    }
}

fn get_window_from_window_context<R: Runtime>(
    app_handle: &AppHandle<R>,
    window_context: &WindowContext,
) -> Option<WebviewWindow<R>> {
    let label = match window_context {
        WindowContext::Label { label } => label,
        WindowContext::None => {
            return app_handle.webview_windows().iter().next().map(|(_, w)| w.to_owned());
        }
    };

    let window = app_handle.webview_windows().iter().find_map(|(_, w)| {
        if w.label() == label {
            Some(w.to_owned())
        } else {
            None
        }
    });

    if window.is_none() {
        error!("Failed to find window by {window_context:?}");
    }

    window
}

fn workspace_id_from_window<R: Runtime>(window: &WebviewWindow<R>) -> Option<String> {
    let url = window.url().unwrap();
    let re = Regex::new(r"/workspaces/(?<wid>\w+)").unwrap();
    match re.captures(url.as_str()) {
        None => None,
        Some(captures) => captures.name("wid").map(|c| c.as_str().to_string()),
    }
}

async fn workspace_from_window<R: Runtime>(window: &WebviewWindow<R>) -> Option<Workspace> {
    match workspace_id_from_window(&window) {
        None => None,
        Some(id) => get_workspace(window, id.as_str()).await.ok(),
    }
}

fn environment_id_from_window<R: Runtime>(window: &WebviewWindow<R>) -> Option<String> {
    let url = window.url().unwrap();
    let mut query_pairs = url.query_pairs();
    query_pairs.find(|(k, _v)| k == "environment_id").map(|(_k, v)| v.to_string())
}

async fn environment_from_window<R: Runtime>(window: &WebviewWindow<R>) -> Option<Environment> {
    match environment_id_from_window(&window) {
        None => None,
        Some(id) => get_environment(window, id.as_str()).await.ok(),
    }
}

fn cookie_jar_id_from_window<R: Runtime>(window: &WebviewWindow<R>) -> Option<String> {
    let url = window.url().unwrap();
    let mut query_pairs = url.query_pairs();
    query_pairs.find(|(k, _v)| k == "cookie_jar_id").map(|(_k, v)| v.to_string())
}

async fn cookie_jar_from_window<R: Runtime>(window: &WebviewWindow<R>) -> Option<CookieJar> {
    match cookie_jar_id_from_window(&window) {
        None => None,
        Some(id) => get_cookie_jar(window, id.as_str()).await.ok(),
    }
}
