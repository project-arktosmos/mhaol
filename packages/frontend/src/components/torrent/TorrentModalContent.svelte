<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { torrentService } from '$services/torrent.service';
	import { libraryService } from '$services/library.service';
	import TorrentAddInput from '$components/torrent/TorrentAddInput.svelte';
	import TorrentSettings from '$components/torrent/TorrentSettings.svelte';
	import TorrentStats from '$components/torrent/TorrentStats.svelte';
	import TorrentList from '$components/torrent/TorrentList.svelte';
	import TorrentSearch from '$components/torrent/TorrentSearch.svelte';

	const state = torrentService.state;

	onMount(() => {
		Promise.all([torrentService.initialize(), libraryService.initialize()]);
	});

	onDestroy(() => {
		torrentService.destroy();
	});
</script>

<!-- Header -->
<div class="flex items-center justify-between pr-8">
	<div>
		<h3 class="text-lg font-bold">Torrent Manager</h3>
		<p class="text-sm text-base-content/60">Download and manage torrents via magnet links</p>
	</div>
	{#if !$state.initialized && $state.loading}
		<span class="loading loading-md loading-spinner"></span>
	{/if}
</div>

<!-- Error display -->
{#if $state.error}
	<div class="mt-4 alert alert-error">
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

<!-- Torrent search -->
<div class="mt-4">
	<TorrentSearch />
</div>

<div class="mt-6 grid grid-cols-1 gap-6 lg:grid-cols-3">
	<!-- Left column: Add input and settings -->
	<div class="flex flex-col gap-4 lg:col-span-1">
		<TorrentAddInput />
		<TorrentSettings />
	</div>

	<!-- Right column: Stats and list -->
	<div class="flex flex-col gap-4 lg:col-span-2">
		<TorrentStats />
		<TorrentList />
	</div>
</div>
