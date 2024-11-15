use crate::error::Result;
use crate::sync::model_hash;
use chrono::NaiveDateTime;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use tauri::{AppHandle, Runtime};
use ts_rs::TS;
use yaak_models::models::{json_col, Environment, Folder, GrpcRequest, HttpRequest, Workspace};
use yaak_models::queries::{
    get_workspace, list_environments, list_folders, list_grpc_requests, list_http_requests,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct SyncBranch {
    #[ts(type = "\"sync_branch\"")]
    pub model: String,
    pub id: String, // Commit hash
    pub workspace_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub name: String,
    pub commit_ids: Vec<String>,
}

#[derive(sea_query::Iden)]
pub enum SyncBranchIden {
    #[iden = "sync_branches"]
    Table,
    Model,
    Id,
    CreatedAt,
    UpdatedAt,
    WorkspaceId,
    CommitIds,
    Name,
}

impl<'s> TryFrom<&rusqlite::Row<'s>> for SyncBranch {
    type Error = rusqlite::Error;

    fn try_from(r: &rusqlite::Row<'_>) -> std::result::Result<Self, Self::Error> {
        Ok(SyncBranch {
            id: r.get("id")?,
            model: r.get("model")?,
            created_at: r.get("created_at")?,
            updated_at: r.get("updated_at")?,
            workspace_id: r.get("workspace_id")?,

            commit_ids: json_col(r.get::<_, String>("commit_ids")?.as_str()),
            name: r.get("name")?,
        })
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct SyncCommit {
    #[ts(type = "\"sync_commit\"")]
    pub model: String,
    pub id: String,
    pub workspace_id: String,
    pub created_at: NaiveDateTime,
    pub message: Option<String>,
    pub object_ids: Vec<String>,
}

#[derive(sea_query::Iden)]
pub enum SyncCommitIden {
    #[iden = "sync_commits"]
    Table,
    Model,
    Id,
    CreatedAt,
    WorkspaceId,

    Message,
    ObjectIds,
}

impl<'s> TryFrom<&rusqlite::Row<'s>> for SyncCommit {
    type Error = rusqlite::Error;

    fn try_from(r: &rusqlite::Row<'_>) -> std::result::Result<Self, Self::Error> {
        Ok(SyncCommit {
            id: r.get("id")?,
            model: r.get("model")?,
            created_at: r.get("created_at")?,
            workspace_id: r.get("workspace_id")?,

            object_ids: json_col(r.get::<_, String>("object_ids")?.as_str()),
            message: r.get("message")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct SyncObject {
    #[ts(type = "\"sync_object\"")]
    pub model: String,
    pub id: String, // Model hash
    pub created_at: NaiveDateTime,
    pub workspace_id: String,
    pub data: Vec<u8>,
    pub model_id: String,
    pub model_model: String,
}

#[derive(sea_query::Iden)]
pub enum SyncObjectIden {
    #[iden = "sync_objects"]
    Table,
    Model,
    Id,
    CreatedAt,
    WorkspaceId,

    Data,
    ModelId,
    ModelModel,
}

impl<'s> TryFrom<&rusqlite::Row<'s>> for SyncObject {
    type Error = rusqlite::Error;

    fn try_from(r: &rusqlite::Row<'s>) -> std::result::Result<Self, Self::Error> {
        let data: Vec<u8> = r.get("data")?;
        Ok(SyncObject {
            id: r.get("id")?,
            model: r.get("model")?,
            created_at: r.get("created_at")?,
            workspace_id: r.get("workspace_id")?,
            data,
            model_id: r.get("model_id")?,
            model_model: r.get("model_model")?,
        })
    }
}

impl Into<SyncModel> for SyncObject {
    fn into(self) -> SyncModel {
        let model: SyncModel = serde_json::from_slice(&self.data.as_slice()).unwrap();
        model
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "snake_case", untagged)]
#[ts(export, export_to = "models.ts")]
pub enum SyncModel {
    Workspace(Workspace),
    Environment(Environment),
    Folder(Folder),
    HttpRequest(HttpRequest),
    GrpcRequest(GrpcRequest),
}

impl<'de> Deserialize<'de> for SyncModel {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json: serde_json::value::Value = serde_json::value::Value::deserialize(deserializer)?;
        let obj = json.as_object().unwrap();
        let model = obj.get("model").unwrap().as_str().unwrap_or("__UNKNOWN__");
        match model {
            "workspace" => Ok(SyncModel::Workspace(
                serde_json::from_value(json).unwrap(),
            )),
            "environment" => Ok(SyncModel::Environment(
                serde_json::from_value(json).unwrap(),
            )),
            "folder" => Ok(SyncModel::Folder(
                serde_json::from_value(json).unwrap(),
            )),
            "http_request" => Ok(SyncModel::HttpRequest(
                serde_json::from_value(json).unwrap(),
            )),
            "grpc_request" => Ok(SyncModel::GrpcRequest(
                serde_json::from_value(json).unwrap(),
            )),
            model => Err(Error::unknown_variant(
                model,
                &[
                    "workspace",
                    "environment",
                    "folder",
                    "http_request",
                    "grpc_request",
                ],
            )),
        }
    }
}

impl SyncModel {
    pub fn model_id(&self) -> String {
        match self {
            SyncModel::Workspace(m) => m.to_owned().id,
            SyncModel::Environment(m) => m.to_owned().id,
            SyncModel::Folder(m) => m.to_owned().id,
            SyncModel::HttpRequest(m) => m.to_owned().id,
            SyncModel::GrpcRequest(m) => m.to_owned().id,
        }
    }

    pub fn is_child_of(&self, parent: &Self) -> bool {
        let parent_id = parent.model_id();
        match self {
            SyncModel::Workspace(_) => false,
            SyncModel::Environment(m) => m.workspace_id == parent_id,
            SyncModel::Folder(m) => match m.to_owned().folder_id {
                None => m.workspace_id == parent_id,
                Some(folder_id) => folder_id == parent_id,
            },
            SyncModel::HttpRequest(m) => match m.to_owned().folder_id {
                None => m.workspace_id == parent_id,
                Some(folder_id) => folder_id == parent_id,
            },
            SyncModel::GrpcRequest(m) => match m.to_owned().folder_id {
                None => m.workspace_id == parent_id,
                Some(folder_id) => folder_id == parent_id,
            },
        }
    }
}

impl Into<SyncObject> for SyncModel {
    fn into(self) -> SyncObject {
        let id = format!("so_{}", model_hash(&self));

        match self.clone() {
            SyncModel::Workspace(m) => SyncObject {
                model: "sync_object".into(),
                created_at: Default::default(),
                id,
                workspace_id: m.id.clone(),
                data: serde_json::to_vec(&self).unwrap(),
                model_id: m.id.clone(),
                model_model: m.model.clone(),
            },
            SyncModel::Environment(m) => SyncObject {
                model: "sync_object".into(),
                created_at: Default::default(),
                id,
                workspace_id: m.workspace_id.clone(),
                data: serde_json::to_vec(&self).unwrap(),
                model_id: m.id.clone(),
                model_model: m.model.clone(),
            },
            SyncModel::Folder(m) => SyncObject {
                model: "sync_object".into(),
                created_at: Default::default(),
                id,
                workspace_id: m.workspace_id.clone(),
                data: serde_json::to_vec(&self).unwrap(),
                model_id: m.id.clone(),
                model_model: m.model.clone(),
            },
            SyncModel::HttpRequest(m) => SyncObject {
                model: "sync_object".into(),
                created_at: Default::default(),
                id,
                workspace_id: m.workspace_id.clone(),
                data: serde_json::to_vec(&self).unwrap(),
                model_id: m.id.clone(),
                model_model: m.model.clone(),
            },
            SyncModel::GrpcRequest(m) => SyncObject {
                model: "sync_object".into(),
                created_at: Default::default(),
                id,
                workspace_id: m.workspace_id.clone(),
                data: serde_json::to_vec(&self).unwrap(),
                model_id: m.id.clone(),
                model_model: m.model.clone(),
            },
        }
    }
}

pub async fn find_all_models<R: Runtime>(
    app_handle: &AppHandle<R>,
    workspace_id: &str,
) -> Result<Vec<SyncModel>> {
    let mut models = Vec::new();

    models.push(SyncModel::Workspace(
        get_workspace(app_handle, workspace_id).await?,
    ));

    models.append(
        &mut list_environments(app_handle, workspace_id)
            .await?
            .iter()
            .map(|m| SyncModel::Environment(m.to_owned()))
            .collect(),
    );

    models.append(
        &mut list_folders(app_handle, workspace_id)
            .await?
            .iter()
            .map(|m| SyncModel::Folder(m.to_owned()))
            .collect(),
    );

    models.append(
        &mut list_http_requests(app_handle, workspace_id)
            .await?
            .iter()
            .map(|m| SyncModel::HttpRequest(m.to_owned()))
            .collect(),
    );

    models.append(
        &mut list_grpc_requests(app_handle, workspace_id)
            .await?
            .iter()
            .map(|m| SyncModel::GrpcRequest(m.to_owned()))
            .collect(),
    );

    Ok(models)
}
