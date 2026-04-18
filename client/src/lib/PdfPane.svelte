<script lang="ts">
  import { api, type ResourceDto, type ReadingContentChunkDto } from '$lib/api';

  interface Props {
    resource: ResourceDto;
    scroll?: (delta: number) => void;
  }
  let { resource, scroll = $bindable() }: Props = $props();

  type Meta = {
    status: string;
    source_type: string;
    word_count: number;
    total_blocks: number;
  };

  let meta = $state<Meta | null>(null);
  let htmlParts = $state<string[]>([]);
  let nextOffset = $state(0);
  let hasMore = $state(false);
  let loading = $state(true);
  let fetching = $state(false);
  let error = $state('');
  let currentId: number | null = null;
  let extractedEl: HTMLDivElement | null = $state(null);
  let scroller: HTMLElement | null = null;

  async function loadFirst() {
    const id = Number(resource.id);
    currentId = id;
    loading = true;
    error = '';
    meta = null;
    htmlParts = [];
    nextOffset = 0;
    hasMore = false;
    try {
      const c = await api.getContentChunk(id, 0);
      if (currentId !== id) return;
      if (c) {
        meta = {
          status: c.status,
          source_type: c.source_type,
          word_count: c.word_count,
          total_blocks: c.total_blocks,
        };
        htmlParts = c.html ? [c.html] : [];
        nextOffset = c.next_offset;
        hasMore = c.has_more;
      }
    } catch (e: any) {
      error = e.message;
    } finally {
      if (currentId === id) loading = false;
      queueMicrotask(fillViewport);
    }
  }

  async function loadMore() {
    if (fetching || !hasMore) return;
    const id = currentId;
    if (id == null) return;
    fetching = true;
    try {
      const c = await api.getContentChunk(id, nextOffset);
      if (currentId !== id || !c) return;
      if (c.html) htmlParts = [...htmlParts, c.html];
      nextOffset = c.next_offset;
      hasMore = c.has_more;
    } catch (e: any) {
      error = e.message;
    } finally {
      if (currentId === id) fetching = false;
      queueMicrotask(fillViewport);
    }
  }

  function onScroll() {
    if (!scroller || !hasMore || fetching) return;
    const dist = scroller.scrollHeight - scroller.scrollTop - scroller.clientHeight;
    if (dist < 500) loadMore();
  }

  function fillViewport() {
    if (!scroller || !hasMore || fetching) return;
    if (scroller.scrollHeight <= scroller.clientHeight + 500) loadMore();
  }

  $effect(() => { if (resource.id) loadFirst(); });

  $effect(() => {
    if (!extractedEl) return;
    const s = extractedEl.closest('.pane-body') as HTMLElement | null;
    scroller = s;
    s?.addEventListener('scroll', onScroll, { passive: true });
    scroll = (delta: number) => s?.scrollBy({ top: delta, behavior: 'smooth' });
    return () => {
      s?.removeEventListener('scroll', onScroll);
      scroll = undefined;
    };
  });

  let isPdf = $derived(Boolean(resource.file_path && resource.file_path.toLowerCase().endsWith('.pdf')));
</script>

{#if loading}
  <p class="dim">loading…</p>
{:else if error}
  <p class="flash err">{error}</p>
{:else if meta?.status === 'ok' && htmlParts.length > 0}
  <div class="extracted" bind:this={extractedEl}>
    {#each htmlParts as part}{@html part}{/each}
    {#if fetching}<p class="dim">loading more…</p>{/if}
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
  :global(.pdf-body) { padding: 0; display: flex; flex-direction: column; }
  .pdf-iframe { width: 100%; flex: 1; min-height: 400px; border: 0; display: block; }
  .extracted { max-width: none; padding: 14px 16px; }
</style>
