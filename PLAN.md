# Plan: Reading Frontend for Web & PDF Content

## Overview

Add a "reader mode" to rl that extracts structured, readable content from web
URLs and PDF files, stores it in SQLite, and renders it in a clean,
distraction-free reading view at `GET /resources/:id/read`.

Currently, `indexing/url.rs` and `indexing/pdf.rs` extract **plain text chunks**
solely for FTS5 search. This feature adds a parallel pipeline that preserves
document structure (headings, paragraphs, lists, code blocks, images) as clean
HTML, stores it once, and serves it through a dedicated reader view.

---

## Step 1: Database Migration

**File**: `migrations/0002_reading_content.sql`

Add a `reading_content` table to store extracted readable HTML separately from
search chunks:

```sql
CREATE TABLE IF NOT EXISTS reading_content (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    resource_id  INTEGER NOT NULL UNIQUE REFERENCES resources(id) ON DELETE CASCADE,
    content_html TEXT NOT NULL,            -- cleaned, structured HTML
    source_type  TEXT NOT NULL CHECK (source_type IN ('url','pdf')),
    word_count   INTEGER NOT NULL DEFAULT 0,
    extracted_at INTEGER NOT NULL DEFAULT (unixepoch())
);
```

Key decisions:
- **One row per resource** (UNIQUE on resource_id) — a resource has at most one
  readable document
- `content_html` stores the cleaned article/document HTML (not raw page HTML)
- `source_type` tracks whether content came from a URL or PDF
- `word_count` enables estimated reading time display (word_count / 200 wpm)
- Separate table (not columns on `resources`) keeps the main table lean and
  makes it easy to re-extract without touching resource metadata

---

## Step 2: New Dependency — `readability`

**File**: `Cargo.toml`

Add the `readability` crate (Rust port of Mozilla's Readability algorithm used
in Firefox Reader Mode):

```toml
readability = "0.3"
```

This replaces the naive scraper-based text extraction for reading purposes.
Benefits over the current `html_to_text()` approach:
- Preserves semantic HTML structure (h1-h6, p, ul/ol/li, blockquote, pre/code, img, a)
- Intelligent content scoring to find the main article body
- Strips navigation, ads, sidebars, footers automatically
- Extracts metadata (title, byline) as bonus

The existing `scraper`-based FTS indexing remains unchanged — it serves a
different purpose (flat text for search ranking).

---

## Step 3: Content Extraction Module

**File**: `src/indexing/reader.rs` (new)

Two public functions that extract structured HTML and store it:

### `extract_url_readable(pool, resource_id, url)`

1. Fetch HTML with `reqwest` (reuse existing client pattern from `url.rs`)
2. Run `readability::extractor::extract()` on the HTML
3. Sanitize the output HTML (strip any remaining scripts/styles, limit tag set)
4. Compute word count from text content
5. `INSERT OR REPLACE INTO reading_content` (upsert on resource_id)

### `extract_pdf_readable(pool, resource_id, file_path)`

1. Read PDF bytes, extract text with `pdf-extract` (reuse from `pdf.rs`)
2. Convert plain text to structured HTML:
   - Detect paragraph breaks (double newlines → `<p>` tags)
   - Detect headings heuristically (short ALL-CAPS lines, or lines followed by
     underlines → `<h2>` tags)
   - Preserve line breaks within paragraphs where meaningful
3. Compute word count
4. `INSERT OR REPLACE INTO reading_content`

### Wire into `src/indexing/mod.rs`

Add `pub mod reader;` to the module declarations.

---

## Step 4: Trigger Extraction on Resource Create/Edit

**File**: `src/handlers/resources.rs`

Modify existing `create()`, `update()`, and `quick_set()` handlers to also
spawn reader extraction alongside the existing FTS indexing:

```rust
// After existing FTS indexing spawn...
// Also extract readable content for the reader view
if let Some(u) = &input.url {
    let u = u.clone();
    let pool2 = pool.clone();
    tokio::spawn(async move {
        reader::extract_url_readable(&pool2, id, &u).await;
    });
}
if let Some(fp) = &input.file_path {
    let fp = fp.clone();
    let pool2 = pool.clone();
    tokio::spawn(async move {
        reader::extract_pdf_readable(&pool2, id, &fp).await;
    });
}
```

Extraction is fire-and-forget (same pattern as existing indexing). If it fails,
the "read" button simply won't appear until content is available.

---

## Step 5: Model Layer

**File**: `src/models/reading.rs` (new)

