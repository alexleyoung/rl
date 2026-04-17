<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { pageActions } from '$lib/paletteActions';
  interface Props { children?: import('svelte').Snippet }
  let { children }: Props = $props();

  const path = $derived(page.url.pathname);
  function active(p: string) { return path === p || path.startsWith(p + '/'); }

  const SETTINGS_SUBPAGES: { slug: string; label: string }[] = [
    { slug: 'general',      label: 'general' },
    { slug: 'appearance',   label: 'appearance' },
    { slug: 'reading',      label: 'reading' },
    { slug: 'import',       label: 'import / export' },
    { slug: 'keymap',       label: 'keymap' },
    { slug: 'smart',        label: 'smart views' },
    { slug: 'integrations', label: 'integrations' },
    { slug: 'sync',         label: 'sync' },
    { slug: 'data',         label: 'data' },
  ];

  $effect(() => {
    pageActions.set(
      SETTINGS_SUBPAGES.map(p => ({
        label: `go to settings: ${p.label}`,
        run: () => goto(`/settings/${p.slug}`),
      })),
    );
    return () => pageActions.set([]);
  });
</script>

<div class="settings-head">
  <h1>settings</h1>
  <span class="muted">· {path.split('/').pop() || 'general'}</span>
</div>

<div class="settings-grid">
  <nav class="s-nav">
    <div class="h">preferences</div>
    <a href="/settings/general" class:on={active('/settings/general')}>general</a>
    <a href="/settings/appearance" class:on={active('/settings/appearance')}>appearance</a>
    <a href="/settings/reading" class:on={active('/settings/reading')}>reading</a>
    <a href="/settings/import" class:on={active('/settings/import')}>import / export</a>
    <div class="h">power</div>
    <a href="/settings/keymap" class:on={active('/settings/keymap')}>keymap</a>
    <a href="/settings/smart" class:on={active('/settings/smart')}>smart views</a>
    <a href="/settings/integrations" class:on={active('/settings/integrations')}>integrations</a>
    <div class="h">account</div>
    <a href="/settings/sync" class:on={active('/settings/sync')}>sync</a>
    <a href="/settings/data" class:on={active('/settings/data')}>data</a>
  </nav>
  <div>
    {@render children?.()}
  </div>
</div>
