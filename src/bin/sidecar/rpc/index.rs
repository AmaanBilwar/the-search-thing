use serde::Deserialize;
use serde_json::json;

use crate::sidecar::backend_proxy::{proxy_index_start, proxy_index_status};
use crate::sidecar::protocol::{
    err_response, ok_response, parse_params, JsonRpcRequest, JsonRpcResponse,
};

#[derive(Debug, Deserialize)]
struct IndexStartParams {
    dir: String,
}

#[derive(Debug, Deserialize)]
struct IndexStatusParams {
    job_id: String,
}

pub fn handle_start(request: &JsonRpcRequest) -> JsonRpcResponse {
    let parsed: IndexStartParams = match parse_params(request) {
        Ok(parsed) => parsed,
        Err(error_response) => return error_response,
    };

    match proxy_index_start(&parsed.dir) {
        Ok(result) => ok_response(request.id.clone(), result),
        Err((code, message)) => err_response(
            request.id.clone(),
            code,
            "Index start failed",
            Some(json!({ "reason": message })),
        ),
    }
}

pub fn handle_status(request: &JsonRpcRequest) -> JsonRpcResponse {
    let parsed: IndexStatusParams = match parse_params(request) {
        Ok(parsed) => parsed,
        Err(error_response) => return error_response,
    };

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
