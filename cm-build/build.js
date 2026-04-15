import { EditorState } from "@codemirror/state";
import { EditorView, keymap, lineNumbers } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { markdown } from "@codemirror/lang-markdown";
import { vim, Vim, CodeMirror } from "@replit/codemirror-vim";

// The built-in :w handler calls CodeMirror.commands.save — set it here.
// This is the correct hook point; no defineEx needed for :w.
CodeMirror.commands.save = function() {
  doSave();
};

function doSave() {
  const view = window._cmView;
  if (!view) return;

  const bodyMd   = document.getElementById("note-body-md");
  const bodyHtml = document.getElementById("note-body-html");
  const form     = document.getElementById("note-save-form");
  if (!form || !bodyMd) return;

  bodyMd.value = view.state.doc.toString();
  if (bodyHtml) bodyHtml.value = "";

  // Sync title from editable input if present
  const titleInput  = document.getElementById("note-title-input");
  const titleHidden = document.getElementById("note-title-hidden");
  if (titleInput && titleHidden) {
    titleHidden.value = titleInput.value || "Untitled";
  }

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

  // :q and :wq — defineEx AFTER editor creation so the dispatcher is ready
  Vim.defineEx("quit", "q", function() {
    window.dispatchEvent(new CustomEvent("rl:exitEdit"));
  });

  Vim.defineEx("wq", "wq", function() {
    doSave();
  });

  // Ctrl+S / Cmd+S — works in both insert and normal mode
  window.addEventListener("keydown", function(e) {
    if ((e.ctrlKey || e.metaKey) && e.key === "s") {
      e.preventDefault();
      doSave();
    }
  }, { once: false });

  return view;
};
