<script lang="ts">
	import classNames from 'classnames';

	let {
		query = '',
		searching = false,
		onsearch
	}: {
		query?: string;
		searching?: boolean;
		onsearch?: (query: string) => void;
	} = $props();

	let inputValue = $state(query);

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && inputValue.trim() && !searching) {
			onsearch?.(inputValue.trim());
		}
	}

	let buttonClasses = $derived(
		classNames('btn btn-primary join-item', {
			'btn-disabled': !inputValue.trim() || searching
		})
	);
</script>

<div class="join w-full">
	<input
		type="text"
		bind:value={inputValue}
		onkeydown={handleKeydown}
		placeholder="Search YouTube videos..."
		class="input-bordered input join-item flex-1"
	/>
	<button
		class={buttonClasses}
		onclick={() => onsearch?.(inputValue.trim())}
		disabled={!inputValue.trim() || searching}
	>
		{#if searching}
			<span class="loading loading-sm loading-spinner"></span>
		{:else}
			Search
		{/if}
	</button>
</div>
