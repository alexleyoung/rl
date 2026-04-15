import { EditorState } from "@codemirror/state";
import { EditorView, keymap, lineNumbers } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { markdown } from "@codemirror/lang-markdown";
import { vim, Vim, CodeMirror } from "@replit/codemirror-vim";

// :w — delegate to page-defined saveNote()
CodeMirror.commands.save = function() {
  if (typeof window.saveNote === "function") window.saveNote();
};

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

  window._view = view;

  // :q — fire rl:exitEdit so each page can handle it (exit edit mode or navigate away)
  Vim.defineEx("quit", "q", function() {
    window.dispatchEvent(new Event("rl:exitEdit"));
  });

  // :wq — save then exit
  Vim.defineEx("wq", "wq", function() {
    if (typeof window.saveNote === "function") window.saveNote();
  });

  // Ctrl+S / Cmd+S
  window.addEventListener("keydown", function(e) {
    if ((e.ctrlKey || e.metaKey) && e.key === "s") {
      e.preventDefault();
      if (typeof window.saveNote === "function") window.saveNote();
    }
  });

  return view;
};
