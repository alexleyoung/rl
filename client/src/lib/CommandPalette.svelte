<script lang="ts">
  import { goto } from '$app/navigation';
  import { api, type SearchHitDto, type TagDto } from '$lib/api';

  type Scope = 'all' | 'name' | 'notes' | 'tags' | 'actions';

  interface Props { open: boolean }
  let { open = $bindable(false) }: Props = $props();

  let q = $state('');
  let scope = $state<Scope>('all');
  let hits = $state<SearchHitDto[]>([]);
  let tags = $state<TagDto[]>([]);
  let sel = $state(0);
  let input: HTMLInputElement;
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  const SCOPES: Scope[] = ['all', 'name', 'notes', 'tags', 'actions'];

  interface Action { label: string; kbd?: string; run: () => void }
  const actions: Action[] = [
    { label: 'add new resource…', kbd: '⌘N', run: () => { close();
      setTimeout(() => document.dispatchEvent(new KeyboardEvent('keydown', { key: 'n', metaKey: true })), 0); } },
    { label: 'go to inbox',    run: () => { close(); goto('/?status=inbox'); } },
    { label: 'go to reading',  run: () => { close(); goto('/?status=reading'); } },
    { label: 'go to queue',    run: () => { close(); goto('/?status=queue'); } },
    { label: 'go to done',     run: () => { close(); goto('/?status=done'); } },
    { label: 'settings',       run: () => { close(); goto('/settings/keymap'); } },
  ];

  function close() { open = false; }

  async function runSearch() {
    if (!q.trim()) { hits = []; return; }
    try {
      const res = await api.search(q.trim(), 20);
      hits = res.hits;
    } catch { hits = []; }
  }

  async function runTags() {
    try { tags = await api.listTags(); } catch { tags = []; }
  }

  $effect(() => {
    if (!open) return;
    runTags();
    sel = 0;
    setTimeout(() => input?.select(), 20);
  });

  $effect(() => {
    const _ = q;
    sel = 0;
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(runSearch, 120);
  });

  let resourceHits = $derived(hits.filter(h => h.source_kind === 'resource'));
  let noteHits     = $derived(hits.filter(h => h.source_kind === 'note'));
  let tagHits      = $derived(q.trim() ? tags.filter(t => t.name.toLowerCase().includes(q.trim().toLowerCase())) : []);
  let actionHits   = $derived(q.trim() ? actions.filter(a => a.label.toLowerCase().includes(q.trim().toLowerCase())) : actions);

  type Section =
    | { kind: 'resource'; head: string; rows: { idx: number; h: SearchHitDto }[] }
    | { kind: 'note';     head: string; rows: { idx: number; h: SearchHitDto }[] }
    | { kind: 'tag';      head: string; rows: { idx: number; t: TagDto }[] }
    | { kind: 'action';   head: string; rows: { idx: number; a: Action }[] };

  // Build sections with stable flat indices
  let sections = $derived.by(() => {
    const sects: Section[] = [];
    let idx = 0;

    if ((scope === 'all' || scope === 'name') && resourceHits.length > 0) {
      sects.push({ kind: 'resource', head: `resources · ${resourceHits.length}`,
        rows: resourceHits.map(h => ({ idx: idx++, h })) });
    }
    if ((scope === 'all' || scope === 'notes') && noteHits.length > 0) {
      sects.push({ kind: 'note', head: `in notes · ${noteHits.length}`,
        rows: noteHits.map(h => ({ idx: idx++, h })) });
    }
    if ((scope === 'all' || scope === 'tags') && tagHits.length > 0) {
      sects.push({ kind: 'tag', head: `tags · ${tagHits.length}`,
        rows: tagHits.map(t => ({ idx: idx++, t })) });
    }
    if ((scope === 'all' || scope === 'actions') && actionHits.length > 0) {
      sects.push({ kind: 'action', head: 'actions',
        rows: actionHits.map(a => ({ idx: idx++, a })) });
    }
    return sects;
  });

  let totalItems = $derived(sections.reduce((n, s) => n + s.rows.length, 0));

  // Flat run-function lookup by index
  function run(i: number) {
    let offset = 0;
    for (const s of sections) {
      if (i < offset + s.rows.length) {
        const row = s.rows[i - offset];
        if (s.kind === 'resource') { close(); goto(`/resources/${(row as any).h.resource_id}`); }
        else if (s.kind === 'note')   { close(); goto(`/resources/${(row as any).h.resource_id}`); }
        else if (s.kind === 'tag')    { close(); goto(`/t/${encodeURIComponent((row as any).t.name)}`); }
        else if (s.kind === 'action') { (row as any).a.run(); }
        return;
      }
      offset += s.rows.length;
    }
  }

  function moveDown() { sel = totalItems > 0 ? Math.min(totalItems - 1, sel + 1) : 0; }
  function moveUp()   { sel = Math.max(0, sel - 1); }

  function onKey(e: KeyboardEvent) {
    if (!open) return;
    if (e.key === 'Escape') { e.preventDefault(); close(); return; }
    if (e.key === 'ArrowDown' || (e.ctrlKey && e.key === 'n')) { e.preventDefault(); moveDown(); return; }
    if (e.key === 'ArrowUp'   || (e.ctrlKey && e.key === 'p')) { e.preventDefault(); moveUp();   return; }
    if (e.key === 'Enter')     { e.preventDefault(); run(sel); return; }
    if (e.key === 'Tab')       {
      e.preventDefault();
      const i = SCOPES.indexOf(scope);
      scope = SCOPES[(i + (e.shiftKey ? -1 : 1) + SCOPES.length) % SCOPES.length];
      sel = 0;
    }
  }

  function onOverlayClick() { close(); }
