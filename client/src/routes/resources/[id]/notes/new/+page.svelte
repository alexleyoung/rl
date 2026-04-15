<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { api, type NoteInputDto } from '$lib/api';

  const rid = $derived(Number(page.params.id));
  let title = $state('');
  let body_md = $state('');
  let error = $state('');
  let saving = $state(false);

  async function save() {
    if (!title.trim()) { error = 'title required'; return; }
    error = '';
    saving = true;
    try {
      const n = await api.createNote(rid, { title: title.trim(), body_md });
      goto(`/resources/${rid}/notes/${n.id}`);
    } catch (e: any) {
      error = e.message;
      saving = false;
    }
  }
</script>

<h1>new note</h1>

{#if error}<p class="flash err">{error}</p>{/if}

<form onsubmit={(e) => { e.preventDefault(); save(); }}>
  <div>
    <label for="title">title *</label>
    <input id="title" type="text" bind:value={title} />
  </div>
  <div>
    <label for="body">body (markdown)</label>
    <textarea id="body" rows={16} bind:value={body_md}></textarea>
  </div>
  <div class="row-actions">
    <button type="submit" class="primary" disabled={saving}>{saving ? 'saving…' : 'save'}</button>
    <a href="/resources/{rid}" class="btn">cancel</a>
  </div>
</form>
