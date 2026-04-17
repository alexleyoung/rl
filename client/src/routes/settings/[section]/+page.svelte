<script lang="ts">
  import { page } from '$app/state';
  const section = $derived(page.params.section);
</script>

<div style="display:flex; align-items:baseline; gap:10px; margin-bottom:6px;">
  <b>{section}</b>
  <span class="muted">· not implemented yet</span>
</div>
<p class="dim">this settings section is a placeholder in v1.</p>
