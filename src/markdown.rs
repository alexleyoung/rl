use pulldown_cmark::{html as cm_html, Options, Parser};

pub fn render(md: &str) -> String {
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(md, opts);
    let mut out = String::new();
    cm_html::push_html(&mut out, parser);
    out
}
