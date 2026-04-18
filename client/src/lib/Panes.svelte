<script lang="ts">
  import type { Snippet } from 'svelte';

  export type PaneMode = 'both' | 'pdf-only' | 'notes-only';

  interface Props {
    mode?: PaneMode;
    pdf?: Snippet;
    notesPane?: Snippet;
    pdfLabel?: string;
    notesLabel?: string;
    onClosePdf?: () => void;
    onCloseNotes?: () => void;
  }

  let {
    mode = $bindable('both'),
    pdf,
    notesPane,
    pdfLabel = 'pdf',
    notesLabel = 'notes · markdown',
    onClosePdf,
    onCloseNotes,
  }: Props = $props();
</script>

<div class="panes {mode}">
  {#if mode !== 'notes-only'}
    <div class="pane">
      <div class="pane-head">
        <span>{pdfLabel}</span>
        <button class="close" title="close pane" onclick={() => { mode = 'notes-only'; onClosePdf?.(); }}>×</button>
      </div>
      <div class="pane-body pdf-body">
        {#if pdf}{@render pdf()}{/if}
      </div>
    </div>
  {/if}
  {#if mode === 'both'}<div class="divider"></div>{/if}
  {#if mode !== 'pdf-only'}
    <div class="pane">
      <div class="pane-head">
        <span>{notesLabel}</span>
        <button class="close" title="close pane" onclick={() => { mode = 'pdf-only'; onCloseNotes?.(); }}>×</button>
      </div>
      <div class="pane-body notes-body">
        {#if notesPane}{@render notesPane()}{/if}
      </div>
    </div>
  {/if}
</div>
