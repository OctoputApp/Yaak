use chrono::NaiveDateTime;
use rusqlite::Row;
use sea_query::Iden;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", tag = "type")]
#[ts(export, export_to = "models.ts")]
pub enum ProxySetting {
    Enabled {
        http: String,
        https: String,
        auth: Option<ProxySettingAuth>,
    },
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct ProxySettingAuth {
    pub user: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct Settings {
    #[ts(type = "\"settings\"")]
    pub model: String,
    pub id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,

    pub appearance: String,
    pub editor_font_size: i32,
    pub editor_soft_wrap: bool,
    pub interface_font_size: i32,
    pub interface_scale: f32,
    pub open_workspace_new_window: Option<bool>,
    pub telemetry: bool,
    pub theme: String,
    pub theme_dark: String,
    pub theme_light: String,
    pub update_channel: String,
    pub proxy: Option<ProxySetting>,
}

#[derive(Iden)]
pub enum SettingsIden {
    #[iden = "settings"]
    Table,
    Model,
    Id,
    CreatedAt,
    UpdatedAt,

    Appearance,
    EditorFontSize,
    EditorSoftWrap,
    InterfaceFontSize,
    InterfaceScale,
    OpenWorkspaceNewWindow,
    Proxy,
    Telemetry,
    Theme,
    ThemeDark,
    ThemeLight,
    UpdateChannel,
}

impl<'s> TryFrom<&Row<'s>> for Settings {
    type Error = rusqlite::Error;

    fn try_from(r: &Row<'s>) -> Result<Self, Self::Error> {
        let proxy: Option<String> = r.get("proxy")?;
        Ok(Settings {
            id: r.get("id")?,
            model: r.get("model")?,
            created_at: r.get("created_at")?,
            updated_at: r.get("updated_at")?,
            appearance: r.get("appearance")?,
            editor_font_size: r.get("editor_font_size")?,
            editor_soft_wrap: r.get("editor_soft_wrap")?,
            interface_font_size: r.get("interface_font_size")?,
            interface_scale: r.get("interface_scale")?,
            open_workspace_new_window: r.get("open_workspace_new_window")?,
            proxy: proxy.map(|p| -> ProxySetting { serde_json::from_str(p.as_str()).unwrap() }),
            telemetry: r.get("telemetry")?,
            theme: r.get("theme")?,
            theme_dark: r.get("theme_dark")?,
            theme_light: r.get("theme_light")?,
            update_channel: r.get("update_channel")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct Workspace {
    #[ts(type = "\"workspace\"")]
    pub model: String,
    pub id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub name: String,
    pub description: String,
    pub variables: Vec<EnvironmentVariable>,

    // Settings
    #[serde(default = "default_true")]
    pub setting_validate_certificates: bool,
    #[serde(default = "default_true")]
    pub setting_follow_redirects: bool,
    pub setting_request_timeout: i32,
}

#[derive(Iden)]
pub enum WorkspaceIden {
    #[iden = "workspaces"]
    Table,
    Model,
    Id,
    CreatedAt,
    UpdatedAt,

    Description,
    Name,
    SettingFollowRedirects,
    SettingRequestTimeout,
    SettingValidateCertificates,
    Variables,
}

impl<'s> TryFrom<&Row<'s>> for Workspace {
    type Error = rusqlite::Error;

    fn try_from(r: &Row<'s>) -> Result<Self, Self::Error> {
        let variables: String = r.get("variables")?;
        Ok(Workspace {
            id: r.get("id")?,
            model: r.get("model")?,
            created_at: r.get("created_at")?,
            updated_at: r.get("updated_at")?,
            name: r.get("name")?,
            description: r.get("description")?,
            variables: json_col(variables.as_str()),
            setting_validate_certificates: r.get("setting_validate_certificates")?,
            setting_follow_redirects: r.get("setting_follow_redirects")?,
            setting_request_timeout: r.get("setting_request_timeout")?,
        })
    }
}

impl Workspace {
    pub fn new(name: String) -> Self {
        Self {
            name,
            model: "workspace".to_string(),
            setting_validate_certificates: true,
            setting_follow_redirects: true,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "models.ts")]
enum CookieDomain {
    HostOnly(String),
    Suffix(String),
    NotPresent,
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "models.ts")]
enum CookieExpires {
    AtUtc(String),
    SessionEnd,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "models.ts")]
pub struct Cookie {
    raw_cookie: String,
    domain: CookieDomain,
    expires: CookieExpires,
    path: (String, bool),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct CookieJar {
    #[ts(type = "\"cookie_jar\"")]
    pub model: String,
    pub id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub workspace_id: String,

    pub cookies: Vec<Cookie>,
    pub name: String,
}

#[derive(Iden)]
pub enum CookieJarIden {
    #[iden = "cookie_jars"]
    Table,
    Id,
    Model,
    WorkspaceId,
    CreatedAt,
    UpdatedAt,

    Cookies,
    Name,
}

impl<'s> TryFrom<&Row<'s>> for CookieJar {
    type Error = rusqlite::Error;

    fn try_from(r: &Row<'s>) -> Result<Self, Self::Error> {
        let cookies: String = r.get("cookies")?;
        Ok(CookieJar {
            id: r.get("id")?,
            model: r.get("model")?,
            workspace_id: r.get("workspace_id")?,
            created_at: r.get("created_at")?,
            updated_at: r.get("updated_at")?,
            name: r.get("name")?,
            cookies: json_col(cookies.as_str()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct Environment {
    #[ts(type = "\"environment\"")]
    pub model: String,
    pub id: String,
    pub workspace_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,

    pub name: String,
    pub variables: Vec<EnvironmentVariable>,
}

#[derive(Iden)]
pub enum EnvironmentIden {
    #[iden = "environments"]
    Table,
    Model,
    Id,
    CreatedAt,
    UpdatedAt,
    WorkspaceId,

    Name,
    Variables,
}

impl<'s> TryFrom<&Row<'s>> for Environment {
    type Error = rusqlite::Error;

    fn try_from(r: &Row<'s>) -> Result<Self, Self::Error> {
        let variables: String = r.get("variables")?;
        Ok(Environment {
            id: r.get("id")?,
            model: r.get("model")?,
            workspace_id: r.get("workspace_id")?,
            created_at: r.get("created_at")?,
            updated_at: r.get("updated_at")?,
            name: r.get("name")?,
            variables: json_col(variables.as_str()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct EnvironmentVariable {
    #[serde(default = "default_true")]
    #[ts(optional, as = "Option<bool>")]
    pub enabled: bool,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct Folder {
    #[ts(type = "\"folder\"")]
    pub model: String,
    pub id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub workspace_id: String,
    pub folder_id: Option<String>,

    pub name: String,
    pub sort_priority: f32,
}

#[derive(Iden)]
pub enum FolderIden {
    #[iden = "folders"]
    Table,
    Id,
    Model,
    WorkspaceId,
    FolderId,
    CreatedAt,
    UpdatedAt,

    Name,
    SortPriority,
}

impl<'s> TryFrom<&Row<'s>> for Folder {
    type Error = rusqlite::Error;

    fn try_from(r: &Row<'s>) -> Result<Self, Self::Error> {
        Ok(Folder {
            id: r.get("id")?,
            model: r.get("model")?,
            sort_priority: r.get("sort_priority")?,
            workspace_id: r.get("workspace_id")?,
            created_at: r.get("created_at")?,
            updated_at: r.get("updated_at")?,
            folder_id: r.get("folder_id")?,
            name: r.get("name")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct HttpRequestHeader {
    #[serde(default = "default_true")]
    #[ts(optional, as = "Option<bool>")]
    pub enabled: bool,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct HttpUrlParameter {
    #[serde(default = "default_true")]
    #[ts(optional, as = "Option<bool>")]
    pub enabled: bool,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct HttpRequest {
    #[ts(type = "\"http_request\"")]
    pub model: String,
    pub id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub workspace_id: String,
    pub folder_id: Option<String>,

    #[ts(type = "Record<string, any>")]
    pub authentication: BTreeMap<String, Value>,
    pub authentication_type: Option<String>,
    #[ts(type = "Record<string, any>")]
    pub body: BTreeMap<String, Value>,
    pub body_type: Option<String>,
    pub headers: Vec<HttpRequestHeader>,
    #[serde(default = "default_http_request_method")]
    pub method: String,
    pub name: String,
    pub sort_priority: f32,
    pub url: String,
    pub url_parameters: Vec<HttpUrlParameter>,
}

#[derive(Iden)]
pub enum HttpRequestIden {
    #[iden = "http_requests"]
    Table,
    Id,
    Model,
    CreatedAt,
    UpdatedAt,
    WorkspaceId,
    FolderId,

    Authentication,
    AuthenticationType,
    Body,
    BodyType,
    Headers,
    Method,
    Name,
    SortPriority,
    Url,
    UrlParameters,
}

impl<'s> TryFrom<&Row<'s>> for HttpRequest {
    type Error = rusqlite::Error;

    fn try_from(r: &Row<'s>) -> Result<Self, Self::Error> {
        let url_parameters: String = r.get("url_parameters")?;
        let body: String = r.get("body")?;
        let authentication: String = r.get("authentication")?;
        let headers: String = r.get("headers")?;
        Ok(HttpRequest {
            id: r.get("id")?,
            model: r.get("model")?,
            sort_priority: r.get("sort_priority")?,
            workspace_id: r.get("workspace_id")?,
            created_at: r.get("created_at")?,
            updated_at: r.get("updated_at")?,
            url: r.get("url")?,
            url_parameters: json_col(url_parameters.as_str()),
            method: r.get("method")?,
            body: json_col(body.as_str()),
            body_type: r.get("body_type")?,
            authentication: json_col(authentication.as_str()),
            authentication_type: r.get("authentication_type")?,
            headers: json_col(headers.as_str()),
            folder_id: r.get("folder_id")?,
            name: r.get("name")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct HttpResponseHeader {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "models.ts")]
pub enum HttpResponseState {
    Initialized,
    Connected,
    Closed,
}

impl Default for HttpResponseState {
    fn default() -> Self {
        Self::Initialized
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct HttpResponse {
    #[ts(type = "\"http_response\"")]
    pub model: String,
    pub id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub workspace_id: String,
    pub request_id: String,

    pub body_path: Option<String>,
    pub content_length: Option<i32>,
    pub elapsed: i32,
    pub elapsed_headers: i32,
    pub error: Option<String>,
    pub headers: Vec<HttpResponseHeader>,
    pub remote_addr: Option<String>,
    pub status: i32,
    pub status_reason: Option<String>,
    pub state: HttpResponseState,
    pub url: String,
    pub version: Option<String>,
}

#[derive(Iden)]
pub enum HttpResponseIden {
    #[iden = "http_responses"]
    Table,
    Model,
    Id,
    CreatedAt,
    UpdatedAt,
    WorkspaceId,
    RequestId,

    BodyPath,
    ContentLength,
    Elapsed,
    ElapsedHeaders,
    Error,
    Headers,
    RemoteAddr,
    Status,
    StatusReason,
    State,
    Url,
    Version,
}

impl<'s> TryFrom<&Row<'s>> for HttpResponse {
    type Error = rusqlite::Error;

    fn try_from(r: &Row<'s>) -> Result<Self, Self::Error> {
        let headers: String = r.get("headers")?;
        let state: String = r.get("state")?;
        Ok(HttpResponse {
            id: r.get("id")?,
            model: r.get("model")?,
            workspace_id: r.get("workspace_id")?,
            request_id: r.get("request_id")?,
            created_at: r.get("created_at")?,
            updated_at: r.get("updated_at")?,
            error: r.get("error")?,
            url: r.get("url")?,
            content_length: r.get("content_length")?,
            version: r.get("version")?,
            elapsed: r.get("elapsed")?,
            elapsed_headers: r.get("elapsed_headers")?,
            remote_addr: r.get("remote_addr")?,
            status: r.get("status")?,
            status_reason: r.get("status_reason")?,
            state: serde_json::from_str(format!(r#""{state}""#).as_str()).unwrap(),
            body_path: r.get("body_path")?,
            headers: json_col(headers.as_str()),
        })
    }
}

impl HttpResponse {
    pub fn new() -> Self {
        Self {
            model: "http_response".to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct GrpcMetadataEntry {
    #[serde(default = "default_true")]
    #[ts(optional, as = "Option<bool>")]
    pub enabled: bool,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct GrpcRequest {
    #[ts(type = "\"grpc_request\"")]
    pub model: String,
    pub id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub workspace_id: String,
    pub folder_id: Option<String>,

    pub authentication_type: Option<String>,
    #[ts(type = "Record<string, any>")]
    pub authentication: BTreeMap<String, Value>,
    pub message: String,
    pub metadata: Vec<GrpcMetadataEntry>,
    pub method: Option<String>,
    pub name: String,
    pub service: Option<String>,
    pub sort_priority: f32,
    pub url: String,
}

#[derive(Iden)]
pub enum GrpcRequestIden {
    #[iden = "grpc_requests"]
    Table,
    Id,
    Model,
    CreatedAt,
    UpdatedAt,
    WorkspaceId,
    FolderId,

    Authentication,
    AuthenticationType,
    Message,
    Metadata,
    Method,
    Name,
    Service,
    SortPriority,
    Url,
}

impl<'s> TryFrom<&Row<'s>> for GrpcRequest {
    type Error = rusqlite::Error;

    fn try_from(r: &Row<'s>) -> Result<Self, Self::Error> {
        let authentication: String = r.get("authentication")?;
        let metadata: String = r.get("metadata")?;
        Ok(GrpcRequest {
            id: r.get("id")?,
            model: r.get("model")?,
            workspace_id: r.get("workspace_id")?,
            created_at: r.get("created_at")?,
            updated_at: r.get("updated_at")?,
            folder_id: r.get("folder_id")?,
            name: r.get("name")?,
            service: r.get("service")?,
            method: r.get("method")?,
            message: r.get("message")?,
            authentication_type: r.get("authentication_type")?,
            authentication: json_col(authentication.as_str()),
            url: r.get("url")?,
            sort_priority: r.get("sort_priority")?,
            metadata: json_col(metadata.as_str()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "models.ts")]
pub enum GrpcConnectionState {
    Initialized,
    Connected,
    Closed,
}

impl Default for GrpcConnectionState {
    fn default() -> Self {
        Self::Initialized
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct GrpcConnection {
    #[ts(type = "\"grpc_connection\"")]
    pub model: String,
    pub id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub workspace_id: String,
    pub request_id: String,

    pub elapsed: i32,
    pub error: Option<String>,
    pub method: String,
    pub service: String,
    pub status: i32,
    pub state: GrpcConnectionState,
    pub trailers: BTreeMap<String, String>,
    pub url: String,
}

#[derive(Iden)]
pub enum GrpcConnectionIden {
    #[iden = "grpc_connections"]
    Table,
    Model,
    Id,
    CreatedAt,
    UpdatedAt,
    WorkspaceId,
    RequestId,

    Elapsed,
    Error,
    Method,
    Service,
    State,
    Status,
    Trailers,
    Url,
}

impl<'s> TryFrom<&Row<'s>> for GrpcConnection {
    type Error = rusqlite::Error;

    fn try_from(r: &Row<'s>) -> Result<Self, Self::Error> {
        let trailers: String = r.get("trailers")?;
        let state: String = r.get("state")?;
        Ok(GrpcConnection {
            id: r.get("id")?,
            model: r.get("model")?,
            workspace_id: r.get("workspace_id")?,
            request_id: r.get("request_id")?,
            created_at: r.get("created_at")?,
            updated_at: r.get("updated_at")?,
            service: r.get("service")?,
            method: r.get("method")?,
            elapsed: r.get("elapsed")?,
            state: serde_json::from_str(format!(r#""{state}""#).as_str()).unwrap(),
            status: r.get("status")?,
            url: r.get("url")?,
            error: r.get("error")?,
            trailers: json_col(trailers.as_str()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "models.ts")]
pub enum GrpcEventType {
    Info,
    Error,
    ClientMessage,
    ServerMessage,
    ConnectionStart,
    ConnectionEnd,
}

impl Default for GrpcEventType {
    fn default() -> Self {
        GrpcEventType::Info
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct GrpcEvent {
    #[ts(type = "\"grpc_event\"")]
    pub model: String,
    pub id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub workspace_id: String,
    pub request_id: String,
    pub connection_id: String,

    pub content: String,
    pub error: Option<String>,
    pub event_type: GrpcEventType,
    pub metadata: BTreeMap<String, String>,
    pub status: Option<i32>,
}

#[derive(Iden)]
pub enum GrpcEventIden {
    #[iden = "grpc_events"]
    Table,
    Model,
    Id,
    CreatedAt,
    UpdatedAt,
    WorkspaceId,
    RequestId,
    ConnectionId,

    Content,
    Error,
    EventType,
    Metadata,
    Status,
}

impl<'s> TryFrom<&Row<'s>> for GrpcEvent {
    type Error = rusqlite::Error;

    fn try_from(r: &Row<'s>) -> Result<Self, Self::Error> {
        let event_type: String = r.get("event_type")?;
        let metadata: String = r.get("metadata")?;
        Ok(GrpcEvent {
            id: r.get("id")?,
            model: r.get("model")?,
            workspace_id: r.get("workspace_id")?,
            request_id: r.get("request_id")?,
            connection_id: r.get("connection_id")?,
            created_at: r.get("created_at")?,
            updated_at: r.get("updated_at")?,
            content: r.get("content")?,
            event_type: json_col(event_type.as_str()),
            metadata: json_col(metadata.as_str()),
            status: r.get("status")?,
            error: r.get("error")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct Plugin {
    #[ts(type = "\"plugin\"")]
    pub model: String,
    pub id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,

    pub checked_at: Option<NaiveDateTime>,
    pub directory: String,
    pub enabled: bool,
    pub url: Option<String>,
}

#[derive(Iden)]
pub enum PluginIden {
    #[iden = "plugins"]
    Table,
    Model,
    Id,
    CreatedAt,
    UpdatedAt,

    CheckedAt,
    Directory,
    Enabled,
    Url,
}

impl<'s> TryFrom<&Row<'s>> for Plugin {
    type Error = rusqlite::Error;

    fn try_from(r: &Row<'s>) -> Result<Self, Self::Error> {
        Ok(Plugin {
            id: r.get("id")?,
            model: r.get("model")?,
            created_at: r.get("created_at")?,
            updated_at: r.get("updated_at")?,
            checked_at: r.get("checked_at")?,
            url: r.get("url")?,
            directory: r.get("directory")?,
            enabled: r.get("enabled")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct KeyValue {
    #[ts(type = "\"key_value\"")]
    pub model: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,

    pub key: String,
    pub namespace: String,
    pub value: String,
}

#[derive(Iden)]
pub enum KeyValueIden {
    #[iden = "key_values"]
    Table,
    Model,
    CreatedAt,
    UpdatedAt,

    Key,
    Namespace,
    Value,
}

impl<'s> TryFrom<&Row<'s>> for KeyValue {
    type Error = rusqlite::Error;

    fn try_from(r: &Row<'s>) -> Result<Self, Self::Error> {
        Ok(KeyValue {
            model: r.get("model")?,
            created_at: r.get("created_at")?,
            updated_at: r.get("updated_at")?,
            namespace: r.get("namespace")?,
            key: r.get("key")?,
            value: r.get("value")?,
        })
    }
}

fn default_true() -> bool {
    true
}

fn default_http_request_method() -> String {
    "GET".to_string()
}

pub enum ModelType {
    TypeCookieJar,
    TypeEnvironment,
    TypeFolder,
    TypeGrpcConnection,
    TypeGrpcEvent,
    TypeGrpcRequest,
    TypeHttpRequest,
    TypeHttpResponse,
    TypePlugin,
    TypeWorkspace,
}

impl ModelType {
    pub fn id_prefix(&self) -> String {
        match self {
            ModelType::TypeCookieJar => "cj",
            ModelType::TypeEnvironment => "ev",
            ModelType::TypeFolder => "fl",
            ModelType::TypeGrpcConnection => "gc",
            ModelType::TypeGrpcEvent => "ge",
            ModelType::TypeGrpcRequest => "gr",
            ModelType::TypeHttpRequest => "rq",
            ModelType::TypeHttpResponse => "rs",
            ModelType::TypePlugin => "pg",
            ModelType::TypeWorkspace => "wk",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase", untagged)]
#[ts(export, export_to = "models.ts")]
pub enum AnyModel {
    CookieJar(CookieJar),
    Environment(Environment),
    Folder(Folder),
    GrpcConnection(GrpcConnection),
    GrpcEvent(GrpcEvent),
    GrpcRequest(GrpcRequest),
    HttpRequest(HttpRequest),
    HttpResponse(HttpResponse),
    Plugin(Plugin),
    Settings(Settings),
    KeyValue(KeyValue),
    Workspace(Workspace),
}

pub fn json_col<'a, T>(s: &'a str) -> T
where
    T: Deserialize<'a> + Default,
{
    serde_json::from_str(s).unwrap_or_default()
}
