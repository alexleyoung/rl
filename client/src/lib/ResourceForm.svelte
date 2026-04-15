<script lang="ts">
  import { api, type ResourceDto, type ResourceInputDto } from '$lib/api';
  import FileDropInput from '$lib/FileDropInput.svelte';
  import TagsInput from '$lib/TagsInput.svelte';

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
  let tags = $state<string[]>([]);
  let error = $state('');
  let saving = $state(false);
  let extracting = $state(false);

  // Populate from initial when it arrives (async load from parent)
  $effect(() => {
    if (initial) {
      kind = initial.kind;
      title = initial.title;
      author = initial.author ?? '';
      url = initial.url ?? '';
      file_path = initial.file_path ?? '';
      tags = initial.tags ?? [];
    }
  });

  // Apply extracted metadata — only fill fields the user hasn't typed yet
  async function applyMeta(input: { file_path?: string; url?: string }) {
    extracting = true;
    try {
      const meta = await api.extractMeta(input);
      if (meta.title && !title.trim()) title = meta.title;
      if (meta.author && !author.trim()) author = meta.author;
    } catch {
      // extraction failure is silent — fields just stay empty
    } finally {
      extracting = false;
    }
  }

  function onFilePath(path: string) {
    file_path = path;
    if (path) applyMeta({ file_path: path });
  }

  async function onUrlBlur() {
    const u = url.trim();
    if (u) applyMeta({ url: u });
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!title.trim()) { error = 'title required'; return; }
    error = '';
    saving = true;
    try {
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
    <label for="title">title *{#if extracting}<span class="extracting"> extracting…</span>{/if}</label>
    <input id="title" type="text" bind:value={title} />
  </div>
  <div>
    <label for="author">author</label>
    <input id="author" type="text" bind:value={author} />
  </div>
  <div>
    <label for="url">url</label>
    <input id="url" type="url" bind:value={url} onblur={onUrlBlur} />
  </div>
  <div>
    <label for="file_path">file</label>
    <FileDropInput bind:value={file_path} onchange={onFilePath} />
  </div>
  <div>
    <label>tags</label>
    <TagsInput bind:tags />
  </div>
  <div class="row-actions">
    <button type="submit" class="primary" disabled={saving}>{saving ? 'saving…' : submitLabel}</button>
    <button type="button" onclick={() => history.back()}>cancel</button>
  </div>
</form>

<style>
  .extracting {
    font-weight: normal;
    color: var(--dim);
    font-size: 0.8em;
  }
</style>
