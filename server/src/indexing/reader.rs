//! Content extraction for the reader frontend.
//!
//! Unlike `indexing/url.rs` and `indexing/pdf.rs` which produce flat text
//! chunks for FTS search, this module produces structured HTML (and a plain
//! text copy) suitable for a distraction-free reading view.

use crate::models::reading;
use sqlx::SqlitePool;
use tracing::{info, warn};
use url::Url;

use super::pymupdf;

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
    let content = match tokio::task::spawn_blocking(move || pymupdf::extract_content(&path)).await {
        Ok(Ok(c)) => c,
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

    let (content_html, content_text) = pdf_blocks_to_html(&content);

    if content_text.trim().is_empty() {
        let _ = reading::mark_failed(pool, resource_id, "pdf").await;
        return;
    }

    let wc = word_count(&content_text);

    if let Err(e) = reading::upsert_ok(pool, resource_id, &content_html, &content_text, "pdf", wc).await {
        warn!("reader: pdf upsert failed: {e}");
        let _ = reading::mark_failed(pool, resource_id, "pdf").await;
        return;
    }
    info!("reader: extracted {wc} words from pdf for resource {resource_id}");
}

/// Convert PyMuPDF blocks into `(content_html, content_text)`.
fn pdf_blocks_to_html(content: &pymupdf::PdfContent) -> (String, String) {
    let mut html = String::new();
    let mut text = String::new();
    let mut first_page = true;

    for page in &content.pages {
        if !first_page {
            html.push_str("<hr class=\"page-break\">\n");
        }
        first_page = false;

        for block in &page.blocks {
            match block {
                pymupdf::PdfBlock::Heading { level, text: t } => {
                    let level = level.clamp(&2, &6);
                    html.push_str(&format!("<h{level}>{}</h{level}>\n", html_escape(t)));
                    text.push_str(t);
                    text.push('\n');
                }
                pymupdf::PdfBlock::Paragraph { text: t } => {
                    html.push_str(&format!("<p>{}</p>\n", html_escape(t)));
                    text.push_str(t);
                    text.push('\n');
                }
            }
        }
    }

    (html, text)
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


fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Split extractor-produced HTML into its top-level block elements.
///
/// The producers in this module emit a flat sequence of `<p>`, `<hN>`, and
/// self-closing `<hr ...>` elements separated by newlines. `readability` for
/// URLs is similar. Returned slices borrow from the input; interior text
/// nodes between blocks are preserved as their own slices.
pub fn split_top_level_blocks(html: &str) -> Vec<&str> {
    let bytes = html.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        if bytes[i] != b'<' {
            let start = i;
            while i < bytes.len() && bytes[i] != b'<' {
                i += 1;
            }
            let s = html[start..i].trim();
            if !s.is_empty() {
                out.push(s);
            }
            continue;
        }
        let tag_start = i + 1;
        let mut j = tag_start;
        while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'-') {
            j += 1;
        }
        let tag = &html[tag_start..j];
        if tag.is_empty() {
            // stray '<' — skip one byte to avoid infinite loop
            i += 1;
            continue;
        }
        // Void elements we care about (only `hr` is produced here).
        if tag.eq_ignore_ascii_case("hr") || tag.eq_ignore_ascii_case("br") {
            let end = html[i..]
                .find('>')
                .map(|k| i + k + 1)
                .unwrap_or(bytes.len());
            out.push(html[i..end].trim());
            i = end;
            continue;
        }
        let close = format!("</{}>", tag);
        let end = html[i..]
            .find(close.as_str())
            .map(|k| i + k + close.len())
            .or_else(|| {
                // case-insensitive fallback for uppercase tags
                html[i..]
                    .to_ascii_lowercase()
                    .find(&close.to_ascii_lowercase())
                    .map(|k| i + k + close.len())
            })
            .unwrap_or(bytes.len());
        let slice = html[i..end].trim();
        if !slice.is_empty() {
            out.push(slice);
        }
        i = end;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::split_top_level_blocks;

    #[test]
    fn splits_paragraphs() {
        let html = "<p>one</p>\n<p>two</p>\n<p>three</p>\n";
        let blocks = split_top_level_blocks(html);
        assert_eq!(blocks, vec!["<p>one</p>", "<p>two</p>", "<p>three</p>"]);
    }

    #[test]
    fn splits_heading_hr_paragraph() {
        let html = "<h2>Title</h2>\n<hr class=\"page-break\">\n<p>body</p>\n";
        let blocks = split_top_level_blocks(html);
        assert_eq!(
            blocks,
            vec!["<h2>Title</h2>", "<hr class=\"page-break\">", "<p>body</p>"]
        );
    }

    #[test]
    fn empty_input() {
        assert!(split_top_level_blocks("").is_empty());
        assert!(split_top_level_blocks("   \n  ").is_empty());
    }

    #[test]
    fn nested_wrapper_is_one_block() {
        let html = "<div><p>a</p><p>b</p></div>\n<p>c</p>";
        let blocks = split_top_level_blocks(html);
        assert_eq!(blocks.len(), 2);
        assert!(blocks[0].starts_with("<div>"));
        assert_eq!(blocks[1], "<p>c</p>");
    }
}
