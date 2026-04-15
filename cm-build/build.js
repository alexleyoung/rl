import { EditorState } from "@codemirror/state";
import { EditorView, keymap, lineNumbers } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { markdown } from "@codemirror/lang-markdown";
import { vim, Vim } from "@replit/codemirror-vim";

// Register :w / :wq / :q BEFORE any editor is created.
// Vim.defineEx is global to the vim extension — all editor instances share it.
Vim.defineEx("write", "w", function() {
  submitEditor();
});
Vim.defineEx("wq", "wq", function() {
  submitEditor();
  window.location.reload();
});
Vim.defineEx("quit", "q", function() {
  window.dispatchEvent(new CustomEvent("cm:cancel"));
});

function submitEditor() {
  const view = window._cmView;
  if (!view) return;
  const md = view.state.doc.toString();
  const bodyMd = document.getElementById("note-body-md");
  const bodyHtml = document.getElementById("note-body-html");
  const form = document.getElementById("note-save-form");
  if (!form || !bodyMd) return;
  bodyMd.value = md;
  if (bodyHtml) bodyHtml.value = "";
  form.requestSubmit();
}

window.initEditor = function(containerId, initialContent) {
  const container = document.getElementById(containerId);
  if (!container) return null;

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
      ],
    }),
    parent: container,
  });

  // Store globally so submitEditor and saveNote() can reach it
  window._cmView = view;

  // Ctrl+S / Cmd+S fallback
  container.addEventListener("keydown", function(e) {
    if ((e.ctrlKey || e.metaKey) && e.key === "s") {
      e.preventDefault();
      submitEditor();
    }
  });

  // :q cancels edit
  window.addEventListener("cm:cancel", function() {
    window.dispatchEvent(new CustomEvent("rl:exitEdit"));
  });

  return view;
};
