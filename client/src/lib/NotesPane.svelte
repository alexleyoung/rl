<script lang="ts">
  import { api, type NoteDto } from '$lib/api';
  import Editor from '$lib/Editor.svelte';

  function focusOnMount(el: HTMLElement) { el.focus(); }

  interface Props {
    rid: number;
    notes: NoteDto[];
    oncreated?: (n: NoteDto) => void;
    onupdated?: (n: NoteDto) => void;
    ondeleted?: (id: number) => void;
  }

  let { rid, notes = [], oncreated, onupdated, ondeleted }: Props = $props();

  let activeId = $state<number | null>(null);
  let editing = $state(false);
  let draft = $state('');
  let saving = $state(false);
  let saved = $state(false);
  let renamingId = $state<number | null>(null);
  let renameValue = $state('');

  let note = $derived(
    activeId != null
      ? (notes.find(n => Number(n.id) === activeId) ?? notes[notes.length - 1] ?? null)
      : (notes[notes.length - 1] ?? null)
  );

  function select(n: NoteDto) {
    activeId = Number(n.id);
    editing = false;
  }

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
        activeId = Number(created.id);
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

  function startRename(n: NoteDto) {
    renamingId = Number(n.id);
    renameValue = n.title;
  }

  async function commitRename(n: NoteDto) {
    const trimmed = renameValue.trim();
    renamingId = null;
    if (!trimmed || trimmed === n.title) return;
    const updated = await api.updateNote(rid, Number(n.id), { title: trimmed, body_md: n.body_md });
    onupdated?.(updated);
  }

  function onRenameKey(e: KeyboardEvent, n: NoteDto) {
    if (e.key === 'Enter') { e.preventDefault(); (e.target as HTMLElement).blur(); }
    if (e.key === 'Escape') { renamingId = null; }
  }

  let deletingId = $state<number | null>(null);
  let deleteTimer: ReturnType<typeof setTimeout> | null = null;

  async function deleteNote(n: NoteDto, e: MouseEvent) {
    e.stopPropagation();
    const id = Number(n.id);
    if (deletingId !== id) {
      deletingId = id;
      deleteTimer = setTimeout(() => { deletingId = null; }, 3000);
      return;
    }
    if (deleteTimer) clearTimeout(deleteTimer);
    deletingId = null;
    await api.deleteNote(rid, id);
    if (activeId === id) activeId = null;
    ondeleted?.(id);
  }
</script>

{#if notes.length > 0}
  <div class="note-tabs" onfocusout={(e) => { if (!e.currentTarget.contains(e.relatedTarget as Node)) { if (deleteTimer) clearTimeout(deleteTimer); deletingId = null; } }}>
    {#each notes as n}
      {#if renamingId === Number(n.id)}
        <input
          class="tab-rename"
          bind:value={renameValue}
          onblur={() => commitRename(n)}
          onkeydown={(e) => onRenameKey(e, n)}
          use:focusOnMount
        />
      {:else}
        <div
          class="tab"
          class:active={Number(n.id) === Number(note?.id)}
          role="button" tabindex="0"
          onclick={() => select(n)}
          ondblclick={() => startRename(n)}
          onkeydown={(e) => e.key === 'Enter' && select(n)}
          title="double-click to rename"
        >
          <span class="tab-label">{n.title}</span>
          <button
            class="tab-del"
            class:confirm={deletingId === Number(n.id)}
            onclick={(e) => deleteNote(n, e)}
            title={deletingId === Number(n.id) ? 'confirm delete' : 'delete note'}
          >{deletingId === Number(n.id) ? '?' : '×'}</button>
        </div>
      {/if}
    {/each}
  </div>
{/if}

{#if editing}
  <div class="notes-editor">
    <Editor value={draft} onsave={save} onsaveAndClose={saveAndClose} oncancel={cancel} />
    <div class="notes-foot">
      <span class="dim">{saving ? 'saving…' : saved ? 'saved' : ':w save · :wq save & close · :q cancel'}</span>
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
  :global(.notes-body) { display: flex; flex-direction: column; }

  .note-tabs {
    display: flex; flex-wrap: wrap; gap: 4px;
    padding-bottom: 8px; border-bottom: 1px solid var(--border);
    margin-bottom: 8px; flex-shrink: 0;
  }
  .tab {
    display: flex; align-items: center; gap: 4px;
    border: 1px solid var(--border); padding: 2px 4px 2px 8px;
    font: inherit; font-size: 11px; cursor: pointer; color: var(--dim);
    max-width: 150px; white-space: nowrap; user-select: none;
  }
  .tab.active { color: var(--fg); border-color: var(--fg); }
  .tab:hover:not(.active) { color: var(--fg); }
  .tab-label { overflow: hidden; text-overflow: ellipsis; flex: 1; min-width: 0; }
  .tab-del {
    background: none; border: none; padding: 0 2px; font: inherit; font-size: 11px;
    cursor: pointer; color: var(--dim); flex-shrink: 0; line-height: 1;
  }
  .tab-del:hover, .tab-del.confirm { color: var(--danger); }
  .tab-rename {
    background: none; border: 1px solid var(--fg); padding: 2px 8px;
    font: inherit; font-size: 11px; color: var(--fg); outline: none;
    max-width: 120px; min-width: 40px;
  }

  .notes-editor {
    display: flex; flex-direction: column; flex: 1; min-height: 0;
  }
  .notes-editor :global(#editor-wrap) {
    flex: 1; min-height: 0; overflow: auto;
  }
  .rendered {
    flex: 1; min-height: 0; overflow: auto;
  }
  .empty {
    flex: 1; display: flex; flex-direction: column; gap: 10px; align-items: flex-start;
  }
  .notes-foot {
    display: flex; justify-content: space-between; align-items: center;
    gap: 10px; padding-top: 8px; font-size: 11px; flex-shrink: 0;
  }
</style>
