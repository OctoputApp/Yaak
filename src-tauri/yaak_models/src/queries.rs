use std::fs;

use crate::error::Result;
use crate::models::{
    CookieJar, CookieJarIden, Environment, EnvironmentIden, Folder, FolderIden, GrpcConnection,
    GrpcConnectionIden, GrpcEvent, GrpcEventIden, GrpcRequest, GrpcRequestIden, HttpRequest,
    HttpRequestIden, HttpResponse, HttpResponseHeader, HttpResponseIden, KeyValue, KeyValueIden,
    ModelType, Settings, SettingsIden, Workspace, WorkspaceIden,
};
use log::error;
use rand::distributions::{Alphanumeric, DistString};
use sea_query::ColumnRef::Asterisk;
use sea_query::Keyword::CurrentTimestamp;
use sea_query::{Cond, Expr, OnConflict, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, WebviewWindow, Wry};
use crate::plugin::SqliteConnection;

pub async fn set_key_value_string(
    mgr: &impl Manager<Wry>,
    namespace: &str,
    key: &str,
    value: &str,
) -> (KeyValue, bool) {
    let encoded = serde_json::to_string(value);
    set_key_value_raw(mgr, namespace, key, &encoded.unwrap()).await
}

pub async fn set_key_value_int(
    mgr: &impl Manager<Wry>,
    namespace: &str,
    key: &str,
    value: i32,
) -> (KeyValue, bool) {
    let encoded = serde_json::to_string(&value);
    set_key_value_raw(mgr, namespace, key, &encoded.unwrap()).await
}

pub async fn get_key_value_string(
    mgr: &impl Manager<Wry>,
    namespace: &str,
    key: &str,
    default: &str,
) -> String {
    match get_key_value_raw(mgr, namespace, key).await {
        None => default.to_string(),
        Some(v) => {
            let result = serde_json::from_str(&v.value);
            match result {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to parse string key value: {}", e);
                    default.to_string()
                }
            }
        }
    }
}

pub async fn get_key_value_int(
    mgr: &impl Manager<Wry>,
    namespace: &str,
    key: &str,
    default: i32,
) -> i32 {
    match get_key_value_raw(mgr, namespace, key).await {
        None => default.clone(),
        Some(v) => {
            let result = serde_json::from_str(&v.value);
            match result {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to parse int key value: {}", e);
                    default.clone()
                }
            }
        }
    }
}

pub async fn set_key_value_raw(
    mgr: &impl Manager<Wry>,
    namespace: &str,
    key: &str,
    value: &str,
) -> (KeyValue, bool) {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let existing = get_key_value_raw(mgr, namespace, key).await;
    let (sql, params) = Query::insert()
        .into_table(KeyValueIden::Table)
        .columns([
            KeyValueIden::CreatedAt,
            KeyValueIden::UpdatedAt,
            KeyValueIden::Namespace,
            KeyValueIden::Key,
            KeyValueIden::Value,
        ])
        .values_panic([
            CurrentTimestamp.into(),
            CurrentTimestamp.into(),
            namespace.into(),
            key.into(),
            value.into(),
        ])
        .on_conflict(
            OnConflict::new()
                .update_columns([KeyValueIden::UpdatedAt, KeyValueIden::Value])
                .to_owned(),
        )
        .returning_all()
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db
        .prepare(sql.as_str())
        .expect("Failed to prepare KeyValue upsert");
    let kv = stmt
        .query_row(&*params.as_params(), |row| row.try_into())
        .expect("Failed to upsert KeyValue");
    (kv, existing.is_none())
}

pub async fn get_key_value_raw(
    mgr: &impl Manager<Wry>,
    namespace: &str,
    key: &str,
) -> Option<KeyValue> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let (sql, params) = Query::select()
        .from(KeyValueIden::Table)
        .column(Asterisk)
        .cond_where(
            Cond::all()
                .add(Expr::col(KeyValueIden::Namespace).eq(namespace))
                .add(Expr::col(KeyValueIden::Key).eq(key)),
        )
        .build_rusqlite(SqliteQueryBuilder);

    db.query_row(sql.as_str(), &*params.as_params(), |row| row.try_into())
        .ok()
}

pub async fn list_workspaces(mgr: &impl Manager<Wry>) -> Result<Vec<Workspace>> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let (sql, params) = Query::select()
        .from(WorkspaceIden::Table)
        .column(Asterisk)
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    let items = stmt.query_map(&*params.as_params(), |row| row.try_into())?;
    Ok(items.map(|v| v.unwrap()).collect())
}

pub async fn get_workspace(mgr: &impl Manager<Wry>, id: &str) -> Result<Workspace> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let (sql, params) = Query::select()
        .from(WorkspaceIden::Table)
        .column(Asterisk)
        .cond_where(Expr::col(WorkspaceIden::Id).eq(id))
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    Ok(stmt.query_row(&*params.as_params(), |row| row.try_into())?)
}

