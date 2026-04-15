import { EditorState } from "@codemirror/state";
import { EditorView, keymap, lineNumbers } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { markdown } from "@codemirror/lang-markdown";
import { vim } from "@replit/codemirror-vim";

// Expose initEditor on window for use by the page
window.initEditor = function(containerId, initialContent, saveUrl) {
  const container = document.getElementById(containerId);
  if (!container) return;

  const view = new EditorView({
    state: EditorState.create({
      doc: initialContent,
      extensions: [
        vim(),
        history(),
        keymap.of([...defaultKeymap, ...historyKeymap]),
        lineNumbers(),
        markdown(),
        EditorView.lineWrapping,
        // Save on :w (vim write command triggers blur on the underlying textarea,
        // but we hook the vim ex command via a custom save function)
      ],
    }),
    parent: container,
  });

  // Intercept vim :w to submit the form
  // @replit/codemirror-vim dispatches a custom event or we can override via the
  // standard vim.js API: Vim.defineEx
  // The vim extension exposes Vim via the global set by the lib
  // We listen for a custom "cm:save" event we'll fire from Vim.defineEx
  if (window.Vim) {
    window.Vim.defineEx("write", "w", function() {
      submitEditor(view, saveUrl);
    });
    window.Vim.defineEx("wq", "wq", function() {
      submitEditor(view, saveUrl);
    });
    window.Vim.defineEx("quit", "q", function() {
      window.location.reload();
    });
  }

  // Also wire Ctrl+S as fallback
  container.addEventListener("keydown", function(e) {
    if ((e.ctrlKey || e.metaKey) && e.key === "s") {
      e.preventDefault();
      submitEditor(view, saveUrl);
    }
  });

  return view;
};

function submitEditor(view, saveUrl) {
  const content = view.state.doc.toString();
  const form = document.getElementById("note-save-form");
  if (form) {
    document.getElementById("note-body-md").value = content;
    form.requestSubmit();
  }
}
