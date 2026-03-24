<script lang="ts">
	import { torrentService } from 'ui-lib/services/torrent.service';
	import { playerService } from 'ui-lib/services/player.service';
	import TorrentListItem from './TorrentListItem.svelte';

	const torrentState = torrentService.state;

	function handlePause(infoHash: string) {
		torrentService.pauseTorrent(infoHash);
	}

	function handleResume(infoHash: string) {
		torrentService.resumeTorrent(infoHash);
	}

	function handleRemove(infoHash: string) {
		torrentService.removeTorrent(infoHash);
	}

	function handleRemoveAll() {
		torrentService.removeAll();
	}

	function handleStream(infoHash: string) {
		const torrent = $torrentState.torrents.find((t) => t.infoHash === infoHash);
		if (!torrent) return;

		playerService.play({
			id: `torrent:${torrent.infoHash}`,
			type: 'torrent',
			name: torrent.name,
			outputPath: torrent.outputPath ?? '',
			mode: 'video',
			format: null,
			videoFormat: null,
			thumbnailUrl: null,
			durationSeconds: null,
			size: torrent.size,
			completedAt: '',
		});
	}
</script>

<div class="card bg-base-200">
	<div class="card-body">
		<div class="flex items-center justify-between">
			<h2 class="card-title text-lg">Torrents</h2>
			{#if $torrentState.torrents.length > 0}
				<button class="btn btn-ghost btn-sm" onclick={handleRemoveAll}> Remove All </button>
			{/if}
		</div>

		{#if $torrentState.torrents.length === 0}
			<div class="py-8 text-center text-base-content/50">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="mx-auto h-12 w-12 opacity-50"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10"
					/>
				</svg>
				<p class="mt-2">No torrents</p>
				<p class="text-sm">Add a magnet link to get started</p>
			</div>
		{:else}
			<div class="flex flex-col gap-3">
				{#each $torrentState.torrents as torrent (torrent.infoHash)}
					<TorrentListItem
						{torrent}
						onpause={handlePause}
						onresume={handleResume}
						onremove={handleRemove}
						onstream={handleStream}
					/>
				{/each}
			</div>
		{/if}
	</div>
</div>
