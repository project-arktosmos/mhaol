<script lang="ts">
	interface Props {
		description: string;
	}

	let { description }: Props = $props();
</script>

<div class="card border border-base-content/10 bg-base-200 p-4">
	<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">Description</h2>
	{#if description}
		<p class="text-sm [overflow-wrap:anywhere] whitespace-pre-wrap">{description}</p>
	{:else}
		<p class="text-sm text-base-content/60 italic">No description.</p>
	{/if}
</div>
