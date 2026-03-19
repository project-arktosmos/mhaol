<script lang="ts">
	import classNames from 'classnames';

	let {
		page,
		totalPages,
		loading = false,
		onpage
	}: {
		page: number;
		totalPages: number;
		loading?: boolean;
		onpage: (page: number) => void;
	} = $props();

	let hasPrev = $derived(page > 1);
	let hasNext = $derived(page < totalPages);
</script>

{#if totalPages > 1}
	<div class="mt-4 flex items-center justify-center gap-2">
		<button
			class={classNames('btn btn-sm', { 'btn-disabled': !hasPrev || loading })}
			onclick={() => onpage(page - 1)}
			disabled={!hasPrev || loading}
		>
			Prev
		</button>
		<span class="text-sm opacity-70">
			Page {page} of {totalPages}
		</span>
		<button
			class={classNames('btn btn-sm', { 'btn-disabled': !hasNext || loading })}
			onclick={() => onpage(page + 1)}
			disabled={!hasNext || loading}
		>
			Next
		</button>
	</div>
{/if}
