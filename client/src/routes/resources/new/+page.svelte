<script lang="ts">
  import { goto } from '$app/navigation';
  import { api, type ResourceInputDto, type MetadataDto } from '$lib/api';
  import FileDropInput from '$lib/FileDropInput.svelte';
  import ResourceForm from '$lib/ResourceForm.svelte';
  import TagsInput from '$lib/TagsInput.svelte';

  // ── Step state ──────────────────────────────────────────────────────────────
  // 'input'   → drop zone / URL entry
  // 'review'  → pre-filled form ready to save
  let step = $state<'input' | 'review'>('input');

  // ── Input step state ────────────────────────────────────────────────────────
  let urlInput = $state('');
  let filePath = $state('');
  let extracting = $state(false);
  let extractError = $state('');

  // ── Review step state ───────────────────────────────────────────────────────
  // Holds the pre-filled ResourceInputDto; passed as `initial` to ResourceForm
  // via a thin adapter so we don't need to change ResourceForm's interface.
  let prefilled = $state<ResourceInputDto | null>(null);

  // ResourceForm expects a ResourceDto for `initial`, but we only have a plain
  // ResourceInputDto here. Fake the required extra fields so the $effect fires.
  let fakeInitial = $derived(prefilled ? {
    id: 0 as unknown as bigint,
    added_at: 0 as unknown as bigint,
    last_read_at: null,
    ...prefilled,
  } : null);

  // ── Extraction ──────────────────────────────────────────────────────────────
  async function runExtract(input: { file_path?: string; url?: string }) {
    extracting = true;
    extractError = '';
    try {
      const meta: MetadataDto = await api.extractMeta(input);
      // Infer kind from context
      const kind = input.file_path ? 'book' : inferKind(input.url ?? '');
      prefilled = {
        kind,
        title: meta.title ?? '',
        author: meta.author ?? undefined,
        url: input.url ?? undefined,
        file_path: input.file_path ?? undefined,
        tags: [],
      };
      step = 'review';
    } catch (e: any) {
      extractError = e.message;
    } finally {
      extracting = false;
    }
  }

  function inferKind(url: string): string {
    const u = url.toLowerCase();
    if (u.includes('github.com') || u.includes('gitlab.com')) return 'repo';
    if (u.includes('arxiv.org') || u.includes('/paper') || u.includes('.pdf')) return 'paper';
    return 'article';
  }

  function onFileDrop(path: string) {
    filePath = path;
    if (path) runExtract({ file_path: path });
  }

  function submitUrl() {
    const u = urlInput.trim();
    if (u) runExtract({ url: u });
  }

  // ── Save ────────────────────────────────────────────────────────────────────
  async function create(data: ResourceInputDto) {
    const r = await api.createResource(data);
    goto(`/resources/${r.id}`);
  }

  function reset() {
    step = 'input';
    urlInput = '';
    filePath = '';
    prefilled = null;
    extractError = '';
  }
</script>

<h1>add resource</h1>

{#if step === 'input'}
  <div class="add-input">
    <div class="drop-section">
      <p class="dim small">drop a file (PDF, epub, …) or paste a URL to get started</p>
      <FileDropInput value={filePath} onchange={onFileDrop} placeholder="drag file here or click to browse" />
    </div>

    <div class="divider"><span>or</span></div>

    <form class="url-section" onsubmit={(e) => { e.preventDefault(); submitUrl(); }}>
      <input
        type="url"
        bind:value={urlInput}
        placeholder="https://…"
        disabled={extracting}
      />
      <button type="submit" disabled={extracting || !urlInput.trim()}>
        {extracting ? 'parsing…' : 'go'}
      </button>
    </form>

    {#if extractError}
      <p class="flash err">{extractError}</p>
    {/if}

    <div class="manual-link">
      <button type="button" class="dim-link" onclick={() => {
        prefilled = { kind: 'book', title: '', tags: [] };
        step = 'review';
      }}>enter details manually</button>
    </div>
  </div>

{:else}
  <button type="button" class="back-btn dim-link" onclick={reset}>← back</button>
  <ResourceForm initial={fakeInitial as any} onsubmit={create} submitLabel="add" />
{/if}

<style>
  .add-input {
    max-width: 480px;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .drop-section p { margin: 0 0 0.4rem; }

  .divider {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    color: var(--dim);
    font-size: 0.85rem;
  }
  .divider::before,
  .divider::after {
    content: '';
    flex: 1;
    height: 1px;
    background: var(--border);
  }

  .url-section {
    display: flex;
    gap: 0.5rem;
  }
  .url-section input { flex: 1; }

  .manual-link { font-size: 0.85rem; }

  .dim-link {
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    color: var(--dim);
    cursor: pointer;
    text-decoration: underline;
    text-underline-offset: 2px;
  }
  .dim-link:hover { color: var(--fg); }

  .back-btn { margin-bottom: 1rem; display: block; }
</style>
