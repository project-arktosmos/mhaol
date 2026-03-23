<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		children: Snippet;
		cellA?: Snippet;
		cellB?: Snippet;
	}

	let { children, cellA, cellB }: Props = $props();
</script>

<div class="flex min-w-0 flex-1 flex-col overflow-y-auto">
	<div class="grid w-full grid-cols-3 gap-4 p-4">
		<div class="flex flex-col gap-4">
			{@render children()}
		</div>
		<div class="flex flex-col gap-4">
			{#if cellA}
				{@render cellA()}
			{:else}
				<div class="flex items-start justify-center pt-4 text-2xl font-bold opacity-40">A</div>
			{/if}
		</div>
		<div class="flex flex-col gap-4">
			{#if cellB}
				{@render cellB()}
			{:else}
				<div class="flex items-start justify-center pt-4 text-2xl font-bold opacity-40">B</div>
			{/if}
		</div>
	</div>
</div>
