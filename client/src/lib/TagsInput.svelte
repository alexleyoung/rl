<script lang="ts">
  interface Props {
    tags?: string[];
    onchange?: (tags: string[]) => void;
    placeholder?: string;
  }

  let {
    tags = $bindable([]),
    onchange,
    placeholder = 'add tag…',
  }: Props = $props();

  let inputValue = $state('');
  let inputEl: HTMLInputElement;

  function notify() {
    onchange?.(tags);
  }

  function addTag(raw: string) {
    const tag = raw.trim().toLowerCase();
    if (tag && !tags.includes(tag)) {
      tags = [...tags, tag];
      notify();
    }
    inputValue = '';
  }

  function removeTag(tag: string) {
    tags = tags.filter(t => t !== tag);
    notify();
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      addTag(inputValue);
    } else if (e.key === 'Backspace' && inputValue === '' && tags.length > 0) {
      removeTag(tags[tags.length - 1]);
    }
  }

  function onBlur() {
    if (inputValue.trim()) addTag(inputValue);
  }

  // Accept pasted comma-separated lists
  function onPaste(e: ClipboardEvent) {
    const text = e.clipboardData?.getData('text') ?? '';
    if (text.includes(',')) {
      e.preventDefault();
      for (const part of text.split(',')) addTag(part);
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="tags-input" onclick={() => inputEl.focus()}>
  {#each tags as tag}
    <span class="chip">
      {tag}
      <button
        type="button"
        class="remove"
        onclick={(e) => { e.stopPropagation(); removeTag(tag); }}
        aria-label="remove {tag}"
      >×</button>
    </span>
  {/each}
  <input
    bind:this={inputEl}
    bind:value={inputValue}
    type="text"
    {placeholder}
    onkeydown={onKeydown}
    onblur={onBlur}
    onpaste={onPaste}
    size={Math.max(placeholder.length, inputValue.length + 2)}
  />
</div>

<style>
  .tags-input {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    align-items: center;
    padding: 0.25rem 0.4rem;
    border: 1px solid var(--border);
    background: var(--bg);
    cursor: text;
    min-height: 2.2rem;
  }
  .tags-input:focus-within { border-color: var(--accent); }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 0.2rem;
    padding: 0.1rem 0.4rem;
    background: color-mix(in srgb, var(--accent) 12%, var(--bg));
    border: 1px solid color-mix(in srgb, var(--accent) 35%, var(--border));
    font-size: 0.8rem;
    line-height: 1.4;
    white-space: nowrap;
  }

  .remove {
    background: none;
    border: none;
    padding: 0;
    margin: 0;
    font: inherit;
    font-size: 0.9rem;
    line-height: 1;
    color: var(--dim);
    cursor: pointer;
    display: flex;
    align-items: center;
  }
  .remove:hover { color: var(--danger); }

  input {
    border: none;
    outline: none;
    background: transparent;
    font: inherit;
    font-size: 0.9rem;
    padding: 0.1rem 0;
    min-width: 4rem;
    flex: 1;
  }
</style>
