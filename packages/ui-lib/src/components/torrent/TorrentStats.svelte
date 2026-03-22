<script lang="ts">
	import { torrentService } from 'ui-lib/services/torrent.service';
	import { formatBytes, formatSpeed } from 'ui-lib/types/torrent.type';

	const torrentState = torrentService.state;
</script>

{#if $torrentState.stats}
	<div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
		<div class="rounded-lg bg-base-200 p-3 text-center">
			<p class="text-xs text-base-content/60">Download</p>
			<p class="text-lg font-bold text-primary">
				{formatSpeed($torrentState.stats.downloadSpeed)}
			</p>
		</div>
		<div class="rounded-lg bg-base-200 p-3 text-center">
			<p class="text-xs text-base-content/60">Upload</p>
			<p class="text-lg font-bold text-secondary">
				{formatSpeed($torrentState.stats.uploadSpeed)}
			</p>
		</div>
		<div class="rounded-lg bg-base-200 p-3 text-center">
			<p class="text-xs text-base-content/60">Downloaded</p>
			<p class="text-lg font-bold">
				{formatBytes($torrentState.stats.totalDownloaded)}
			</p>
		</div>
		<div class="rounded-lg bg-base-200 p-3 text-center">
			<p class="text-xs text-base-content/60">Active</p>
			<p class="text-lg font-bold">{$torrentState.stats.activeTorrents}</p>
		</div>
	</div>
{/if}
