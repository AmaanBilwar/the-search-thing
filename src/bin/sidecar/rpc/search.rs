use serde::Deserialize;
use serde_json::json;

use crate::sidecar::backend_proxy::proxy_search_query;
use crate::sidecar::protocol::{
    err_response, ok_response, parse_params, JsonRpcRequest, JsonRpcResponse,
};

#[derive(Debug, Deserialize)]
struct SearchQueryParams {
    q: String,
}

pub fn handle_query(request: &JsonRpcRequest) -> JsonRpcResponse {
    let parsed: SearchQueryParams = match parse_params(request) {
        Ok(parsed) => parsed,
        Err(error_response) => return error_response,
    };

    match proxy_search_query(&parsed.q) {
        Ok(result) => ok_response(request.id.clone(), result),
        Err((code, message)) => err_response(
            request.id.clone(),
            code,
            "Search query failed",
            Some(json!({ "reason": message })),
        ),
    }
}
