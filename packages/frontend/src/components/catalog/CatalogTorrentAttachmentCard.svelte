<script lang="ts">
	import classNames from 'classnames';
	import { Icon } from 'cloud-ui';

	interface Props {
		/** A `torrent magnet` is persisted on the firkin's `files` — the
		 * Download flow, where the torrent client persists bytes to
		 * `<data_root>/downloads/torrents/`. */
		downloadAttached: boolean;
		/** A `torrent stream magnet` is persisted on the firkin's `files` —
		 * the Stream flow, where the torrent client writes to the ephemeral
		 * `<data_root>/downloads/torrent-streams/` (wiped on each new
		 * stream). The persisted magnet records the user's most-recent
		 * stream pick so the panel survives reloads. */
		streamAttached: boolean;
	}

	let { downloadAttached, streamAttached }: Props = $props();
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
				streamAttached ? 'bg-base-300/40 text-base-content' : 'text-base-content/40'
			)}
		>
			<Icon name="lorc/magnet" size={32} title="Stream mode" />
			<span class="text-xs font-medium">Stream</span>
			<span class="text-[10px] text-base-content/60">
				{streamAttached ? 'Magnet attached' : 'Not attached'}
			</span>
		</div>
	</div>
</div>
