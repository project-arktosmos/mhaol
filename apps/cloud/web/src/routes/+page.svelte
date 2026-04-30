<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import classNames from 'classnames';
	import HealthCard from '../components/HealthCard.svelte';
	import {
		cloudHealthService,
		type PackageHealth,
		type PackageHealthStatus
	} from '../lib/cloud-health.service';

	const state = cloudHealthService.state;

	const packageTone = (
		s: PackageHealthStatus | undefined
	): 'success' | 'warning' | 'error' | 'neutral' => {
		switch (s) {
			case 'ok':
				return 'success';
			case 'warning':
				return 'warning';
			case 'error':
				return 'error';
			default:
				return 'neutral';
		}
	};

	const packageValue = (pkg: PackageHealth | undefined): string => {
		if (!pkg) return '—';
		switch (pkg.status) {
			case 'ok':
				return 'Healthy';
			case 'warning':
				return 'Warning';
			case 'error':
				return 'Error';
			default:
				return 'Unavailable';
		}
	};

	const packageHint = (pkg: PackageHealth | undefined): string | null => {
		if (!pkg) return null;
		if (pkg.message) return pkg.message;
		const d = pkg.details ?? {};
		const parts: string[] = [];
		if (pkg.name === 'yt-dlp') {
			if (d.ytdlpVersion) parts.push(`v${d.ytdlpVersion}`);
			if (typeof d.active === 'number') parts.push(`${d.active} active`);
			if (typeof d.queued === 'number' && d.queued > 0) parts.push(`${d.queued} queued`);
		} else if (pkg.name === 'torrent') {
			if (typeof d.activeTorrents === 'number') parts.push(`${d.activeTorrents} active`);
		} else if (pkg.name === 'ed2k') {
			if (d.serverConnected && d.serverName) parts.push(`server: ${d.serverName}`);
			if (typeof d.activeFiles === 'number') parts.push(`${d.activeFiles} active`);
		} else if (pkg.name === 'ipfs') {
			if (typeof d.state === 'string') parts.push(String(d.state).toLowerCase());
			if (typeof d.connectedPeers === 'number') parts.push(`${d.connectedPeers} peers`);
			if (typeof d.pinnedCount === 'number') parts.push(`${d.pinnedCount} pinned`);
			if (d.privateNetwork === true) parts.push('private');
		} else if (pkg.name === 'p2p-stream') {
			parts.push(d.gstreamerInitialized ? 'GStreamer ready' : 'GStreamer offline');
		}
		return parts.length > 0 ? parts.join(' · ') : null;
	};

	onMount(() => {
		cloudHealthService.start(5000);
	});

	onDestroy(() => {
		cloudHealthService.stop();
	});

	const formatUptime = (seconds: number): string => {
		if (seconds < 60) return `${seconds}s`;
		const minutes = Math.floor(seconds / 60);
		if (minutes < 60) return `${minutes}m ${seconds % 60}s`;
		const hours = Math.floor(minutes / 60);
		if (hours < 24) return `${hours}h ${minutes % 60}m`;
		const days = Math.floor(hours / 24);
		return `${days}d ${hours % 24}h`;
	};

	const formatTime = (ms: number | null): string => {
		if (ms === null) return '—';
		try {
			return new Date(ms).toLocaleTimeString();
		} catch {
			return '—';
		}
	};

	const truncate = (value: string | null, lead: number = 6, tail: number = 4): string => {
		if (!value) return '—';
		if (value.length <= lead + tail + 1) return value;
		return `${value.slice(0, lead)}…${value.slice(-tail)}`;
	};

	const statusTone = $derived.by((): 'success' | 'error' | 'warning' => {
		const s = $state;
		if (s.loading && !s.status) return 'warning';
		if (!s.online) return 'error';
		return 'success';
	});

	const statusLabel = $derived.by((): string => {
		const s = $state;
		if (s.loading && !s.status) return 'Checking…';
		if (!s.online) return 'Offline';
		return 'Online';
	});
</script>

