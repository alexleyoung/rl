<script lang="ts">
  import { defaultKeymap, loadOverrides, saveOverrides, countModified, type Binding } from '$lib/keymap';

  type Entry = typeof defaultKeymap[string];
  const entries = Object.values(defaultKeymap);

  let overrides = $state<Record<string, Binding[]>>({});
  let query = $state('');
  let editing = $state<{ action: string; index: number } | null>(null);
  let captured = $state<string[]>([]);
  let modified = $state(0);

  function reload() {
    overrides = loadOverrides();
    modified = countModified();
  }
  $effect(() => { reload(); });

  function bindingsFor(action: string): Binding[] {
    return overrides[action] ?? defaultKeymap[action].bindings;
  }

  const grouped = $derived.by(() => {
    const q = query.trim().toLowerCase();
    const match = (e: Entry) =>
      !q || e.action.includes(q) || e.context.includes(q) || bindingsFor(e.action).some(b => b.toLowerCase().includes(q));
    const groups: Record<string, Entry[]> = { global: [], navigation: [], 'in list': [], reader: [] };
    for (const e of entries) if (match(e)) groups[e.group].push(e);
    return groups;
  });

  function startEdit(action: string, index: number) {
    editing = { action, index };
    captured = [];
  }

  function cancelEdit() { editing = null; captured = []; }

  function commitEdit() {
    if (!editing || captured.length === 0) { cancelEdit(); return; }
    const current = [...bindingsFor(editing.action)];
    current[editing.index] = captured.join(' ');
    const next = { ...overrides, [editing.action]: current };
    if (JSON.stringify(current) === JSON.stringify(defaultKeymap[editing.action].bindings)) {
      delete next[editing.action];
    }
    overrides = next;
    saveOverrides(next);
    modified = countModified();
    cancelEdit();
  }

  function formatKey(e: KeyboardEvent): string {
    const parts: string[] = [];
    if (e.metaKey) parts.push('⌘');
    if (e.ctrlKey) parts.push('⌃');
    if (e.altKey) parts.push('⌥');
    if (e.shiftKey) parts.push('⇧');
    let k = e.key;
    if (k === 'Enter') k = '↵';
    else if (k === 'Backspace') k = '⌫';
    else if (k === 'Escape') k = 'Esc';
    else if (k === 'Tab') k = '⇥';
    else if (k === ' ') k = 'Space';
    else if (k === 'ArrowUp') k = '↑';
    else if (k === 'ArrowDown') k = '↓';
    else if (k === 'ArrowLeft') k = '←';
    else if (k === 'ArrowRight') k = '→';
    else if (k.length === 1) k = k.toUpperCase();
    return parts.length ? parts.join('') + k : k;
  }

  function onKey(e: KeyboardEvent) {
    if (!editing) return;
    if (e.key === 'Escape') { e.preventDefault(); cancelEdit(); return; }
    if (e.key === 'Enter' && !e.metaKey && !e.ctrlKey && !e.altKey && !e.shiftKey) {
      e.preventDefault(); commitEdit(); return;
    }
    if (['Meta', 'Control', 'Alt', 'Shift'].includes(e.key)) return;
    e.preventDefault();
    captured = [...captured, formatKey(e)];
  }

  function resetAll() {
    overrides = {};
    saveOverrides({});
    modified = 0;
  }

  function exportJson() {
    const blob = new Blob([JSON.stringify(overrides, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url; a.download = 'rl-keymap.json'; a.click();
    URL.revokeObjectURL(url);
  }

  function renderBinding(b: Binding) {
    return b.split(' ').map(tok => tok.split('').map(c => ({ c }))); // not used
  }

  function tokens(b: Binding): string[] {
    return b.split(' ').filter(Boolean);
  }
</script>

<svelte:window onkeydown={onKey} />

<div style="display:flex; align-items:baseline; gap:10px; margin-bottom:6px;">
  <b>keymap</b>
  <span class="muted">click a binding to rebind. press new keys, then <span class="kbd">↵</span>.</span>
</div>

<div class="keymap-search">
  <input type="text" placeholder="search bindings…" bind:value={query} />
</div>

<table class="keymap">
  <thead>
    <tr><th>action</th><th>binding</th><th>context</th></tr>
  </thead>
  <tbody>
    {#each Object.entries(grouped) as [group, items]}
      {#if items.length > 0}
        <tr class="group-divider"><td colspan="3">{group}</td></tr>
        {#each items as e}
          <tr>
            <td>{e.action}</td>
            <td>
              {#each bindingsFor(e.action) as b, i}
                {@const isEditing = editing?.action === e.action && editing?.index === i}
                <button
                  class="binding"
                  class:editing={isEditing}
                  onclick={() => startEdit(e.action, i)}
                  type="button"
                >
                  {#if isEditing}
                    {#if captured.length === 0}
                      <span class="muted">press keys…</span>
                    {:else}
                      {#each captured as tok}
                        <span class="kbd">{tok}</span>
                      {/each}
                    {/if}
                  {:else}
                    {#each tokens(b) as tok}
                      <span class="kbd">{tok}</span>
                    {/each}
                  {/if}
                </button>
                {#if i < bindingsFor(e.action).length - 1}{' '}{/if}
              {/each}
            </td>
            <td><span class="ctx">{e.context}</span></td>
          </tr>
        {/each}
      {/if}
    {/each}
  </tbody>
</table>

<div class="sep"></div>
<div style="display:flex; justify-content:space-between; align-items:center;">
  <div class="muted">{modified} binding{modified === 1 ? '' : 's'} modified from defaults</div>
  <div style="display:flex; gap:8px;">
    <button class="btn" onclick={resetAll}>reset all</button>
    <button class="btn" onclick={exportJson}>export json</button>
  </div>
</div>

<style>
  .binding { background: transparent; cursor: pointer; font: inherit; }
</style>
