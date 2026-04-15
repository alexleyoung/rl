import { EditorState } from "@codemirror/state";
import { EditorView, keymap, lineNumbers } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { markdown } from "@codemirror/lang-markdown";
import { vim, Vim } from "@replit/codemirror-vim";

// Define ex commands at module load time — before any editor is created.
// This ensures :w/:wq/:q are available as soon as vim mode is active.
Vim.defineEx("write", "w", function() {
  doSave();
});

Vim.defineEx("wq", "wq", function() {
  doSave();
});

Vim.defineEx("quit", "q", function() {
  window.dispatchEvent(new CustomEvent("rl:exitEdit"));
});

function doSave() {
  const view = window._cmView;
  if (!view) return;

  const bodyMd   = document.getElementById("note-body-md");
  const bodyHtml = document.getElementById("note-body-html");
  const form     = document.getElementById("note-save-form");

  if (!form || !bodyMd) return;

  bodyMd.value = view.state.doc.toString();
  if (bodyHtml) bodyHtml.value = "";

  // Sync title from input if present (new note page)
  const titleInput  = document.getElementById("note-title-input");
  const titleHidden = document.getElementById("note-title-hidden");
  if (titleInput && titleHidden) {
    titleHidden.value = titleInput.value || "Untitled";
  }

  // Use plain .submit() — requestSubmit() can fail when form is display:none
  // in some browsers even though form itself is outside hidden containers here.
  form.submit();
}

window.initEditor = function(containerId, initialContent) {
  const container = document.getElementById(containerId);
  if (!container) return null;

  const view = new EditorView({
    state: EditorState.create({
      doc: initialContent || "",
      extensions: [
        vim(),
        history(),
        keymap.of([...defaultKeymap, ...historyKeymap]),
        lineNumbers(),
        markdown(),
        EditorView.lineWrapping,
      ],
    }),
    parent: container,
  });

  window._cmView = view;

  // Ctrl+S / Cmd+S fallback (works whether in insert or normal mode)
  window.addEventListener("keydown", function(e) {
    if ((e.ctrlKey || e.metaKey) && e.key === "s") {
      e.preventDefault();
      doSave();
    }
  });

  return view;
};