pub async fn upsert_workspace(window: &WebviewWindow, workspace: Workspace) -> Result<Workspace> {
    let id = match workspace.id.as_str() {
        "" => generate_model_id(ModelType::TypeWorkspace),
        _ => workspace.id.to_string(),
    };
    let trimmed_name = workspace.name.trim();

    let dbm = &*window.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::insert()
        .into_table(WorkspaceIden::Table)
        .columns([
            WorkspaceIden::Id,
            WorkspaceIden::CreatedAt,
            WorkspaceIden::UpdatedAt,
            WorkspaceIden::Name,
            WorkspaceIden::Description,
            WorkspaceIden::Variables,
            WorkspaceIden::SettingRequestTimeout,
            WorkspaceIden::SettingFollowRedirects,
            WorkspaceIden::SettingValidateCertificates,
        ])
        .values_panic([
            id.as_str().into(),
            CurrentTimestamp.into(),
            CurrentTimestamp.into(),
            trimmed_name.into(),
            workspace.description.into(),
            serde_json::to_string(&workspace.variables).unwrap().into(),
            workspace.setting_request_timeout.into(),
            workspace.setting_follow_redirects.into(),
            workspace.setting_validate_certificates.into(),
        ])
        .on_conflict(
            OnConflict::column(GrpcRequestIden::Id)
                .update_columns([
                    WorkspaceIden::UpdatedAt,
                    WorkspaceIden::Name,
                    WorkspaceIden::Description,
                    WorkspaceIden::Variables,
                    WorkspaceIden::SettingRequestTimeout,
                    WorkspaceIden::SettingFollowRedirects,
                    WorkspaceIden::SettingValidateCertificates,
                ])
                .to_owned(),
        )
        .returning_all()
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db.prepare(sql.as_str())?;
    let m = stmt.query_row(&*params.as_params(), |row| row.try_into())?;
    Ok(emit_upserted_model(window, m))
}

pub async fn delete_workspace(window: &WebviewWindow, id: &str) -> Result<Workspace> {
    let dbm = &*window.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let workspace = get_workspace(window, id).await?;

    let (sql, params) = Query::delete()
        .from_table(WorkspaceIden::Table)
        .cond_where(Expr::col(WorkspaceIden::Id).eq(id))
        .build_rusqlite(SqliteQueryBuilder);
    db.execute(sql.as_str(), &*params.as_params())?;

    for r in list_responses_by_workspace_id(window, id).await? {
        delete_http_response(window, &r.id).await?;
    }

    emit_deleted_model(window, workspace)
}

pub async fn get_cookie_jar(mgr: &impl Manager<Wry>, id: &str) -> Result<CookieJar> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::select()
        .from(CookieJarIden::Table)
        .column(Asterisk)
        .cond_where(Expr::col(CookieJarIden::Id).eq(id))
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    Ok(stmt.query_row(&*params.as_params(), |row| row.try_into())?)
}

pub async fn list_cookie_jars(
    mgr: &impl Manager<Wry>,
    workspace_id: &str,
) -> Result<Vec<CookieJar>> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let (sql, params) = Query::select()
        .from(CookieJarIden::Table)
        .column(Asterisk)
        .cond_where(Expr::col(CookieJarIden::WorkspaceId).eq(workspace_id))
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    let items = stmt.query_map(&*params.as_params(), |row| row.try_into())?;
    Ok(items.map(|v| v.unwrap()).collect())
}

pub async fn delete_cookie_jar(window: &WebviewWindow, id: &str) -> Result<CookieJar> {
    let cookie_jar = get_cookie_jar(window, id).await?;
    let dbm = &*window.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::delete()
        .from_table(CookieJarIden::Table)
        .cond_where(Expr::col(WorkspaceIden::Id).eq(id))
        .build_rusqlite(SqliteQueryBuilder);
    db.execute(sql.as_str(), &*params.as_params())?;

    emit_deleted_model(window, cookie_jar)
}

pub async fn duplicate_grpc_request(window: &WebviewWindow, id: &str) -> Result<GrpcRequest> {
    let mut request = get_grpc_request(window, id).await?.clone();
    request.id = "".to_string();
    upsert_grpc_request(window, &request).await
}

pub async fn delete_grpc_request(window: &WebviewWindow, id: &str) -> Result<GrpcRequest> {
    let req = get_grpc_request(window, id).await?;

    let dbm = &*window.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let (sql, params) = Query::delete()
        .from_table(GrpcRequestIden::Table)
        .cond_where(Expr::col(GrpcRequestIden::Id).eq(id))
        .build_rusqlite(SqliteQueryBuilder);
    db.execute(sql.as_str(), &*params.as_params())?;

    emit_deleted_model(window, req)
}

