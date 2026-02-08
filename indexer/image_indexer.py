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

    def summarize_image_bytes(
        image_id: str, image_bytes: bytes, mime_hint: str = "jpeg"
    ) -> dict:
        data_uri = _bytes_to_data_uri(image_bytes, mime_hint)
        prompt = (
            "You are an expert vision assistant. Provide a concise JSON summary for "
            "the provided video frame. Respond with JSON only (no code fences). Use the schema: "
            '{"summary": "<1-2 sentences>", "objects": ["..."], "actions": ["..."], '
            '"setting": "<location or scene>", "quality": "<good|low>"}'
        )
        response = client.chat.completions.create(
            model="meta-llama/llama-4-maverick-17b-128e-instruct",
            messages=[
                {
                    "role": "user",
                    "content": [
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
        return {"image": image_id, "summary": summary_payload}

    def process_batch(batch: list[tuple[str, int, bytes]]) -> list[dict]:
        batch_results = []
        for chunk_key, idx, img_bytes in batch:
            image_id = f"{chunk_key}_{idx}"
            try:
                batch_results.append(summarize_image_bytes(image_id, img_bytes, "jpeg"))
            except Exception as batch_err:
                batch_results.append(
                    {"image": image_id, "summary": None, "error": str(batch_err)}
                )
        return batch_results

    def run_batches():
        results = []
        batches = [
            flat_items[i : i + batch_size]
            for i in range(0, len(flat_items), batch_size)
        ]
        with ThreadPoolExecutor(max_workers=max_workers) as executor:
            future_to_batch = {
                executor.submit(process_batch, batch): batch for batch in batches
            }
            for future in as_completed(future_to_batch):
                results.extend(future.result())
        return results

    loop = asyncio.get_event_loop()
    raw_results = await loop.run_in_executor(None, run_batches)

    grouped: dict[str, list[dict]] = {}
    for entry in raw_results:
        image_id = entry["image"]
        chunk_key = image_id.rsplit("_", 1)[0] if "_" in image_id else image_id
        grouped.setdefault(chunk_key, []).append(entry)

    for chunk_key, entries in grouped.items():
        print(f"[OK] Summarized {len(entries)} frames for chunk {chunk_key}")

    return grouped