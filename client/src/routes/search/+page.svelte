<script lang="ts">
  import { page } from '$app/state';
  import { api, type SearchResponseDto } from '$lib/api';

  let results = $state<SearchResponseDto | null>(null);
  let error = $state('');

  $effect(() => {
    const q = page.url.searchParams.get('q') ?? '';
    if (!q.trim()) { results = null; return; }
    api.search(q)
      .then(r => { results = r; })
      .catch(e => { error = e.message; });
  });

  function hitUrl(hit: SearchResponseDto['hits'][number]): string {
    if (hit.note_id) return `/resources/${hit.resource_id}/notes/${hit.note_id}`;
    return `/resources/${hit.resource_id}`;
  }
</script>

{#if error}<p class="flash err">{error}</p>{/if}

{#if results}
  <p class="dim small mb">{results.hits.length} result{results.hits.length !== 1 ? 's' : ''} for "{results.query}"</p>

  {#if results.hits.length === 0}
    <p class="dim">no results</p>
  {:else}
    <table>
      <thead><tr><th>title</th><th>kind</th><th>snippet</th></tr></thead>
      <tbody>
        {#each results.hits as h}
          <tr>
            <td><a href={hitUrl(h)}>{h.title}</a></td>
            <td><span class="kind">{h.source_kind}</span></td>
            <td class="small">{h.snippet}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
{/if}