pub async fn upsert_grpc_request(
    window: &WebviewWindow,
    request: &GrpcRequest,
) -> Result<GrpcRequest> {
    let dbm = &*window.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let id = match request.id.as_str() {
        "" => generate_model_id(ModelType::TypeGrpcRequest),
        _ => request.id.to_string(),
    };
    let trimmed_name = request.name.trim();

    let (sql, params) = Query::insert()
        .into_table(GrpcRequestIden::Table)
        .columns([
            GrpcRequestIden::Id,
            GrpcRequestIden::CreatedAt,
            GrpcRequestIden::UpdatedAt,
            GrpcRequestIden::Name,
            GrpcRequestIden::WorkspaceId,
            GrpcRequestIden::FolderId,
            GrpcRequestIden::SortPriority,
            GrpcRequestIden::Url,
            GrpcRequestIden::Service,
            GrpcRequestIden::Message,
            GrpcRequestIden::AuthenticationType,
            GrpcRequestIden::Authentication,
            GrpcRequestIden::Metadata,
        ])
        .values_panic([
            id.as_str().into(),
            CurrentTimestamp.into(),
            CurrentTimestamp.into(),
            trimmed_name.into(),
            request.workspace_id.as_str().into(),
            request.folder_id.as_ref().map(|s| s.as_str()).into(),
            request.sort_priority.into(),
            request.url.as_str().into(),
            request.service.as_ref().map(|s| s.as_str()).into(),
            request.method.as_ref().map(|s| s.as_str()).into(),
            request.message.as_str().into(),
            request
                .authentication_type
                .as_ref()
                .map(|s| s.as_str())
                .into(),
            serde_json::to_string(&request.authentication)
                .unwrap()
                .into(),
            serde_json::to_string(&request.metadata).unwrap().into(),
        ])
        .on_conflict(
            OnConflict::column(GrpcRequestIden::Id)
                .update_columns([
                    GrpcRequestIden::UpdatedAt,
                    GrpcRequestIden::WorkspaceId,
                    GrpcRequestIden::Name,
                    GrpcRequestIden::FolderId,
                    GrpcRequestIden::SortPriority,
                    GrpcRequestIden::Url,
                    GrpcRequestIden::Service,
                    GrpcRequestIden::Method,
                    GrpcRequestIden::Message,
                    GrpcRequestIden::AuthenticationType,
                    GrpcRequestIden::Authentication,
                    GrpcRequestIden::Metadata,
                ])
                .to_owned(),
        )
        .returning_all()
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db.prepare(sql.as_str())?;
    let m = stmt.query_row(&*params.as_params(), |row| row.try_into())?;
    Ok(emit_upserted_model(window, m))
}

pub async fn get_grpc_request(mgr: &impl Manager<Wry>, id: &str) -> Result<GrpcRequest> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::select()
        .from(GrpcRequestIden::Table)
        .column(Asterisk)
        .cond_where(Expr::col(GrpcRequestIden::Id).eq(id))
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    Ok(stmt.query_row(&*params.as_params(), |row| row.try_into())?)
}

pub async fn list_grpc_requests(
    mgr: &impl Manager<Wry>,
    workspace_id: &str,
) -> Result<Vec<GrpcRequest>> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let (sql, params) = Query::select()
        .from(GrpcRequestIden::Table)
        .cond_where(Expr::col(GrpcRequestIden::WorkspaceId).eq(workspace_id))
        .column(Asterisk)
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    let items = stmt.query_map(&*params.as_params(), |row| row.try_into())?;
    Ok(items.map(|v| v.unwrap()).collect())
}

pub async fn upsert_grpc_connection(
    window: &WebviewWindow,
    connection: &GrpcConnection,
) -> Result<GrpcConnection> {
    let dbm = &*window.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let id = match connection.id.as_str() {
        "" => generate_model_id(ModelType::TypeGrpcConnection),
        _ => connection.id.to_string(),
    };
    let (sql, params) = Query::insert()
        .into_table(GrpcConnectionIden::Table)
        .columns([
            GrpcConnectionIden::Id,
            GrpcConnectionIden::CreatedAt,
            GrpcConnectionIden::UpdatedAt,
            GrpcConnectionIden::WorkspaceId,
            GrpcConnectionIden::RequestId,
            GrpcConnectionIden::Service,
            GrpcConnectionIden::Method,
            GrpcConnectionIden::Elapsed,
            GrpcConnectionIden::Status,
            GrpcConnectionIden::Error,
            GrpcConnectionIden::Trailers,
            GrpcConnectionIden::Url,
        ])
        .values_panic([
            id.as_str().into(),
            CurrentTimestamp.into(),
            CurrentTimestamp.into(),
            connection.workspace_id.as_str().into(),
            connection.request_id.as_str().into(),
            connection.service.as_str().into(),
            connection.method.as_str().into(),
            connection.elapsed.into(),
            connection.status.into(),
            connection.error.as_ref().map(|s| s.as_str()).into(),
            serde_json::to_string(&connection.trailers).unwrap().into(),
            connection.url.as_str().into(),
        ])
        .on_conflict(
            OnConflict::column(GrpcConnectionIden::Id)
                .update_columns([
                    GrpcConnectionIden::UpdatedAt,
                    GrpcConnectionIden::Service,
                    GrpcConnectionIden::Method,
                    GrpcConnectionIden::Elapsed,
                    GrpcConnectionIden::Status,
                    GrpcConnectionIden::Error,
                    GrpcConnectionIden::Trailers,
                    GrpcConnectionIden::Url,
                ])
                .to_owned(),
        )
        .returning_all()
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db.prepare(sql.as_str())?;
    let m = stmt.query_row(&*params.as_params(), |row| row.try_into())?;
    Ok(emit_upserted_model(window, m))
}

pub async fn get_grpc_connection(mgr: &impl Manager<Wry>, id: &str) -> Result<GrpcConnection> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let (sql, params) = Query::select()
        .from(GrpcConnectionIden::Table)
        .column(Asterisk)
        .cond_where(Expr::col(GrpcConnectionIden::Id).eq(id))
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    Ok(stmt.query_row(&*params.as_params(), |row| row.try_into())?)
}

