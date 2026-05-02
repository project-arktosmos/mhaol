<script lang="ts" module>
	export interface AttachmentInfo {
		title: string;
		seeders: number | null;
		leechers: number | null;
		sizeBytes: number | null;
	}
</script>

<script lang="ts">
	import classNames from 'classnames';
	import { Icon } from 'cloud-ui';
	import { formatSizeBytes } from '$lib/search.service';

	interface Props {
		/** The persisted `torrent magnet` (Download flow — long-lived in
		 * `<data_root>/downloads/torrents/`). `null` when nothing's attached. */
		download: AttachmentInfo | null;
		/** The persisted `torrent stream magnet` (Stream flow — ephemeral,
		 * `<data_root>/downloads/torrent-streams/`, wiped on next stream).
		 * `null` when nothing's attached. */
		stream: AttachmentInfo | null;
	}

	let { download, stream }: Props = $props();
</script>

{#snippet cell(info: AttachmentInfo | null, iconName: string, label: string, iconTitle: string)}
	<div
		class={classNames(
			'flex flex-col items-center gap-1 rounded-md border border-base-content/10 p-3 text-center',
			info ? 'bg-base-300/40 text-base-content' : 'text-base-content/40'
		)}
	>
		<Icon name={iconName} size={32} title={iconTitle} />
		<span class="text-xs font-medium">{label}</span>
		{#if info}
			<span class="block max-w-full truncate text-[10px] text-base-content/70" title={info.title}>
				{info.title}
			</span>
			<div class="flex items-center gap-1.5 text-[10px] text-base-content/60">
				<span class="text-success" title="Seeders">↑ {info.seeders ?? '—'}</span>
				<span class="text-warning" title="Leechers">↓ {info.leechers ?? '—'}</span>
				<span title="File size"
					>· {info.sizeBytes != null ? formatSizeBytes(info.sizeBytes) : '—'}</span
				>
			</div>
		{:else}
			<span class="text-[10px] text-base-content/60">Not attached</span>
		{/if}
	</div>
{/snippet}

<div class="card border border-base-content/10 bg-base-200 p-4">
	<h2 class="mb-3 text-sm font-semibold text-base-content/70 uppercase">Torrent attachment</h2>
	<div class="grid grid-cols-2 gap-3">
		{@render cell(download, 'delapouite/cloud-download', 'Download', 'Download mode')}
		{@render cell(stream, 'lorc/magnet', 'Stream', 'Stream mode')}
	</div>
</div>
