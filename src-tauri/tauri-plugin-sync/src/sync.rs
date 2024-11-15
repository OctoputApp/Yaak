use crate::SyncModel;
use hex;
use sha1::{Digest, Sha1};

pub fn model_hash(m: &SyncModel) -> String {
    let mut value = serde_json::to_value(&m)
        .unwrap()
        .as_object()
        .unwrap()
        .to_owned();

    // Remove fields we don't want to be in the final model
    value.remove("createdAt");
    value.remove("updatedAt");

    // println!("PRETTY {}", serde_json::to_string_pretty(&value).unwrap());
    let model_bytes = serde_json::to_vec(&value).unwrap();

    let mut hasher = Sha1::new();
    hasher.update(&model_bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use crate::sync::model_hash;
    use crate::SyncModel;
    use chrono::{TimeDelta, Utc};
    use serde_json::json;
    use std::collections::BTreeMap;
    use std::ops::{Add, Sub};
    use yaak_models::models::HttpRequest;

    fn debug_http_request() -> HttpRequest {
        let mut auth = BTreeMap::new();
        auth.insert("zzz".into(), json!("bar"));
        auth.insert("foo".into(), json!("bar"));
        auth.insert("baz".into(), json!("bar"));
        auth.insert("bar".into(), json!("bar"));
        HttpRequest {
            id: "req_123".to_string(),
            name: "My Request".to_string(),
            created_at: Utc::now().sub(TimeDelta::seconds(1234567)).naive_utc(),
            updated_at: Utc::now().naive_utc(),
            workspace_id: "wk_123".to_string(),
            folder_id: Some("fl_345".to_string()),
            authentication: auth,
            ..Default::default()
        }
    }

    #[test]
    fn test_model_hash_determinism() {
        for _i in 1..1000 {
            assert_eq!(
                model_hash(&SyncModel::HttpRequest(debug_http_request())),
                model_hash(&SyncModel::HttpRequest(debug_http_request())),
            );
        }
    }

    #[test]
    fn test_model_hash_ignore_updated_at() {
        let r1 = debug_http_request();
        let mut r2 = r1.clone();
        r2.updated_at = r2.updated_at.add(TimeDelta::seconds(999));

        assert_eq!(
            model_hash(&SyncModel::HttpRequest(r1)),
            model_hash(&SyncModel::HttpRequest(r2)),
        );
    }

    #[test]
    fn test_model_hash_different() {
        let r1 = debug_http_request();
        let mut r2 = r1.clone();
        r2.name = "Different".to_string();

        assert_ne!(
            model_hash(&SyncModel::HttpRequest(r1)),
            model_hash(&SyncModel::HttpRequest(r2)),
        );
    }
}