pub async fn list_grpc_connections(
    mgr: &impl Manager<Wry>,
    request_id: &str,
) -> Result<Vec<GrpcConnection>> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::select()
        .from(GrpcConnectionIden::Table)
        .cond_where(Expr::col(GrpcConnectionIden::RequestId).eq(request_id))
        .column(Asterisk)
        .order_by(GrpcConnectionIden::CreatedAt, Order::Desc)
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    let items = stmt.query_map(&*params.as_params(), |row| row.try_into())?;
    Ok(items.map(|v| v.unwrap()).collect())
}

pub async fn delete_grpc_connection(window: &WebviewWindow, id: &str) -> Result<GrpcConnection> {
    let resp = get_grpc_connection(window, id).await?;

    let dbm = &*window.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::delete()
        .from_table(GrpcConnectionIden::Table)
        .cond_where(Expr::col(GrpcConnectionIden::Id).eq(id))
        .build_rusqlite(SqliteQueryBuilder);

    db.execute(sql.as_str(), &*params.as_params())?;
    emit_deleted_model(window, resp)
}

pub async fn delete_all_grpc_connections(window: &WebviewWindow, request_id: &str) -> Result<()> {
    for r in list_grpc_connections(window, request_id).await? {
        delete_grpc_connection(window, &r.id).await?;
    }
    Ok(())
}

pub async fn upsert_grpc_event(window: &WebviewWindow, event: &GrpcEvent) -> Result<GrpcEvent> {
    let dbm = &*window.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let id = match event.id.as_str() {
        "" => generate_model_id(ModelType::TypeGrpcEvent),
        _ => event.id.to_string(),
    };

    let (sql, params) = Query::insert()
        .into_table(GrpcEventIden::Table)
        .columns([
            GrpcEventIden::Id,
            GrpcEventIden::CreatedAt,
            GrpcEventIden::UpdatedAt,
            GrpcEventIden::WorkspaceId,
            GrpcEventIden::RequestId,
            GrpcEventIden::ConnectionId,
            GrpcEventIden::Content,
            GrpcEventIden::EventType,
            GrpcEventIden::Metadata,
            GrpcEventIden::Status,
            GrpcEventIden::Error,
        ])
        .values_panic([
            id.as_str().into(),
            CurrentTimestamp.into(),
            CurrentTimestamp.into(),
            event.workspace_id.as_str().into(),
            event.request_id.as_str().into(),
            event.connection_id.as_str().into(),
            event.content.as_str().into(),
            serde_json::to_string(&event.event_type).unwrap().into(),
            serde_json::to_string(&event.metadata).unwrap().into(),
            event.status.into(),
            event.error.as_ref().map(|s| s.as_str()).into(),
        ])
        .on_conflict(
            OnConflict::column(GrpcEventIden::Id)
                .update_columns([
                    GrpcEventIden::UpdatedAt,
                    GrpcEventIden::Content,
                    GrpcEventIden::EventType,
                    GrpcEventIden::Metadata,
                    GrpcEventIden::Status,
                    GrpcEventIden::Error,
                ])
                .to_owned(),
        )
        .returning_all()
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db.prepare(sql.as_str())?;
    let m = stmt.query_row(&*params.as_params(), |row| row.try_into())?;
    Ok(emit_upserted_model(window, m))
}

pub async fn get_grpc_event(mgr: &impl Manager<Wry>, id: &str) -> Result<GrpcEvent> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let (sql, params) = Query::select()
        .from(GrpcEventIden::Table)
        .column(Asterisk)
        .cond_where(Expr::col(GrpcEventIden::Id).eq(id))
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    Ok(stmt.query_row(&*params.as_params(), |row| row.try_into())?)
}

pub async fn list_grpc_events(
    mgr: &impl Manager<Wry>,
    connection_id: &str,
) -> Result<Vec<GrpcEvent>> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::select()
        .from(GrpcEventIden::Table)
        .cond_where(Expr::col(GrpcEventIden::ConnectionId).eq(connection_id))
        .column(Asterisk)
        .order_by(GrpcEventIden::CreatedAt, Order::Desc)
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    let items = stmt.query_map(&*params.as_params(), |row| row.try_into())?;
    Ok(items.map(|v| v.unwrap()).collect())
}

pub async fn upsert_cookie_jar(
    window: &WebviewWindow,
    cookie_jar: &CookieJar,
) -> Result<CookieJar> {
    let id = match cookie_jar.id.as_str() {
        "" => generate_model_id(ModelType::TypeCookieJar),
        _ => cookie_jar.id.to_string(),
    };
    let trimmed_name = cookie_jar.name.trim();

    let dbm = &*window.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::insert()
        .into_table(CookieJarIden::Table)
        .columns([
            CookieJarIden::Id,
            CookieJarIden::CreatedAt,
            CookieJarIden::UpdatedAt,
            CookieJarIden::WorkspaceId,
            CookieJarIden::Name,
            CookieJarIden::Cookies,
        ])
        .values_panic([
            id.as_str().into(),
            CurrentTimestamp.into(),
            CurrentTimestamp.into(),
            cookie_jar.workspace_id.as_str().into(),
            trimmed_name.into(),
            serde_json::to_string(&cookie_jar.cookies).unwrap().into(),
        ])
        .on_conflict(
            OnConflict::column(GrpcEventIden::Id)
                .update_columns([
                    CookieJarIden::UpdatedAt,
                    CookieJarIden::Name,
                    CookieJarIden::Cookies,
                ])
                .to_owned(),
        )
        .returning_all()
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db.prepare(sql.as_str())?;
    let m = stmt.query_row(&*params.as_params(), |row| row.try_into())?;
    Ok(emit_upserted_model(window, m))
}

