<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { api } from '$lib/api';
  import Editor from '$lib/Editor.svelte';

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

  async function saveAndClose() {
    await save();
  }
</script>

<h1>new note</h1>

{#if error}<p class="flash err">{error}</p>{/if}

<form onsubmit={(e) => { e.preventDefault(); save(); }} style="gap:0.5rem;">
  <div>
    <label for="title">title *</label>
    <input id="title" type="text" bind:value={title} />
  </div>
  <div class="row-actions">
    <button type="submit" class="primary" disabled={saving}>{saving ? 'saving…' : 'save (:w)'}</button>
    <a href="/resources/{rid}" class="btn">cancel (:q)</a>
  </div>
</form>

<div class="mt">
  <Editor
    value={body_md}
    onsave={(_v) => { body_md = _v; save(); }}
    oncancel={() => goto(`/resources/${rid}`)}
    onsaveAndClose={(_v) => { body_md = _v; saveAndClose(); }}
    onchange={(v) => { body_md = v; }}
  />
</div>
