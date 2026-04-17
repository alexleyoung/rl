<script lang="ts">
  import { api, type ResourceDto } from '$lib/api';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { pageActions } from '$lib/paletteActions';

  type Status = 'inbox' | 'reading' | 'queue' | 'done' | 'all';

  let resources = $state<ResourceDto[]>([]);
  let status = $state<Status>('inbox');
  let filter = $state('');
  let error = $state('');
  let selected = $state(0);

  async function load() {
    try {
      const st = status === 'all' ? undefined : status;
      resources = await api.listResources({ status: st });
    } catch (e: any) {
      error = e.message;
    }
  }

  $effect(() => {
    const s = page.url.searchParams.get('status') as Status | null;
    status = s && ['inbox','reading','queue','done','all'].includes(s) ? s : 'inbox';
    load();
  });

  function setStatus(s: Status) {
    const qs = new URLSearchParams(page.url.searchParams);
    qs.set('status', s);
    goto(`/?${qs}`, { replaceState: true, keepFocus: true });
  }

  async function toggleDone(r: ResourceDto, e: Event) {
    e.stopPropagation();
    const next = r.status === 'done' ? 'inbox' : 'done';
    try {
      const updated = await api.setStatus(Number(r.id), next);
      resources = resources.map(x => x.id === r.id ? updated : x);
    } catch (ex: any) { error = ex.message; }
  }

  function open(r: ResourceDto) {
    goto(`/resources/${r.id}`);
  }

  let visible = $derived.by(() => {
    const f = filter.trim().toLowerCase();
    if (!f) return resources;
    return resources.filter(r =>
      r.title.toLowerCase().includes(f) ||
      (r.author ?? '').toLowerCase().includes(f) ||
      r.tags.some(t => t.toLowerCase().includes(f))
    );
  });

  let unreadCount = $derived(resources.filter(r => !r.last_read_at).length);

  $effect(() => {
    const el = document.querySelector('.row-item.selected');
    if (el) el.scrollIntoView({ block: 'nearest' });
  });

  function fmtTag(t: string) { return '#' + t; }

  $effect(() => {
    pageActions.set([
      { label: 'go to inbox',   run: () => goto('/?status=inbox') },
      { label: 'go to reading', run: () => goto('/?status=reading') },
      { label: 'go to queue',   run: () => goto('/?status=queue') },
      { label: 'go to done',    run: () => goto('/?status=done') },
      { label: 'go to all',     run: () => goto('/?status=all') },
    ]);
    return () => pageActions.set([]);
  });

  function isTypingTarget(el: EventTarget | null) {
    if (!(el instanceof HTMLElement)) return false;
    const tag = el.tagName;
    return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || el.isContentEditable;
  }

  function onKey(e: KeyboardEvent) {
    if (isTypingTarget(e.target)) return;
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    if (e.key === 'j' || e.key === 'ArrowDown') {
      e.preventDefault();
      selected = Math.min(visible.length - 1, selected + 1);
    } else if (e.key === 'k' || e.key === 'ArrowUp') {
      e.preventDefault();
      selected = Math.max(0, selected - 1);
    } else if (e.key === 'Enter') {
      const r = visible[selected];
      if (r) { e.preventDefault(); goto(`/resources/${r.id}`); }
    } else if (e.key === 'e') {
      const r = visible[selected];
      if (r) { e.preventDefault(); toggleDone(r, e); }
    } else if (e.key === 'q') {
      const r = visible[selected];
      if (r && r.status !== 'queue') {
        e.preventDefault();
        api.setStatus(Number(r.id), 'queue').then(updated => {
          resources = resources.map(x => x.id === r.id ? updated : x);
        }).catch((ex: any) => { error = ex.message; });
      }
    }
  }
</script>

<svelte:window onkeydown={onKey} />

<h2 class="title">resources</h2>

{#if error}<p class="flash err">{error}</p>{/if}

<div class="controls">
  <div class="search">
    <input
      type="text"
      bind:value={filter}
      placeholder="filter this list…  /  to focus, ⌘K for global"
    />
  </div>
  <div class="seg">
    {#each (['inbox','reading','queue','done','all'] as Status[]) as s}
      <button class:on={status === s} onclick={() => setStatus(s)}>{s}</button>
    {/each}
  </div>
</div>

<div class="status-row">
  <span>{status} · {visible.length} {visible.length === 1 ? 'item' : 'items'}{#if status === 'inbox' && unreadCount > 0} · {unreadCount} unread{/if}</span>
</div>

<ul class="list">
  {#each visible as r, i}
    <li
      class="row-item"
      class:read={r.status === 'done' || r.last_read_at}
      class:selected={selected === i}
      onmouseenter={() => selected = i}
      role="presentation"
    >
      <button class="ck" class:done={r.status === 'done'} onclick={(e) => toggleDone(r, e)} aria-label="toggle done">
        {r.status === 'done' ? '✓' : '○'}
      </button>
      <span class="ty">{r.kind}</span>
      <a class="title" href="/resources/{r.id}" onclick={(e) => { e.preventDefault(); open(r); }}>
        {r.title}{#if r.author} — {r.author}{/if}
      </a>
      <span class="tags">
        {#each r.tags as t, i}
          {#if i > 0}{' '}{/if}
          <a href="/t/{encodeURIComponent(t)}">{fmtTag(t)}</a>
        {/each}
      </span>
    </li>
  {/each}
</ul>

{#if visible.length === 0 && !error}
  <p class="dim" style="margin-top:18px">nothing here. press <span class="kbd">⌘N</span> to add.</p>
{/if}
