use scraper::{Html, Selector};

use crate::api::dto::MetadataDto;
use crate::indexing::pymupdf;

// ─── PDF ─────────────────────────────────────────────────────────────────────

pub fn extract_pdf_meta(file_path: &str) -> MetadataDto {
    match pymupdf::extract_meta(file_path) {
        Ok(m) => MetadataDto { title: m.title, author: m.author, description: m.description },
        Err(_) => MetadataDto::default(),
    }
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
