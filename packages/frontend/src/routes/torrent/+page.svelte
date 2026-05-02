<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import classNames from 'classnames';
	import HealthCard from '../../components/HealthCard.svelte';
	import { cloudHealthService } from '$lib/cloud-health.service';
	import {
		torrentStatusService,
		type TorrentInfo,
		type TorrentState
	} from '$lib/torrent-status.service';

	const healthState = cloudHealthService.state;
	const torrentState = torrentStatusService.state;

	onMount(() => {
		cloudHealthService.start(5000);
		torrentStatusService.start(5000);
	});

	onDestroy(() => {
		cloudHealthService.stop();
		torrentStatusService.stop();
	});

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const units = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(1024));
		const value = bytes / Math.pow(1024, i);
		return `${value.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
	}

	function formatSpeed(bytesPerSec: number): string {
		if (!bytesPerSec) return '0 B/s';
		return `${formatBytes(bytesPerSec)}/s`;
	}

	function formatEta(seconds: number | null): string {
		if (seconds === null || seconds <= 0) return '—';
		const hours = Math.floor(seconds / 3600);
		const minutes = Math.floor((seconds % 3600) / 60);
		const secs = Math.floor(seconds % 60);
		if (hours > 0) return `${hours}h ${minutes}m`;
		if (minutes > 0) return `${minutes}m ${secs}s`;
		return `${secs}s`;
	}

	function formatTime(ts: number): string {
		try {
			return new Date(ts * 1000).toLocaleString();
		} catch {
			return '—';
		}
	}

	function stateBadgeClass(state: TorrentState): string {
		return classNames('badge badge-sm', {
			'badge-info': state === 'initializing' || state === 'checking',
			'badge-primary': state === 'downloading',
			'badge-success': state === 'seeding',
			'badge-warning': state === 'paused',
			'badge-error': state === 'error'
		});
	}

	function progressBarClass(t: TorrentInfo): string {
		return classNames('progress w-full', {
			'progress-success': t.state === 'seeding' || t.progress >= 1,
			'progress-primary': t.state === 'downloading' && t.progress < 1,
			'progress-warning': t.state === 'paused',
			'progress-error': t.state === 'error',
			'progress-info': t.state === 'initializing' || t.state === 'checking'
		});
	}

	const torrentPkg = $derived($healthState.status?.packages?.torrent ?? null);

	const clientStatusLabel = $derived.by((): string => {
		const pkg = torrentPkg;
		if (!pkg) return $healthState.loading ? 'Checking…' : 'Unknown';
		switch (pkg.status) {
			case 'ok':
				return 'Ready';
			case 'warning':
				return pkg.message ?? 'Warming up';
			case 'error':
				return 'Error';
			default:
				return 'Unavailable';
		}
	});

	const clientStatusTone = $derived.by((): 'success' | 'warning' | 'error' | 'neutral' => {
		const pkg = torrentPkg;
		if (!pkg) return 'neutral';
		if (pkg.status === 'ok') return 'success';
		if (pkg.status === 'warning') return 'warning';
		if (pkg.status === 'error') return 'error';
		return 'neutral';
	});

	const details = $derived((torrentPkg?.details ?? {}) as Record<string, unknown>);

	const activeCount = $derived(
		typeof details.activeTorrents === 'number' ? (details.activeTorrents as number) : null
	);
	const downloadSpeed = $derived(
		typeof details.downloadSpeed === 'number' ? (details.downloadSpeed as number) : null
	);
	const uploadSpeed = $derived(
		typeof details.uploadSpeed === 'number' ? (details.uploadSpeed as number) : null
	);
	const totalDownloaded = $derived(
		typeof details.totalDownloaded === 'number' ? (details.totalDownloaded as number) : null
	);
	const totalUploaded = $derived(
		typeof details.totalUploaded === 'number' ? (details.totalUploaded as number) : null
	);

	function refreshAll() {
		cloudHealthService.refresh();
		torrentStatusService.refresh();
	}
</script>

<svelte:head>
	<title>Mhaol Cloud — Torrent</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<header class="flex items-center justify-between gap-4">
		<div>
			<h1 class="text-2xl font-bold">Torrent Client</h1>
			<p class="text-sm text-base-content/60">
				Live status of the cloud's embedded librqbit torrent session. Refreshes every 5 seconds.
			</p>
		</div>
		<button
			class="btn btn-outline btn-sm"
			onclick={refreshAll}
			disabled={$healthState.loading || $torrentState.loading}
		>
			Refresh
		</button>
	</header>

	{#if $healthState.error && !$healthState.status}
		<div class="alert alert-error">
			<span>Cannot reach the cloud node: {$healthState.error}</span>
		</div>
	{/if}

	{#if $torrentState.error}
		<div class="alert alert-error">
			<span>{$torrentState.error}</span>
		</div>
	{/if}

	<section class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5">
		<HealthCard
			label="Client"
			value={clientStatusLabel}
			tone={clientStatusTone}
			hint={torrentPkg?.message ?? null}
		/>
		<HealthCard label="Active torrents" value={activeCount !== null ? String(activeCount) : '—'} />
		<HealthCard
			label="Download"
			value={downloadSpeed !== null ? formatSpeed(downloadSpeed) : '—'}
			hint={totalDownloaded !== null ? `Total: ${formatBytes(totalDownloaded)}` : null}
		/>
		<HealthCard
			label="Upload"
			value={uploadSpeed !== null ? formatSpeed(uploadSpeed) : '—'}
			hint={totalUploaded !== null ? `Total: ${formatBytes(totalUploaded)}` : null}
		/>
		<HealthCard
			label="In list"
			value={String($torrentState.torrents.length)}
			hint={$torrentState.lastCheckedAt
				? `Updated ${new Date($torrentState.lastCheckedAt).toLocaleTimeString()}`
				: null}
		/>
	</section>

	<section class="flex flex-col gap-3">
		<h2 class="text-lg font-semibold">Torrents</h2>

		{#if $torrentState.loading && $torrentState.torrents.length === 0}
			<p class="text-sm text-base-content/60">Loading…</p>
		{:else if $torrentState.torrents.length === 0}
			<p class="text-sm text-base-content/60">
				No torrents in the session. Add one via the catalog detail page or the firkin bookmark flow.
			</p>
		{:else}
			<div class="overflow-x-auto rounded-box border border-base-content/10">
				<table class="table table-sm">
					<thead>
						<tr>
							<th>Name</th>
							<th class="w-24">State</th>
							<th class="w-64">Progress</th>
							<th class="w-24 text-right">Size</th>
							<th class="w-28 text-right">↓ Speed</th>
							<th class="w-28 text-right">↑ Speed</th>
							<th class="w-20 text-right">Peers</th>
							<th class="w-20 text-right">Seeds</th>
							<th class="w-20 text-right">ETA</th>
							<th class="w-44">Added</th>
						</tr>
					</thead>
					<tbody>
						{#each $torrentState.torrents as t (t.infoHash)}
							<tr>
								<td>
									<div class="flex flex-col gap-0.5">
										<span class="font-medium break-all">{t.name}</span>
										<span class="font-mono text-xs break-all text-base-content/50">
											{t.infoHash}
										</span>
										{#if t.outputPath}
											<span class="font-mono text-xs break-all text-base-content/40">
												{t.outputPath}
											</span>
										{/if}
									</div>
								</td>
								<td>
									<span class={stateBadgeClass(t.state)}>{t.state}</span>
								</td>
								<td>
									<div class="flex flex-col gap-1">
										<progress
											class={progressBarClass(t)}
											value={Math.max(0, Math.min(1, t.progress)) * 100}
											max="100"
										></progress>
										<span class="text-xs text-base-content/60">
											{(Math.max(0, Math.min(1, t.progress)) * 100).toFixed(1)}%
										</span>
									</div>
								</td>
								<td class="text-right text-xs">{formatBytes(t.size)}</td>
								<td class="text-right font-mono text-xs">{formatSpeed(t.downloadSpeed)}</td>
								<td class="text-right font-mono text-xs">{formatSpeed(t.uploadSpeed)}</td>
								<td class="text-right text-xs">{t.peers}</td>
								<td class="text-right text-xs">{t.seeds}</td>
								<td class="text-right text-xs">{formatEta(t.eta)}</td>
								<td class="text-xs text-base-content/60">{formatTime(t.addedAt)}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</section>
</div>
