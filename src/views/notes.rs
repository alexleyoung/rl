use crate::models::note::Note;
use crate::models::resource::Resource;
use crate::views::layout::page;
use maud::{html, Markup, PreEscaped};

pub fn new_page(r: &Resource) -> Markup {
    let create_url = format!("/resources/{}/notes", r.id);
    let back_url   = format!("/resources/{}", r.id);
    page("new note", html! {
        div.row-actions.mb {
            a.dim.small href=(back_url) { "← " (r.title) }
        }
        h1 { "new note" }

        div.mb {
            input #note-title-input type="text" value="Untitled"
                placeholder="title"
                style="font-size:1.1rem;border:none;border-bottom:1px solid var(--border);width:100%;outline:none;background:transparent;"
                autocomplete="off";
        }

        div #editor-wrap {
            div #cm-container {}
        }

        div.row-actions.mt {
            button.primary onclick="window._doSave()" type="button" { ":w save" }
            a.btn href=(back_url) { "cancel" }
        }

        script src="/static/codemirror.bundle.js" {}
        script {
            (PreEscaped(format!(r#"
window.addEventListener('rl:exitEdit', function() {{
    window.location.href = {back};
}});

window.addEventListener('DOMContentLoaded', function() {{
    window.initEditor('cm-container', '', {save_url});
    if (window._cmView) window._cmView.focus();
}});
"#,
                back     = serde_json::to_string(&back_url).unwrap(),
                save_url = serde_json::to_string(&create_url).unwrap(),
            )))
        }
    })
}

pub fn view_page(r: &Resource, note: &Note) -> Markup {
    let edit_url = format!("/resources/{}/notes/{}/edit", r.id, note.id);
    let del_url  = format!("/resources/{}/notes/{}/delete", r.id, note.id);
    let back_url = format!("/resources/{}", r.id);

    page(&note.title, html! {
        div.row-actions.mb {
            a.dim.small href=(back_url) { "← " (r.title) }
        }

        // Rendered view
        div #note-rendered {
            h1 { (note.title) }
            div.note-body {
                (PreEscaped(&note.body_html))
            }
            p.small.dim.mt { "press " code { "e" } " to edit" }
        }

        // Editor — hidden by default
        div #note-editor style="display:none" {
            div.mb {
                input #note-title-input type="text" value=(note.title)
                    style="font-size:1.1rem;border:none;border-bottom:1px solid var(--border);width:100%;outline:none;background:transparent;"
                    autocomplete="off";
            }
            div #editor-wrap {
                div #cm-container {}
            }
            div.row-actions.mt {
                button.primary onclick="window._doSave()" type="button" { ":w save" }
                button onclick="exitEdit()" type="button" { "cancel" }
                form method="post" action=(del_url) style="display:inline" {
                    button.danger type="submit"
                        onclick="return confirm('delete this note?')" { "delete" }
                }
            }
        }

        script src="/static/codemirror.bundle.js" {}
        script {
            (PreEscaped(format!(r#"
var _editorInit = false;

function enterEdit() {{
    document.getElementById('note-rendered').style.display = 'none';
    document.getElementById('note-editor').style.display = '';
    if (!_editorInit) {{
        _editorInit = true;
        window.initEditor('cm-container', {md}, {edit_url});
    }}
    if (window._cmView) window._cmView.focus();
}}

function exitEdit() {{
    document.getElementById('note-editor').style.display = 'none';
    document.getElementById('note-rendered').style.display = '';
}}

// :q exits edit mode; :w/:wq navigates away (server redirects back here)
window.addEventListener('rl:exitEdit', exitEdit);

document.addEventListener('keydown', function(e) {{
    if (e.key === 'e' && !e.ctrlKey && !e.metaKey && !e.altKey) {{
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
                md       = serde_json::to_string(&note.body_md).unwrap(),
                edit_url = serde_json::to_string(&edit_url).unwrap(),
            )))
        }
    })
}

pub fn edit_page(r: &Resource, note: &Note) -> Markup {
    // Plain fallback (no-JS)
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