<svelte:head>
	<title>Mhaol Cloud — Health</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<header class="flex items-center justify-between gap-4">
		<div>
			<h1 class="text-2xl font-bold">Cloud Node Health</h1>
			<p class="text-sm text-base-content/60">
				Live status of this Mhaol cloud instance. Refreshes every 5 seconds.
			</p>
		</div>
		<div class="flex items-center gap-3">
			<div
				class={classNames('flex items-center gap-2 rounded-full border px-3 py-1.5 text-sm', {
					'border-success/40 bg-success/10 text-success': statusTone === 'success',
					'border-error/40 bg-error/10 text-error': statusTone === 'error',
					'border-warning/40 bg-warning/10 text-warning': statusTone === 'warning'
				})}
			>
				<span
					class={classNames('inline-block h-2 w-2 rounded-full', {
						'bg-success': statusTone === 'success',
						'bg-error': statusTone === 'error',
						'animate-pulse bg-warning': statusTone === 'warning'
					})}
				></span>
				<span class="font-medium">{statusLabel}</span>
			</div>
			<button
				class="btn btn-outline btn-sm"
				onclick={() => cloudHealthService.refresh()}
				disabled={$state.loading}
			>
				Refresh
			</button>
		</div>
	</header>

	{#if $state.error && !$state.status}
		<div class="alert alert-error">
			<span>Cannot reach the cloud node: {$state.error}</span>
		</div>
	{/if}

	<section class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
		<HealthCard
			label="Status"
			value={statusLabel}
			tone={statusTone}
			hint={$state.lastCheckedAt
				? `Last checked at ${formatTime($state.lastCheckedAt)}`
				: 'Awaiting first check'}
		/>
		<HealthCard
			label="Latency"
			value={$state.latencyMs !== null ? `${$state.latencyMs} ms` : '—'}
			hint="Round-trip to /api/cloud/status"
		/>
		<HealthCard
			label="Uptime"
			value={$state.status ? formatUptime($state.status.uptime_seconds) : '—'}
			hint={$state.status ? `Started at ${formatTime($state.status.started_at)}` : null}
		/>
		<HealthCard label="Version" value={$state.status?.version ?? '—'} hint="mhaol-cloud crate" />
		<HealthCard
			label="Bind"
			value={$state.status ? `${$state.status.host}:${$state.status.port}` : '—'}
			hint={$state.status?.local_ip ? `LAN: ${$state.status.local_ip}` : null}
			mono
		/>
		<HealthCard
			label="Public IP"
			value={$state.status?.public_ip ?? '—'}
			hint={$state.status?.public_ip ? 'Resolved via api.ipify.org' : 'Unavailable'}
			mono
		/>
		<HealthCard
			label="Database"
			value={$state.status?.db.engine ?? '—'}
			tone={$state.status?.db.connected === false ? 'error' : 'success'}
			hint={$state.status
				? `${$state.status.db.namespace}/${$state.status.db.database}${
						$state.status.db.version ? ` · ${$state.status.db.version}` : ''
					}`
				: null}
			mono
		/>
		<HealthCard
			label="Signaling wallet"
			value={truncate($state.status?.signaling_address ?? null)}
			hint={$state.status?.signaling_address ?? null}
			mono
		/>
	</section>

	{#if $state.status?.packages}
		<section class="flex flex-col gap-3">
			<h2 class="text-lg font-semibold">Packages</h2>
			<div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5">
				{#each [{ key: 'p2p-stream', pkg: $state.status.packages.p2pStream }, { key: 'yt-dlp', pkg: $state.status.packages.ytDlp }, { key: 'torrent', pkg: $state.status.packages.torrent }, { key: 'ed2k', pkg: $state.status.packages.ed2k }, { key: 'ipfs', pkg: $state.status.packages.ipfs }] as { key, pkg } (key)}
					<HealthCard
						label={key}
						value={packageValue(pkg)}
						tone={packageTone(pkg?.status)}
						hint={packageHint(pkg)}
					/>
				{/each}
			</div>
		</section>
	{/if}

	{#if $state.status?.client_address}
		<section class="card bg-base-200 p-4">
			<h2 class="mb-2 text-lg font-semibold">Identities</h2>
			<dl class="grid grid-cols-1 gap-2 sm:grid-cols-[max-content_1fr] sm:gap-x-4">
				<dt class="text-sm text-base-content/60">Signaling address</dt>
				<dd class="font-mono text-sm break-all">
					{$state.status.signaling_address ?? '—'}
				</dd>
				<dt class="text-sm text-base-content/60">Client address</dt>
				<dd class="font-mono text-sm break-all">
					{$state.status.client_address ?? '—'}
				</dd>
			</dl>
		</section>
	{/if}
</div>
