use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::sidecar::backend_proxy::{proxy_index_start, proxy_index_status};
use crate::sidecar::protocol::{
    err_response, ok_response, parse_params, JsonRpcRequest, JsonRpcResponse,
};
use crate::sidecar::rpc::indexing::adapters::hash::Sha256PathHasher;
use crate::sidecar::rpc::indexing::adapters::helix::HelixTextStore;
use crate::sidecar::rpc::indexing::text_indexer::file_indexer;

#[derive(Debug, Deserialize)]
struct IndexStartParams {
    dir: String,
}

#[derive(Debug, Deserialize)]
struct IndexStatusParams {
    job_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct IndexJobStatus {
    job_id: String,
    dir: String,
    status: String,
    phase: String,
    batch_size: usize,
    text_found: usize,
    text_indexed: usize,
    text_errors: usize,
    text_skipped: usize,
    video_found: usize,
    video_indexed: usize,
    video_errors: usize,
    video_skipped: usize,
    image_found: usize,
    image_indexed: usize,
    image_errors: usize,
    image_skipped: usize,
    message: String,
    error: String,
    started_at: String,
    updated_at: String,
    finished_at: Option<String>,
}

static JOB_COUNTER: AtomicU64 = AtomicU64::new(1);
static JOB_STORE: OnceLock<Mutex<HashMap<String, IndexJobStatus>>> = OnceLock::new();

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

fn index_mode() -> String {
    env::var("SIDECAR_INDEX_MODE").unwrap_or_else(|_| "python-proxy".to_string())
}

fn store() -> &'static Mutex<HashMap<String, IndexJobStatus>> {
    JOB_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn make_job_id() -> String {
    let seq = JOB_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("rust-text-{}-{}", now_string(), seq)
}

fn put_job(status: IndexJobStatus) -> Result<(), String> {
    let mut jobs = store().lock().map_err(|e| e.to_string())?;
    jobs.insert(status.job_id.clone(), status);
    Ok(())
}

fn update_job<F>(job_id: &str, updater: F) -> Result<(), String>
where
    F: FnOnce(&mut IndexJobStatus),
{
    let mut jobs = store().lock().map_err(|e| e.to_string())?;
    let job = jobs
        .get_mut(job_id)
        .ok_or_else(|| format!("job not found: {}", job_id))?;
    updater(job);
    job.updated_at = now_string();
    Ok(())
}

fn get_job(job_id: &str) -> Result<Option<IndexJobStatus>, String> {
    let jobs = store().lock().map_err(|e| e.to_string())?;
    Ok(jobs.get(job_id).cloned())
}

fn spawn_rust_text_job(job_id: String, dir: String) {
    thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(error) => {
                let _ = update_job(&job_id, |job| {
                    job.status = "failed".to_string();
                    job.phase = "done".to_string();
                    job.error = format!("failed to init runtime: {}", error);
                    job.message = "Indexing failed".to_string();
                    job.finished_at = Some(now_string());
                });
                return;
            }
        };

        let hasher = Sha256PathHasher;
        let store = match HelixTextStore::from_env() {
            Ok(store) => store,
            Err(error) => {
                let _ = update_job(&job_id, |job| {
                    job.status = "failed".to_string();
                    job.phase = "done".to_string();
                    job.error = error;
                    job.message = "Indexing failed".to_string();
                    job.finished_at = Some(now_string());
                });
                return;
            }
        };

        let _ = update_job(&job_id, |job| {
            job.phase = "index_text".to_string();
            job.message = "Indexing text files (Rust orchestrator)".to_string();
        });

        let results = runtime.block_on(file_indexer(vec![dir.clone()], &hasher, &store));

        let text_found = results.len();
        let text_indexed = results.iter().filter(|r| r.indexed).count();
        let text_skipped = results
            .iter()
            .filter(|r| r.error.as_deref() == Some("Duplicate content hash"))
            .count();
        let text_errors = results
            .iter()
            .filter(|r| !r.indexed && r.error.as_deref() != Some("Duplicate content hash"))
            .count();

        let failed_example = results
            .iter()
            .find(|r| !r.indexed && r.error.as_deref() != Some("Duplicate content hash"))
            .and_then(|r| r.error.clone())
            .unwrap_or_default();

        let _ = update_job(&job_id, |job| {
            job.text_found = text_found;
            job.text_indexed = text_indexed;
            job.text_skipped = text_skipped;
            job.text_errors = text_errors;
            job.phase = "done".to_string();
            job.finished_at = Some(now_string());

            if text_errors > 0 {
                job.status = "failed".to_string();
                job.message = "Text indexing failed".to_string();
                job.error = failed_example;
            } else {
                job.status = "completed".to_string();
                job.message = "Text indexing complete".to_string();
                job.error.clear();
            }
        });
    });
}

pub fn handle_start(request: &JsonRpcRequest) -> JsonRpcResponse {
    let parsed: IndexStartParams = match parse_params(request) {
        Ok(parsed) => parsed,
        Err(error_response) => return error_response,
    };

    if index_mode() != "rust-text" {
        return match proxy_index_start(&parsed.dir) {
            Ok(result) => ok_response(request.id.clone(), result),
            Err((code, message)) => err_response(
                request.id.clone(),
                code,
                "Index start failed",
                Some(json!({ "reason": message })),
            ),
        };
    }

    let job_id = make_job_id();
    let now = now_string();
    let status = IndexJobStatus {
        job_id: job_id.clone(),
        dir: parsed.dir.clone(),
        status: "running".to_string(),
        phase: "scan_text".to_string(),
        batch_size: 0,
        text_found: 0,
        text_indexed: 0,
        text_errors: 0,
        text_skipped: 0,
        video_found: 0,
        video_indexed: 0,
        video_errors: 0,
        video_skipped: 0,
        image_found: 0,
        image_indexed: 0,
        image_errors: 0,
        image_skipped: 0,
        message: "Starting Rust text indexer".to_string(),
        error: String::new(),
        started_at: now.clone(),
        updated_at: now,
        finished_at: None,
    };

    if let Err(error) = put_job(status) {
        return err_response(
            request.id.clone(),
            -32603,
            "Index start failed",
            Some(json!({ "reason": error })),
        );
    }

    spawn_rust_text_job(job_id.clone(), parsed.dir);
    ok_response(
        request.id.clone(),
        json!({ "success": true, "job_id": job_id }),
    )
}

pub fn handle_status(request: &JsonRpcRequest) -> JsonRpcResponse {
    let parsed: IndexStatusParams = match parse_params(request) {
        Ok(parsed) => parsed,
        Err(error_response) => return error_response,
    };

    match get_job(&parsed.job_id) {
        Ok(Some(status)) => return ok_response(request.id.clone(), json!(status)),
        Ok(None) => {}
        Err(error) => {
            return err_response(
                request.id.clone(),
                -32603,
                "Index status failed",
                Some(json!({ "reason": error })),
            )
        }
    }

    match proxy_index_status(&parsed.job_id) {
        Ok(result) => ok_response(request.id.clone(), result),
        Err((code, message)) => err_response(
            request.id.clone(),
            code,
            "Index status failed",
            Some(json!({ "reason": message })),
        ),
    }
}
