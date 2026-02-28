use serde_json::json;

use crate::sidecar::backend_proxy::backend_base_url;
use crate::sidecar::protocol::{ok_response, JsonRpcResponse};

pub fn handle(id: serde_json::Value) -> JsonRpcResponse {
    ok_response(
        id,
        json!({
            "ok": true,
            "service": "the-search-thing-sidecar",
            "version": env!("CARGO_PKG_VERSION"),
            "backend_url": backend_base_url(),
            "index_mode": "python-proxy",
            "search_mode": "python-proxy",
        }),
    )
}
