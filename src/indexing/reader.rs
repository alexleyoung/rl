//! Content extraction for the reader frontend.
//!
//! Unlike `indexing/url.rs` and `indexing/pdf.rs` which produce flat text
//! chunks for FTS search, this module produces structured HTML (and a plain
//! text copy) suitable for a distraction-free reading view.

use crate::models::reading;
use sqlx::SqlitePool;
use tracing::{info, warn};
use url::Url;

/// Extract reader content from a URL. Uses the `readability` crate for
/// Mozilla-style article extraction; falls back to a simple paragraph wrap
/// of stripped body text if that fails.
pub async fn extract_url(pool: &SqlitePool, resource_id: i64, url: &str) {
    let _ = reading::mark_pending(pool, resource_id, "url").await;

    let html = match fetch_html(url).await {
        Ok(h) => h,
        Err(e) => {
            warn!("reader: fetch failed for {url}: {e}");
            let _ = reading::mark_failed(pool, resource_id, "url").await;
            return;
        }
    };

    let parsed_url = match Url::parse(url) {
        Ok(u) => u,
        Err(e) => {
            warn!("reader: bad url {url}: {e}");
            let _ = reading::mark_failed(pool, resource_id, "url").await;
            return;
        }
    };

    // readability is a sync CPU task — run on the blocking thread pool.
    let html_clone = html.clone();
    let extracted = tokio::task::spawn_blocking(move || {
        let mut bytes = html_clone.as_bytes();
        readability::extractor::extract(&mut bytes, &parsed_url)
    })
    .await;

    match extracted {
        Ok(Ok(product)) if !product.text.trim().is_empty() => {
            let word_count = word_count(&product.text);
            if let Err(e) = reading::upsert_ok(
                pool,
                resource_id,
                &product.content,
                &product.text,
                "url",
                word_count,
            )
            .await
            {
                warn!("reader: db upsert failed for resource {resource_id}: {e}");
                let _ = reading::mark_failed(pool, resource_id, "url").await;
                return;
            }
            info!(
                "reader: extracted {} words from url for resource {resource_id}",
                word_count
            );
        }
        other => {
            if let Ok(Err(e)) = &other {
                warn!("reader: readability failed for {url}: {e:?}");
            } else if let Err(e) = &other {
                warn!("reader: spawn_blocking failed: {e}");
            }
            // Fallback: strip body text and wrap paragraphs.
            let text = strip_body_text(&html);
            if text.trim().is_empty() {
                let _ = reading::mark_failed(pool, resource_id, "url").await;
                return;
            }
            let content_html = wrap_paragraphs(&text);
            let word_count = word_count(&text);
            if let Err(e) = reading::upsert_ok(
                pool, resource_id, &content_html, &text, "url", word_count,
            )
            .await
            {
                warn!("reader: fallback upsert failed: {e}");
                let _ = reading::mark_failed(pool, resource_id, "url").await;
                return;
            }
            info!(
                "reader: fallback extracted {} words for resource {resource_id}",
                word_count
            );
        }
    }
}

/// Extract reader content from a local PDF file.
pub async fn extract_pdf(pool: &SqlitePool, resource_id: i64, file_path: &str) {
    let _ = reading::mark_pending(pool, resource_id, "pdf").await;

    let path = file_path.to_string();
    let text = match tokio::task::spawn_blocking(move || {
        let bytes = std::fs::read(&path)?;
        pdf_extract::extract_text_from_mem(&bytes)
            .map_err(|e| anyhow::anyhow!("pdf extract: {e}"))
    })
    .await
    {
        Ok(Ok(t)) => t,
        Ok(Err(e)) => {
            warn!("reader: pdf extract failed for {file_path}: {e}");
            let _ = reading::mark_failed(pool, resource_id, "pdf").await;
            return;
        }
        Err(e) => {
            warn!("reader: spawn_blocking failed: {e}");
            let _ = reading::mark_failed(pool, resource_id, "pdf").await;
            return;
        }
    };

    if text.trim().is_empty() {
        let _ = reading::mark_failed(pool, resource_id, "pdf").await;
        return;
    }

    let content_html = pdf_text_to_html(&text);
    let word_count = word_count(&text);

    if let Err(e) =
        reading::upsert_ok(pool, resource_id, &content_html, &text, "pdf", word_count).await
    {
        warn!("reader: pdf upsert failed: {e}");
        let _ = reading::mark_failed(pool, resource_id, "pdf").await;
        return;
    }
    info!(
        "reader: extracted {} words from pdf for resource {resource_id}",
        word_count
    );
}

