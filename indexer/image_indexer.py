import asyncio
import json
import uuid
from pathlib import Path
from typing import List

from the_search_thing import search_images

from utils.clients import get_helix_client

# creating image node
async def create_img(file_id: str, content: str, path: str) -> str:
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