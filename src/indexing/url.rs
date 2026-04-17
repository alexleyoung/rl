use scraper::{Html, Selector};
use sqlx::SqlitePool;
use tracing::warn;

const CHUNK_SIZE: usize = 1000;

pub async fn index_url(pool: &SqlitePool, resource_id: i64, url: &str) {
    let url = url.to_string();
    let text = match fetch_text(&url).await {
        Ok(t) => t,
        Err(e) => { warn!("fetch_text failed for {url}: {e}"); return; }
    };

    if text.trim().is_empty() {
        return;
    }

    // Remove stale URL chunks for this resource
    let _ = sqlx::query(
        "DELETE FROM search_fts WHERE source_kind='url' AND source_id=?"
    )
    .bind(resource_id)
    .execute(pool)
    .await;

    let chunks = chunk_text(&text, CHUNK_SIZE);
    for (i, chunk) in chunks.iter().enumerate() {
        let title = format!("Article chunk {}", i + 1);
        let res = sqlx::query(
            "INSERT INTO search_fts(source_kind, source_id, title, body) VALUES ('url', ?, ?, ?)"
        )
        .bind(resource_id)
        .bind(&title)
        .bind(chunk)
        .execute(pool)
        .await;

        if let Err(e) = res {
            warn!("FTS insert failed for url chunk {i}: {e}");
            break;
        }
    }

    tracing::info!("indexed {} URL chunks for resource {resource_id}", chunks.len());
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

    // Remove script and style elements
    let sel_body = Selector::parse("body").unwrap();
    let _sel_skip = Selector::parse("script, style, nav, footer, header").unwrap();

    let body = doc.select(&sel_body).next();
    let root = if let Some(b) = body { b.inner_html() } else { html.to_string() };

    // Re-parse just the body fragment
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

    // Basic cleanup: collapse whitespace
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn chunk_text(text: &str, size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut buf = String::with_capacity(size);
    for word in text.split_whitespace() {
        if buf.len() + word.len() + 1 > size && !buf.is_empty() {
            chunks.push(buf.trim().to_string());
            buf.clear();
        }
        if !buf.is_empty() { buf.push(' '); }
        buf.push_str(word);
    }
    if !buf.trim().is_empty() {
        chunks.push(buf.trim().to_string());
    }
    chunks
}
