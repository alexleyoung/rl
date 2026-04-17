<script lang="ts">
  import { defaultKeymap, getBindingsForAction } from '$lib/keymap';

  interface Props { open: boolean }
  let { open = $bindable(false) }: Props = $props();

  function close() { open = false; }

  function onKey(e: KeyboardEvent) {
    if (!open) return;
    if (e.key === 'Escape' || e.key === '?') { e.preventDefault(); close(); }
  }

  const groups = ['global', 'navigation', 'in list', 'reader'] as const;
  const entries = Object.values(defaultKeymap);

  function tokens(b: string): string[] { return b.split(' ').filter(Boolean); }
</script>

<svelte:window onkeydown={onKey} />

{#if open}
  <div class="overlay" onclick={close} role="presentation"></div>
  <div class="help" role="dialog" aria-modal="true">
    <div class="help-head">
      <span>keyboard shortcuts</span>
      <button onclick={close}>×</button>
    </div>
    <div class="help-body">
      {#each groups as group}
        {@const rows = entries.filter(e => e.group === group)}
        {#if rows.length}
          <div class="h-group">{group}</div>
          {#each rows as e}
            <div class="h-row">
              <span class="h-action">{e.action}</span>
              <span class="h-bindings">
                {#each getBindingsForAction(e.action) as b, i}
                  {#if i > 0}<span class="dim"> / </span>{/if}
                  {#each tokens(b) as tok}<span class="kbd">{tok}</span>{/each}
                {/each}
              </span>
              <span class="h-ctx dim">{e.context}</span>
            </div>
          {/each}
        {/if}
      {/each}
    </div>
    <div class="help-foot dim">
      <span class="kbd">?</span> or <span class="kbd">esc</span> to close ·
      <a href="/settings/keymap">edit bindings ↗</a>
    </div>
  </div>
{/if}

<style>
  .help {
    position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%);
    z-index: 200;
    width: 520px; max-width: calc(100vw - 32px);
    max-height: 80vh;
    display: flex; flex-direction: column;
    background: var(--bg-2); border: 1px solid var(--line); border-radius: 2px;
  }
  .help-head {
    display: flex; justify-content: space-between; align-items: center;
    padding: 10px 14px; border-bottom: 1px solid var(--line);
    font-size: 12px; color: var(--ink-2);
  }
  .help-head button { color: var(--ink-3); font-size: 16px; line-height: 1; }
  .help-head button:hover { color: var(--ink); }
  .help-body { overflow: auto; flex: 1; padding: 8px 0; }
  .h-group {
    padding: 6px 14px 2px;
    font-size: 10px; text-transform: uppercase; letter-spacing: 0.08em;
    color: var(--ink-3);
  }
  .h-row {
    display: flex; align-items: center; gap: 10px;
    padding: 3px 14px; font-size: 12px;
  }
  .h-action { flex: 1; color: var(--ink); }
  .h-bindings { display: flex; align-items: center; gap: 2px; flex-shrink: 0; }
  .h-ctx { font-size: 11px; min-width: 90px; text-align: right; flex-shrink: 0; }
  .help-foot {
    padding: 8px 14px; border-top: 1px solid var(--line);
    font-size: 11px;
  }
  .help-foot a { color: var(--ink-2); }
  .help-foot a:hover { color: var(--ink); }
</style>