```rust
pub struct ReadingContent {
    pub id: i64,
    pub resource_id: i64,
    pub content_html: String,
    pub source_type: String,
    pub word_count: i64,
    pub extracted_at: i64,
}

pub async fn get_for_resource(pool: &SqlitePool, resource_id: i64) -> Result<Option<ReadingContent>>
pub async fn upsert(pool: &SqlitePool, resource_id: i64, content_html: &str, source_type: &str, word_count: i64) -> Result<()>
pub async fn has_content(pool: &SqlitePool, resource_id: i64) -> Result<bool>
```

Wire into `src/models/mod.rs`.

---

## Step 6: Handler — Reader View

**File**: `src/handlers/resources.rs` (add to existing)

New handler function:

```rust
pub async fn read_view(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let r = resource::get(&pool, id).await?.ok_or(AppError::NotFound)?;
    let content = reading::get_for_resource(&pool, id).await?
        .ok_or(AppError::NotFound)?;
    Ok(view::reader_page(&r, &content))
}
```

---

## Step 7: Route Registration

**File**: `src/main.rs`

Add the route:

```rust
.route("/resources/:id/read", get(handlers::resources::read_view))
```

---

## Step 8: Reader View Template

**File**: `src/views/reader.rs` (new)

A dedicated, distraction-free reading layout. Unlike the standard `page()`
layout, the reader uses a more focused design:

```rust
pub fn reader_page(resource: &Resource, content: &ReadingContent) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (resource.title) " — reader — rl" }
                link rel="stylesheet" href="/static/app.css";
                link rel="stylesheet" href="/static/reader.css";
            }
            body.reader-body {
                // Minimal top bar: back link, title, reading time
                header.reader-header {
                    a.reader-back href=(format!("/resources/{}", resource.id)) { "< back" }
                    span.reader-title { (resource.title) }
                    span.reader-meta {
                        (reading_time(content.word_count))
                    }
                }

                // Progress bar at top of viewport
                div.reader-progress {
                    div.reader-progress-bar #progress-bar {}
                }

                // Article content
                article.reader-content {
                    @if let Some(author) = &resource.author {
                        p.reader-byline { (author) }
                    }
                    (PreEscaped(&content.content_html))
                }

                // Keyboard shortcut help (toggled with ?)
                div.reader-help #reader-help style="display:none" {
                    h3 { "keyboard shortcuts" }
                    p { code { "j/k" } " — scroll down/up" }
                    p { code { "g g" } " — go to top" }
                    p { code { "G" } " — go to bottom" }
                    p { code { "q" } " — back to resource" }
                    p { code { "?" } " — toggle this help" }
                }

                script src="/static/reader.js" {}
            }
        }
    }
}

fn reading_time(words: i64) -> String {
    let mins = (words as f64 / 200.0).ceil() as i64;
    if mins <= 1 { "1 min read".into() }
    else { format!("{mins} min read") }
}
```

Wire into `src/views/mod.rs`.

---

## Step 9: Reader CSS

**File**: `static/reader.css` (new)

Dedicated stylesheet for the reading view. Key design choices:
- **Proportional font** for body text (unlike the rest of the app which uses
  monospace) — reading long-form content in monospace is fatiguing
- **Constrained line length** (~65ch) for comfortable reading
- **Generous line height** (1.8) and paragraph spacing
- Monospace preserved only for `code`/`pre` blocks

