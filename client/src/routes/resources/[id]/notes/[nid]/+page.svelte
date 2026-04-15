<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { api, type NoteDto, type ResourceDto } from '$lib/api';

  const rid = $derived(Number(page.params.id));
  const nid = $derived(Number(page.params.nid));

  let resource = $state<ResourceDto | null>(null);
  let note = $state<NoteDto | null>(null);
  let error = $state('');

  $effect(() => {
    if (!rid || !nid) return;
    Promise.all([api.getResource(rid), api.getNote(rid, nid)])
      .then(([d, n]) => { resource = d.resource; note = n; })
      .catch(e => { error = e.message; });
  });

  async function deleteNote() {
    if (!confirm('Delete this note?')) return;
    await api.deleteNote(rid, nid);
    goto(`/resources/${rid}`);
  }
</script>

{#if error}<p class="flash err">{error}</p>{/if}

{#if resource && note}
  <p class="dim small mb"><a href="/resources/{rid}">{resource.title}</a></p>
  <h1>{note.title}</h1>

  <div class="row-actions mb">
    <a href="/resources/{rid}/notes/{nid}/edit" class="btn primary">edit</a>
    <button class="danger" onclick={deleteNote}>delete</button>
  </div>

  <div class="note-body">
    {@html note.body_html}
  </div>
{/if}
