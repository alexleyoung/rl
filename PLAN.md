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
    content_html TEXT NOT NULL DEFAULT '',  -- cleaned, structured HTML
    content_text TEXT NOT NULL DEFAULT '',  -- plain text (for word count, future FTS)
    source_type  TEXT NOT NULL CHECK (source_type IN ('url','pdf')),
    word_count   INTEGER NOT NULL DEFAULT 0,
    status       TEXT NOT NULL DEFAULT 'pending'
                 CHECK (status IN ('pending','ok','failed')),
    extracted_at INTEGER NOT NULL DEFAULT (unixepoch())
);
```

Key decisions:
- **One row per resource** (UNIQUE on resource_id) — a resource has at most one
  readable document
- `content_html` stores the cleaned article/document HTML (not raw page HTML)
- `content_text` stores the plain text version — used for word count and could
  feed improved FTS indexing later
- `source_type` tracks whether content came from a URL or PDF
- `word_count` enables estimated reading time display (~230 wpm)
- **`status` column** (`pending` → `ok` | `failed`) — since extraction is async
  and fire-and-forget, the reader handler needs to know whether extraction is
  in progress, succeeded, or failed. This enables proper UX: show a loading
  page while pending, the reader when ok, or an error page with retry when failed
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
    pub content_text: String,
    pub source_type: String,
    pub word_count: i64,
    pub status: String,        // "pending", "ok", "failed"
    pub extracted_at: i64,
}

pub async fn get_for_resource(pool: &SqlitePool, resource_id: i64) -> Result<Option<ReadingContent>>
pub async fn upsert(pool: &SqlitePool, resource_id: i64, html: &str, text: &str, source_type: &str, word_count: i64) -> Result<()>
pub async fn mark_pending(pool: &SqlitePool, resource_id: i64, source_type: &str) -> Result<()>
pub async fn mark_failed(pool: &SqlitePool, resource_id: i64) -> Result<()>
pub async fn delete_for_resource(pool: &SqlitePool, resource_id: i64) -> Result<()>
```

- `upsert` sets status to `'ok'` on successful extraction
- `mark_pending` creates/updates to `'pending'` when extraction starts
- `mark_failed` sets status to `'failed'` on extraction error
- `delete_for_resource` used when URL or file_path changes (invalidation)

Wire into `src/models/mod.rs`.

---

## Step 6: Handler — Reader View

**File**: `src/handlers/reader.rs` (new file, separate from resources handler)

Reader-specific handlers in their own module for cleanliness:

```rust
pub async fn read_view(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let r = resource::get(&pool, id).await?.ok_or(AppError::NotFound)?;

    // Guard: only resources with a URL (article/blog/paper) or file_path can be read
    let can_read = (r.url.is_some() && matches!(r.kind.as_str(), "article" | "blog" | "paper"))
                   || r.file_path.is_some();
    if !can_read { return Err(AppError::NotFound); }

    let content = reading::get_for_resource(&pool, id).await?;

    match content {
        Some(c) if c.status == "ok" => Ok(view::reader_page(&r, &c)),
        Some(c) if c.status == "pending" => Ok(view::reader_pending_page(&r)),
        Some(c) if c.status == "failed" => Ok(view::reader_failed_page(&r)),
        None => {
            // No extraction attempted yet — kick one off and show pending
            trigger_extraction(&pool, &r).await;
            Ok(view::reader_pending_page(&r))
        }
        _ => Ok(view::reader_failed_page(&r)),
    }
}
```

This status-driven approach means clicking "read" always does something useful:
- First visit → triggers extraction, shows auto-refreshing "extracting..." page
- Extraction done → shows the reader
- Extraction failed → shows error with retry button

The `reader_pending_page` auto-refreshes via
`<script>setTimeout(function(){ location.reload(); }, 3000);</script>` so the
user just sees the reader appear once extraction completes.

Wire into `src/handlers/mod.rs`.

---

## Step 7: Route Registration

**File**: `src/main.rs`

Add two routes:

```rust
.route("/resources/:id/read", get(handlers::reader::read_view))
.route("/resources/:id/read/refresh", post(handlers::reader::refresh_content))
```

---

## Step 8: Reader View Template

**File**: `src/views/reader.rs` (new)

Three template functions for the reader's three states:

### `reader_page` — the main reader view (status = "ok")

Uses the standard `page()` layout but with reader-specific content. The
article body renders inside a `.reader-content` container styled by `reader.css`.

```rust
pub fn reader_page(resource: &Resource, content: &ReadingContent) -> Markup {
    // Uses page() layout but renders article body + reader chrome
    // Header: back link to resource, title, author, word count, reading time, refresh button
    // Progress bar: fixed 2px bar at top, filled by reader.js as user scrolls
    // Article body: content.content_html rendered via PreEscaped inside article.reader-content
    // Help panel: hidden by default, toggled with ? key
    // Script: reader.js for keyboard nav + progress
}
```

