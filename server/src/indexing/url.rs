use std::sync::Arc;

use scraper::{Html, Selector};
use sqlx::SqlitePool;
use tracing::warn;

use super::{chunk::chunk_text, embed::{reindex_chunks, Embedder}};

pub async fn index_url(pool: &SqlitePool, embedder: Option<Arc<Embedder>>, resource_id: i64, url: &str) {
    let text = match fetch_text(url).await {
        Ok(t) => t,
        Err(e) => { warn!("fetch_text failed for {url}: {e}"); return; }
    };

    if text.trim().is_empty() {
        return;
    }

    let chunks = chunk_text(&text);

    // FTS indexing
    let _ = sqlx::query("DELETE FROM search_fts WHERE source_kind='url' AND source_id=?")
        .bind(resource_id)
        .execute(pool)
        .await;

    for (i, chunk) in chunks.iter().enumerate() {
        let title = format!("Article chunk {}", i + 1);
        if let Err(e) = sqlx::query(
            "INSERT INTO search_fts(source_kind, source_id, title, body) VALUES ('url', ?, ?, ?)"
        )
        .bind(resource_id)
        .bind(&title)
        .bind(chunk)
        .execute(pool)
        .await
        {
            warn!("FTS insert failed for url chunk {i}: {e}");
            break;
        }
    }

    tracing::info!("FTS: indexed {} URL chunks for resource {resource_id}", chunks.len());

    // Vector indexing
    if let Some(emb) = embedder {
        reindex_chunks(pool, emb, "url", resource_id, chunks).await;
    }
}

async fn fetch_text(url: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::builder()
        .user_agent("rl-indexer/0.1")
        .timeout(std::time::Duration::from_secs(15))
        .build()?;
    let html = client.get(url).send().await?.text().await?;
    Ok(html_to_text(&html))
}

fn html_to_text(html: &str) -> String {
    let doc = Html::parse_document(html);
    let sel_body = Selector::parse("body").unwrap();
    let root = doc.select(&sel_body).next()
        .map(|b| b.inner_html())
        .unwrap_or_else(|| html.to_string());

    let fragment = Html::parse_fragment(&root);
    let mut text = String::new();
    for node in fragment.tree.values() {
        if let Some(t) = node.as_text() {
            let s = t.trim();
            if !s.is_empty() {
                text.push_str(s);
                text.push(' ');
            }
        }
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
