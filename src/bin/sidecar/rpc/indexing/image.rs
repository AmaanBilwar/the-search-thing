use serde_json::{json, Map, Value};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct ImageIndexResult {
    pub path: String,
    pub image_id: Option<String>,
    pub indexed: bool,
    pub error: Option<String>,
}

pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

pub fn mime_hint_from_path(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => "jpeg",
        Some("png") => "png",
        Some("webp") => "webp",
        Some("gif") => "gif",
        Some("bmp") => "bmp",
        Some("tiff") | Some("tif") => "tiff",
        _ => "jpeg",
    }
}

fn strip_code_fences(content: &str) -> String {
    let text = content.trim();
    if !text.starts_with("```") {
        return text.to_string();
    }

    let mut lines: Vec<&str> = text.lines().collect();
    if !lines.is_empty() && lines[0].starts_with("```") {
        lines.remove(0);
    }
    if !lines.is_empty() && lines[lines.len() - 1].trim().starts_with("```") {
        lines.pop();
    }

    lines.join("\n").trim().to_string()
}

fn string_list_field(map: &Map<String, Value>, key: &str) -> Vec<String> {
    map.get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<String>>()
        })
        .unwrap_or_default()
}

fn string_field(map: &Map<String, Value>, key: &str) -> String {
    map.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .unwrap_or_default()
}

fn normalized_summary_from_map(map: &Map<String, Value>, fallback_text: &str) -> Value {
    let mut summary = string_field(map, "summary");
    if summary.starts_with("```") {
        summary = normalize_summary_content(&summary)
            .get("summary")
            .and_then(Value::as_str)
            .unwrap_or(&summary)
            .to_string();
    }

    if summary.is_empty() {
        summary = fallback_text.to_string();
    }

    json!({
        "summary": summary,
        "objects": string_list_field(map, "objects"),
        "actions": string_list_field(map, "actions"),
        "setting": string_field(map, "setting"),
        "ocr": string_field(map, "ocr"),
        "quality": string_field(map, "quality"),
    })
}

pub fn normalize_summary_content(content: &str) -> Value {
    let text = strip_code_fences(content);

    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(map)) => normalized_summary_from_map(&map, &text),
        _ => json!({
            "summary": text,
            "objects": [],
            "actions": [],
            "setting": "",
            "ocr": "",
            "quality": "",
        }),
    }
}

pub fn build_embedding_text(summary: &Value) -> String {
    let mut parts = Vec::new();

    let add_part = |parts: &mut Vec<String>, label: &str, value: String| {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            parts.push(format!("{}: {}", label, trimmed));
        }
    };

    if let Some(text) = summary.get("summary").and_then(Value::as_str) {
        add_part(&mut parts, "summary", text.to_string());
    }

    if let Some(items) = summary.get("objects").and_then(Value::as_array) {
        let joined = items
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .collect::<Vec<&str>>()
            .join(", ");
        add_part(&mut parts, "objects", joined);
    }

    if let Some(items) = summary.get("actions").and_then(Value::as_array) {
        let joined = items
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .collect::<Vec<&str>>()
            .join(", ");
        add_part(&mut parts, "actions", joined);
    }

    if let Some(text) = summary.get("setting").and_then(Value::as_str) {
        add_part(&mut parts, "setting", text.to_string());
    }

    if let Some(text) = summary.get("ocr").and_then(Value::as_str) {
        add_part(&mut parts, "ocr", text.to_string());
    }

    if let Some(text) = summary.get("quality").and_then(Value::as_str) {
        add_part(&mut parts, "quality", text.to_string());
    }

    parts.join(" | ")
}
