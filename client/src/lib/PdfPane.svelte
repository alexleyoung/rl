<script lang="ts">
  import { tick } from 'svelte';
  import { api, type ResourceDto } from '$lib/api';

  interface Props {
    resource: ResourceDto;
    scroll?: (delta: number) => void;
  }
  let { resource, scroll = $bindable() }: Props = $props();

  type Meta  = { status: string; source_type: string; word_count: number; total_blocks: number };
  type Chunk = { offset: number; nextOffset: number; html: string };

  const WINDOW = 5;
  const NEAR   = 600;

  let meta        = $state<Meta | null>(null);
  let chunks      = $state<Chunk[]>([]);
  let totalBlocks = $state(0);
  let topPad      = $state(0);
  let botPad      = $state(0);
  let loading     = $state(true);
  let fetching    = $state(false);
  let error       = $state('');
  let currentId: number | null = null;

  // Offsets of chunks dropped off the top, in order (oldest first).
  // Used to re-request the exact chunk when scrolling back up.
  let droppedHeadOffsets: number[] = [];

  let wrapEl:   HTMLDivElement | null = $state(null);
  let scroller: HTMLElement    | null = null;

  // Map from chunk offset → rendered div element.
  const chunkDivs = new Map<number, HTMLDivElement>();

  function trackChunk(el: HTMLDivElement, offset: number) {
    chunkDivs.set(offset, el);
    return { destroy: () => chunkDivs.delete(offset) };
  }

  async function loadFirst() {
    const id = Number(resource.id);
    currentId = id;
    loading = true; error = '';
    meta = null; chunks = [];
    topPad = 0; botPad = 0;
    droppedHeadOffsets = [];
    chunkDivs.clear();

    try {
      const c = await api.getContentChunk(id, 0);
      if (currentId !== id || !c) return;
      meta = { status: c.status, source_type: c.source_type, word_count: c.word_count, total_blocks: c.total_blocks };
      totalBlocks = c.total_blocks;
      if (c.html) chunks = [{ offset: 0, nextOffset: c.next_offset, html: c.html }];
    } catch (e: any) { error = e.message; }
    finally {
      if (currentId === id) loading = false;
      queueMicrotask(fillViewport);
    }
  }

  async function loadForward() {
    const tail = chunks.length ? chunks[chunks.length - 1].nextOffset : 0;
    if (fetching || tail >= totalBlocks) return;
    const id = currentId; if (id == null) return;
    fetching = true;
    try {
      const c = await api.getContentChunk(id, tail);
      if (currentId !== id || !c || !c.html) return;
      totalBlocks = c.total_blocks;
      const incoming: Chunk = { offset: tail, nextOffset: c.next_offset, html: c.html };

      if (chunks.length >= WINDOW) {
        const head = chunks[0];
        topPad += chunkDivs.get(head.offset)?.offsetHeight ?? 0;
        droppedHeadOffsets.push(head.offset);
        chunks = [...chunks.slice(1), incoming];
      } else {
        chunks = [...chunks, incoming];
      }
    } catch (e: any) { error = e.message; }
    finally {
      if (currentId === id) fetching = false;
      queueMicrotask(fillViewport);
    }
  }

  async function loadBackward() {
    if (fetching || droppedHeadOffsets.length === 0) return;
    const id = currentId; if (id == null) return;
    fetching = true;
    try {
      const reqOffset = droppedHeadOffsets[droppedHeadOffsets.length - 1];
      const c = await api.getContentChunk(id, reqOffset);
      if (currentId !== id || !c || !c.html) return;
      totalBlocks = c.total_blocks;
      const incoming: Chunk = { offset: reqOffset, nextOffset: c.next_offset, html: c.html };

      const prevHeight = scroller?.scrollHeight ?? 0;
      const prevTop    = scroller?.scrollTop    ?? 0;

      if (chunks.length >= WINDOW) {
        const tail = chunks[chunks.length - 1];
        botPad += chunkDivs.get(tail.offset)?.offsetHeight ?? 0;
        chunks = [incoming, ...chunks.slice(0, -1)];
      } else {
        chunks = [incoming, ...chunks];
      }
      droppedHeadOffsets.pop();

      // After Svelte inserts the new head chunk, restore scroll so viewport doesn't jump.
      await tick();
      if (scroller) {
        const added = scroller.scrollHeight - prevHeight;
        scroller.scrollTop = prevTop + added;
        // Deduct the real rendered height of the restored chunk from the spacer.
        const realH = chunkDivs.get(reqOffset)?.offsetHeight ?? 0;
        topPad = Math.max(0, topPad - realH);
      }
    } catch (e: any) { error = e.message; }
    finally {
      if (currentId === id) fetching = false;
    }
  }

  function onScroll() {
    if (!scroller || fetching) return;
    const distBottom = scroller.scrollHeight - scroller.scrollTop - scroller.clientHeight;
    const distTop    = scroller.scrollTop;
    if (distBottom < NEAR) loadForward();
    if (distTop < NEAR && droppedHeadOffsets.length > 0) loadBackward();
  }

  function fillViewport() {
    if (!scroller || fetching) return;
    if (scroller.scrollHeight <= scroller.clientHeight + NEAR) loadForward();
  }

  $effect(() => { if (resource.id) loadFirst(); });

  $effect(() => {
    if (!wrapEl) return;
    const s = wrapEl.closest('.pane-body') as HTMLElement | null;
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
{:else if meta?.status === 'ok' && chunks.length > 0}
  <div class="extracted" bind:this={wrapEl}>
    {#if topPad > 0}<div style="height:{topPad}px"></div>{/if}
    {#each chunks as chunk (chunk.offset)}
      <div use:trackChunk={chunk.offset}>
        {@html chunk.html}
      </div>
    {/each}
    {#if botPad > 0}<div style="height:{botPad}px"></div>{/if}
    {#if fetching}<p class="dim fetching">loading…</p>{/if}
  </div>
{:else if isPdf && resource.file_path}
  <div class="iframe-wrap">
    <iframe
      class="pdf-iframe"
      src={api.fileUrl(Number(resource.id))}
      title="pdf viewer"
    ></iframe>
  </div>
{:else if resource.url}
  <p class="dim">no extracted content yet.</p>
  <p><a href={resource.url} target="_blank" rel="noreferrer">open {resource.url}</a></p>
{:else}
  <p class="dim">nothing to show. add a url or upload a file.</p>
{/if}

<style>
  .iframe-wrap { position: absolute; inset: 0; display: flex; flex-direction: column; }
  .pdf-iframe  { width: 100%; flex: 1; border: 0; display: block; }
  .extracted   { max-width: none; }
  .fetching    { text-align: center; padding: 8px 0; }
</style>
