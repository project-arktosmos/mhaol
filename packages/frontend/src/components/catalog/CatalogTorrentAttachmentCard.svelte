<script lang="ts">
	import classNames from 'classnames';
	import { Icon } from 'cloud-ui';

	interface Props {
		/** A `torrent magnet` is attached to the firkin's `files` — the
		 * Download flow, where the torrent client persists bytes to
		 * `<data_root>/downloads/torrents/`. */
		downloadAttached: boolean;
		/** A torrent stream session is currently active for this firkin —
		 * the Stream flow, where the torrent client writes to
		 * `<data_root>/downloads/torrent-streams/` and the inline player is
		 * pulling bytes off it. */
		streamActive: boolean;
	}

	let { downloadAttached, streamActive }: Props = $props();
</script>

<div class="card border border-base-content/10 bg-base-200 p-4">
	<h2 class="mb-3 text-sm font-semibold text-base-content/70 uppercase">Torrent attachment</h2>
	<div class="grid grid-cols-2 gap-3">
		<div
			class={classNames(
				'flex flex-col items-center gap-1 rounded-md border border-base-content/10 p-3 text-center',
				downloadAttached ? 'bg-base-300/40 text-base-content' : 'text-base-content/40'
			)}
		>
			<Icon name="delapouite/cloud-download" size={32} title="Download mode" />
			<span class="text-xs font-medium">Download</span>
			<span class="text-[10px] text-base-content/60">
				{downloadAttached ? 'Magnet attached' : 'Not attached'}
			</span>
		</div>
		<div
			class={classNames(
				'flex flex-col items-center gap-1 rounded-md border border-base-content/10 p-3 text-center',
				streamActive ? 'bg-base-300/40 text-base-content' : 'text-base-content/40'
			)}
		>
			<Icon name="lorc/magnet" size={32} title="Stream mode" />
			<span class="text-xs font-medium">Stream</span>
			<span class="text-[10px] text-base-content/60">
				{streamActive ? 'Streaming now' : 'Inactive'}
			</span>
		</div>
	</div>
</div>
