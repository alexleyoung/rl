<script lang="ts">
  import '../app.css';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import CommandPalette from '$lib/CommandPalette.svelte';
  import AddModal from '$lib/AddModal.svelte';
  import HelpOverlay from '$lib/HelpOverlay.svelte';
  import { getBindingsForAction, defaultKeymap } from '$lib/keymap';

  interface Props { children: import('svelte').Snippet }
  let { children }: Props = $props();

  let paletteOpen = $state(false);
  let addOpen = $state(false);
  let helpOpen = $state(false);
  let chordPending = $state<string | null>(null);
  let chordTimer: ReturnType<typeof setTimeout> | null = null;

  function openPalette() { paletteOpen = true; }
  function openAdd() { addOpen = true; }

  function clearChord() {
    chordPending = null;
    if (chordTimer) { clearTimeout(chordTimer); chordTimer = null; }
  }

  function isTypingTarget(el: EventTarget | null) {
    if (!(el instanceof HTMLElement)) return false;
    const tag = el.tagName;
    return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || el.isContentEditable;
  }

  function matchKey(e: KeyboardEvent, binding: string): boolean {
    // binding is like "⌘K", "⌘⇧H", "/", "?", "g i" (chord — not handled here)
    if (binding.includes(' ')) return false; // chords handled separately
    const parts = binding.split('');
    const needMeta  = parts.includes('⌘'); // macOS Command — metaKey only
    const needCtrl  = parts.includes('⌃'); // explicit Ctrl — ctrlKey only
    const needShift = parts.includes('⇧');
    const mainKey = parts.filter(p => !'⌘⇧⌥⌃'.includes(p)).join('');
    if (!mainKey) return false;
    if (needMeta  && !e.metaKey)  return false;
    if (!needMeta && e.metaKey)   return false;
    if (needCtrl  && !e.ctrlKey)  return false;
    if (!needCtrl && !needMeta && e.ctrlKey) return false; // unmodified binding shouldn't fire on ctrl
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
      case 'this help': helpOpen = true; return true;
      default: return false;
    }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (chordPending) { clearChord(); e.preventDefault(); return; }
      if (paletteOpen) { paletteOpen = false; return; }
      if (addOpen) { addOpen = false; return; }
      if (helpOpen) { helpOpen = false; return; }
    }
    const inInput = isTypingTarget(e.target);

    // If we have a pending chord first-key, try to complete it
    if (chordPending) {
      const candidate = `${chordPending} ${e.key.toLowerCase()}`;
      clearChord();
      for (const action of Object.keys(defaultKeymap)) {
        const bindings = getBindingsForAction(action);
        for (const b of bindings) {
          if (b === candidate) {
            if (fireAction(action)) { e.preventDefault(); return; }
          }
        }
      }
      return; // consumed regardless
    }

    // Check if this key starts any chord
    if (!inInput && !e.metaKey && !e.ctrlKey && !e.altKey) {
      const k = e.key.toLowerCase();
      const startsChord = Object.values(defaultKeymap).some(entry =>
        getBindingsForAction(entry.action).some(b => b.includes(' ') && b.split(' ')[0] === k)
      );
      if (startsChord) {
        chordPending = k;
        chordTimer = setTimeout(clearChord, 1500);
        e.preventDefault();
        return;
      }
    }

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
    {#if chordPending}<span class="chord-hint"><span class="kbd">{chordPending}</span> –</span>{/if}
    <button class="nav-add" onclick={openAdd}>+ add <span class="kbd">⌘N</span></button>
    <button class="nav-search" onclick={openPalette}>search <span class="kbd">⌘K</span></button>
  </nav>

  <main>
    {@render children()}
  </main>
</div>

<CommandPalette bind:open={paletteOpen} />
<AddModal bind:open={addOpen} />
<HelpOverlay bind:open={helpOpen} />

<style>
  .spacer { flex: 1; }
  .nav-add, .nav-search { color: var(--ink-2); font-size: 12px; }
  .nav-add:hover, .nav-search:hover { color: var(--ink); text-decoration: underline; }
  .chord-hint { font-size: 12px; color: var(--ink-2); }
</style>
