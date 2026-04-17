<script lang="ts">
  import { api, type NoteDto } from '$lib/api';
  import Editor from '$lib/Editor.svelte';

  interface Props {
    rid: number;
    note: NoteDto | null;
    oncreated?: (n: NoteDto) => void;
    onupdated?: (n: NoteDto) => void;
  }

  let { rid, note, oncreated, onupdated }: Props = $props();

  let editing = $state(false);
  let draft = $state('');
  let saving = $state(false);
  let saved = $state(false);

  function startEdit() {
    draft = note?.body_md ?? '';
    editing = true;
  }

  async function save(value: string) {
    saving = true;
    try {
      if (note) {
        const updated = await api.updateNote(rid, Number(note.id), { title: note.title, body_md: value });
        onupdated?.(updated);
      } else {
        const created = await api.createNote(rid, { title: 'notes', body_md: value });
        oncreated?.(created);
      }
      saved = true;
      setTimeout(() => saved = false, 1500);
    } finally { saving = false; }
  }

  async function saveAndClose(value: string) {
    await save(value);
    editing = false;
  }

  function cancel() { editing = false; }
</script>

{#if editing}
  <div class="notes-editor">
    <Editor value={draft} onsave={save} onsaveAndClose={saveAndClose} oncancel={cancel} />
    <div class="notes-foot">
      <span class="dim">{saving ? 'saving…' : saved ? 'saved' : 'press :w to save · :wq save & close · :q cancel'}</span>
      <button class="btn" onclick={cancel}>close</button>
    </div>
  </div>
{:else if note}
  <div class="rendered" ondblclick={startEdit} role="presentation">
    {@html note.body_html}
  </div>
  <div class="notes-foot">
    <button class="btn" onclick={startEdit}>edit</button>
    <span class="dim">double-click to edit</span>
  </div>
{:else}
  <div class="empty" role="presentation">
    <p class="dim">no notes yet.</p>
    <button class="btn primary" onclick={startEdit}>+ start writing</button>
  </div>
{/if}

<style>
  .notes-editor :global(#editor-wrap) { min-height: 360px; }
  .notes-foot {
    display: flex; justify-content: space-between; align-items: center;
    gap: 10px; margin-top: 10px; font-size: 11px;
  }
  .empty { display: flex; flex-direction: column; gap: 10px; align-items: flex-start; }
  .rendered { min-height: 200px; }
</style>
