<script lang="ts">
  import { onMount } from "svelte";
  import { documentsService } from "ui-lib/services/documents.service";

  const docs = documentsService.state;

  onMount(() => {
    documentsService.refresh();
  });

  function formatDate(value: string): string {
    try {
      return new Date(value).toLocaleString();
    } catch {
      return value;
    }
  }
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
    <div class="overflow-x-auto rounded-box border border-base-content/10">
      <table class="table table-sm">
        <thead>
          <tr>
            <th>ID</th>
            <th>Type</th>
            <th>Source</th>
            <th>Title</th>
            <th>Year</th>
            <th>Artists</th>
            <th>Images</th>
            <th>Files</th>
            <th>Description</th>
            <th>Created</th>
          </tr>
        </thead>
        <tbody>
          {#each $docs.documents as doc (doc.id)}
            <tr>
              <td class="font-mono text-xs text-base-content/70">{doc.id}</td>
              <td class="text-xs">{doc.type}</td>
              <td class="text-xs">{doc.source}</td>
              <td class="font-medium">{doc.title}</td>
              <td class="text-xs">{doc.year ?? ""}</td>
              <td class="text-xs"
                >{(doc.artists ?? []).map((a) => a.name).join(", ")}</td
              >
              <td class="text-xs">{(doc.images ?? []).length}</td>
              <td class="text-xs">{(doc.files ?? []).length}</td>
              <td
                class="max-w-md text-xs whitespace-pre-wrap text-base-content/80"
                >{doc.description}</td
              >
              <td class="text-xs text-base-content/60"
                >{formatDate(doc.created_at)}</td
              >
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
