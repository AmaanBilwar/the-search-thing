use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use std::collections::HashSet;
use std::fs;
use walkdir::WalkDir;

use crate::sidecar::protocol::{
    err_response, ok_response, parse_params, JsonRpcRequest, JsonRpcResponse,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WalkTextBatchParams {
    dir: String,
    text_exts: Vec<String>,
    #[serde(default)]
    ignore_exts: Vec<String>,
    #[serde(default)]
    ignore_files: Vec<String>,
    cursor: usize,
    batch_size: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WalkTextBatchResult {
    batch: Vec<(String, String)>,
    cursor: usize,
    done: bool,
    scanned_count: usize,
    skipped_count: usize,
}

fn normalize_extensions(values: Vec<String>) -> HashSet<String> {
    values
        .into_iter()
        .map(|ext| {
            let mut normalized = ext.trim().to_lowercase();
            if !normalized.is_empty() && !normalized.starts_with('.') {
                normalized = format!(".{}", normalized);
            }
            normalized
        })
        .filter(|ext| !ext.is_empty())
        .collect()
}

fn normalize_file_names(values: Vec<String>) -> HashSet<String> {
    values
        .into_iter()
        .map(|name| name.trim().to_lowercase())
        .filter(|name| !name.is_empty())
        .collect()
}

fn walk_text_batch(params: WalkTextBatchParams) -> Result<WalkTextBatchResult, String> {
    let text_exts = normalize_extensions(params.text_exts);
    let ignore_exts = normalize_extensions(params.ignore_exts);
    let ignore_files = normalize_file_names(params.ignore_files);

    let mut batch: Vec<(String, String)> = Vec::new();
    let mut scanned_count = 0usize;
    let mut skipped_count = 0usize;
    let mut next_cursor = params.cursor;

    for (idx, entry) in WalkDir::new(&params.dir).into_iter().enumerate() {
        let entry = entry.map_err(|e| e.to_string())?;
        if idx < params.cursor {
            continue;
        }

        next_cursor = idx + 1;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        scanned_count += 1;

        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            if ignore_files.contains(&name.to_lowercase()) {
                skipped_count += 1;
                continue;
            }
        }

        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| format!(".{}", s.to_lowercase()));

        if let Some(ref extension) = ext {
            if ignore_exts.contains(extension) {
                skipped_count += 1;
                continue;
            }
        }

        match ext {
            Some(ref extension) if text_exts.contains(extension) => {
                if let Ok(content) = fs::read_to_string(path) {
                    batch.push((path.to_string_lossy().to_string(), content));
                }
            }
            _ => {
                skipped_count += 1;
            }
        }

        if batch.len() >= params.batch_size {
            return Ok(WalkTextBatchResult {
                batch,
                cursor: next_cursor,
                done: false,
                scanned_count,
                skipped_count,
            });
        }
    }

    Ok(WalkTextBatchResult {
        batch,
        cursor: next_cursor,
        done: true,
        scanned_count,
        skipped_count,
    })
}

pub fn handle_walk_text_batch(request: &JsonRpcRequest) -> JsonRpcResponse {
    let parsed: WalkTextBatchParams = match parse_params(request) {
        Ok(parsed) => parsed,
        Err(error_response) => return error_response,
    };

    match walk_text_batch(parsed) {
        Ok(result) => match serde_json::to_value(result) {
            Ok(value) => ok_response(request.id.clone(), value),
            Err(error) => err_response(
                request.id.clone(),
                -32603,
                "Internal error",
                Some(json!({ "reason": error.to_string() })),
            ),
        },
        Err(error) => err_response(
            request.id.clone(),
            -32603,
            "Internal error",
            Some(json!({ "reason": error })),
        ),
    }
}
