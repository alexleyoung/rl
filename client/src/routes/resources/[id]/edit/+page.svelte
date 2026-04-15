<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { api, type ResourceDto, type ResourceInputDto } from '$lib/api';
  import ResourceForm from '$lib/ResourceForm.svelte';

  const rid = $derived(Number(page.params.id));
  let resource = $state<ResourceDto | null>(null);
  let error = $state('');

  $effect(() => {
    if (!rid) return;
    api.getResource(rid)
      .then(d => { resource = d.resource; })
      .catch(e => { error = e.message; });
  });

  async function update(data: ResourceInputDto) {
    await api.updateResource(rid, data);
    goto(`/resources/${rid}`);
  }
</script>

{#if error}<p class="flash err">{error}</p>{/if}

{#if resource}
  <h1>edit — {resource.title}</h1>
  <ResourceForm initial={resource} onsubmit={update} />
{/if}
