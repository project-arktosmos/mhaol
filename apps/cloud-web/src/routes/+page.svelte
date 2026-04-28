<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import classNames from 'classnames';
	import HealthCard from '../components/HealthCard.svelte';
	import { cloudHealthService } from '../lib/cloud-health.service';

	const state = cloudHealthService.state;

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
