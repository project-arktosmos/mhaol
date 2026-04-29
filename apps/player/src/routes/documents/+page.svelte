<script lang="ts">
  import { onMount } from "svelte";
  import { documentsService } from "ui-lib/services/documents.service";
  import DocumentCard from "ui-lib/components/documents/DocumentCard.svelte";

  const docs = documentsService.state;

  onMount(() => {
    documentsService.refresh();
  });
</script>

<svelte:head>
  <title>Mhaol Player — Documents</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
  <header class="flex items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-bold">Documents</h1>
      <p class="text-sm text-base-content/60">
        Documents stored in the cloud's SurrealDB, fetched via the active
        transport (HTTP or WebRTC RPC).
      </p>
    </div>
    <button
      class="btn btn-outline btn-sm"
      onclick={() => documentsService.refresh()}
      disabled={$docs.loading}
    >
      {$docs.loading ? "Loading…" : "Refresh"}
    </button>
  </header>

  {#if $docs.error}
    <div class="alert alert-error">
      <span>{$docs.error}</span>
    </div>
  {/if}

  {#if $docs.loading && $docs.documents.length === 0}
    <p class="text-sm text-base-content/60">Loading…</p>
  {:else if $docs.documents.length === 0}
    <p class="text-sm text-base-content/60">No documents in the cloud yet.</p>
  {:else}
    <div
      class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4"
    >
      {#each $docs.documents as doc (doc.id)}
        <DocumentCard document={doc} />
      {/each}
    </div>
  {/if}
</div>
