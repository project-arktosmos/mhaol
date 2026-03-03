<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import classNames from 'classnames';
	import { downloadsService } from '$services/downloads.service';
	import { modalRouterService } from '$services/modal-router.service';
	import { formatBytes } from '$types/torrent.type';

	const downloadState = downloadsService.state;

	const ACTIVE_STATES = new Set([
		'downloading',
		'processing',
		'pending',
		'initializing',
		'checking'
	]);

	let activeDownloads = $derived(
		$downloadState.downloads.filter((dl) => ACTIVE_STATES.has(dl.state))
	);
	let completedCount = $derived(
		$downloadState.downloads.filter((dl) => dl.state === 'completed' || dl.state === 'seeding')
			.length
	);
	let errorCount = $derived(
		$downloadState.downloads.filter(
			(dl) => dl.state === 'error' || dl.state === 'failed' || dl.state === 'cancelled'
		).length
	);

	function stateColor(s: string): string {
		switch (s) {
			case 'completed':
			case 'seeding':
				return 'badge-success';
			case 'downloading':
			case 'processing':
				return 'badge-primary';
			case 'pending':
			case 'initializing':
			case 'checking':
				return 'badge-info';
			case 'paused':
				return 'badge-warning';
			case 'error':
			case 'failed':
			case 'cancelled':
				return 'badge-error';
			default:
				return 'badge-neutral';
		}
	}

	onMount(() => {
		downloadsService.startPolling();
	});

	onDestroy(() => {
		downloadsService.stopPolling();
	});
</script>

<div class="mb-2 flex items-center justify-between">
	<h2 class="text-sm font-semibold uppercase tracking-wide text-base-content/50">Downloads</h2>
	<button
		class="btn btn-ghost btn-xs text-base-content/50"
		onclick={() => modalRouterService.openNavbar('downloads')}
	>
		View All
	</button>
</div>

{#if $downloadState.loading && $downloadState.downloads.length === 0}
	<div class="flex justify-center py-4">
		<span class="loading loading-spinner loading-sm"></span>
	</div>
{:else if $downloadState.error && $downloadState.downloads.length === 0}
	<p class="text-xs text-error">{$downloadState.error}</p>
{:else}
	<div class="grid grid-cols-2 gap-2">
		<div class="rounded-lg bg-base-100 p-2 text-center">
			<p class="text-xs text-base-content/60">Total</p>
			<p class="text-lg font-bold">{$downloadState.downloads.length}</p>
		</div>
		<div class="rounded-lg bg-base-100 p-2 text-center">
			<p class="text-xs text-base-content/60">Active</p>
			<p class="text-lg font-bold text-primary">{activeDownloads.length}</p>
		</div>
		<div class="rounded-lg bg-base-100 p-2 text-center">
			<p class="text-xs text-base-content/60">Done</p>
			<p class="text-lg font-bold text-success">{completedCount}</p>
		</div>
		<div class="rounded-lg bg-base-100 p-2 text-center">
			<p class="text-xs text-base-content/60">Errors</p>
			<p class={classNames('text-lg font-bold', { 'text-error': errorCount > 0 })}>
				{errorCount}
			</p>
		</div>
	</div>

	{#if activeDownloads.length > 0}
		<div class="mt-3 flex flex-col gap-2">
			{#each activeDownloads as dl (dl.id)}
				<div class="rounded-lg bg-base-100 p-2">
					<div class="mb-1 flex items-center justify-between gap-2">
						<span class="min-w-0 truncate text-xs" title={dl.name}>{dl.name}</span>
						<span class={classNames('badge badge-xs shrink-0', stateColor(dl.state))}>
							{dl.state}
						</span>
					</div>
					<div class="flex items-center gap-2">
						<progress
							class="progress progress-primary h-1.5 flex-1"
							value={dl.progress * 100}
							max="100"
						></progress>
						<span class="shrink-0 text-xs text-base-content/60">
							{(dl.progress * 100).toFixed(1)}%
						</span>
					</div>
					{#if dl.size > 0}
						<p class="mt-0.5 text-right text-xs text-base-content/40">
							{formatBytes(dl.size)}
						</p>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
{/if}
