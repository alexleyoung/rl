use scraper::{Html, Selector};

use crate::api::dto::MetadataDto;

// ─── PDF ─────────────────────────────────────────────────────────────────────

pub fn extract_pdf_meta(bytes: &[u8]) -> MetadataDto {
    let doc = match lopdf::Document::load_mem(bytes) {
        Ok(d) => d,
        Err(_) => return MetadataDto::default(),
    };

    let info = match doc.trailer.get(b"Info") {
        Ok(obj) => match doc.dereference(obj) {
            Ok((_, lopdf::Object::Dictionary(d))) => d,
            _ => return MetadataDto::default(),
        },
        Err(_) => return MetadataDto::default(),
    };

    let title = pdf_string(info.get(b"Title").ok());
    let author = pdf_string(info.get(b"Author").ok());
    let description = pdf_string(info.get(b"Subject").ok());

    MetadataDto { title, author, description }
}

/// Decode a lopdf string object: UTF-16BE if BOM present, else UTF-8/Latin-1.
fn pdf_string(obj: Option<&lopdf::Object>) -> Option<String> {
    let bytes = match obj? {
        lopdf::Object::String(b, _) => b,
        _ => return None,
    };
    if bytes.is_empty() {
        return None;
    }
    let s = if bytes.starts_with(&[0xFE, 0xFF]) {
        // UTF-16BE with BOM
        let words: Vec<u16> = bytes[2..]
            .chunks(2)
            .map(|c| u16::from_be_bytes([c[0], c.get(1).copied().unwrap_or(0)]))
            .collect();
        String::from_utf16_lossy(&words).to_string()
    } else {
        // Try UTF-8, fall back to Latin-1
        match std::str::from_utf8(bytes) {
            Ok(s) => s.to_string(),
            Err(_) => bytes.iter().map(|&b| b as char).collect(),
        }
    };
    let s = s.trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

// ─── URL ─────────────────────────────────────────────────────────────────────

pub async fn extract_url_meta(url: &str) -> MetadataDto {
    let html = match fetch_html(url).await {
        Ok(h) => h,
        Err(_) => return MetadataDto::default(),
    };
    parse_html_meta(&html)
}

async fn fetch_html(url: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::builder()
        .user_agent("rl-indexer/0.1")
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    Ok(client.get(url).send().await?.text().await?)
}

fn parse_html_meta(html: &str) -> MetadataDto {
    let doc = Html::parse_document(html);

    let title = meta_attr(&doc, "meta[property='og:title']", "content")
        .or_else(|| meta_attr(&doc, "meta[name='twitter:title']", "content"))
        .or_else(|| select_text(&doc, "title"));

    let author = meta_attr(&doc, "meta[name='author']", "content")
        .or_else(|| meta_attr(&doc, "meta[property='article:author']", "content"));

    let description = meta_attr(&doc, "meta[property='og:description']", "content")
        .or_else(|| meta_attr(&doc, "meta[name='description']", "content"));

    MetadataDto { title, author, description }
}

fn meta_attr(doc: &Html, selector: &str, attr: &str) -> Option<String> {
    let sel = Selector::parse(selector).ok()?;
    let val = doc.select(&sel).next()?.value().attr(attr)?.trim().to_string();
    if val.is_empty() { None } else { Some(val) }
}

fn select_text(doc: &Html, selector: &str) -> Option<String> {
    let sel = Selector::parse(selector).ok()?;
    let text: String = doc.select(&sel).next()?.text().collect();
    let t = text.trim().to_string();
    if t.is_empty() { None } else { Some(t) }
}
