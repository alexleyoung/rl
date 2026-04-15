<script lang="ts">
  import { api } from '$lib/api';

  interface Props {
    value?: string;           // bound file_path string
    onchange?: (path: string) => void;
    placeholder?: string;
    fileUrl?: string;         // if set, filename becomes a link to open the file
  }

  let {
    value = $bindable(''),
    onchange,
    placeholder = 'drag file here or click to browse',
    fileUrl,
  }: Props = $props();

  let uploading = $state(false);
  let error = $state('');
  let dragover = $state(false);
  let fileInput: HTMLInputElement;

  async function handleFile(file: File) {
    uploading = true;
    error = '';
    try {
      const res = await api.uploadFile(file);
      value = res.path;
      onchange?.(res.path);
    } catch (e: any) {
      error = e.message;
    } finally {
      uploading = false;
    }
  }

  function onDrop(e: DragEvent) {
    e.preventDefault();
    dragover = false;
    const file = e.dataTransfer?.files?.[0];
    if (file) handleFile(file);
  }

  function onPickerChange(e: Event) {
    const file = (e.currentTarget as HTMLInputElement).files?.[0];
    if (file) handleFile(file);
  }

  // Basename for display
  function basename(p: string) {
    return p.split('/').pop() ?? p;
  }
</script>

<!-- Hidden real file input -->
<input
  bind:this={fileInput}
  type="file"
  style="display:none"
  onchange={onPickerChange}
/>

<div
  class="drop-zone"
  class:dragover
  class:has-value={!!value}
  role="button"
  tabindex="0"
  onclick={() => !value && fileInput.click()}
  onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') fileInput.click(); }}
  ondragover={(e) => { e.preventDefault(); dragover = true; }}
  ondragleave={() => { dragover = false; }}
  ondrop={onDrop}
>
  {#if uploading}
    <span class="dim">uploading…</span>
  {:else if value}
    {#if fileUrl}
      <a class="file-link" href={fileUrl} target="_blank" onclick={(e) => e.stopPropagation()}>{basename(value)}</a>
    {:else}
      <span class="path" title={value}>{basename(value)}</span>
    {/if}
    <span class="file-actions">
      <button class="action-btn" type="button" title="replace file" onclick={(e) => { e.stopPropagation(); fileInput.click(); }}>replace</button>
      <button class="action-btn danger-btn" type="button" title="clear" onclick={(e) => { e.stopPropagation(); value = ''; onchange?.(''); }}>✕</button>
    </span>
  {:else}
    <span class="prompt">{placeholder}</span>
  {/if}
</div>

{#if error}<p class="drop-error">{error}</p>{/if}

<style>
  .drop-zone {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-height: 2.2rem;
    padding: 0.35rem 0.6rem;
    border: 1px solid var(--border);
    cursor: pointer;
    background: var(--bg);
    font: inherit;
    font-size: 0.9rem;
    transition: border-color 0.1s, background 0.1s;
    outline: none;
    width: 100%;
    box-sizing: border-box;
  }
  .drop-zone:focus { border-color: var(--accent); }
  .drop-zone.has-value { cursor: default; }
  .drop-zone.dragover {
    border-color: var(--accent);
    background: color-mix(in srgb, var(--accent) 6%, var(--bg));
    cursor: copy;
  }
  .drop-zone .prompt { color: var(--dim); }
  .file-link, .path {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.85rem;
  }
  .file-link { color: var(--accent); }
  .file-actions {
    display: flex;
    gap: 0.3rem;
    flex-shrink: 0;
    opacity: 0;
    transition: opacity 0.1s;
  }
  .drop-zone:hover .file-actions { opacity: 1; }
  .action-btn {
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    font-size: 0.8rem;
    color: var(--dim);
    cursor: pointer;
    text-decoration: underline;
    text-underline-offset: 2px;
  }
  .danger-btn:hover { color: var(--danger); }
  .drop-error { color: var(--danger); font-size: 0.8rem; margin: 0.2rem 0 0; }
</style>