pub async fn list_environments(
    mgr: &impl Manager<Wry>,
    workspace_id: &str,
) -> Result<Vec<Environment>> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::select()
        .from(EnvironmentIden::Table)
        .cond_where(Expr::col(EnvironmentIden::WorkspaceId).eq(workspace_id))
        .column(Asterisk)
        .order_by(EnvironmentIden::CreatedAt, Order::Desc)
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    let items = stmt.query_map(&*params.as_params(), |row| row.try_into())?;
    Ok(items.map(|v| v.unwrap()).collect())
}

pub async fn delete_environment(window: &WebviewWindow, id: &str) -> Result<Environment> {
    let dbm = &*window.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let env = get_environment(window, id).await?;

    let (sql, params) = Query::delete()
        .from_table(EnvironmentIden::Table)
        .cond_where(Expr::col(EnvironmentIden::Id).eq(id))
        .build_rusqlite(SqliteQueryBuilder);

    db.execute(sql.as_str(), &*params.as_params())?;
    emit_deleted_model(window, env)
}

async fn get_settings(mgr: &impl Manager<Wry>) -> Result<Settings> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::select()
        .from(SettingsIden::Table)
        .column(Asterisk)
        .cond_where(Expr::col(SettingsIden::Id).eq("default"))
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    Ok(stmt.query_row(&*params.as_params(), |row| row.try_into())?)
}

pub async fn get_or_create_settings(mgr: &impl Manager<Wry>) -> Settings {
    if let Ok(settings) = get_settings(mgr).await {
        return settings;
    }

    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::insert()
        .into_table(SettingsIden::Table)
        .columns([SettingsIden::Id])
        .values_panic(["default".into()])
        .returning_all()
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db
        .prepare(sql.as_str())
        .expect("Failed to prepare Settings insert");
    stmt.query_row(&*params.as_params(), |row| row.try_into())
        .expect("Failed to insert Settings")
}

pub async fn update_settings(window: &WebviewWindow, settings: Settings) -> Result<Settings> {
    let dbm = &*window.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::update()
        .table(SettingsIden::Table)
        .cond_where(Expr::col(SettingsIden::Id).eq("default"))
        .values([
            (SettingsIden::Id, "default".into()),
            (SettingsIden::CreatedAt, CurrentTimestamp.into()),
            (
                SettingsIden::Appearance,
                settings.appearance.as_str().into(),
            ),
            (SettingsIden::ThemeDark, settings.theme_dark.as_str().into()),
            (
                SettingsIden::ThemeLight,
                settings.theme_light.as_str().into(),
            ),
            (
                SettingsIden::UpdateChannel,
                settings.update_channel.as_str().into(),
            ),
            (
                SettingsIden::InterfaceFontSize,
                settings.interface_font_size.into(),
            ),
            (
                SettingsIden::InterfaceScale,
                settings.interface_scale.into(),
            ),
            (
                SettingsIden::EditorFontSize,
                settings.editor_font_size.into(),
            ),
            (
                SettingsIden::EditorSoftWrap,
                settings.editor_soft_wrap.into(),
            ),
            (
                SettingsIden::OpenWorkspaceNewWindow,
                settings.open_workspace_new_window.into(),
            ),
        ])
        .returning_all()
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db.prepare(sql.as_str())?;
    let m = stmt.query_row(&*params.as_params(), |row| row.try_into())?;
    Ok(emit_upserted_model(window, m))
}

pub async fn upsert_environment(
    window: &WebviewWindow,
    environment: Environment,
) -> Result<Environment> {
    let id = match environment.id.as_str() {
        "" => generate_model_id(ModelType::TypeEnvironment),
        _ => environment.id.to_string(),
    };
    let trimmed_name = environment.name.trim();

    let dbm = &*window.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::insert()
        .into_table(EnvironmentIden::Table)
        .columns([
            EnvironmentIden::Id,
            EnvironmentIden::CreatedAt,
            EnvironmentIden::UpdatedAt,
            EnvironmentIden::WorkspaceId,
            EnvironmentIden::Name,
            EnvironmentIden::Variables,
        ])
        .values_panic([
            id.as_str().into(),
            CurrentTimestamp.into(),
            CurrentTimestamp.into(),
            environment.workspace_id.as_str().into(),
            trimmed_name.into(),
            serde_json::to_string(&environment.variables)
                .unwrap()
                .into(),
        ])
        .on_conflict(
            OnConflict::column(GrpcEventIden::Id)
                .update_columns([
                    EnvironmentIden::UpdatedAt,
                    EnvironmentIden::Name,
                    EnvironmentIden::Variables,
                ])
                .to_owned(),
        )
        .returning_all()
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db.prepare(sql.as_str())?;
    let m = stmt.query_row(&*params.as_params(), |row| row.try_into())?;
    Ok(emit_upserted_model(window, m))
}

pub async fn get_environment(mgr: &impl Manager<Wry>, id: &str) -> Result<Environment> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::select()
        .from(EnvironmentIden::Table)
        .column(Asterisk)
        .cond_where(Expr::col(EnvironmentIden::Id).eq(id))
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    Ok(stmt.query_row(&*params.as_params(), |row| row.try_into())?)
}

