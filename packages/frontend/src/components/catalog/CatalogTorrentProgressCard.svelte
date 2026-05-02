<script lang="ts">
	import classNames from 'classnames';
	import {
		formatBytes,
		formatSpeed,
		formatEta,
		getStateColor,
		getStateLabel,
		type TorrentInfo
	} from '$types/torrent.type';

	interface Row {
		title: string | null;
		torrent: TorrentInfo;
	}

	interface Props {
		rows: Row[];
	}

	let { rows }: Props = $props();

	const badgeClass = (state: TorrentInfo['state']) => {
		const tone = getStateColor(state);
		return classNames('badge badge-sm', {
			'badge-info': tone === 'info',
			'badge-primary': tone === 'primary',
			'badge-success': tone === 'success',
			'badge-warning': tone === 'warning',
			'badge-error': tone === 'error',
			'badge-neutral': tone === 'neutral'
		});
	};

	const progressClass = (t: TorrentInfo) =>
		classNames('progress w-full', {
			'progress-success': t.state === 'seeding' || t.progress >= 1,
			'progress-primary': t.state === 'downloading' && t.progress < 1,
			'progress-warning': t.state === 'paused',
			'progress-error': t.state === 'error',
			'progress-info': t.state === 'initializing' || t.state === 'checking'
		});
</script>

{#if rows.length > 0}
	<section class="card border border-base-content/10 bg-base-200">
		<div class="card-body gap-4 p-4">
			<h2 class="card-title text-lg">Torrent activity</h2>
			<div class="flex flex-col gap-4">
				{#each rows as { title, torrent } (torrent.infoHash)}
					<div class="flex flex-col gap-2 rounded-md bg-base-100 p-3">
						<div class="flex items-start justify-between gap-3">
							<div class="min-w-0 flex-1">
								<p class="truncate font-medium" title={title ?? torrent.name}>
									{title ?? torrent.name}
								</p>
								<p class="font-mono text-xs break-all text-base-content/50">
									{torrent.infoHash}
								</p>
							</div>
							<span class={badgeClass(torrent.state)}>{getStateLabel(torrent.state)}</span>
						</div>
						<div class="flex items-center gap-2">
							<progress
								class={progressClass(torrent)}
								value={Math.max(0, Math.min(1, torrent.progress)) * 100}
								max="100"
							></progress>
							<span class="w-14 text-right text-xs text-base-content/60">
								{(Math.max(0, Math.min(1, torrent.progress)) * 100).toFixed(1)}%
							</span>
						</div>
						<dl class="grid grid-cols-2 gap-x-4 gap-y-1 text-xs sm:grid-cols-3 md:grid-cols-5">
							<div class="flex justify-between gap-2">
								<dt class="text-base-content/50">Size</dt>
								<dd class="font-mono">{formatBytes(torrent.size)}</dd>
							</div>
							<div class="flex justify-between gap-2">
								<dt class="text-base-content/50">↓</dt>
								<dd class="font-mono">{formatSpeed(torrent.downloadSpeed)}</dd>
							</div>
							<div class="flex justify-between gap-2">
								<dt class="text-base-content/50">↑</dt>
								<dd class="font-mono">{formatSpeed(torrent.uploadSpeed)}</dd>
							</div>
							<div class="flex justify-between gap-2">
								<dt class="text-base-content/50">Peers</dt>
								<dd class="font-mono">{torrent.peers} / {torrent.seeds}</dd>
							</div>
							<div class="flex justify-between gap-2">
								<dt class="text-base-content/50">ETA</dt>
								<dd class="font-mono">{formatEta(torrent.eta)}</dd>
							</div>
						</dl>
					</div>
				{/each}
			</div>
		</div>
	</section>
{/if}
