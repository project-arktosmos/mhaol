<script lang="ts">
	import { torrentService } from 'frontend/services/torrent.service';
	import TorrentAddInput from 'ui-lib/components/torrent/TorrentAddInput.svelte';
	import TorrentSettings from 'ui-lib/components/torrent/TorrentSettings.svelte';
	import TorrentStats from 'ui-lib/components/torrent/TorrentStats.svelte';
	import TorrentList from 'ui-lib/components/torrent/TorrentList.svelte';
	import TorrentSearch from 'ui-lib/components/torrent/TorrentSearch.svelte';

	const state = torrentService.state;
</script>

<div class="p-6">
	<div class="mb-6 flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Torrent Manager</h1>
			<p class="text-sm text-base-content/60">Download and manage torrents via magnet links</p>
		</div>
		{#if !$state.initialized && $state.loading}
			<span class="loading loading-md loading-spinner"></span>
		{/if}
	</div>

	{#if $state.error}
		<div class="mb-4 alert alert-error">
			<span>{$state.error}</span>
			<button
				class="btn btn-ghost btn-sm"
				onclick={() => torrentService.state.update((s) => ({ ...s, error: null }))}
			>
				Dismiss
			</button>
		</div>
	{/if}

	<div class="mb-6">
		<TorrentSearch />
	</div>

	<div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
		<div class="flex flex-col gap-4 lg:col-span-1">
			<TorrentAddInput />
			<TorrentSettings />
		</div>
		<div class="flex flex-col gap-4 lg:col-span-2">
			<TorrentStats />
			<TorrentList />
		</div>
	</div>
</div>
