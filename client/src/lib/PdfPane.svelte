<script lang="ts">
  import { api, type ResourceDto, type ReadingContentDto } from '$lib/api';

  interface Props { resource: ResourceDto }
  let { resource }: Props = $props();

  let content = $state<ReadingContentDto | null>(null);
  let loading = $state(true);
  let error = $state('');

  async function load() {
    loading = true; error = '';
    try {
      content = await api.getContent(Number(resource.id));
    } catch (e: any) {
      error = e.message;
    } finally { loading = false; }
  }

  $effect(() => { if (resource.id) load(); });

  let isPdf = $derived(Boolean(resource.file_path && resource.file_path.toLowerCase().endsWith('.pdf')));
</script>

{#if loading}
  <p class="dim">loading…</p>
{:else if error}
  <p class="flash err">{error}</p>
{:else if content?.status === 'ok' && content.content_html}
  <div class="extracted">
    {@html content.content_html}
  </div>
{:else if isPdf && resource.file_path}
  <iframe
    class="pdf-iframe"
    src={api.fileUrl(Number(resource.id))}
    title="pdf viewer"
  ></iframe>
{:else if resource.url}
  <p class="dim">no extracted content yet.</p>
  <p><a href={resource.url} target="_blank" rel="noreferrer">open {resource.url}</a></p>
{:else}
  <p class="dim">nothing to show. add a url or upload a file.</p>
{/if}

<style>
  .pdf-iframe { width: 100%; height: 600px; border: 0; background: #fff; }
  .extracted { max-width: none; }
</style>
