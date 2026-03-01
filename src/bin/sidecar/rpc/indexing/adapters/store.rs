use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct ExistingFileRecord {
    pub file_id: String,
}

#[async_trait]
pub trait TextIndexStore: Send + Sync {
    async fn get_file_by_hash(
        &self,
        content_hash: &str,
    ) -> Result<Option<ExistingFileRecord>, String>;

    async fn create_file(
        &self,
        file_id: &str,
        content_hash: &str,
        content: &str,
        path: &str,
    ) -> Result<(), String>;

    async fn create_file_embeddings(
        &self,
        file_id: &str,
        content: &str,
        path: &str,
    ) -> Result<(), String>;
}
