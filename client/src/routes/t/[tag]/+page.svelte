<script lang="ts">
  import { page } from '$app/state';
  import { api, type ResourceDto, type TagDto, type SearchHitDto } from '$lib/api';

  const tag = $derived(decodeURIComponent(page.params.tag));

  let resources = $state<ResourceDto[]>([]);
  let tags = $state<TagDto[]>([]);
  let noteHits = $state<SearchHitDto[]>([]);
  let error = $state('');

  async function load() {
    try {
      const [rs, ts, search] = await Promise.all([
        api.listResources({ tag }),
        api.listTags(),
        api.search(tag, 10).catch(() => ({ hits: [] })),
      ]);
      resources = rs;
      tags = ts.filter(t => t.name !== tag);
      noteHits = (search.hits ?? []).filter(h => h.source_kind === 'note');
    } catch (e: any) { error = e.message; }
  }

  $effect(() => { if (tag) load(); });

  const STATUSES = ['reading', 'queue', 'inbox', 'done'] as const;
  const byStatus = $derived.by(() => {
    const map: Record<string, ResourceDto[]> = { reading: [], queue: [], inbox: [], done: [] };
    for (const r of resources) {
      const s = (r.status ?? 'inbox') as string;
      (map[s] ??= []).push(r);
    }
    return map;
  });

  const readCount = $derived(resources.filter(r => r.status === 'done').length);
  const pct = $derived(resources.length ? Math.round((readCount / resources.length) * 100) : 0);
</script>

{#if error}<p class="flash err">{error}</p>{/if}

<div class="tag-head">
  <h1>#{tag}</h1>
  <span class="meta">{resources.length} item{resources.length === 1 ? '' : 's'} · {pct}% read · grouped by status</span>
</div>

<div class="tag-grid">
  <div>
    {#each STATUSES as s}
      {@const items = byStatus[s] ?? []}
      {#if items.length > 0}
        <div class="group-head">{s} · {items.length}</div>
        <ul class="list">
          {#each items as r}
            <li class="row-item {r.status === 'done' ? 'read' : ''}">
              <span class="ck {r.status === 'done' ? 'done' : ''}">{r.status === 'done' ? '✓' : '○'}</span>
              <span class="ty">{r.kind}</span>
              <a class="title" href="/resources/{r.id}">{r.title}</a>
              <span class="tags">
                {#each r.tags.filter(t => t !== tag) as t}
                  <a href="/t/{encodeURIComponent(t)}" class="muted">#{t}</a>{' '}
                {/each}
              </span>
            </li>
          {/each}
        </ul>
      {/if}
    {/each}
    {#if resources.length === 0}
      <p class="dim">no resources tagged #{tag}.</p>
    {/if}
  </div>

  <aside class="aside">
    <div class="block">
      <div class="h">other tags</div>
      <div class="tagcloud">
        {#each tags.slice(0, 24) as t}
          <a href="/t/{encodeURIComponent(t.name)}" class={t.count >= 5 ? 'lg' : t.count >= 2 ? 'md' : ''}>#{t.name}</a>
        {/each}
      </div>
    </div>

    {#if noteHits.length > 0}
      <div class="block">
        <div class="h">notes mentioning</div>
        {#each noteHits as h}
          <div class="note-snip">
            {@html h.snippet}
            <div class="from">from: <a href="/resources/{h.resource_id}">{h.title}</a></div>
          </div>
        {/each}
      </div>
    {/if}
  </aside>
</div>