pub async fn get_folder(mgr: &impl Manager<Wry>, id: &str) -> Result<Folder> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::select()
        .from(FolderIden::Table)
        .column(Asterisk)
        .cond_where(Expr::col(FolderIden::Id).eq(id))
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    Ok(stmt.query_row(&*params.as_params(), |row| row.try_into())?)
}

pub async fn list_folders(mgr: &impl Manager<Wry>, workspace_id: &str) -> Result<Vec<Folder>> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::select()
        .from(FolderIden::Table)
        .cond_where(Expr::col(FolderIden::WorkspaceId).eq(workspace_id))
        .column(Asterisk)
        .order_by(FolderIden::CreatedAt, Order::Desc)
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    let items = stmt.query_map(&*params.as_params(), |row| row.try_into())?;
    Ok(items.map(|v| v.unwrap()).collect())
}

pub async fn delete_folder(window: &WebviewWindow, id: &str) -> Result<Folder> {
    let folder = get_folder(window, id).await?;
    let dbm = &*window.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::delete()
        .from_table(FolderIden::Table)
        .cond_where(Expr::col(FolderIden::Id).eq(id))
        .build_rusqlite(SqliteQueryBuilder);
    db.execute(sql.as_str(), &*params.as_params())?;

    emit_deleted_model(window, folder)
}

pub async fn upsert_folder(window: &WebviewWindow, r: Folder) -> Result<Folder> {
    let id = match r.id.as_str() {
        "" => generate_model_id(ModelType::TypeFolder),
        _ => r.id.to_string(),
    };
    let trimmed_name = r.name.trim();

    let dbm = &*window.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::insert()
        .into_table(FolderIden::Table)
        .columns([
            FolderIden::Id,
            FolderIden::CreatedAt,
            FolderIden::UpdatedAt,
            FolderIden::WorkspaceId,
            FolderIden::FolderId,
            FolderIden::Name,
            FolderIden::SortPriority,
        ])
        .values_panic([
            id.as_str().into(),
            CurrentTimestamp.into(),
            CurrentTimestamp.into(),
            r.workspace_id.as_str().into(),
            r.folder_id.as_ref().map(|s| s.as_str()).into(),
            trimmed_name.into(),
            r.sort_priority.into(),
        ])
        .on_conflict(
            OnConflict::column(GrpcEventIden::Id)
                .update_columns([
                    FolderIden::UpdatedAt,
                    FolderIden::Name,
                    FolderIden::FolderId,
                    FolderIden::SortPriority,
                ])
                .to_owned(),
        )
        .returning_all()
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db.prepare(sql.as_str())?;
    let m = stmt.query_row(&*params.as_params(), |row| row.try_into())?;
    Ok(emit_upserted_model(window, m))
}

pub async fn duplicate_http_request(window: &WebviewWindow, id: &str) -> Result<HttpRequest> {
    let mut request = get_http_request(window, id).await?.clone();
    request.id = "".to_string();
    upsert_http_request(window, request).await
}

pub async fn upsert_http_request(window: &WebviewWindow, r: HttpRequest) -> Result<HttpRequest> {
    let id = match r.id.as_str() {
        "" => generate_model_id(ModelType::TypeHttpRequest),
        _ => r.id.to_string(),
    };
    let trimmed_name = r.name.trim();

    let dbm = &*window.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::insert()
        .into_table(HttpRequestIden::Table)
        .columns([
            HttpRequestIden::Id,
            HttpRequestIden::CreatedAt,
            HttpRequestIden::UpdatedAt,
            HttpRequestIden::WorkspaceId,
            HttpRequestIden::FolderId,
            HttpRequestIden::Name,
            HttpRequestIden::Url,
            HttpRequestIden::UrlParameters,
            HttpRequestIden::Method,
            HttpRequestIden::Body,
            HttpRequestIden::BodyType,
            HttpRequestIden::Authentication,
            HttpRequestIden::AuthenticationType,
            HttpRequestIden::Headers,
            HttpRequestIden::SortPriority,
        ])
        .values_panic([
            id.as_str().into(),
            CurrentTimestamp.into(),
            CurrentTimestamp.into(),
            r.workspace_id.as_str().into(),
            r.folder_id.as_ref().map(|s| s.as_str()).into(),
            trimmed_name.into(),
            r.url.as_str().into(),
            serde_json::to_string(&r.url_parameters).unwrap().into(),
            r.method.as_str().into(),
            serde_json::to_string(&r.body).unwrap().into(),
            r.body_type.as_ref().map(|s| s.as_str()).into(),
            serde_json::to_string(&r.authentication).unwrap().into(),
            r.authentication_type.as_ref().map(|s| s.as_str()).into(),
            serde_json::to_string(&r.headers).unwrap().into(),
            r.sort_priority.into(),
        ])
        .on_conflict(
            OnConflict::column(GrpcEventIden::Id)
                .update_columns([
                    HttpRequestIden::UpdatedAt,
                    HttpRequestIden::WorkspaceId,
                    HttpRequestIden::Name,
                    HttpRequestIden::FolderId,
                    HttpRequestIden::Method,
                    HttpRequestIden::Headers,
                    HttpRequestIden::Body,
                    HttpRequestIden::BodyType,
                    HttpRequestIden::Authentication,
                    HttpRequestIden::AuthenticationType,
                    HttpRequestIden::Url,
                    HttpRequestIden::UrlParameters,
                    HttpRequestIden::SortPriority,
                ])
                .to_owned(),
        )
        .returning_all()
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db.prepare(sql.as_str())?;
    let m = stmt.query_row(&*params.as_params(), |row| row.try_into())?;
    Ok(emit_upserted_model(window, m))
}

