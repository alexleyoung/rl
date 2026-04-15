use crate::handlers::search::SearchHit;
use crate::views::layout::page;
use maud::{html, Markup};

pub fn search_page(query: &str, hits: &[SearchHit]) -> Markup {
    page("search", html! {
        h1 { "search" }
        form action="/search" method="get" {
            div #search-bar {
                input type="text" name="q" value=(query) placeholder="search…" autocomplete="off";
                button type="submit" { "go" }
            }
        }

        @if !query.is_empty() {
            @if hits.is_empty() {
                p.dim.mt { "no results for "" (query) """ }
            } @else {
                p.dim.small.mt { (hits.len()) " result(s) for "" (query) """ }
                table.mt {
                    thead {
                        tr { th { "title" } th { "kind" } th { "source" } }
                    }
                    tbody {
                        @for h in hits {
                            tr {
                                td { a href=(h.url) { (h.title) } }
                                td { span.kind { (h.source_kind) } }
                                td.small.dim { (h.snippet) }
                            }
                        }
                    }
                }
            }
        }
    })
}
