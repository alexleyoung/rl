<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorState } from '@codemirror/state';
  import { EditorView, keymap, lineNumbers } from '@codemirror/view';
  import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
  import { markdown } from '@codemirror/lang-markdown';
  import { vim, Vim } from '@replit/codemirror-vim';

  interface Props {
    value?: string;
    onsave?: (value: string) => void;
    oncancel?: () => void;
    onsaveAndClose?: (value: string) => void;
    onchange?: (value: string) => void;
  }

  let {
    value = '',
    onsave,
    oncancel,
    onsaveAndClose,
    onchange,
  }: Props = $props();

  let host: HTMLDivElement;
  let view: EditorView;

  onMount(() => {
    view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: value,
        extensions: [
          vim(),
          history(),
          keymap.of([...defaultKeymap, ...historyKeymap]),
          lineNumbers(),
          markdown(),
          EditorView.lineWrapping,
          EditorView.updateListener.of(update => {
            if (update.docChanged) {
              onchange?.(update.state.doc.toString());
            }
          }),
        ],
      }),
    });

    // Ex commands captured via closure — no window globals
    Vim.defineEx('write', 'w', () => onsave?.(view.state.doc.toString()));
    Vim.defineEx('quit', 'q', () => oncancel?.());
    Vim.defineEx('wq', 'wq', () => onsaveAndClose?.(view.state.doc.toString()));

    function onKeyDown(e: KeyboardEvent) {
      if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault();
        onsave?.(view.state.doc.toString());
      }
    }

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  });

  onDestroy(() => view?.destroy());
</script>

<div id="editor-wrap" bind:this={host}></div>
