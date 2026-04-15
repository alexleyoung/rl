<script lang="ts">
  import { api, type ResourceDto, type TagDto } from '$lib/api';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';

  type SortKey = 'tag' | 'title' | 'kind' | 'added' | 'read';
  type SortDir = 'asc' | 'desc';

  let resources = $state<ResourceDto[]>([]);
  let tags = $state<TagDto[]>([]);
  let activeTag = $state<string | null>(null);
  let sort = $state<SortKey>('tag');
  let dir = $state<SortDir>('asc');
  let error = $state('');

  async function load() {
    try {
      const params = page.url.searchParams;
      const tag = params.get('tag') ?? undefined;
      activeTag = tag ?? null;
      sort = (params.get('sort') as SortKey) || 'tag';
      dir = (params.get('dir') as SortDir) || 'asc';
      [resources, tags] = await Promise.all([api.listResources(tag), api.listTags()]);
    } catch (e: any) {
      error = e.message;
    }
  }

  $effect(() => { load(); });

  // Client-side sort
  let sorted = $derived.by(() => {
    const cmp = (a: ResourceDto, b: ResourceDto): number => {
      let av: string | number, bv: string | number;
      switch (sort) {
        case 'tag':   av = a.tags[0] ?? '\uffff'; bv = b.tags[0] ?? '\uffff'; break;
        case 'title': av = a.title.toLowerCase();  bv = b.title.toLowerCase();  break;
        case 'kind':  av = a.kind;                  bv = b.kind;                  break;
        case 'added': av = Number(a.added_at);      bv = Number(b.added_at);      break;
        case 'read':  av = Number(a.last_read_at ?? 0); bv = Number(b.last_read_at ?? 0); break;
      }
      if (av < bv) return -1;
      if (av > bv) return 1;
      return 0;
    };
    const result = [...resources].sort(cmp);
    return dir === 'desc' ? result.reverse() : result;
  });

  function setSort(key: SortKey) {
    const params = new URLSearchParams(page.url.searchParams);
    if (sort === key) {
      params.set('dir', dir === 'asc' ? 'desc' : 'asc');
    } else {
      params.set('sort', key);
      params.set('dir', 'asc');
    }
    goto(`/?${params}`, { replaceState: true, keepFocus: true });
  }

  function filterTag(name: string) {
    const params = new URLSearchParams(page.url.searchParams);
    if (activeTag === name) params.delete('tag');
    else params.set('tag', name);
    goto(`/?${params}`, { replaceState: true });
  }

  async function deleteResource(id: number, title: string) {
    if (!confirm(`Delete "${title}"?`)) return;
    await api.deleteResource(id);
    resources = resources.filter(r => Number(r.id) !== id);
  }

  const LABELS: Record<SortKey, string> = {
    tag: 'tag', title: 'title', kind: 'kind', added: 'added', read: 'last read',
  };

  function thClass(key: SortKey) {
    return sort === key ? `sort-active ${dir}` : '';
  }
</script>

{#if error}<p class="flash err">{error}</p>{/if}

<div class="tags">
  {#each tags as t}
    <button class="tag" class:active={activeTag === t.name} onclick={() => filterTag(t.name)}>
      {t.name} <span class="dim">({t.count})</span>
    </button>
  {/each}
</div>

<table>
  <thead>
    <tr>
      {#each (['title', 'kind', 'tag', 'added', 'read'] as SortKey[]) as key}
        <th>
          <button class="sort-btn {thClass(key)}" onclick={() => setSort(key)}>
            {LABELS[key]}{#if sort === key}<span class="arrow">{dir === 'asc' ? ' ↑' : ' ↓'}</span>{/if}
          </button>
        </th>
      {/each}
      <th></th>
    </tr>
  </thead>
  <tbody>
    {#each sorted as r}
      <tr>
        <td><a href="/resources/{r.id}">{r.title}</a></td>
        <td><span class="kind">{r.kind}</span></td>
        <td class="small dim">{r.tags.join(', ')}</td>
        <td class="small dim">{r.added_at ? new Date(Number(r.added_at) * 1000).toLocaleDateString() : ''}</td>
        <td class="small dim">{r.last_read_at ? new Date(Number(r.last_read_at) * 1000).toLocaleDateString() : '—'}</td>
        <td>
          <span class="row-actions">
            <a href="/resources/{r.id}/edit" class="btn small">edit</a>
            <button class="danger small" onclick={() => deleteResource(Number(r.id), r.title)}>del</button>
          </span>
        </td>
      </tr>
    {/each}
  </tbody>
</table>

<style>
  .sort-btn {
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    font-size: inherit;
    font-weight: bold;
    color: var(--dim);
    cursor: pointer;
    white-space: nowrap;
  }
  .sort-btn:hover { color: var(--fg); }
  .sort-btn.sort-active { color: var(--fg); }
  .arrow { color: var(--accent); }
</style>
