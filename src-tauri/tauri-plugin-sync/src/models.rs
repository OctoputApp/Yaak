use crate::sync::model_hash;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use ts_rs::TS;
use yaak_models::models::{json_col, Environment, Folder, GrpcRequest, HttpRequest, Workspace};

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

    fn try_from(r: &rusqlite::Row<'_>) -> Result<Self, Self::Error> {
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
    pub id: String, // Commit hash
    pub workspace_id: String,
    pub created_at: NaiveDateTime,
    pub message: Option<String>,
    pub object_ids: Vec<String>,
}

impl SyncCommit {
    pub fn generate_id(&self) -> String {
        let mut hasher = Sha1::new();
        for id in self.object_ids.iter() {
            hasher.update(id.as_bytes());
        }
        let id = hex::encode(hasher.finalize());
        format!("sc_{id}")
    }
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

    fn try_from(r: &rusqlite::Row<'_>) -> Result<Self, Self::Error> {
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

    fn try_from(r: &rusqlite::Row<'s>) -> Result<Self, Self::Error> {
        let data: Vec<u8> = r.get("data")?;
        Ok(SyncObject {
            id: r.get("id")?,
            model: r.get("model")?,
            created_at: r.get("created_at")?,
            workspace_id: r.get("workspace_id")?,
            data: serde_json::from_slice(data.as_slice()).unwrap_or_default(),
            model_id: r.get("model_id")?,
            model_model: r.get("model_model")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case", tag = "model_type", content = "model")]
#[ts(export, export_to = "models.ts")]
pub enum SyncModel {
    Workspace(Workspace),
    Environment(Environment),
    Folder(Folder),
    HttpRequest(HttpRequest),
    GrpcRequest(GrpcRequest),
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
}

impl Into<SyncObject> for SyncModel {
    fn into(self) -> SyncObject {
        match self.clone() {
            SyncModel::Workspace(m) => SyncObject {
                model: "sync_object".into(),
                created_at: Default::default(),
                id: model_hash(&self),
                workspace_id: m.id.clone(),
                data: serde_json::to_vec(&self).unwrap(),
                model_id: m.id.clone(),
                model_model: m.model.clone(),
            },
            SyncModel::Environment(m) => SyncObject {
                model: "sync_object".into(),
                created_at: Default::default(),
                id: model_hash(&self),
                workspace_id: m.workspace_id.clone(),
                data: serde_json::to_vec(&self).unwrap(),
                model_id: m.id.clone(),
                model_model: m.model.clone(),
            },
            SyncModel::Folder(m) => SyncObject {
                model: "sync_object".into(),
                created_at: Default::default(),
                id: model_hash(&self),
                workspace_id: m.workspace_id.clone(),
                data: serde_json::to_vec(&self).unwrap(),
                model_id: m.id.clone(),
                model_model: m.model.clone(),
            },
            SyncModel::HttpRequest(m) => SyncObject {
                model: "sync_object".into(),
                created_at: Default::default(),
                id: model_hash(&self),
                workspace_id: m.workspace_id.clone(),
                data: serde_json::to_vec(&self).unwrap(),
                model_id: m.id.clone(),
                model_model: m.model.clone(),
            },
            SyncModel::GrpcRequest(m) => SyncObject {
                model: "sync_object".into(),
                created_at: Default::default(),
                id: model_hash(&self),
                workspace_id: m.workspace_id.clone(),
                data: serde_json::to_vec(&self).unwrap(),
                model_id: m.id.clone(),
                model_model: m.model.clone(),
            },
        }
    }
}