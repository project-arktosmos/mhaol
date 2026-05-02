<script lang="ts">
	// The load function in `+page.ts` always redirects to /catalog/[id],
	// so this page is only ever visible for a frame while the POST to
	// /api/firkins is in flight.
</script>

<svelte:head>
	<title>Mhaol Cloud — Loading…</title>
</svelte:head>

<div class="flex min-h-full items-center justify-center p-6">
	<div class="flex items-center gap-3 text-base-content/60">
		<span class="loading loading-spinner"></span>
		<span class="text-sm">Opening catalog item…</span>
	</div>
</div>