</script>

<svelte:window onkeydown={onKey} />

{#if open}
  <div class="overlay" onclick={onOverlayClick} role="presentation"></div>
  <div class="palette" role="dialog" aria-modal="true">
    <div class="palette-head">
      <span class="muted">&gt;</span>
      <input bind:this={input} bind:value={q} type="text" placeholder="search everything…" autofocus />
    </div>
    <div class="palette-scopes">
      {#each SCOPES as s}
        <button class="s" class:on={scope === s} onclick={() => { scope = s; sel = 0; }}>{s}</button>
      {/each}
    </div>
    <div class="palette-body">
      {#each sections as section}
        <div class="p-section-head">{section.head}</div>
        {#each section.rows as row}
          <button
            class="p-row"
            class:sel={sel === row.idx}
            onmouseenter={() => sel = row.idx}
            onclick={() => run(row.idx)}
          >
            {#if section.kind === 'resource'}
              <span class="ty">resource</span>
              <span>{(row as any).h.title}</span>
              <span class="rt">{(row as any).h.snippet.slice(0, 40)}</span>
            {:else if section.kind === 'note'}
              <span class="ty">note</span>
              <span class="snip">{(row as any).h.snippet}</span>
              <span class="rt">{(row as any).h.title}</span>
            {:else if section.kind === 'tag'}
              <span class="ty">tag</span>
              <span>#{(row as any).t.name}</span>
              <span class="rt">{(row as any).t.count} items</span>
            {:else if section.kind === 'action'}
              <span class="ty">do</span>
              <span>{(row as any).a.label}</span>
              <span class="rt">{#if (row as any).a.kbd}<span class="kbd">{(row as any).a.kbd}</span>{/if}</span>
            {/if}
          </button>
        {/each}
      {/each}

      {#if totalItems === 0}
        <div class="p-section-head" style="padding:18px 12px">no results.</div>
      {/if}
    </div>
    <div class="palette-foot">
      <span><span class="kbd">↑↓</span> <span class="kbd">⌃n</span><span class="kbd">⌃p</span> navigate</span>
      <span><span class="kbd">↵</span> open</span>
      <span><span class="kbd">tab</span> cycle scope</span>
      <span class="right-ml"><span class="kbd">esc</span> close</span>
    </div>
  </div>
{/if}

<style>
  .p-row { background: none; border: none; width: 100%; text-align: left; font: inherit; }
</style>
