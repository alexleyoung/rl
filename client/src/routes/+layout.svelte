<script lang="ts">
  import '../app.css';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import CommandPalette from '$lib/CommandPalette.svelte';
  import AddModal from '$lib/AddModal.svelte';
  import { getBindingsForAction, defaultKeymap } from '$lib/keymap';

  interface Props { children: import('svelte').Snippet }
  let { children }: Props = $props();

  let paletteOpen = $state(false);
  let addOpen = $state(false);

  function openPalette() { paletteOpen = true; }
  function openAdd() { addOpen = true; }

  function isTypingTarget(el: EventTarget | null) {
    if (!(el instanceof HTMLElement)) return false;
    const tag = el.tagName;
    return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || el.isContentEditable;
  }

  function matchKey(e: KeyboardEvent, binding: string): boolean {
    // binding is like "⌘K", "⌘⇧H", "/", "?", "g i" (chord — not handled here)
    if (binding.includes(' ')) return false; // chords handled separately
    const parts = binding.split('');
    const needMeta = parts.includes('⌘');
    const needShift = parts.includes('⇧');
    const mainKey = parts.filter(p => p !== '⌘' && p !== '⇧' && p !== '⌥' && p !== '⌃').join('');
    if (!mainKey) return false;
    const metaOrCtrl = e.metaKey || e.ctrlKey;
    if (needMeta && !metaOrCtrl) return false;
    if (!needMeta && metaOrCtrl) return false;
    if (needShift && !e.shiftKey) return false;
    return e.key.toLowerCase() === mainKey.toLowerCase();
  }

  function fireAction(action: string): boolean {
    switch (action) {
      case 'command palette': openPalette(); return true;
      case 'add new resource': openAdd(); return true;
      case 'focus filter': {
        const i = document.querySelector('main input[type="text"], main input[type="search"]') as HTMLInputElement | null;
        if (i) { i.focus(); return true; }
        return false;
      }
      case 'go to inbox': goto('/?status=inbox'); return true;
      case 'go to reading': goto('/?status=reading'); return true;
      case 'go to queue': goto('/?status=queue'); return true;
      case 'go to done': goto('/?status=done'); return true;
      case 'this help': goto('/settings/keymap'); return true;
      default: return false;
    }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (paletteOpen) { paletteOpen = false; return; }
      if (addOpen) { addOpen = false; return; }
    }
    const inInput = isTypingTarget(e.target);
    // Global bindings: always fire (even from inputs) if they include ⌘
    for (const action of Object.keys(defaultKeymap)) {
      const bindings = getBindingsForAction(action);
      for (const b of bindings) {
        const hasModifier = b.includes('⌘') || b.includes('⇧');
        if (inInput && !hasModifier) continue;
        if (matchKey(e, b)) {
          if (fireAction(action)) {
            e.preventDefault();
            return;
          }
        }
      }
    }
  }

  $effect(() => {
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  });

  let currentPath = $derived(page.url.pathname);
  function isOn(prefix: string) {
    if (prefix === '/') return currentPath === '/';
    return currentPath.startsWith(prefix);
  }
</script>

<div class="page">
  <nav class="nav">
    <a href="/" class="brand">rl</a>
    <a href="/" class:on={isOn('/')}>resources</a>
    <a href="/settings/keymap" class:on={isOn('/settings')}>settings</a>
    <span class="spacer"></span>
    <button class="nav-add" onclick={openAdd}>+ add <span class="kbd">⌘N</span></button>
    <button class="nav-search" onclick={openPalette}>search <span class="kbd">⌘K</span></button>
  </nav>

  <main>
    {@render children()}
  </main>
</div>

<CommandPalette bind:open={paletteOpen} />
<AddModal bind:open={addOpen} />

<style>
  .spacer { flex: 1; }
  .nav-add, .nav-search { color: var(--ink-2); font-size: 12px; }
  .nav-add:hover, .nav-search:hover { color: var(--ink); text-decoration: underline; }
</style>
