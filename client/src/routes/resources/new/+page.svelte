<script lang="ts">
  import { goto } from '$app/navigation';
  import { api, type ResourceInputDto } from '$lib/api';
  import ResourceForm from '$lib/ResourceForm.svelte';

  async function create(data: ResourceInputDto) {
    const r = await api.createResource(data);
    goto(`/resources/${r.id}`);
  }
</script>

<h1>add resource</h1>
<ResourceForm onsubmit={create} submitLabel="add" />
