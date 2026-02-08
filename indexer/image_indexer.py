import asyncio
import json
import uuid
from pathlib import Path
from typing import List

from the_search_thing import get_base64_bytes

from utils.clients import get_groq_client, get_helix_client


async def img_indexer(
    file_paths: List[str] | str,
) -> List[dict]:
    if isinstance(file_paths, str):
        file_paths = [file_paths]
    if not file_paths:
        print("No file paths provided. exiting image indexing.")
        return []

    results: List[dict] = []
    for path in file_paths:
        p = Path(path)
        if not p.exists():
            print(f"[WARN] Skipping (not found): {path}")
            results.append(
                {
                    "path": path,
                    "file_id": None,
                    "indexed": False,
                    "error": "Path not found",
                }
            )
            continue

        try:
            img_base64 = get_base64_bytes(path)
        except Exception as e:
            print(f"[WARN] Skipping (Bytes Extraction failed): {path} — {e}")
            results.append(
                {"path": path, "file_id": None, "indexed": False, "error": str(e)}
            )
            continue
            
        try:
            summary_payload, embedding_text = await generate_summary(img_base64)
        except Exception as e:
            print(f"[WARN] Skipping (Summary failed): {path} — {e}")
            results.append(
                {"path": path, "file_id": None, "indexed": False, "error": str(e)}
            )
            continue
            
        file_id = str(uuid.uuid4())
        try:
            await create_img(file_id, json.dumps(summary_payload), path=path)
            await create_img_embeddings(file_id, embedding_text, path=path)
            results.append({"path": path, "file_id": file_id, "indexed": True})
            print(f"[OK] Indexed image: {path}")
        except Exception as e:
            print(f"[ERROR] Indexing failed for {path}: {e}")
            results.append(
                {"path": path, "file_id": file_id, "indexed": False, "error": str(e)}
            )
            
    return results

    
# creating image node
async def create_img(file_id: str, content: str, path: str) -> str:
    # here content is a raw json summary
    file_params = {"file_id": file_id, "content": content, "path": path}

    def _query() -> str:
        helix_client = get_helix_client()
        return json.dumps(helix_client.query("CreateImage", file_params))

    return await asyncio.to_thread(_query)


async def create_img_embeddings(file_id: str, content: str, path: str) -> str:
    file_params = {"file_id": file_id, "content": content, "path": path}

    def _query() -> str:
        helix_client = get_helix_client()
        return json.dumps(
            helix_client.query(
                "CreateImageEmbeddings",
                file_params,
            )
        )

    return await asyncio.to_thread(_query)

def _base64_to_uri(img_base64: str, mime_hint: str = "jpeg" ) -> str:
    return f"data:image/{mime_hint};base64,{img_base64}"

def _normalize_summary_content(content_str: str) -> dict:
    """
    Normalize model output into a JSON-friendly dict without code fences.
    Accepts raw text (possibly with ```json fences) and tries to JSON-parse.
    Falls back to wrapping the text under 'summary'.
    """
    text = content_str.strip()

    if text.startswith("```"):
        lines = text.splitlines()
        if lines and lines[0].startswith("```"):
            lines = lines[1:]
        if lines and lines[-1].strip().startswith("```"):
            lines = lines[:-1]
        text = "\n".join(lines).strip()

    parsed_obj = None
    try:
        parsed_obj = json.loads(text)
    except Exception:
        parsed_obj = None

    if not isinstance(parsed_obj, dict):
        return {"summary": text}

    summary_text = parsed_obj.get("summary")
    if isinstance(summary_text, str) and summary_text.strip().startswith("```"):
        nested = _normalize_summary_content(summary_text)
        if isinstance(nested, dict):
            summary_text = nested.get("summary", summary_text)

    objects = (
        parsed_obj.get("objects") if isinstance(parsed_obj.get("objects"), list) else []
    )
    actions = (
        parsed_obj.get("actions") if isinstance(parsed_obj.get("actions"), list) else []
    )
    setting = (
        parsed_obj.get("setting") if isinstance(parsed_obj.get("setting"), str) else ""
    )
    ocr = (
        parsed_obj.get("ocr") if isinstance(parsed_obj.get("ocr"), str) else ""
    )
    quality = (
        parsed_obj.get("quality") if isinstance(parsed_obj.get("quality"), str) else ""
    )

    if summary_text is None:
        summary_text = text

    return {
        "summary": summary_text,
        "objects": objects,
        "actions": actions,
        "setting": setting,
        "ocr": ocr,
        "quality": quality,
    }
    
def _build_embedding_text(summary: dict) -> str:
    

async def generate_summary(
    image_base64: str,
) -> tuple[dict, str]:
    """
    Summarize a single image (base64) and return both:
    - structured JSON payload
    - normalized text string for embeddings
    """
    if not image_base64:
        print("[WARN] No bytes data provided")
        return {},""

    client = get_groq_client()
    data_uri = _base64_to_uri(image_base64, "jpeg")
    
    prompt = (
        "You are an expert vision assistant. Provide a concise JSON summary for "
        "the provided image. Respond with JSON only (no code fences). Use the schema: "
        '{"summary": "<1-2 sentences>", "objects": ["..."], "actions": ["..."], '
        '"setting": "<location or scene>", "ocr": "<visible text or empty>", "quality": "<good|low>"}'
    )
    
    response = client.chat.completions.create(
        model="meta-llama/llama-4-maverick-17b-128e-instruct",
        messages=[
            {
                "role": "user",
                "content" : [
                    {"type": "text", "text": prompt},
                    {"type": "image_url", "image_url": {"url": data_uri}},
                ],
            }
        ],
        max_tokens=500,
        temperature=0.2,
    )
    content = response.choices[0].message.content
    if isinstance(content, list):
        parts = []
        for part in content:
            if isinstance(part, dict) and "text" in part:
                parts.append(part["text"])
            else:
                parts.append(str(part))
        content = " ".join(parts)
    summary_payload = (
        _normalize_summary_content(content)
        if isinstance(content, str)
        else {"summary": str(content)}
    )
    embedding_text = _build_embedding_text(summary_payload)
    
    return summary_payload, embedding_text
    