pub async fn list_http_requests(
    mgr: &impl Manager<Wry>,
    workspace_id: &str,
) -> Result<Vec<HttpRequest>> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let (sql, params) = Query::select()
        .from(HttpRequestIden::Table)
        .cond_where(Expr::col(HttpRequestIden::WorkspaceId).eq(workspace_id))
        .column(Asterisk)
        .order_by(HttpRequestIden::CreatedAt, Order::Desc)
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    let items = stmt.query_map(&*params.as_params(), |row| row.try_into())?;
    Ok(items.map(|v| v.unwrap()).collect())
}

pub async fn get_http_request(mgr: &impl Manager<Wry>, id: &str) -> Result<HttpRequest> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::select()
        .from(HttpRequestIden::Table)
        .column(Asterisk)
        .cond_where(Expr::col(HttpRequestIden::Id).eq(id))
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    Ok(stmt.query_row(&*params.as_params(), |row| row.try_into())?)
}

pub async fn delete_http_request(window: &WebviewWindow, id: &str) -> Result<HttpRequest> {
    let req = get_http_request(window, id).await?;

    // DB deletes will cascade but this will delete the files
    delete_all_http_responses(window, id).await?;

    let dbm = &*window.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let (sql, params) = Query::delete()
        .from_table(HttpRequestIden::Table)
        .cond_where(Expr::col(HttpRequestIden::Id).eq(id))
        .build_rusqlite(SqliteQueryBuilder);
    db.execute(sql.as_str(), &*params.as_params())?;

    emit_deleted_model(window, req)
}

#[allow(clippy::too_many_arguments)]
pub async fn create_http_response(
    window: &WebviewWindow,
    request_id: &str,
    elapsed: i64,
    elapsed_headers: i64,
    url: &str,
    status: i64,
    status_reason: Option<&str>,
    content_length: Option<i64>,
    body_path: Option<&str>,
    headers: Vec<HttpResponseHeader>,
    version: Option<&str>,
    remote_addr: Option<&str>,
) -> Result<HttpResponse> {
    let req = get_http_request(window, request_id).await?;
    let id = generate_model_id(ModelType::TypeHttpResponse);
    let dbm = &*window.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::insert()
        .into_table(HttpResponseIden::Table)
        .columns([
            HttpResponseIden::Id,
            HttpResponseIden::CreatedAt,
            HttpResponseIden::UpdatedAt,
            HttpResponseIden::RequestId,
            HttpResponseIden::WorkspaceId,
            HttpResponseIden::Elapsed,
            HttpResponseIden::ElapsedHeaders,
            HttpResponseIden::Url,
            HttpResponseIden::Status,
            HttpResponseIden::StatusReason,
            HttpResponseIden::ContentLength,
            HttpResponseIden::BodyPath,
            HttpResponseIden::Headers,
            HttpResponseIden::Version,
            HttpResponseIden::RemoteAddr,
        ])
        .values_panic([
            id.as_str().into(),
            CurrentTimestamp.into(),
            CurrentTimestamp.into(),
            req.id.as_str().into(),
            req.workspace_id.as_str().into(),
            elapsed.into(),
            elapsed_headers.into(),
            url.into(),
            status.into(),
            status_reason.into(),
            content_length.into(),
            body_path.into(),
            serde_json::to_string(&headers).unwrap().into(),
            version.into(),
            remote_addr.into(),
        ])
        .returning_all()
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db.prepare(sql.as_str())?;
    let m = stmt.query_row(&*params.as_params(), |row| row.try_into())?;
    Ok(emit_upserted_model(window, m))
}

pub async fn cancel_pending_grpc_connections(app: &AppHandle) -> Result<()> {
    let dbm = &*app.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::update()
        .table(GrpcConnectionIden::Table)
        .value(GrpcConnectionIden::Elapsed, -1)
        .cond_where(Expr::col(GrpcConnectionIden::Elapsed).eq(0))
        .build_rusqlite(SqliteQueryBuilder);

    db.execute(sql.as_str(), &*params.as_params())?;
    Ok(())
}

pub async fn cancel_pending_responses(app: &AppHandle) -> Result<()> {
    let dbm = &*app.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::update()
        .table(HttpResponseIden::Table)
        .values([
            (HttpResponseIden::Elapsed, (-1i32).into()),
            (HttpResponseIden::StatusReason, "Cancelled".into()),
        ])
        .cond_where(Expr::col(HttpResponseIden::Elapsed).eq(0))
        .build_rusqlite(SqliteQueryBuilder);

    db.execute(sql.as_str(), &*params.as_params())?;
    Ok(())
}

pub async fn update_response_if_id(
    window: &WebviewWindow,
    response: &HttpResponse,
) -> Result<HttpResponse> {
    if response.id.is_empty() {
        Ok(response.clone())
    } else {
        update_response(window, response).await
    }
}

