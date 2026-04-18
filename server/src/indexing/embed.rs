use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use parking_lot::Mutex;
use sqlx::SqlitePool;
use tracing::{info, warn};

pub struct Embedder {
    inner: Mutex<TextEmbedding>,
}

impl Embedder {
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        info!("loading embedding model (first run may download ~100MB)…");
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::BGESmallENV15)
                .with_cache_dir(cache_dir)
                .with_show_download_progress(true),
        )?;
        info!("embedding model ready");
        Ok(Self { inner: Mutex::new(model) })
    }

    /// Embed a batch of texts. Runs on a blocking thread.
    pub async fn embed(self: Arc<Self>, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        tokio::task::spawn_blocking(move || {
            let guard = self.inner.lock();
            Ok(guard.embed(texts, None)?)
        })
        .await?
    }
}

/// Serialize f32 slice to little-endian BLOB.
pub fn vec_to_blob(v: &[f32]) -> Vec<u8> {
    bytemuck::cast_slice(v).to_vec()
}

/// Deserialize BLOB to f32 vec.
pub fn blob_to_vec(b: &[u8]) -> Vec<f32> {
    bytemuck::cast_slice(b).to_vec()
}

/// Cosine similarity between two equal-length vectors.
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 { 0.0 } else { dot / (na * nb) }
}

const EMBED_BATCH: usize = 32;

/// Delete all embedding rows for a (source_kind, source_id) pair, then insert fresh rows.
/// Chunks are embedded in batches of `EMBED_BATCH` to bound peak memory usage.
pub async fn reindex_chunks(
    pool: &SqlitePool,
    embedder: Arc<Embedder>,
    source_kind: &str,
    source_id: i64,
    chunks: Vec<String>,
) {
    let _ = sqlx::query("DELETE FROM embeddings WHERE source_kind=? AND source_id=?")
        .bind(source_kind)
        .bind(source_id)
        .execute(pool)
        .await;

    if chunks.is_empty() {
        return;
    }

    let sk = source_kind.to_string();
    let mut global_idx: i64 = 0;
    let mut dim: i64 = 0;

    for batch in chunks.chunks(EMBED_BATCH) {
        let batch_owned: Vec<String> = batch.to_vec();
        let emb = embedder.clone();
        let vectors = match emb.embed(batch_owned.clone()).await {
            Ok(v) => v,
            Err(e) => { warn!("embed failed for {sk}/{source_id} at idx {global_idx}: {e}"); return; }
        };

        if dim == 0 {
            dim = vectors.first().map(|v| v.len()).unwrap_or(0) as i64;
        }

        for (text, vec) in batch_owned.iter().zip(vectors.iter()) {
            let blob = vec_to_blob(vec);
            let res = sqlx::query(
                "INSERT INTO embeddings(source_kind, source_id, chunk_idx, text, dim, vector)
                 VALUES (?, ?, ?, ?, ?, ?)"
            )
            .bind(&sk)
            .bind(source_id)
            .bind(global_idx)
            .bind(text)
            .bind(dim)
            .bind(&blob)
            .execute(pool)
            .await;

            if let Err(e) = res {
                warn!("embed insert failed for chunk {global_idx}: {e}");
                return;
            }
            global_idx += 1;
        }
    }

    info!("embedded {} chunks for {sk}/{source_id}", global_idx);
}
