use maud::{html, Markup, DOCTYPE};

pub fn page(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " — rl" }
                link rel="stylesheet" href="/static/app.css";
                script src="/static/htmx.min.js" {}
            }
            body {
                div id="wrap" {
                    nav {
                        a.brand href="/" { "rl" }
                        a href="/" { "resources" }
                        a href="/search" { "search" }
                    }
                    (content)
                }
            }
        }
    }
}
