use crate::models::note::Note;
use crate::models::resource::Resource;
use crate::views::layout::page;
use maud::{html, Markup, PreEscaped};

pub fn new_page(r: &Resource) -> Markup {
    page("new note", html! {
        h1 { "new note — " a href=(format!("/resources/{}", r.id)) { (r.title) } }
        form method="post" action=(format!("/resources/{}/notes", r.id)) {
            label { "title"
                input type="text" name="title" value="Untitled" required autocomplete="off";
            }
            label { "body (markdown)"
                textarea name="body_md" rows="20" {}
            }
            div.row-actions {
                button.primary type="submit" { "save" }
                a.btn href=(format!("/resources/{}", r.id)) { "cancel" }
            }
        }
    })
}

pub fn view_page(r: &Resource, note: &Note) -> Markup {
    page(&note.title, html! {
        div.row-actions.mb {
            a.dim.small href=(format!("/resources/{}", r.id)) { "← " (r.title) }
        }
        h1 { (note.title) }
        div #note-rendered.note-body {
            (PreEscaped(&note.body_html))
        }
        // Hidden edit form — toggled by JS
        div #note-editor style="display:none" {
            // Actual save form
            form #note-save-form method="post"
                action=(format!("/resources/{}/notes/{}/edit", r.id, note.id)) {
                input type="hidden" name="title" id="note-title-hidden" value=(note.title);
                input type="hidden" name="body_md" id="note-body-md";
                input type="hidden" name="body_html" id="note-body-html";
            }
            // Title edit
            div.mb {
                input #note-title-input type="text" value=(note.title)
                    style="font-size:1.1rem;border:none;border-bottom:1px solid var(--border);width:100%;outline:none;background:transparent;"
                    oninput="document.getElementById('note-title-hidden').value=this.value";
            }
            // CodeMirror mounts here
            div #editor-wrap {
                div #cm-container {}
            }
            div.row-actions.mt {
                button.primary onclick="saveNote()" type="button" { ":w save" }
                button onclick="exitEdit()" type="button" { "cancel" }
                form method="post" action=(format!("/resources/{}/notes/{}/delete", r.id, note.id))
                    style="display:inline" {
                    button.danger type="submit"
                        onclick="return confirm('delete this note?')" { "delete" }
                }
            }
        }
        script src="/static/codemirror.bundle.js" {}
        script {
            (PreEscaped(format!(r#"
var _view = null;
var _initialMd = {md};

function enterEdit() {{
    document.getElementById('note-rendered').style.display = 'none';
    document.getElementById('note-editor').style.display = '';
    if (!_view) {{
        _view = window.initEditor('cm-container', _initialMd, null);
    }}
    if (_view) _view.focus();
}}

function exitEdit() {{
    document.getElementById('note-editor').style.display = 'none';
    document.getElementById('note-rendered').style.display = '';
}}

function saveNote() {{
    if (!_view) return;
    var md = _view.state.doc.toString();
    document.getElementById('note-body-md').value = md;
    // body_html is generated server-side; send empty, server will render
    document.getElementById('note-body-html').value = '';
    document.getElementById('note-save-form').submit();
}}

document.addEventListener('keydown', function(e) {{
    if (e.key === 'e' && !e.ctrlKey && !e.metaKey) {{
        var tag = document.activeElement && document.activeElement.tagName;
        if (tag !== 'INPUT' && tag !== 'TEXTAREA') {{
            var editor = document.getElementById('note-editor');
            if (editor && editor.style.display === 'none') {{
                e.preventDefault();
                enterEdit();
            }}
        }}
    }}
}});
"#,
                md = serde_json::to_string(&note.body_md).unwrap()
            )))
        }
    })
}

pub fn edit_page(r: &Resource, note: &Note) -> Markup {
    // Simple fallback non-JS edit page
    page(&format!("edit — {}", note.title), html! {
        h1 { "edit note" }
        form method="post" action=(format!("/resources/{}/notes/{}/edit", r.id, note.id)) {
            label { "title"
                input type="text" name="title" value=(note.title) required autocomplete="off";
            }
            label { "body (markdown)"
                textarea name="body_md" rows="30" { (note.body_md) }
            }
            input type="hidden" name="body_html" value="";
            div.row-actions {
                button.primary type="submit" { "save" }
                a.btn href=(format!("/resources/{}/notes/{}", r.id, note.id)) { "cancel" }
            }
        }
    })
}
