use crate::models::reading::ReadingContent;
use crate::models::resource::Resource;
use crate::views::layout::page;
use maud::{html, Markup, PreEscaped};

/// Estimated reading time in minutes using ~230 wpm.
fn reading_time(words: i64) -> String {
    let mins = (words.max(1) as f64 / 230.0).ceil() as i64;
    if mins <= 1 {
        "1 min read".to_string()
    } else {
        format!("{mins} min read")
    }
}

/// Reader view — status = "ok".
pub fn reader_page(r: &Resource, content: &ReadingContent) -> Markup {
    let back_url = format!("/resources/{}", r.id);
    let refresh_url = format!("/resources/{}/read/refresh", r.id);

    page(&format!("read — {}", r.title), html! {
        // Progress bar — fixed at top of viewport, filled as user scrolls.
        div #reader-progress {
            div #reader-progress-bar {}
        }

        div.reader-header {
            div.row-actions.mb {
                a.reader-back.dim.small href=(back_url) { "← " (r.title) }
            }
            h1.reader-title { (r.title) }
            div.reader-meta.dim.small {
                @if let Some(a) = &r.author {
                    span { (a) }
                    span { " · " }
                }
                span { (content.word_count) " words" }
                span { " · " }
                span { (reading_time(content.word_count)) }
                span { " · " }
                form method="post" action=(refresh_url) style="display:inline" {
                    button.small type="submit" title="re-extract content" {
                        "refresh"
                    }
                }
            }
        }

        article #reader-content.reader-body {
            (PreEscaped(&content.content_html))
        }

        // Keyboard shortcut help panel — toggled with ?.
        div #reader-help.reader-help style="display:none" {
            h3 { "keyboard shortcuts" }
            table {
                tbody {
                    tr { td { code { "j / Space" } } td { "scroll down" } }
                    tr { td { code { "k / Shift+Space" } } td { "scroll up" } }
                    tr { td { code { "g g" } } td { "go to top" } }
                    tr { td { code { "G" } } td { "go to bottom" } }
                    tr { td { code { "q / Esc" } } td { "back to resource" } }
                    tr { td { code { "?" } } td { "toggle this help" } }
                }
            }
        }

        script src="/static/reader.js" {}
    })
}

/// Reader view — status = "pending".
/// Auto-refreshes the page every few seconds so the reader appears once
/// extraction finishes.
pub fn reader_pending_page(r: &Resource) -> Markup {
    let back_url = format!("/resources/{}", r.id);
    let read_url = format!("/resources/{}/read", r.id);

    page(&format!("read — {}", r.title), html! {
        div.row-actions.mb {
            a.dim.small href=(back_url) { "← " (r.title) }
        }
        h1 { (r.title) }
        div.flash {
            "extracting content… this page will refresh automatically."
        }
        p.dim.small.mt {
            "if it doesn't refresh, "
            a href=(read_url) { "reload manually" }
            "."
        }
        script {
            (PreEscaped(r#"
setTimeout(function() { location.reload(); }, 3000);
"#))
        }
    })
}

/// Reader view — status = "failed".
pub fn reader_failed_page(r: &Resource) -> Markup {
    let back_url = format!("/resources/{}", r.id);
    let refresh_url = format!("/resources/{}/read/refresh", r.id);

    page(&format!("read — {}", r.title), html! {
        div.row-actions.mb {
            a.dim.small href=(back_url) { "← " (r.title) }
        }
        h1 { (r.title) }
        div.flash.err { "content extraction failed." }
        p.dim {
            @if r.url.is_some() {
                "the page may require javascript, block automated access, or have an unusual structure. "
            }
            @if r.file_path.is_some() {
                "the PDF may be image-based (scanned) or malformed. "
            }
        }
        div.row-actions.mt {
            form method="post" action=(refresh_url) style="display:inline" {
                button.primary type="submit" { "retry extraction" }
            }
            @if let Some(u) = &r.url {
                a.btn href=(u) target="_blank" rel="noopener" { "open original" }
            }
            a.btn href=(back_url) { "back" }
        }
    })
}
