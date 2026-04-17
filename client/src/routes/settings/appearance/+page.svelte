<script lang="ts">
  import { onMount } from 'svelte';

  let theme = $state<'dark' | 'light'>('dark');

  onMount(() => {
    theme = (localStorage.getItem('rl_theme') as 'dark' | 'light') || 'dark';
  });

  function setTheme(t: 'dark' | 'light') {
    theme = t;
    localStorage.setItem('rl_theme', t);
    document.documentElement.setAttribute('data-theme', t);
  }
</script>

<div style="display:flex; align-items:baseline; gap:10px; margin-bottom:18px;">
  <b>appearance</b>
</div>

<div class="setting-row">
  <span class="setting-label">theme</span>
  <div class="seg">
    <button class:on={theme === 'dark'}  onclick={() => setTheme('dark')}>dark</button>
    <button class:on={theme === 'light'} onclick={() => setTheme('light')}>light</button>
  </div>
</div>

<style>
  .setting-row {
    display: flex; align-items: center; gap: 24px;
    padding: 10px 0; border-bottom: 1px solid var(--line);
  }
  .setting-label { color: var(--ink-2); min-width: 100px; }
</style>
