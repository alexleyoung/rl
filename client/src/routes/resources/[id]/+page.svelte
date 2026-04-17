<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { api, type ResourceDto, type NoteDto } from '$lib/api';
  import Panes, { type PaneMode } from '$lib/Panes.svelte';
  import NotesPane from '$lib/NotesPane.svelte';
  import PdfPane from '$lib/PdfPane.svelte';
  import { pageActions, pageHandlers } from '$lib/paletteActions';
  import { STATUSES } from '$lib/status';

  const rid = $derived(Number(page.params.id));
  let resource = $state<ResourceDto | null>(null);
  let notes = $state<NoteDto[]>([]);
  let error = $state('');
  let paneMode = $state<PaneMode>('both');

  async function load() {
    try {
      const detail = await api.getResource(rid);
      resource = detail.resource;
      notes = detail.notes;
      api.markRead(rid).catch(() => {});
    } catch (e: any) { error = e.message; }
  }

  $effect(() => { if (rid) load(); });

  async function setStatus(s: string) {
    if (!resource) return;
    try {
      resource = await api.setStatus(rid, s);
    } catch (e: any) { error = e.message; }
  }

  function togglePdf() {
    paneMode = paneMode === 'notes-only' ? 'both' : 'notes-only';
  }
  function toggleNotes() {
    paneMode = paneMode === 'pdf-only' ? 'both' : 'pdf-only';
  }

  let confirmDelete = $state(false);

  async function deleteResource() {
    if (!confirmDelete) { confirmDelete = true; setTimeout(() => confirmDelete = false, 4000); return; }
    await api.deleteResource(rid);
    goto('/');
  }

  let primaryNote = $derived(notes.length > 0 ? notes[notes.length - 1] : null);

  function onNoteCreated(n: NoteDto) { notes = [...notes, n]; }
  function onNoteUpdated(n: NoteDto) { notes = notes.map(x => x.id === n.id ? n : x); }

  $effect(() => {
    const id = rid;
    pageActions.set([
      ...STATUSES.map(s => ({ label: `mark as ${s}`, run: () => setStatus(s) })),
      { label: 'close pdf',       run: () => { paneMode = 'notes-only'; } },
      { label: 'close notes',     run: () => { paneMode = 'pdf-only'; } },
      { label: 'show both panes', run: () => { paneMode = 'both'; } },
      { label: 'edit resource',   run: () => goto(`/resources/${id}/edit`) },
      { label: 'new note',        run: () => goto(`/resources/${id}/notes/new`) },
      { label: 'delete resource', run: () => deleteResource() },
    ]);
    pageHandlers.set({
      'toggle pdf pane':   togglePdf,
      'toggle notes pane': toggleNotes,
      'swap panes':        () => { paneMode = paneMode === 'both' ? 'pdf-only' : paneMode === 'pdf-only' ? 'notes-only' : 'both'; },
    });
    return () => { pageActions.set([]); pageHandlers.set({}); };
  });
</script>

{#if error}<p class="flash err">{error}</p>{/if}

{#if resource}
  <div class="reader-top">
    <a href="/">← resources</a>
    {#each resource.tags as t, i}
      <span class="crumb">/</span>
      <a href="/t/{encodeURIComponent(t)}" class="muted">#{t}</a>
    {/each}
  </div>

  <div class="reader-head">
    <h1>{resource.title}</h1>
    <div class="meta">
      {#if resource.author}<span>{resource.author}</span>{/if}
      <span>{resource.kind}</span>
      {#if resource.added_at}<span class="dim">added {new Date(Number(resource.added_at) * 1000).toLocaleDateString()}</span>{/if}
      {#if resource.url}<span><a href={resource.url} target="_blank" rel="noreferrer">open ↗</a></span>{/if}
    </div>
    <div class="actions">
      <span class="muted">status:</span>
      {#each ['inbox','reading','queue','done'] as s}
        <button class:on={resource.status === s} onclick={() => setStatus(s)}>{s}</button>
        {#if s !== 'done'}<span class="dim">·</span>{/if}
      {/each}
      <span class="right-ml">
        <a href="/resources/{rid}/edit" class="muted">edit ↗</a>
      </span>
    </div>
  </div>

  <div class="reader-head" style="border: 0; padding: 0; margin-bottom: 8px;">
    <div class="actions">
      <span class="muted">panes:</span>
      <button class:on={paneMode !== 'notes-only'} onclick={togglePdf}>pdf</button>
      <span class="dim">·</span>
      <button class:on={paneMode !== 'pdf-only'} onclick={toggleNotes}>notes</button>
      <span class="right-ml muted">{notes.length} note{notes.length === 1 ? '' : 's'}</span>
    </div>
  </div>

  <Panes bind:mode={paneMode} pdfLabel={`${resource.file_path ? 'pdf' : resource.url ? 'source' : 'content'}`}>
    {#snippet pdf()}<PdfPane resource={resource!} />{/snippet}
    {#snippet notes()}<NotesPane {rid} note={primaryNote} oncreated={onNoteCreated} onupdated={onNoteUpdated} />{/snippet}
  </Panes>

  <div class="reader-foot">
    <a href="/resources/{rid}/notes/new" class="muted">+ new note</a>
    <span class="dim">·</span>
    <a href="/resources/{rid}/edit" class="muted">edit metadata</a>
    <span class="dim">·</span>
    <button class="link-btn danger" onclick={deleteResource}>{confirmDelete ? 'confirm delete?' : 'delete'}</button>
  </div>
{/if}

<style>
  .reader-foot {
    margin-top: 14px; display: flex; gap: 10px; align-items: center;
    font-size: 12px;
  }
  .link-btn {
    background: none; border: none; padding: 0; font: inherit; cursor: pointer;
  }
  .link-btn.danger { color: var(--danger); }
  .link-btn:hover { text-decoration: underline; }
</style>