pub async fn update_response(
    window: &WebviewWindow,
    response: &HttpResponse,
) -> Result<HttpResponse> {
    let dbm = &*window.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::update()
        .table(HttpResponseIden::Table)
        .cond_where(Expr::col(HttpResponseIden::Id).eq(response.clone().id))
        .values([
            (HttpResponseIden::UpdatedAt, CurrentTimestamp.into()),
            (HttpResponseIden::Elapsed, response.elapsed.into()),
            (HttpResponseIden::Url, response.url.as_str().into()),
            (HttpResponseIden::Status, response.status.into()),
            (
                HttpResponseIden::StatusReason,
                response.status_reason.as_ref().map(|s| s.as_str()).into(),
            ),
            (
                HttpResponseIden::ContentLength,
                response.content_length.into(),
            ),
            (
                HttpResponseIden::BodyPath,
                response.body_path.as_ref().map(|s| s.as_str()).into(),
            ),
            (
                HttpResponseIden::Error,
                response.error.as_ref().map(|s| s.as_str()).into(),
            ),
            (
                HttpResponseIden::Headers,
                serde_json::to_string(&response.headers)
                    .unwrap_or_default()
                    .into(),
            ),
            (
                HttpResponseIden::Version,
                response.version.as_ref().map(|s| s.as_str()).into(),
            ),
            (
                HttpResponseIden::RemoteAddr,
                response.remote_addr.as_ref().map(|s| s.as_str()).into(),
            ),
        ])
        .returning_all()
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db.prepare(sql.as_str())?;
    let m = stmt.query_row(&*params.as_params(), |row| row.try_into())?;
    Ok(emit_upserted_model(window, m))
}

pub async fn get_http_response(mgr: &impl Manager<Wry>, id: &str) -> Result<HttpResponse> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let (sql, params) = Query::select()
        .from(HttpResponseIden::Table)
        .column(Asterisk)
        .cond_where(Expr::col(HttpResponseIden::Id).eq(id))
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    Ok(stmt.query_row(&*params.as_params(), |row| row.try_into())?)
}

pub async fn delete_http_response(window: &WebviewWindow, id: &str) -> Result<HttpResponse> {
    let resp = get_http_response(window, id).await?;

    // Delete the body file if it exists
    if let Some(p) = resp.body_path.clone() {
        if let Err(e) = fs::remove_file(p) {
            error!("Failed to delete body file: {}", e);
        };
    }

    let dbm = &*window.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let (sql, params) = Query::delete()
        .from_table(HttpResponseIden::Table)
        .cond_where(Expr::col(HttpResponseIden::Id).eq(id))
        .build_rusqlite(SqliteQueryBuilder);
    db.execute(sql.as_str(), &*params.as_params())?;

    emit_deleted_model(window, resp)
}

pub async fn delete_all_http_responses(window: &WebviewWindow, request_id: &str) -> Result<()> {
    for r in list_responses(window, request_id, None).await? {
        delete_http_response(window, &r.id).await?;
    }
    Ok(())
}

pub async fn list_responses(
    mgr: &impl Manager<Wry>,
    request_id: &str,
    limit: Option<i64>,
) -> Result<Vec<HttpResponse>> {
    let limit_unwrapped = limit.unwrap_or_else(|| i64::MAX);
    let dbm = mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let (sql, params) = Query::select()
        .from(HttpResponseIden::Table)
        .cond_where(Expr::col(HttpResponseIden::RequestId).eq(request_id))
        .column(Asterisk)
        .order_by(HttpResponseIden::CreatedAt, Order::Desc)
        .limit(limit_unwrapped as u64)
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    let items = stmt.query_map(&*params.as_params(), |row| row.try_into())?;
    Ok(items.map(|v| v.unwrap()).collect())
}

pub async fn list_responses_by_workspace_id(
    mgr: &impl Manager<Wry>,
    workspace_id: &str,
) -> Result<Vec<HttpResponse>> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let (sql, params) = Query::select()
        .from(HttpResponseIden::Table)
        .cond_where(Expr::col(HttpResponseIden::WorkspaceId).eq(workspace_id))
        .column(Asterisk)
        .order_by(HttpResponseIden::CreatedAt, Order::Desc)
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    let items = stmt.query_map(&*params.as_params(), |row| row.try_into())?;
    Ok(items.map(|v| v.unwrap()).collect())
}

pub fn generate_model_id(model: ModelType) -> String {
    let id = generate_id();
    format!("{}_{}", model.id_prefix(), id)
}

pub fn generate_id() -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), 10)
}

#[derive(Clone, Serialize)]
#[serde(default, rename_all = "camelCase")]
struct ModelPayload<M: Serialize + Clone> {
    pub model: M,
    pub window_label: String,
}

fn emit_upserted_model<M: Serialize + Clone>(window: &WebviewWindow, model: M) -> M {
    let payload = ModelPayload {
        model: model.clone(),
        window_label: window.label().to_string(),
    };

    window.emit("upserted_model", payload).unwrap();
    model
}

fn emit_deleted_model<M: Serialize + Clone>(window: &WebviewWindow, model: M) -> Result<M> {
    let payload = ModelPayload {
        model: model.clone(),
        window_label: window.label().to_string(),
    };
    window.emit("deleted_model", payload).unwrap();
    Ok(model)
}
