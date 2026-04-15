<script lang="ts">
  import '../app.css';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';

  interface Props { children: import('svelte').Snippet }
  let { children }: Props = $props();

  let searchQuery = $state('');

  function onSearch(e: Event) {
    e.preventDefault();
    if (searchQuery.trim()) {
      goto(`/search?q=${encodeURIComponent(searchQuery.trim())}`);
    }
  }

  // Sync search input from URL on search page
  $effect(() => {
    if (page.url.pathname === '/search') {
      searchQuery = page.url.searchParams.get('q') ?? '';
    } else {
      searchQuery = '';
    }
  });
</script>

<div id="wrap">
  <nav>
    <a href="/" class="brand">rl</a>
    <a href="/" class:active={page.url.pathname === '/'}>library</a>
    <a href="/resources/new">+ add</a>
  </nav>

  <div id="search-bar">
    <form onsubmit={onSearch} style="flex-direction:row; gap:0.5rem;">
      <input
        type="text"
        placeholder="search…"
        bind:value={searchQuery}
      />
      <button type="submit">search</button>
    </form>
  </div>

  {@render children()}
</div>