```css
/* Reader-specific layout — overlays/extends app.css variables */
.reader-body {
  background: var(--bg);
  color: var(--fg);
}

.reader-header {
  position: sticky;
  top: 0;
  display: flex;
  align-items: baseline;
  gap: 1.5rem;
  padding: 0.5rem 1rem;
  background: var(--bg);
  border-bottom: 1px solid var(--border);
  font-family: var(--mono);
  font-size: 0.8rem;
  z-index: 10;
}
.reader-back { color: var(--dim); }
.reader-title { flex: 1; font-weight: bold; color: var(--fg); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.reader-meta { color: var(--dim); white-space: nowrap; }

/* Progress indicator */
.reader-progress {
  position: sticky;
  top: 2rem; /* below header */
  height: 2px;
  background: var(--border);
  z-index: 10;
}
.reader-progress-bar {
  height: 100%;
  width: 0%;
  background: var(--accent);
  transition: width 100ms ease;
}

/* Article content */
.reader-content {
  max-width: 65ch;
  margin: 2rem auto;
  padding: 0 1rem;
  font-family: Georgia, "Times New Roman", serif;
  font-size: 1.1rem;
  line-height: 1.8;
}

.reader-content h1, .reader-content h2, .reader-content h3,
.reader-content h4, .reader-content h5, .reader-content h6 {
  font-family: var(--mono);
  margin-top: 2rem;
  margin-bottom: 0.5rem;
  line-height: 1.3;
}
.reader-content h1 { font-size: 1.5rem; }
.reader-content h2 { font-size: 1.25rem; }
.reader-content h3 { font-size: 1.1rem; }

.reader-content p { margin-bottom: 1.2rem; }
.reader-content ul, .reader-content ol { margin: 0.75rem 0 1.2rem 1.5rem; }
.reader-content li { margin-bottom: 0.3rem; }

.reader-content blockquote {
  border-left: 3px solid var(--border);
  padding-left: 1rem;
  color: var(--dim);
  font-style: italic;
  margin: 1rem 0;
}

.reader-content code {
  font-family: var(--mono);
  font-size: 0.9em;
  background: #e8e8e8;
  padding: 0.1rem 0.3rem;
  border-radius: 2px;
}
.reader-content pre {
  background: #e8e8e8;
  padding: 1rem;
  overflow-x: auto;
  margin: 1rem 0;
  line-height: 1.5;
}
.reader-content pre code { background: none; padding: 0; }

.reader-content img {
  max-width: 100%;
  height: auto;
  margin: 1rem 0;
}

.reader-content a { color: var(--accent); text-decoration: underline; }

.reader-byline {
  color: var(--dim);
  font-style: italic;
  margin-bottom: 2rem;
  font-size: 0.95rem;
}

/* Help overlay */
.reader-help {
  position: fixed;
  bottom: 1rem;
  right: 1rem;
  background: var(--bg);
  border: 1px solid var(--border);
  padding: 1rem;
  font-family: var(--mono);
  font-size: 0.8rem;
  z-index: 20;
  max-width: 280px;
}
.reader-help h3 { margin-bottom: 0.5rem; }
.reader-help code { background: #e8e8e8; padding: 0 0.25rem; }
```

---

## Step 10: Reader JavaScript (Keyboard Navigation + Progress)

**File**: `static/reader.js` (new)

Minimal vanilla JS for vim-style keyboard navigation and scroll progress:

```javascript
(function() {
  var SCROLL_STEP = 100;
  var lastKey = '';

  // Scroll progress bar
  function updateProgress() {
    var scrollTop = window.scrollY;
    var docHeight = document.documentElement.scrollHeight - window.innerHeight;
    var pct = docHeight > 0 ? (scrollTop / docHeight) * 100 : 0;
    var bar = document.getElementById('progress-bar');
    if (bar) bar.style.width = pct + '%';
  }
  window.addEventListener('scroll', updateProgress);
  updateProgress();

  // Keyboard shortcuts
  document.addEventListener('keydown', function(e) {
    // Don't capture when typing in inputs
    if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;

    var key = e.key;

    if (key === 'j') {
      window.scrollBy(0, SCROLL_STEP);
    } else if (key === 'k') {
      window.scrollBy(0, -SCROLL_STEP);
    } else if (key === 'd' && e.ctrlKey) {
      e.preventDefault();
      window.scrollBy(0, window.innerHeight / 2);
    } else if (key === 'u' && e.ctrlKey) {
      e.preventDefault();
      window.scrollBy(0, -window.innerHeight / 2);
    } else if (key === 'G' && !e.ctrlKey) {
      window.scrollTo(0, document.documentElement.scrollHeight);
    } else if (key === 'g') {
      if (lastKey === 'g') {
        window.scrollTo(0, 0);
        lastKey = '';
        return;
      }
      lastKey = 'g';
      setTimeout(function() { lastKey = ''; }, 500);
      return;
    } else if (key === 'q') {
      // Navigate back to resource detail
      var backLink = document.querySelector('.reader-back');
      if (backLink) window.location.href = backLink.href;
    } else if (key === '?') {
      var help = document.getElementById('reader-help');
      if (help) help.style.display = help.style.display === 'none' ? 'block' : 'none';
    }

    if (key !== 'g') lastKey = '';
  });
})();
```

---

## Step 11: "Read" Button on Resource Detail Page

**File**: `src/views/resources.rs`

Modify `detail_page()` to show a "read" button when reading content exists.
The handler will pass a boolean `has_reading_content` flag:

