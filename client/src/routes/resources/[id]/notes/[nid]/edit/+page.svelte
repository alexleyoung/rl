<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { api, type NoteDto, type ResourceDto } from '$lib/api';

  const rid = $derived(Number(page.params.id));
  const nid = $derived(Number(page.params.nid));

  let resource = $state<ResourceDto | null>(null);
  let note = $state<NoteDto | null>(null);
  let title = $state('');
  let body_md = $state('');
  let error = $state('');
  let saving = $state(false);

  $effect(() => {
    if (!rid || !nid) return;
    Promise.all([api.getResource(rid), api.getNote(rid, nid)])
      .then(([d, n]) => {
        resource = d.resource;
        note = n;
        title = n.title;
        body_md = n.body_md;
      })
      .catch(e => { error = e.message; });
  });

  async function save() {
    if (!title.trim()) { error = 'title required'; return; }
    error = '';
    saving = true;
    try {
      await api.updateNote(rid, nid, { title: title.trim(), body_md });
      goto(`/resources/${rid}/notes/${nid}`);
    } catch (e: any) {
      error = e.message;
      saving = false;
    }
  }
</script>

{#if error}<p class="flash err">{error}</p>{/if}

{#if resource && note}
  <p class="dim small mb"><a href="/resources/{rid}">{resource.title}</a></p>
  <h1>edit — {note.title}</h1>

  <form onsubmit={(e) => { e.preventDefault(); save(); }}>
    <div>
      <label for="title">title</label>
      <input id="title" type="text" bind:value={title} />
    </div>
    <div>
      <label for="body">body (markdown)</label>
      <!-- Editor.svelte replaces this textarea in Phase 6 -->
      <textarea id="body" rows={24} bind:value={body_md}></textarea>
    </div>
    <div class="row-actions">
      <button type="submit" class="primary" disabled={saving}>{saving ? 'saving…' : 'save'}</button>
      <a href="/resources/{rid}/notes/{nid}" class="btn">cancel</a>
    </div>
  </form>
{/if}
