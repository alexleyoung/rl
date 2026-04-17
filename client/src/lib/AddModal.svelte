<script lang="ts">
  import { goto } from '$app/navigation';
  import { api, type ResourceInputDto, type MetadataDto } from '$lib/api';

  interface Props { open: boolean }
  let { open = $bindable(false) }: Props = $props();

  let url = $state('');
  let title = $state('');
  let author = $state('');
  let kind = $state('paper');
  let status = $state('inbox');
  let tagStr = $state('');
  let filePath = $state('');
  let dragover = $state(false);
  let extracting = $state(false);
  let error = $state('');
  let fileInput: HTMLInputElement;

  function reset() {
    url = ''; title = ''; author = ''; kind = 'paper'; status = 'inbox';
    tagStr = ''; filePath = ''; error = ''; extracting = false;
  }

  $effect(() => { if (open) { setTimeout(() => (document.querySelector('.modal input[type=url]') as HTMLInputElement | null)?.focus(), 20); } });

  function inferKind(u: string): string {
    const s = u.toLowerCase();
    if (s.includes('github.com') || s.includes('gitlab.com')) return 'repo';
    if (s.includes('arxiv.org') || s.endsWith('.pdf')) return 'paper';
    return 'article';
  }

  async function extractFromUrl() {
    if (!url.trim()) return;
    extracting = true; error = '';
    try {
      const meta: MetadataDto = await api.extractMeta({ url: url.trim() });
      if (meta.title && !title) title = meta.title;
      if (meta.author && !author) author = meta.author;
      kind = inferKind(url);
    } catch (e: any) {
      error = e.message;
    } finally {
      extracting = false;
    }
  }

  async function uploadFile(file: File) {
    extracting = true; error = '';
    try {
      const res = await api.uploadFile(file);
      filePath = res.path;
      const meta: MetadataDto = await api.extractMeta({ file_path: res.path });
      if (meta.title && !title) title = meta.title;
      if (meta.author && !author) author = meta.author;
      if (!kind) kind = 'paper';
    } catch (e: any) {
      error = e.message;
    } finally {
      extracting = false;
    }
  }

  function onDrop(e: DragEvent) {
    e.preventDefault();
    dragover = false;
    const f = e.dataTransfer?.files?.[0];
    if (f) uploadFile(f);
  }

  function onPick(e: Event) {
    const f = (e.currentTarget as HTMLInputElement).files?.[0];
    if (f) uploadFile(f);
  }

  async function submit(e: Event) {
    e.preventDefault();
    if (!title.trim()) { error = 'title required'; return; }
    const tags = tagStr.split(',').map(t => t.trim().replace(/^#/, '')).filter(Boolean);
    const body: ResourceInputDto = {
      kind, title: title.trim(), author: author.trim() || null,
      url: url.trim() || null, file_path: filePath || null,
      status, tags,
    };
    try {
      const r = await api.createResource(body);
      open = false;
      reset();
      goto(`/resources/${r.id}`);
    } catch (ex: any) { error = ex.message; }
  }

  function overlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) { open = false; reset(); }
  }
</script>

{#if open}
  <div class="modal" onclick={overlayClick} role="presentation">
    <form class="card" onsubmit={submit}>
      <h3>add a resource</h3>
      <div class="sub">paste a url, drop a pdf, or fill in by hand.</div>

      <input bind:this={fileInput} type="file" style="display:none" onchange={onPick} />
      <div
        class="drop"
        class:over={dragover}
        role="button"
        tabindex="0"
        onclick={() => fileInput.click()}
        onkeydown={(e) => { if (e.key === 'Enter') fileInput.click(); }}
        ondragover={(e) => { e.preventDefault(); dragover = true; }}
        ondragleave={() => dragover = false}
        ondrop={onDrop}
      >
        {#if extracting}parsing…
        {:else if filePath}<b>{filePath.split('/').pop()}</b>
        {:else}drop pdf · or <b>click to browse</b>{/if}
      </div>

      <div class="field">
        <label>paste url</label>
        <input type="url" bind:value={url} placeholder="https://arxiv.org/abs/1706.03762" onblur={extractFromUrl} />
      </div>

      <div class="sep"></div>

      <div class="field"><label>title</label><input type="text" bind:value={title} placeholder="paxos made simple" /></div>
      <div class="field"><label>author</label><input type="text" bind:value={author} /></div>
      <div class="row">
        <div class="field">
          <label>type</label>
          <select bind:value={kind}>
            <option>paper</option><option>book</option><option>article</option><option>blog</option><option>repo</option>
          </select>
        </div>
        <div class="field">
          <label>status</label>
          <select bind:value={status}>
            <option>inbox</option><option>queue</option><option>reading</option><option>done</option>
          </select>
        </div>
      </div>
      <div class="field"><label>tags</label><input type="text" bind:value={tagStr} placeholder="#consensus, #distsys" /></div>

      {#if error}<p class="flash err">{error}</p>{/if}

      <div class="foot">
        <button type="button" class="btn" onclick={() => { open = false; reset(); }}>cancel</button>
        <button type="submit" class="btn primary">add to {status}</button>
      </div>
    </form>
  </div>
{/if}
