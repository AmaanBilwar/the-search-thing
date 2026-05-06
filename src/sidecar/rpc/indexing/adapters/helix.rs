use async_trait::async_trait;
use helix_rs::{HelixDB, HelixDBClient};
use serde_json::{json, Value};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::sidecar::rpc::indexing::adapters::store::{
    ExistingFileRecord, ExistingImageRecord, ImageIndexStore, TextIndexStore, VideoIndexStore,
};

#[derive(Debug, Clone)]
pub struct HelixTextStore {
    endpoint: String,
    port: u16,
    api_key: Option<String>,
}

impl HelixTextStore {
    pub fn from_env() -> Result<Self, String> {
        let endpoint =
            env::var("HELIX_ENDPOINT").unwrap_or_else(|_| "http://localhost".to_string());
        let port = env::var("HELIX_PORT")
            .unwrap_or_else(|_| "6969".to_string())
            .parse::<u16>()
            .map_err(|e| format!("invalid HELIX_PORT: {}", e))?;
        let api_key = env::var("HELIX_API_KEY")
            .ok()
            .filter(|v| !v.trim().is_empty());

        Ok(Self {
            endpoint,
            port,
            api_key,
        })
    }

    fn client(&self) -> HelixDB {
        HelixDB::new(
            Some(self.endpoint.as_str()),
            Some(self.port),
            self.api_key.as_deref(),
        )
    }

    fn extract_asset_id(value: &Value) -> Option<String> {
        if let Some(id) = value.get("asset_id").and_then(Value::as_str) {
            return Some(id.to_string());
        }
        if let Some(id) = value.get("id").and_then(Value::as_str) {
            return Some(id.to_string());
        }

        if let Some(array) = value.as_array() {
            for item in array {
                if let Some(id) = Self::extract_asset_id(item) {
                    return Some(id);
                }
            }
        }

        if let Some(obj) = value.as_object() {
            for nested in obj.values() {
                if let Some(id) = Self::extract_asset_id(nested) {
                    return Some(id);
                }
            }
        }

        None
    }

    fn now_rfc3339() -> String {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let sec = (secs % 60) as u32;
        let min = ((secs / 60) % 60) as u32;
        let hour = ((secs / 3600) % 24) as u32;
        let mut days = secs / 86400;

        let mut year = 1970u32;
        loop {
            let diy = if year.is_multiple_of(4)
                && (!year.is_multiple_of(100) || year.is_multiple_of(400))
            {
                366u64
            } else {
                365u64
            };
            if days < diy {
                break;
            }
            days -= diy;
            year += 1;
        }

        let leap =
            year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
        let month_days: [u64; 12] = [
            31,
            if leap { 29 } else { 28 },
            31,
            30,
            31,
            30,
            31,
            31,
            30,
            31,
            30,
            31,
        ];
        let mut month = 1u32;
        for &md in &month_days {
            if days < md {
                break;
            }
            days -= md;
            month += 1;
        }
        let day = days + 1;

        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            year, month, day, hour, min, sec
        )
    }

    fn is_not_found_error(message: &str) -> bool {
        let lowered = message.to_ascii_lowercase();
        lowered.contains("graph error: no value found")
            || lowered.contains("\"error\":\"graph error: no value found\"")
    }
}

#[async_trait]
impl TextIndexStore for HelixTextStore {
    async fn get_file_by_hash(
        &self,
        content_hash: &str,
    ) -> Result<Option<ExistingFileRecord>, String> {
        let payload = json!({ "content_hash": content_hash });
        let client = self.client();
        let result: Value = client
            .query("GetAssetByHash", &payload)
            .await
            .map_err(|e| e.to_string())
            .or_else(|error| {
                if Self::is_not_found_error(&error) {
                    Ok(Value::Null)
                } else {
                    Err(error)
                }
            })?;

        Ok(Self::extract_asset_id(&result).map(|asset_id| ExistingFileRecord { asset_id }))
    }

    async fn create_file_asset(
        &self,
        content_hash: &str,
        kind: &str,
        path: &str,
    ) -> Result<(), String> {
        let payload = json!({
            "content_hash": content_hash,
            "kind": kind,
            "path": path,
        });
        let client = self.client();
        let _: Value = client
            .query("CreateAsset", &payload)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn create_file_asset_embeddings(
        &self,
        content_hash: &str,
        unit_kind: &str,
        unit_key: &str,
        content: &str,
    ) -> Result<(), String> {
        let payload = json!({
            "content_hash": content_hash,
            "unit_kind": unit_kind,
            "unit_key": unit_key,
            "content": content,
            "created_at": Self::now_rfc3339(),
        });
        let client = self.client();
        let _: Value = client
            .query("CreateAssetEmbeddingByHash", &payload)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[async_trait]
impl ImageIndexStore for HelixTextStore {
    async fn get_image_by_hash(
        &self,
        content_hash: &str,
    ) -> Result<Option<ExistingImageRecord>, String> {
        let payload = json!({ "content_hash": content_hash });
        let client = self.client();
        let result: Value = client
            .query("GetAssetByHash", &payload)
            .await
            .map_err(|e| e.to_string())
            .or_else(|error| {
                if Self::is_not_found_error(&error) {
                    Ok(Value::Null)
                } else {
                    Err(error)
                }
            })?;

        Ok(Self::extract_asset_id(&result).map(|image_id| ExistingImageRecord { image_id }))
    }

    async fn create_image_asset(
        &self,
        content_hash: &str,
        kind: &str,
        path: &str,
    ) -> Result<(), String> {
        let payload = json!({
            "content_hash": content_hash,
            "kind": kind,
            "path": path,
        });
        let client = self.client();
        let _: Value = client
            .query("CreateAsset", &payload)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn create_image_asset_embeddings(
        &self,
        content_hash: &str,
        unit_kind: &str,
        unit_key: &str,
        content: &str,
    ) -> Result<(), String> {
        let payload = json!({
            "content_hash": content_hash,
            "unit_kind": unit_kind,
            "unit_key": unit_key,
            "content": content,
            "created_at": Self::now_rfc3339(),
        });
        let client = self.client();
        let _: Value = client
            .query("CreateAssetEmbeddingByHash", &payload)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[async_trait]
impl VideoIndexStore for HelixTextStore {
    async fn create_video_asset(
        &self,
        content_hash: &str,
        kind: &str,
        path: &str,
    ) -> Result<(), String> {
        let payload = json!({
            "content_hash": content_hash,
            "kind": kind,
            "path": path,
        });
        let client = self.client();
        let _: Value = client
            .query("CreateAsset", &payload)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn create_video_asset_embeddings(
        &self,
        content_hash: &str,
        unit_kind: &str,
        unit_key: &str,
        content: &str,
    ) -> Result<(), String> {
        let payload = json!({
            "content_hash": content_hash,
            "unit_kind": unit_kind,
            "unit_key": unit_key,
            "content": content,
            "created_at": Self::now_rfc3339(),
        });
        let client = self.client();
        let _: Value = client
            .query("CreateAssetEmbeddingByHash", &payload)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
