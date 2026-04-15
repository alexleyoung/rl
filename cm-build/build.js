import { EditorState } from "@codemirror/state";
import { EditorView, keymap, lineNumbers } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { markdown } from "@codemirror/lang-markdown";
import { vim, Vim, CodeMirror } from "@replit/codemirror-vim";

// :w calls CodeMirror.commands.save — hook here, not via defineEx.
CodeMirror.commands.save = function() {
  doSave();
};

function doSave() {
  const view = window._cmView;
  if (!view) return;

  const md = view.state.doc.toString();
  const titleInput = document.getElementById("note-title-input");
  const title = (titleInput ? titleInput.value : null) || "Untitled";
  const saveUrl = window._saveUrl;
  if (!saveUrl) return;

  // Use fetch so we never touch form display/visibility at all
  const body = new URLSearchParams({ title, body_md: md, body_html: "" });
  fetch(saveUrl, {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: body.toString(),
    redirect: "follow",
  }).then(function(res) {
    // Navigate to wherever the server redirected us (the note view page)
    window.location.href = res.url;
  }).catch(function(err) {
    console.error("save failed:", err);
  });
}

window.initEditor = function(containerId, initialContent, saveUrl) {
  const container = document.getElementById(containerId);
  if (!container) return null;

  window._saveUrl = saveUrl;

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
  window._doSave = doSave;

  // Register :q and :wq after editor creation (dispatcher is ready)
  Vim.defineEx("quit", "q", function() {
    window.dispatchEvent(new CustomEvent("rl:exitEdit"));
  });

  Vim.defineEx("wq", "wq", function() {
    doSave();
  });

  // Ctrl+S / Cmd+S
  window.addEventListener("keydown", function(e) {
    if ((e.ctrlKey || e.metaKey) && e.key === "s") {
      e.preventDefault();
      doSave();
    }
  });

  return view;
};
