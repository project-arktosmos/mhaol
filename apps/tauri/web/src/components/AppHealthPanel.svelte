<script lang="ts">
	import classNames from 'classnames';
	import HealthCard from './HealthCard.svelte';
	import type { AppHealth } from '../lib/apps-health.service';

	interface Props {
		app: AppHealth;
	}

	let { app }: Props = $props();

	const tone = $derived.by((): 'success' | 'error' | 'warning' => {
		if (app.loading && app.lastCheckedAt === null) return 'warning';
		return app.online ? 'success' : 'error';
	});

	const statusLabel = $derived.by((): string => {
		if (app.loading && app.lastCheckedAt === null) return 'Checking…';
		return app.online ? 'Online' : 'Offline';
	});

	const formatTime = (ms: number | null): string => {
		if (ms === null) return '—';
		try {
			return new Date(ms).toLocaleTimeString();
		} catch {
			return '—';
		}
	};

	const formatUptime = (seconds: number): string => {
		if (seconds < 60) return `${seconds}s`;
		const minutes = Math.floor(seconds / 60);
		if (minutes < 60) return `${minutes}m ${seconds % 60}s`;
		const hours = Math.floor(minutes / 60);
		if (hours < 24) return `${hours}h ${minutes % 60}m`;
		const days = Math.floor(hours / 24);
		return `${days}d ${hours % 24}h`;
	};

	const cloudUptime = $derived.by((): string | null => {
		if (app.id !== 'cloud' || !app.info) return null;
		const seconds = app.info['uptime_seconds'];
		return typeof seconds === 'number' ? formatUptime(seconds) : null;
	});

	const cloudVersion = $derived.by((): string | null => {
		if (app.id !== 'cloud' || !app.info) return null;
		const v = app.info['version'];
		return typeof v === 'string' ? v : null;
	});

	const cloudBind = $derived.by((): string | null => {
		if (app.id !== 'cloud' || !app.info) return null;
		const host = app.info['host'];
		const port = app.info['port'];
		if (typeof host === 'string' && typeof port === 'number') return `${host}:${port}`;
		return null;
	});

	const open = (): void => {
		try {
			window.open(app.url, '_blank', 'noopener,noreferrer');
		} catch {
			// ignored
		}
	};
</script>

<section class="flex flex-col gap-3 rounded-box bg-base-200/40 p-4">
	<header class="flex items-center justify-between gap-3">
		<div class="flex items-center gap-3">
			<span
				class={classNames('inline-block h-2.5 w-2.5 rounded-full', {
					'bg-success': tone === 'success',
					'bg-error': tone === 'error',
					'animate-pulse bg-warning': tone === 'warning'
				})}
			></span>
			<div>
				<h2 class="text-lg font-bold">{app.label}</h2>
				<p class="font-mono text-xs text-base-content/60">{app.url}</p>
			</div>
		</div>
		<button class="btn btn-outline btn-sm" onclick={open}>Open</button>
	</header>

	<div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
		<HealthCard
			label="Status"
			value={statusLabel}
			{tone}
			hint={app.lastCheckedAt
				? `Last checked at ${formatTime(app.lastCheckedAt)}`
				: 'Awaiting first check'}
		/>
		<HealthCard
			label="Latency"
			value={app.latencyMs !== null ? `${app.latencyMs} ms` : '—'}
			hint={app.online ? 'Round-trip to health endpoint' : (app.error ?? 'Unreachable')}
		/>
		{#if app.id === 'cloud'}
			<HealthCard label="Uptime" value={cloudUptime ?? '—'} />
			<HealthCard label="Version" value={cloudVersion ?? '—'} hint={cloudBind} mono />
		{:else}
			<HealthCard label="Mode" value="Static SPA" hint="vite preview/build" />
			<HealthCard label="Port" value="9595" mono />
		{/if}
	</div>
</section>
