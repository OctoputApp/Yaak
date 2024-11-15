use crate::error::Result;
use crate::{SyncBranch, SyncBranchIden, SyncCommit, SyncCommitIden, SyncObject, SyncObjectIden};
use sea_query::ColumnRef::Asterisk;
use sea_query::Keyword::CurrentTimestamp;
use sea_query::{Cond, Expr, OnConflict, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use tauri::{Manager, Runtime, WebviewWindow};
use yaak_models::plugin::SqliteConnection;
use yaak_models::queries::{emit_upserted_model, generate_model_id_with_prefix};

pub async fn query_objects<R: Runtime>(
    mgr: &impl Manager<R>,
    object_ids: Vec<String>,
) -> Result<Vec<SyncObject>> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let (sql, params) = Query::select()
        .from(SyncObjectIden::Table)
        .cond_where(Expr::col(SyncObjectIden::Id).is_in(object_ids))
        .column(Asterisk)
        .order_by(SyncObjectIden::Id, Order::Asc)
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    let items = stmt.query_map(&*params.as_params(), |row| row.try_into())?;
    Ok(items.map(|v| v.unwrap()).collect())
}

pub async fn query_object<R: Runtime>(mgr: &impl Manager<R>, id: &str) -> Result<SyncObject> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let (sql, params) = Query::select()
        .from(SyncObjectIden::Table)
        .column(Asterisk)
        .cond_where(Expr::col(SyncObjectIden::Id).eq(id))
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    Ok(stmt.query_row(&*params.as_params(), |row| row.try_into())?)
}

pub async fn insert_object<R: Runtime>(
    window: &WebviewWindow<R>,
    object: SyncObject,
) -> Result<SyncObject> {
    let dbm = &*window.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::insert()
        .into_table(SyncObjectIden::Table)
        .columns([
            SyncObjectIden::Id,
            SyncObjectIden::WorkspaceId,
            SyncObjectIden::Data,
            SyncObjectIden::ModelId,
            SyncObjectIden::ModelModel,
        ])
        .values_panic([
            object.id.into(),
            object.workspace_id.into(),
            object.data.as_slice().into(),
            object.model_id.into(),
            object.model_model.into(),
        ])
        .returning_all()
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db.prepare(sql.as_str())?;
    let m = stmt.query_row(&*params.as_params(), |row| row.try_into())?;
    Ok(emit_upserted_model(&window, m))
}

pub async fn query_commits<R: Runtime>(
    mgr: &impl Manager<R>,
    commit_ids: Vec<String>,
) -> Result<Vec<SyncCommit>> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let (sql, params) = Query::select()
        .from(SyncCommitIden::Table)
        .cond_where(Expr::col(SyncCommitIden::Id).is_in(commit_ids))
        .column(Asterisk)
        .order_by(SyncCommitIden::CreatedAt, Order::Desc)
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    let items = stmt.query_map(&*params.as_params(), |row| row.try_into())?;
    Ok(items.map(|v| v.unwrap()).collect())
}

pub async fn query_commit<R: Runtime>(mgr: &impl Manager<R>, id: &str) -> Result<SyncCommit> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::select()
        .from(SyncCommitIden::Table)
        .column(Asterisk)
        .cond_where(Expr::col(SyncCommitIden::Id).eq(id))
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    Ok(stmt.query_row(&*params.as_params(), |row| row.try_into())?)
}

pub async fn insert_commit<R: Runtime>(
    window: &WebviewWindow<R>,
    commit: SyncCommit,
) -> Result<SyncCommit> {
    let dbm = &*window.app_handle().state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::insert()
        .into_table(SyncCommitIden::Table)
        .columns([
            SyncCommitIden::Id,
            SyncCommitIden::WorkspaceId,
            SyncCommitIden::Message,
            SyncCommitIden::ObjectIds,
        ])
        .values_panic([
            generate_model_id_with_prefix("sc").into(),
            commit.workspace_id.into(),
            commit.message.clone().into(),
            serde_json::to_string(&commit.object_ids)?.into(),
        ])
        .returning_all()
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db.prepare(sql.as_str())?;
    let m = stmt.query_row(&*params.as_params(), |row| row.try_into())?;
    Ok(emit_upserted_model(&window, m))
}

pub async fn query_branch_by_name<R: Runtime>(
    mgr: &impl Manager<R>,
    workspace_id: &str,
    name: &str,
) -> Result<SyncBranch> {
    let dbm = &*mgr.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();

    let (sql, params) = Query::select()
        .from(SyncBranchIden::Table)
        .column(Asterisk)
        .cond_where(
            Cond::all()
                .add(Expr::col(SyncBranchIden::Name).eq(name))
                .add(Expr::col(SyncBranchIden::WorkspaceId).eq(workspace_id)),
        )
        .build_rusqlite(SqliteQueryBuilder);
    let mut stmt = db.prepare(sql.as_str())?;
    Ok(stmt.query_row(&*params.as_params(), |row| row.try_into())?)
}

pub async fn upsert_branch<R: Runtime>(
    window: &WebviewWindow<R>,
    branch: SyncBranch,
) -> Result<SyncBranch> {
    let id = match branch.id.as_str() {
        "" => generate_model_id_with_prefix("sb"),
        _ => branch.id.to_string(),
    };
    let trimmed_name = branch.name.trim();

    let dbm = &*window.state::<SqliteConnection>();
    let db = dbm.0.lock().await.get().unwrap();
    let (sql, params) = Query::insert()
        .into_table(SyncBranchIden::Table)
        .columns([
            SyncBranchIden::Id,
            SyncBranchIden::CreatedAt,
            SyncBranchIden::UpdatedAt,
            SyncBranchIden::WorkspaceId,
            SyncBranchIden::CommitIds,
            SyncBranchIden::Name,
        ])
        .values_panic([
            id.into(),
            CurrentTimestamp.into(),
            CurrentTimestamp.into(),
            branch.workspace_id.into(),
            serde_json::to_string(&branch.commit_ids)?.into(),
            trimmed_name.into(),
        ])
        .on_conflict(
            OnConflict::new()
                .update_columns([
                    SyncBranchIden::UpdatedAt,
                    SyncBranchIden::Name,
                    SyncBranchIden::CommitIds,
                ])
                .to_owned(),
        )
        .returning_all()
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db.prepare(sql.as_str())?;
    let m = stmt.query_row(&*params.as_params(), |row| row.try_into())?;
    Ok(emit_upserted_model(window, m))
}
