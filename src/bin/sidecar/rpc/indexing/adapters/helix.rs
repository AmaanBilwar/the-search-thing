use async_trait::async_trait;
use helix_rs::{HelixDB, HelixDBClient};
use serde_json::{json, Value};
use std::env;

use crate::sidecar::rpc::indexing::adapters::store::{ExistingFileRecord, TextIndexStore};

#[derive(Debug, Clone)]
pub struct HelixTextStore {
    endpoint: String,
    port: u16,
    api_key: Option<String>,
}

impl HelixTextStore {
    pub fn from_env() -> Result<Self, String> {
        let endpoint = env::var("HELIX_ENDPOINT").unwrap_or_else(|_| "http://localhost".to_string());
        let port = env::var("HELIX_PORT")
            .unwrap_or_else(|_| "7003".to_string())
            .parse::<u16>()
            .map_err(|e| format!("invalid HELIX_PORT: {}", e))?;
        let api_key = env::var("HELIX_API_KEY").ok().filter(|v| !v.trim().is_empty());

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

    fn extract_existing_file_id(value: &Value) -> Option<String> {
        if let Some(file_id) = value.get("file_id").and_then(Value::as_str) {
            return Some(file_id.to_string());
        }

        if let Some(file_node) = value.get("file") {
            if let Some(file_id) = file_node.get("file_id").and_then(Value::as_str) {
                return Some(file_id.to_string());
            }
            if let Some(file_array) = file_node.as_array() {
                if let Some(first) = file_array.first() {
                    if let Some(file_id) = first.get("file_id").and_then(Value::as_str) {
                        return Some(file_id.to_string());
                    }
                }
            }
        }

        if let Some(array) = value.as_array() {
            if let Some(first) = array.first() {
                if let Some(file_id) = first.get("file_id").and_then(Value::as_str) {
                    return Some(file_id.to_string());
                }
            }
        }

        None
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
            .query("GetFileByHash", &payload)
            .await
            .map_err(|e| e.to_string())?;

        Ok(Self::extract_existing_file_id(&result).map(|file_id| ExistingFileRecord { file_id }))
    }

    async fn create_file(
        &self,
        file_id: &str,
        content_hash: &str,
        content: &str,
        path: &str,
    ) -> Result<(), String> {
        let payload = json!({
            "file_id": file_id,
            "content_hash": content_hash,
            "content": content,
            "path": path,
        });
        let client = self.client();
        let _: Value = client
            .query("CreateFile", &payload)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn create_file_embeddings(
        &self,
        file_id: &str,
        content: &str,
        path: &str,
    ) -> Result<(), String> {
        let payload = json!({
            "file_id": file_id,
            "content": content,
            "path": path,
        });
        let client = self.client();
        let _: Value = client
            .query("CreateFileEmbeddings", &payload)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
