use crate::diff::{compute_changes, objects_from_stage_tree, StageTreeNode};
use crate::error::Result;
use crate::queries::{
    insert_commit, insert_object, query_branch_by_name, query_commits, query_object, upsert_branch,
};
use crate::SyncCommit;
use serde::{Deserialize, Serialize};
use tauri::{command, Manager, Runtime, WebviewWindow};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "commands.ts")]
#[serde(rename_all = "camelCase")]
pub struct ChangesPayload {
    workspace_id: String,
    branch: String,
}

#[command]
pub async fn changes<R: Runtime>(
    window: WebviewWindow<R>,
    payload: ChangesPayload,
) -> Result<StageTreeNode> {
    compute_changes(
        &window,
        payload.workspace_id.as_str(),
        payload.branch.as_str(),
    )
    .await
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "commands.ts")]
#[serde(rename_all = "camelCase")]
pub struct CommitsPayload {
    workspace_id: String,
    branch: String,
}

#[command]
pub async fn commits<R: Runtime>(
    window: WebviewWindow<R>,
    payload: CommitsPayload,
) -> Result<Vec<SyncCommit>> {
    let branch = query_branch_by_name(
        window.app_handle(),
        payload.workspace_id.as_str(),
        payload.branch.as_str(),
    )
    .await?;
    query_commits(window.app_handle(), branch.commit_ids).await
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "commands.ts")]
#[serde(rename_all = "camelCase")]
pub struct CommitPayload {
    workspace_id: String,
    message: String,
    branch: String,
    added_ids: Vec<String>,
}

#[command]
pub async fn commit<R: Runtime>(window: WebviewWindow<R>, payload: CommitPayload) -> Result<()> {
    let tree = compute_changes(
        &window,
        payload.workspace_id.as_str(),
        payload.branch.as_str(),
    )
    .await?;

    let objects = objects_from_stage_tree(tree, payload.added_ids);
    for obj in objects.clone() {
        if let Err(_) = query_object(window.app_handle(), obj.id.as_str()).await {
            insert_object(&window, obj).await?;
        }
    }
    let object_ids: Vec<String> = objects.iter().map(|o| o.id.clone()).collect();

    // Insert the commit
    let commit = insert_commit(
        &window,
        SyncCommit {
            workspace_id: payload.workspace_id.clone().into(),
            message: payload.message.into(),
            object_ids,
            ..Default::default()
        },
    )
    .await?;

    let mut branch = query_branch_by_name(
        window.app_handle(),
        payload.workspace_id.as_str(),
        payload.branch.as_str(),
    )
    .await?;
    branch.commit_ids.push(commit.id);
    upsert_branch(&window, branch).await?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "commands.ts")]
#[serde(rename_all = "camelCase")]
pub struct PushPayload {
    workspace_id: String,
    message: String,
}

#[command]
pub async fn push<R: Runtime>(window: WebviewWindow<R>, payload: PushPayload) -> Result<()> {
    // # NOTES: 
    // - You could be pushing multiple local commits
    
    // 1. Fetch branch history from remote
    // 2. Figure out which commits haven't been pushed
    // 3. Push the objects from these new commits
    
    todo!()
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "commands.ts")]
#[serde(rename_all = "camelCase")]
pub struct PullPayload {
    workspace_id: String,
    message: String,
}

#[command]
pub async fn pull<R: Runtime>(window: WebviewWindow<R>, payload: PushPayload) -> Result<()> {
    // 1. Fetch branch history from remote
    // 2. Figure out which commits are new
    // 3. Check local DB to see which objects already exist
    // 4. Pull down new objects

    todo!()
}
