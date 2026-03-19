<script lang="ts">
	import { torrentService } from 'frontend/services/torrent.service';
	import { torrentSearchService } from 'frontend/services/torrent-search.service';
	import TorrentAddInput from 'ui-lib/components/torrent/TorrentAddInput.svelte';
	import TorrentStats from 'ui-lib/components/torrent/TorrentStats.svelte';
	import TorrentList from 'ui-lib/components/torrent/TorrentList.svelte';
	import TorrentSearch from 'ui-lib/components/torrent/TorrentSearch.svelte';

	const torrentState = torrentService.state;
	const searchState = torrentSearchService.state;
	const hasResults = $derived($searchState.results.length > 0);
</script>

<div class="p-6">
	{#if !$torrentState.initialized && $torrentState.loading}
		<div class="mb-6 flex justify-end">
			<span class="loading loading-md loading-spinner"></span>
		</div>
	{/if}

	{#if $torrentState.error}
		<div class="mb-4 alert alert-error">
			<span>{$torrentState.error}</span>
			<button
				class="btn btn-ghost btn-sm"
				onclick={() => torrentService.state.update((s) => ({ ...s, error: null }))}
			>
				Dismiss
			</button>
		</div>
	{/if}

	<div class="mb-6">
		<TorrentStats />
	</div>

	<div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
		<div class={hasResults ? 'lg:col-span-2' : ''}>
			<TorrentSearch />
			<div class="mt-6">
				<TorrentAddInput />
			</div>
		</div>

		<div class={hasResults ? '' : 'lg:col-span-2'}>
			<TorrentList />
		</div>
	</div>
</div>
