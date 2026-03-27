<script lang="ts">
	import classNames from 'classnames';
	import { websurfxService } from 'ui-lib/services/websurfx.service';
	import WebSurfxResultCard from 'ui-lib/components/websurfx/WebSurfxResultCard.svelte';

	const searchState = websurfxService.state;

	let inputValue = $state('');

	function handleSearch() {
		const q = inputValue.trim();
		if (q && !$searchState.searching) {
			websurfxService.search(q);
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			handleSearch();
		}
	}

	function handleClear() {
		inputValue = '';
		websurfxService.clearResults();
	}

	let buttonClasses = $derived(
		classNames('btn btn-primary join-item', {
			'btn-disabled': !inputValue.trim() || $searchState.searching
		})
	);
</script>

<div class="flex h-full flex-col gap-4 p-4">
	<div class="flex items-center justify-between">
		<p class="text-sm text-base-content/60">Search the web using DuckDuckGo</p>
		{#if $searchState.results.length > 0}
			<button class="btn btn-ghost btn-sm" onclick={handleClear}>Clear</button>
		{/if}
	</div>

	<div class="join w-full">
		<input
			type="text"
			bind:value={inputValue}
			onkeydown={handleKeydown}
			placeholder="Search the web..."
			class="input input-bordered join-item flex-1"
		/>
		<button
			class={buttonClasses}
			onclick={handleSearch}
			disabled={!inputValue.trim() || $searchState.searching}
		>
			{#if $searchState.searching}
				<span class="loading loading-sm loading-spinner"></span>
			{:else}
				Search
			{/if}
		</button>
	</div>

	{#if $searchState.error}
		<div class="alert alert-error">
			<span>{$searchState.error}</span>
			<button
				class="btn btn-ghost btn-sm"
				onclick={() => websurfxService.state.update((s) => ({ ...s, error: null }))}
			>
				Dismiss
			</button>
		</div>
	{/if}

	{#if $searchState.searching}
		<div class="flex justify-center py-8">
			<span class="loading loading-lg loading-spinner"></span>
		</div>
	{:else if $searchState.results.length > 0}
		<div class="flex flex-col gap-3">
			{#each $searchState.results as result (result.url)}
				<WebSurfxResultCard {result} />
			{/each}
		</div>
	{:else if $searchState.query && !$searchState.searching}
		<div class="flex flex-col items-center gap-2 py-8 text-base-content/50">
			<p class="text-sm">No results found for "{$searchState.query}"</p>
		</div>
	{/if}
</div>
