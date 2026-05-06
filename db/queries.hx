QUERY CreateAsset(kind: String, path: String, content_hash: String) =>
    existing <- N<Asset>::WHERE(_::{content_hash}::EQ(content_hash))
    asset <- existing::UpsertN({
        kind: kind,
        content_hash: content_hash,
        path: path,
    })
    RETURN asset

QUERY GetAssetByHash(content_hash: String) =>
    asset <- N<Asset>({content_hash: content_hash})
    RETURN asset

QUERY CreateAssetEmbeddingByHash(content_hash: String, unit_kind: String, unit_key: String, content: String) =>
    asset <- N<Asset>({content_hash: content_hash})
    existing_embedding <- asset::Out<HasAssetEmbedding>
        ::WHERE(_::{unit_kind}::EQ(unit_kind))
        ::WHERE(_::{unit_key}::EQ(unit_key))
    embedding <- existing_embedding::UpsertV(Embed(content), {
        unit_kind: unit_kind,
        content: content,
    })
    existing_edge <- E<HasAssetEmbedding>
    has_embedding <- existing_edge::UpsertE({})::From(asset)::To(embedding)
    RETURN embedding

QUERY SearchAssetEmbeddings(query: String) =>
    embeddings <- SearchV<AssetEmbedding>(Embed(query), 120)
        ::RerankMMR(lambda: 0.7)
        ::RANGE(0, 50)
    assets <- embeddings::In<HasAssetEmbedding>
    RETURN assets
