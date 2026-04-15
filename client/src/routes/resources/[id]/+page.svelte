<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { api, type ResourceDto, type NoteDto } from '$lib/api';
  import FileDropInput from '$lib/FileDropInput.svelte';
  import TagsInput from '$lib/TagsInput.svelte';

  const rid = $derived(Number(page.params.id));
  let resource = $state<ResourceDto | null>(null);
  let notes = $state<NoteDto[]>([]);
  let error = $state('');
  let quickValue = $state('');
  let quickSaving = $state(false);
  let editingUrl = $state(false);
  let urlInput = $state<HTMLInputElement | null>(null);

  async function load() {
    try {
      const detail = await api.getResource(rid);
      resource = detail.resource;
      notes = detail.notes;
    } catch (e: any) { error = e.message; }
  }

  $effect(() => { if (rid) load(); });

  async function deleteResource() {
    if (!resource || !confirm(`Delete "${resource.title}"?`)) return;
    await api.deleteResource(rid);
    goto('/');
  }

  async function deleteNote(nid: number) {
    if (!confirm('Delete this note?')) return;
    await api.deleteNote(rid, nid);
    notes = notes.filter(n => Number(n.id) !== nid);
  }

  async function saveUrl() {
    if (!resource) return;
    quickSaving = true;
    try {
      resource = await api.quickSet(rid, { field: 'url', value: quickValue || undefined });
      editingUrl = false;
    } catch (e: any) { error = e.message; }
    finally { quickSaving = false; }
  }

  function startEditUrl() {
    quickValue = resource?.url ?? '';
    editingUrl = true;
    // focus after DOM update
    setTimeout(() => urlInput?.focus(), 0);
  }

  async function saveFilePath(path: string) {
    if (!resource) return;
    try {
      resource = await api.quickSet(rid, { field: 'file_path', value: path || undefined });
    } catch (e: any) { error = e.message; }
  }

  async function saveTags(newTags: string[]) {
    if (!resource) return;
    try {
      resource = await api.setTags(rid, newTags);
    } catch (e: any) { error = e.message; }
  }
</script>

{#if error}<p class="flash err">{error}</p>{/if}

{#if resource}
  <h1>{resource.title}</h1>

  <div class="meta">
    <div class="meta-row"><span class="key">kind</span><span class="kind">{resource.kind}</span></div>
    {#if resource.author}<div class="meta-row"><span class="key">author</span><span>{resource.author}</span></div>{/if}
    <div class="meta-row">
      <span class="key">url</span>
      {#if editingUrl}
        <form class="inline-form" onsubmit={(e) => { e.preventDefault(); saveUrl(); }}>
          <input bind:this={urlInput} type="url" bind:value={quickValue} placeholder="https://…" />
          <button type="submit" disabled={quickSaving}>{quickSaving ? '…' : 'set'}</button>
          <button type="button" onclick={() => { editingUrl = false; }}>cancel</button>
        </form>
      {:else if resource.url}
        <a href={resource.url} target="_blank" rel="noreferrer" onclick={(e) => { if (e.altKey) { e.preventDefault(); startEditUrl(); } }}>{resource.url}</a>
        <button class="inline-edit" type="button" onclick={startEditUrl}>edit</button>
      {:else}
        <button class="unset-link" type="button" onclick={startEditUrl}>set url…</button>
      {/if}
    </div>
    <div class="meta-row">
      <span class="key">file</span>
      <span class="file-drop-wrap">
        <FileDropInput
          bind:value={resource.file_path as string}
          placeholder="drag file here or click to browse"
          onchange={saveFilePath}
          fileUrl={resource.file_path ? api.fileUrl(rid) : undefined}
        />
      </span>
    </div>
    <div class="meta-row">
      <span class="key">tags</span>
      <span class="tags-wrap">
        <TagsInput
          bind:tags={resource.tags}
          onchange={saveTags}
          placeholder="add tag…"
        />
      </span>
    </div>
  </div>

  <div class="row-actions mb">
    <a href="/resources/{rid}/edit" class="btn">edit</a>
    <button class="danger" onclick={deleteResource}>delete</button>
  </div>

  <h2>notes</h2>
  {#if notes.length === 0}
    <p class="dim small">no notes yet</p>
  {:else}
    <table>
      <thead><tr><th>title</th><th></th></tr></thead>
      <tbody>
        {#each notes as n}
          <tr>
            <td><a href="/resources/{rid}/notes/{n.id}">{n.title}</a></td>
            <td>
              <span class="row-actions">
                <a href="/resources/{rid}/notes/{n.id}/edit" class="btn small">edit</a>
                <button class="danger small" onclick={() => deleteNote(Number(n.id))}>del</button>
              </span>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}

  <div class="mt">
    <a href="/resources/{rid}/notes/new" class="btn primary">+ note</a>
  </div>
{/if}

<style>
  .unset-link {
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    color: var(--dim);
    cursor: pointer;
    text-decoration: underline;
    text-underline-offset: 2px;
  }
  .unset-link:hover { color: var(--fg); }

  .inline-edit {
    background: none;
    border: none;
    padding: 0 0 0 0.5rem;
    font: inherit;
    font-size: 0.8em;
    color: var(--dim);
    cursor: pointer;
    text-decoration: underline;
    text-underline-offset: 2px;
    opacity: 0;
  }
  .meta-row:hover .inline-edit { opacity: 1; }

  .inline-form {
    display: flex;
    flex-direction: row;
    gap: 0.4rem;
    align-items: center;
    flex: 1;
  }
  .inline-form input[type="url"] { flex: 1; }

  .file-drop-wrap { flex: 1; }
  .tags-wrap { flex: 1; }
</style>
