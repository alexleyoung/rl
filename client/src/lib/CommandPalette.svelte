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

  // Predefined actions shown in the "actions" section
  interface Action { label: string; kbd?: string; run: () => void }
  const actions: Action[] = [
    { label: 'add new resource…', kbd: '⌘N', run: () => { close(); /* layout listens for ⌘N */
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
    } catch {
      hits = [];
    }
  }

  async function runTags() {
    try {
      tags = await api.listTags();
    } catch { tags = []; }
  }

  $effect(() => {
    if (!open) return;
    runTags();
    // reset on open
    sel = 0;
    setTimeout(() => input?.select(), 20);
  });

  $effect(() => {
    const _ = q;
    sel = 0;
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(runSearch, 120);
  });

  // Filtered sections
  let resourceHits = $derived(hits.filter(h => h.source_kind === 'resource'));
  let noteHits     = $derived(hits.filter(h => h.source_kind === 'note'));
  let tagHits      = $derived(
    q.trim()
      ? tags.filter(t => t.name.toLowerCase().includes(q.trim().toLowerCase()))
      : []
  );
  let actionHits   = $derived(
    q.trim()
      ? actions.filter(a => a.label.toLowerCase().includes(q.trim().toLowerCase()))
      : actions
  );

  interface FlatItem { kind: 'resource' | 'note' | 'tag' | 'action'; run: () => void; el: any }
  let items = $derived.by<FlatItem[]>(() => {
    const arr: FlatItem[] = [];
    if (scope === 'all' || scope === 'name') {
      for (const h of resourceHits) arr.push({ kind: 'resource', run: () => { close(); goto(`/resources/${h.resource_id}`); }, el: h });
    }
    if (scope === 'all' || scope === 'notes') {
      for (const h of noteHits) arr.push({ kind: 'note', run: () => { close(); goto(`/resources/${h.resource_id}/notes/${h.note_id}`); }, el: h });
    }
    if (scope === 'all' || scope === 'tags') {
      for (const t of tagHits) arr.push({ kind: 'tag', run: () => { close(); goto(`/t/${encodeURIComponent(t.name)}`); }, el: t });
    }
    if (scope === 'all' || scope === 'actions') {
      for (const a of actionHits) arr.push({ kind: 'action', run: a.run, el: a });
    }
    return arr;
  });

  function moveDown() { sel = items.length > 0 ? Math.min(items.length - 1, sel + 1) : 0; }
  function moveUp()   { sel = Math.max(0, sel - 1); }

  function onKey(e: KeyboardEvent) {
    if (!open) return;
    if (e.key === 'Escape') { e.preventDefault(); close(); return; }
    if (e.key === 'ArrowDown' || (e.ctrlKey && e.key === 'n')) { e.preventDefault(); moveDown(); return; }
    if (e.key === 'ArrowUp'   || (e.ctrlKey && e.key === 'p')) { e.preventDefault(); moveUp();   return; }
    if (e.key === 'Enter')     { e.preventDefault(); items[sel]?.run(); return; }
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
      <input
        bind:this={input}
        bind:value={q}
        type="text"
        placeholder="search everything…"
        autofocus
      />
    </div>
    <div class="palette-scopes">
      {#each SCOPES as s}
        <button class="s" class:on={scope === s} onclick={() => { scope = s; sel = 0; }}>{s}</button>
      {/each}
    </div>
    <div class="palette-body">
      {#if scope === 'all' || scope === 'name'}
        {#if resourceHits.length > 0}
          <div class="p-section-head">resources · {resourceHits.length}</div>
          {#each resourceHits as h, i}
            {@const flatIdx = items.findIndex(x => x.kind === 'resource' && x.el === h)}
            <button
              class="p-row"
              class:sel={sel === flatIdx}
              onmouseenter={() => sel = flatIdx}
              onclick={() => items[flatIdx]?.run()}
            >
              <span class="ty">resource</span>
              <span>{h.title}</span>
              <span class="rt">{h.snippet.slice(0, 40)}</span>
            </button>
          {/each}
        {/if}
      {/if}

      {#if scope === 'all' || scope === 'notes'}
        {#if noteHits.length > 0}
          <div class="p-section-head">in notes · {noteHits.length}</div>
          {#each noteHits as h}
            {@const flatIdx = items.findIndex(x => x.kind === 'note' && x.el === h)}
            <button
              class="p-row"
              class:sel={sel === flatIdx}
              onmouseenter={() => sel = flatIdx}
              onclick={() => items[flatIdx]?.run()}
            >
              <span class="ty">note</span>
              <span class="snip">{h.snippet}</span>
              <span class="rt">{h.title}</span>
            </button>
          {/each}
        {/if}
      {/if}

      {#if scope === 'all' || scope === 'tags'}
        {#if tagHits.length > 0}
          <div class="p-section-head">tags · {tagHits.length}</div>
          {#each tagHits as t}
            {@const flatIdx = items.findIndex(x => x.kind === 'tag' && x.el === t)}
            <button
              class="p-row"
              class:sel={sel === flatIdx}
              onmouseenter={() => sel = flatIdx}
              onclick={() => items[flatIdx]?.run()}
            >
              <span class="ty">tag</span>
              <span>#{t.name}</span>
              <span class="rt">{t.count} items</span>
            </button>
          {/each}
        {/if}
      {/if}

      {#if scope === 'all' || scope === 'actions'}
        {#if actionHits.length > 0}
          <div class="p-section-head">actions</div>
          {#each actionHits as a}
            {@const flatIdx = items.findIndex(x => x.kind === 'action' && x.el === a)}
            <button
              class="p-row"
              class:sel={sel === flatIdx}
              onmouseenter={() => sel = flatIdx}
              onclick={() => items[flatIdx]?.run()}
            >
              <span class="ty">do</span>
              <span>{a.label}</span>
              <span class="rt">{#if a.kbd}<span class="kbd">{a.kbd}</span>{/if}</span>
            </button>
          {/each}
        {/if}
      {/if}

      {#if items.length === 0}
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
