use crate::models::resource::Resource;
use crate::views::layout::page;
use maud::{html, Markup};

pub fn list_page(resources: &[Resource], all_tags: &[String], active_tag: Option<&str>) -> Markup {
    page("resources", html! {
        div #search-bar {
            form action="/search" method="get" {
                input type="text" name="q" placeholder="search…" autocomplete="off";
                button type="submit" { "go" }
            }
        }

        @if !all_tags.is_empty() {
            div.tags {
                a.tag[active_tag.is_none()].active[active_tag.is_none()]
                    href="/" { "all" }
                @for tag in all_tags {
                    @let is_active = active_tag == Some(tag.as_str());
                    a.tag[is_active].active[is_active]
                        href=(format!("/?tag={}", urlencoding::encode(tag))) {
                        (tag)
                    }
                }
            }
        }

        @if resources.is_empty() {
            p.dim { "no resources yet. " a href="/resources/new" { "add one" } }
        } @else {
            table {
                thead {
                    tr {
                        th { "title" }
                        th { "kind" }
                        th { "tags" }
                        th { "added" }
                    }
                }
                tbody {
                    @for r in resources {
                        tr {
                            td { a href=(format!("/resources/{}", r.id)) { (r.title) } }
                            td { span.kind { (r.kind) } }
                            td { } // filled by tag fetch per row — simplified: tags shown on detail
                            td.small.dim { (fmt_date(r.added_at)) }
                        }
                    }
                }
            }
        }

        div.mt {
            a.btn href="/resources/new" { "+ new resource" }
        }
    })
}

pub fn new_page() -> Markup {
    page("new resource", html! {
        h1 { "new resource" }
        form method="post" action="/resources" {
            label { "kind"
                select name="kind" required {
                    option value="book" { "book" }
                    option value="paper" { "paper" }
                    option value="article" { "article" }
                    option value="blog" { "blog" }
                    option value="repo" { "repo" }
                }
            }
            label { "title"
                input type="text" name="title" required autocomplete="off";
            }
            label { "author"
                input type="text" name="author" autocomplete="off";
            }
            label { "url"
                input type="url" name="url" autocomplete="off";
            }
            label { "local file path"
                input type="text" name="file_path" autocomplete="off";
            }
            label { "tags (comma-separated)"
                input type="text" name="tags" autocomplete="off";
            }
            div.row-actions {
                button.primary type="submit" { "save" }
                a.btn href="/" { "cancel" }
            }
        }
    })
}

pub fn detail_page(r: &Resource, tags: &[String], notes: &[crate::models::note::Note]) -> Markup {
    page(&r.title, html! {
        h1 { (r.title) }

        div.meta {
            div.meta-row { span.key.dim { "kind" } span.kind { (r.kind) } }
            @if let Some(a) = &r.author {
                div.meta-row { span.key.dim { "author" } span { (a) } }
            }
            @if let Some(u) = &r.url {
                div.meta-row { span.key.dim { "url" }
                    a href=(u) target="_blank" rel="noopener" { (u) }
                }
            }
            @if let Some(fp) = &r.file_path {
                div.meta-row { span.key.dim { "file" }
                    a href=(format!("/files/{}", r.id)) { (fp) }
                }
            }
            div.meta-row { span.key.dim { "added" } span.dim { (fmt_date(r.added_at)) } }
            @if !tags.is_empty() {
                div.meta-row { span.key.dim { "tags" }
                    div.tags style="margin:0" {
                        @for t in tags {
                            a.tag href=(format!("/?tag={}", urlencoding::encode(t))) { (t) }
                        }
                    }
                }
            }
        }

        div.row-actions.mb {
            a.btn href=(format!("/resources/{}/edit", r.id)) { "edit" }
            form method="post" action=(format!("/resources/{}/delete", r.id)) style="display:inline" {
                button.danger type="submit"
                    onclick="return confirm('delete this resource and all its notes?')" {
                    "delete"
                }
            }
        }

        h2 { "notes" }

        @if notes.is_empty() {
            p.dim { "no notes yet." }
        } @else {
            table {
                thead {
                    tr { th { "title" } th { "updated" } }
                }
                tbody {
                    @for n in notes {
                        tr {
                            td {
                                a href=(format!("/resources/{}/notes/{}", r.id, n.id)) {
                                    (n.title)
                                }
                            }
                            td.small.dim { (fmt_date(n.updated_at)) }
                        }
                    }
                }
            }
        }

        div.mt {
            a.btn href=(format!("/resources/{}/notes/new", r.id)) { "+ new note" }
        }
    })
}

pub fn edit_page(r: &Resource, tags: &[String]) -> Markup {
    let tags_str = tags.join(", ");
    page(&format!("edit — {}", r.title), html! {
        h1 { "edit resource" }
        form method="post" action=(format!("/resources/{}/edit", r.id)) {
            label { "kind"
                select name="kind" {
                    @for k in &["book","paper","article","blog","repo"] {
                        option value=(k) selected[*k == r.kind] { (k) }
                    }
                }
            }
            label { "title"
                input type="text" name="title" value=(r.title) required autocomplete="off";
            }
            label { "author"
                input type="text" name="author"
                    value=(r.author.as_deref().unwrap_or("")) autocomplete="off";
            }
            label { "url"
                input type="url" name="url"
                    value=(r.url.as_deref().unwrap_or("")) autocomplete="off";
            }
            label { "local file path"
                input type="text" name="file_path"
                    value=(r.file_path.as_deref().unwrap_or("")) autocomplete="off";
            }
            label { "tags (comma-separated)"
                input type="text" name="tags" value=(tags_str) autocomplete="off";
            }
            div.row-actions {
                button.primary type="submit" { "save" }
                a.btn href=(format!("/resources/{}", r.id)) { "cancel" }
            }
        }
    })
}

fn fmt_date(ts: i64) -> String {
    use chrono::{DateTime, Utc};
    DateTime::<Utc>::from_timestamp(ts, 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "—".to_string())
}