async fn fetch_html(url: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::builder()
        .user_agent("rl-reader/0.1")
        .timeout(std::time::Duration::from_secs(20))
        .build()?;
    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("http status {}", resp.status());
    }
    Ok(resp.text().await?)
}

fn word_count(text: &str) -> i64 {
    text.split_whitespace().count() as i64
}

/// Minimal body-text stripper used only as a fallback when the readability
/// crate fails. Produces a flat string; `wrap_paragraphs` adds structure.
fn strip_body_text(html: &str) -> String {
    use scraper::{Html, Selector};
    let doc = Html::parse_document(html);
    let sel_body = Selector::parse("body").unwrap();
    let root = doc
        .select(&sel_body)
        .next()
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

/// Wrap a flat text blob as one big paragraph (fallback path).
fn wrap_paragraphs(text: &str) -> String {
    let escaped = html_escape(text);
    format!("<p>{}</p>", escaped)
}

/// Convert raw PDF text into lightly-structured HTML.
///
/// pdf-extract produces newline-separated lines with blank lines between
/// logical paragraphs and form-feed (`\x0c`) between pages. We:
///   - split on form-feed to detect pages (add `<hr class="page-break">`)
///   - split on blank lines within a page to form paragraphs
///   - heuristically detect headings (short lines <80 chars with no trailing
///     period, followed by a blank line)
fn pdf_text_to_html(text: &str) -> String {
    let mut out = String::new();
    let pages: Vec<&str> = text.split('\x0c').collect();
    let mut first_page = true;

    for page in pages {
        if !first_page {
            out.push_str("<hr class=\"page-break\">\n");
        }
        first_page = false;

        // Split into paragraph blocks on blank lines.
        let blocks: Vec<String> = page
            .split("\n\n")
            .map(|b| b.trim().to_string())
            .filter(|b| !b.is_empty())
            .collect();

        for block in blocks {
            // Collapse internal newlines to spaces within a block.
            let collapsed: String = block
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            if collapsed.is_empty() {
                continue;
            }
            if is_likely_heading(&collapsed) {
                out.push_str("<h2>");
                out.push_str(&html_escape(&collapsed));
                out.push_str("</h2>\n");
            } else {
                out.push_str("<p>");
                out.push_str(&html_escape(&collapsed));
                out.push_str("</p>\n");
            }
        }
    }
    out
}

fn is_likely_heading(line: &str) -> bool {
    let len = line.chars().count();
    if len == 0 || len > 80 {
        return false;
    }
    let trimmed = line.trim_end_matches(|c: char| c.is_whitespace());
    // Ends with sentence punctuation => not a heading
    if matches!(trimmed.chars().last(), Some('.') | Some('?') | Some('!')) {
        return false;
    }
    // All-caps or numbered (e.g., "1. Introduction", "Chapter 2")
    let letters: String = trimmed.chars().filter(|c| c.is_alphabetic()).collect();
    if !letters.is_empty() && letters == letters.to_uppercase() {
        return true;
    }
    // "1. Foo", "1.1 Foo", "Chapter 3", etc.
    if trimmed.starts_with(|c: char| c.is_ascii_digit())
        || trimmed.to_lowercase().starts_with("chapter ")
        || trimmed.to_lowercase().starts_with("section ")
    {
        return true;
    }
    false
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
