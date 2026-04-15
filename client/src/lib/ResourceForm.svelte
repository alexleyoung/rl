<script lang="ts">
  import type { ResourceDto, ResourceInputDto } from '$lib/api';
  import FileDropInput from '$lib/FileDropInput.svelte';

  interface Props {
    initial?: ResourceDto | null;
    onsubmit: (data: ResourceInputDto) => Promise<void>;
    submitLabel?: string;
  }

  let { initial = null, onsubmit, submitLabel = 'save' }: Props = $props();

  let kind = $state('book');
  let title = $state('');
  let author = $state('');
  let url = $state('');
  let file_path = $state('');
  let tagsStr = $state('');
  let error = $state('');
  let saving = $state(false);

  // Populate from initial when it arrives (async load from parent)
  $effect(() => {
    if (initial) {
      kind = initial.kind;
      title = initial.title;
      author = initial.author ?? '';
      url = initial.url ?? '';
      file_path = initial.file_path ?? '';
      tagsStr = (initial.tags ?? []).join(', ');
    }
  });

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!title.trim()) { error = 'title required'; return; }
    error = '';
    saving = true;
    try {
      const tags = tagsStr.split(',').map(s => s.trim()).filter(Boolean);
      await onsubmit({
        kind,
        title: title.trim(),
        author: author.trim() || undefined,
        url: url.trim() || undefined,
        file_path: file_path.trim() || undefined,
        tags,
      });
    } catch (e: any) {
      error = e.message;
    } finally {
      saving = false;
    }
  }
</script>

{#if error}<p class="flash err">{error}</p>{/if}

<form onsubmit={handleSubmit}>
  <div>
    <label for="kind">kind</label>
    <select id="kind" bind:value={kind}>
      <option>book</option>
      <option>paper</option>
      <option>article</option>
      <option>blog</option>
      <option>repo</option>
    </select>
  </div>
  <div>
    <label for="title">title *</label>
    <input id="title" type="text" bind:value={title} />
  </div>
  <div>
    <label for="author">author</label>
    <input id="author" type="text" bind:value={author} />
  </div>
  <div>
    <label for="url">url</label>
    <input id="url" type="url" bind:value={url} />
  </div>
  <div>
    <label for="file_path">file</label>
    <FileDropInput bind:value={file_path} />
  </div>
  <div>
    <label for="tags">tags (comma-separated)</label>
    <input id="tags" type="text" bind:value={tagsStr} placeholder="algorithms, systems" />
  </div>
  <div class="row-actions">
    <button type="submit" class="primary" disabled={saving}>{saving ? 'saving…' : submitLabel}</button>
    <button type="button" onclick={() => history.back()}>cancel</button>
  </div>
</form>
