<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { api, type ResourceDto, type NoteDto } from '$lib/api';

  const rid = $derived(Number(page.params.id));
  let resource = $state<ResourceDto | null>(null);
  let notes = $state<NoteDto[]>([]);
  let error = $state('');
  let quickField = $state<'url'|'file_path'>('url');
  let quickValue = $state('');
  let quickSaving = $state(false);

  async function load() {
    try {
      const detail = await api.getResource(rid);
      resource = detail.resource;
      notes = detail.notes;
      quickValue = resource.url ?? resource.file_path ?? '';
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

  async function saveQuickSet() {
    if (!resource) return;
    quickSaving = true;
    try {
      resource = await api.quickSet(rid, { field: quickField, value: quickValue || undefined });
    } catch (e: any) { error = e.message; }
    finally { quickSaving = false; }
  }
</script>

{#if error}<p class="flash err">{error}</p>{/if}

{#if resource}
  <h1>{resource.title}</h1>

  <div class="meta">
    <div class="meta-row"><span class="key">kind</span><span class="kind">{resource.kind}</span></div>
    {#if resource.author}<div class="meta-row"><span class="key">author</span><span>{resource.author}</span></div>{/if}
    {#if resource.url}
      <div class="meta-row">
        <span class="key">url</span>
        <a href={resource.url} target="_blank" rel="noreferrer">{resource.url}</a>
      </div>
    {/if}
    {#if resource.file_path}
      <div class="meta-row">
        <span class="key">file</span>
        <a href={api.fileUrl(rid)} target="_blank">{resource.file_path}</a>
      </div>
    {/if}
    {#if resource.tags.length}
      <div class="meta-row"><span class="key">tags</span><span>{resource.tags.join(', ')}</span></div>
    {/if}
  </div>

  <!-- Quick-set URL or file path -->
  <div class="mb">
    <form style="flex-direction:row; gap:0.5rem; align-items:center;" onsubmit={(e) => { e.preventDefault(); saveQuickSet(); }}>
      <select bind:value={quickField} style="width:auto;">
        <option value="url">url</option>
        <option value="file_path">file</option>
      </select>
      <input type="text" bind:value={quickValue} placeholder="set value…" />
      <button type="submit" disabled={quickSaving}>{quickSaving ? '…' : 'set'}</button>
    </form>
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
