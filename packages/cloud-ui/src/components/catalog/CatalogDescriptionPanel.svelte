<script lang="ts">
	interface Props {
		description: string;
	}

	let { description }: Props = $props();
</script>

<div class="flex flex-col gap-2">
	<h2 class="text-sm font-semibold text-base-content/70 uppercase">Description</h2>
	{#if description}
		<p class="text-sm [overflow-wrap:anywhere] whitespace-pre-wrap">{description}</p>
	{:else}
		<p class="text-sm text-base-content/60 italic">No description.</p>
	{/if}
</div>