### `reader_pending_page` — extraction in progress (status = "pending")

```rust
pub fn reader_pending_page(resource: &Resource) -> Markup {
    // Shows "extracting content..." flash message
    // Auto-refreshes after 3 seconds via inline JS setTimeout
    // Back link to resource detail
}
```

### `reader_failed_page` — extraction failed (status = "failed")

```rust
pub fn reader_failed_page(resource: &Resource) -> Markup {
    // Shows error flash explaining possible causes:
    //   - URL: page may require JS, block bots, or have unusual structure
    //   - PDF: may be image-based (scanned) or corrupted
    // Retry button (POST to /resources/:id/read/refresh)
    // "Open original" link if URL exists
    // Back link to resource detail
}
```

### `reading_time` helper

```rust
fn reading_time(words: i64) -> String {
    let mins = (words as f64 / 230.0).ceil() as i64;  // ~230 WPM average
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

Modify `detail_page()` to show a "read" button when the resource has a readable
source. Since the reader handler does on-demand extraction (Step 6), the button
should always appear for resources that *could* be read — the reader page
itself handles the pending/failed states:

```rust
// In the row-actions div, add before "edit":
@let has_readable_source = (r.url.is_some() && matches!(r.kind.as_str(), "article" | "blog" | "paper"))
                           || r.file_path.is_some();
@if has_readable_source {
    a.btn.primary href=(format!("/resources/{}/read", r.id)) { "read" }
}
```

No extra DB query needed in the `show` handler — the button visibility is based
purely on the resource's existing fields.

---

## Step 12: Refresh + Invalidation

### Refresh endpoint

**File**: `src/handlers/reader.rs`

`POST /resources/:id/read/refresh` — force re-extraction (used from both the
reader failed page and the reader header's refresh button):

```rust
pub async fn refresh_content(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let r = resource::get(&pool, id).await?.ok_or(AppError::NotFound)?;
    reading::delete_for_resource(&pool, id).await?;
    trigger_extraction(&pool, &r).await;
    Ok(Redirect::to(&format!("/resources/{id}/read")))
}
```

### Content invalidation on source change

**File**: `src/handlers/resources.rs`

When a resource's URL or file_path changes via `update()` or `quick_set()`,
delete the existing readable content so it gets re-extracted on next "read":

```rust
// In update() and quick_set(), after updating URL or file_path:
reading::delete_for_resource(&pool, id).await?;
```

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

- **First click with no content yet**: Reader handler triggers extraction
  on-demand and shows auto-refreshing "extracting..." page (3s refresh)
- **Extraction failure**: Status set to `'failed'`, reader shows error page
  with retry button and "open original" link. Logged via tracing.
- **Concurrent extraction requests**: `mark_pending` is idempotent; if a user
  refreshes the pending page, no duplicate extraction is spawned (check status
  before spawning)
- **Very large documents**: `content_html` could be large. Fine for SQLite
  (up to 1GB per field). Sanity-check: truncate if > 2MB as a safety net.
- **Non-article resources**: Books/papers with file_path get PDF extraction.
  Repos get no reader content (no "read" button). Articles/blogs get URL
  extraction. Papers with URL also get URL extraction.
- **URL/file_path changes**: Existing readable content is invalidated
  (deleted) so the next "read" click triggers fresh extraction
- **Resources without URL or file_path**: No "read" button shown. Handler
  returns 404 if navigated to directly.
- **Readability extraction failure fallback**: If the `readability` crate
  fails on a URL, fall back to the existing scraper-based approach but wrap
  text in `<p>` tags for minimal structure.

---

## New Files Summary

| File | Purpose |
|------|---------|
| `migrations/0002_reading_content.sql` | Schema for stored readable content |
| `src/models/reading.rs` | DB queries for reading_content table |
| `src/indexing/reader.rs` | Content extraction (URL via readability, PDF via pdf-extract + formatting) |
| `src/handlers/reader.rs` | Read view + refresh handlers |
| `src/views/reader.rs` | Reader view Maud templates (ok, pending, failed states) |
| `static/reader.css` | Reader-specific styles |
| `static/reader.js` | Keyboard nav + progress bar |

## Modified Files Summary

| File | Changes |
|------|---------|
| `Cargo.toml` | Add `readability` dependency |
| `src/main.rs` | Add `/resources/:id/read` and `/resources/:id/read/refresh` routes |
| `src/models/mod.rs` | Add `pub mod reading;` |
| `src/indexing/mod.rs` | Add `pub mod reader;` |
| `src/handlers/mod.rs` | Add `pub mod reader;` |
| `src/views/mod.rs` | Add `pub mod reader;` |
| `src/handlers/resources.rs` | Wire extraction into create/update/quick_set; add invalidation on URL/file_path change |
| `src/views/resources.rs` | Add "read" button to detail page |
| `static/app.css` | No changes needed (reader.css is separate) |
| `.sqlx/` | Regenerate offline query cache |
