<script lang="ts">
	import { torrentService } from '$services/torrent.service';
	import { formatBytes, formatSpeed } from '$types/torrent.type';

	const state = torrentService.state;
</script>

{#if $state.stats}
	<div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
		<div class="rounded-lg bg-base-200 p-3 text-center">
			<p class="text-xs text-base-content/60">Download</p>
			<p class="text-lg font-bold text-primary">
				{formatSpeed($state.stats.downloadSpeed)}
			</p>
		</div>
		<div class="rounded-lg bg-base-200 p-3 text-center">
			<p class="text-xs text-base-content/60">Upload</p>
			<p class="text-lg font-bold text-secondary">
				{formatSpeed($state.stats.uploadSpeed)}
			</p>
		</div>
		<div class="rounded-lg bg-base-200 p-3 text-center">
			<p class="text-xs text-base-content/60">Downloaded</p>
			<p class="text-lg font-bold">
				{formatBytes($state.stats.totalDownloaded)}
			</p>
		</div>
		<div class="rounded-lg bg-base-200 p-3 text-center">
			<p class="text-xs text-base-content/60">Active</p>
			<p class="text-lg font-bold">{$state.stats.activeTorrents}</p>
		</div>
	</div>
{/if}
