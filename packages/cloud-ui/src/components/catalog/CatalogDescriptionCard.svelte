<script lang="ts">
	interface Props {
		description: string;
	}
	let { description }: Props = $props();
</script>

{#if description}
	<div class="card border border-base-content/10 bg-base-200 p-4">
		<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">Description</h2>
		<p class="text-sm [overflow-wrap:anywhere] whitespace-pre-wrap">{description}</p>
	</div>
{/if}