```rust
// In the row-actions div, add before "edit":
@if has_reading_content {
    a.btn href=(format!("/resources/{}/read", r.id)) { "read" }
}
```

Update the `show` handler in `handlers/resources.rs` to query
`reading::has_content()` and pass it to the view.

---

## Step 12: Manual Re-Extract Action

**File**: `src/handlers/resources.rs`

Add a `POST /resources/:id/extract` endpoint that re-triggers content
extraction on demand (useful if the first extraction failed or content changed):

```rust
pub async fn re_extract(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let r = resource::get(&pool, id).await?.ok_or(AppError::NotFound)?;
    // Spawn extraction
    if let Some(u) = &r.url {
        let u = u.clone(); let pool2 = pool.clone();
        tokio::spawn(async move { reader::extract_url_readable(&pool2, id, &u).await; });
    }
    if let Some(fp) = &r.file_path {
        let fp = fp.clone(); let pool2 = pool.clone();
        tokio::spawn(async move { reader::extract_pdf_readable(&pool2, id, &fp).await; });
    }
    Ok(Redirect::to(&format!("/resources/{id}")))
}
```

Add a small "extract" or "refresh" button on the resource detail page that
posts to this endpoint.

---

## Implementation Order

| # | Task | Files | Dependencies |
|---|------|-------|-------------|
| 1 | Add migration | `migrations/0002_reading_content.sql` | None |
| 2 | Add `readability` to Cargo.toml | `Cargo.toml` | None |
| 3 | Add reading model | `src/models/reading.rs`, `src/models/mod.rs` | Step 1 |
| 4 | Add reader extraction module | `src/indexing/reader.rs`, `src/indexing/mod.rs` | Steps 2, 3 |
| 5 | Wire extraction into handlers | `src/handlers/resources.rs` | Step 4 |
| 6 | Add reader view template | `src/views/reader.rs`, `src/views/mod.rs` | Step 3 |
| 7 | Add reader CSS + JS | `static/reader.css`, `static/reader.js` | None |
| 8 | Add reader route + handler | `src/main.rs`, `src/handlers/resources.rs` | Steps 5, 6 |
| 9 | Add "read" button to detail page | `src/views/resources.rs`, `src/handlers/resources.rs` | Step 8 |
| 10 | Add re-extract endpoint | `src/handlers/resources.rs`, `src/main.rs`, `src/views/resources.rs` | Step 8 |
| 11 | Update SQLx offline cache | `.sqlx/` | All query changes |
| 12 | Test end-to-end | — | All steps |

Steps 1, 2, and 7 can be done in parallel. Steps 3-4 are sequential.
Steps 6 and 7 can be done in parallel.

---

## Edge Cases & Error Handling

- **No readable content yet**: Don't show "read" button. Show a flash/hint
  that content is being extracted if resource was just created.
- **Extraction failure**: Log via tracing (existing pattern). "read" button
  stays hidden. User can retry via "re-extract" button.
- **Very large documents**: `content_html` could be large. This is fine for
  SQLite (up to 1GB per field). Consider lazy-loading sections later if needed.
- **Non-article resources**: Books/papers with file_path get PDF extraction.
  Repos get no reader content (no "read" button). Articles/blogs get URL
  extraction.
- **Content already exists**: Upsert (INSERT OR REPLACE) on re-extraction.
- **Resources without URL or file_path**: No extraction triggered, no "read"
  button.

---

## New Files Summary

| File | Purpose |
|------|---------|
| `migrations/0002_reading_content.sql` | Schema for stored readable content |
| `src/models/reading.rs` | DB queries for reading_content table |
| `src/indexing/reader.rs` | Content extraction (URL via readability, PDF via pdf-extract + formatting) |
| `src/views/reader.rs` | Reader view Maud template |
| `static/reader.css` | Reader-specific styles |
| `static/reader.js` | Keyboard nav + progress bar |

## Modified Files Summary

| File | Changes |
|------|---------|
| `Cargo.toml` | Add `readability` dependency |
| `src/main.rs` | Add `/resources/:id/read` and `/resources/:id/extract` routes |
| `src/models/mod.rs` | Add `pub mod reading;` |
| `src/indexing/mod.rs` | Add `pub mod reader;` |
| `src/views/mod.rs` | Add `pub mod reader;` |
| `src/handlers/resources.rs` | Add `read_view`, `re_extract` handlers; wire extraction into create/update/quick_set |
| `src/views/resources.rs` | Add "read" and "re-extract" buttons to detail page |
| `.sqlx/` | Regenerate offline query cache |
