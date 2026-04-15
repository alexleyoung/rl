<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { api, type NoteDto, type ResourceDto } from '$lib/api';
  import Editor from '$lib/Editor.svelte';

  const rid = $derived(Number(page.params.id));
  const nid = $derived(Number(page.params.nid));

  let resource = $state<ResourceDto | null>(null);
  let note = $state<NoteDto | null>(null);
  let title = $state('');
  let body_md = $state('');
  let error = $state('');
  let saving = $state(false);
  let loaded = $state(false);

  $effect(() => {
    if (!rid || !nid) return;
    Promise.all([api.getResource(rid), api.getNote(rid, nid)])
      .then(([d, n]) => {
        resource = d.resource;
        note = n;
        title = n.title;
        body_md = n.body_md;
        loaded = true;
      })
      .catch(e => { error = e.message; });
  });

  async function save() {
    if (!title.trim()) { error = 'title required'; return; }
    error = '';
    saving = true;
    try {
      await api.updateNote(rid, nid, { title: title.trim(), body_md });
      // Stay on edit page after :w save (same as old vim UX)
      saving = false;
    } catch (e: any) {
      error = e.message;
      saving = false;
    }
  }

  async function saveAndClose() {
    if (!title.trim()) { error = 'title required'; return; }
    error = '';
    try {
      await api.updateNote(rid, nid, { title: title.trim(), body_md });
      goto(`/resources/${rid}/notes/${nid}`);
    } catch (e: any) {
      error = e.message;
    }
  }
</script>

{#if error}<p class="flash err">{error}</p>{/if}

{#if resource && note && loaded}
  <p class="dim small mb"><a href="/resources/{rid}">{resource.title}</a></p>
  <h1>edit — {note.title}</h1>

  <form onsubmit={(e) => { e.preventDefault(); saveAndClose(); }} style="gap:0.5rem;">
    <div>
      <label for="title">title</label>
      <input id="title" type="text" bind:value={title} />
    </div>
    <div class="row-actions">
      <button type="submit" class="primary" disabled={saving}>{saving ? 'saving…' : 'save & close (:wq)'}</button>
      <button type="button" onclick={() => save()} disabled={saving}>save (:w)</button>
      <a href="/resources/{rid}/notes/{nid}" class="btn">cancel (:q)</a>
    </div>
  </form>

  <div class="mt">
    <Editor
      value={body_md}
      onsave={(v) => { body_md = v; save(); }}
      oncancel={() => goto(`/resources/${rid}/notes/${nid}`)}
      onsaveAndClose={(v) => { body_md = v; saveAndClose(); }}
      onchange={(v) => { body_md = v; }}
    />
  </div>
{/if}
