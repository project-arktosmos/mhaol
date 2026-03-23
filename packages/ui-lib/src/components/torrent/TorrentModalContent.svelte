<script lang="ts">
	import { onMount } from 'svelte';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import TorrentAddInput from 'ui-lib/components/torrent/TorrentAddInput.svelte';
	import TorrentSearch from 'ui-lib/components/torrent/TorrentSearch.svelte';

	const state = torrentService.state;

	onMount(() => {
		torrentService.initialize();
	});
</script>

<!-- Error display -->
{#if $state.error}
	<div class="alert alert-error">
		<svg
			xmlns="http://www.w3.org/2000/svg"
			class="h-6 w-6 shrink-0 stroke-current"
			fill="none"
			viewBox="0 0 24 24"
		>
			<path
				stroke-linecap="round"
				stroke-linejoin="round"
				stroke-width="2"
				d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
			/>
		</svg>
		<span>{$state.error}</span>
		<button
			class="btn btn-ghost btn-sm"
			onclick={() => torrentService.state.update((s) => ({ ...s, error: null }))}
		>
			Dismiss
		</button>
	</div>
{/if}

<TorrentAddInput />

<div class="mt-4">
	<TorrentSearch />
</div>
