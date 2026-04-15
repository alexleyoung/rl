<script lang="ts">
  import { api, type ResourceDto, type TagDto } from '$lib/api';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';

  let resources = $state<ResourceDto[]>([]);
  let tags = $state<TagDto[]>([]);
  let activeTag = $state<string | null>(null);
  let error = $state('');

  async function load() {
    try {
      const tag = page.url.searchParams.get('tag') ?? undefined;
      activeTag = tag ?? null;
      [resources, tags] = await Promise.all([api.listResources(tag), api.listTags()]);
    } catch (e: any) {
      error = e.message;
    }
  }

  $effect(() => { load(); });

  function filterTag(name: string) {
    if (activeTag === name) goto('/');
    else goto(`/?tag=${encodeURIComponent(name)}`);
  }

  async function deleteResource(id: number, title: string) {
    if (!confirm(`Delete "${title}"?`)) return;
    await api.deleteResource(id);
    resources = resources.filter(r => Number(r.id) !== id);
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
      <th>title</th>
      <th>kind</th>
      <th>tags</th>
      <th></th>
    </tr>
  </thead>
  <tbody>
    {#each resources as r}
      <tr>
        <td><a href="/resources/{r.id}">{r.title}</a></td>
        <td><span class="kind">{r.kind}</span></td>
        <td class="small dim">{r.tags.join(', ')}</td>
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
