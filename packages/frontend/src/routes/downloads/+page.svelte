<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import type { UnifiedDownload } from '$types/download.type';
	import { formatBytes } from '$types/torrent.type';

	let downloads = $state<UnifiedDownload[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);

	onMount(() => {
		loadDownloads();
	});

	async function loadDownloads() {
		loading = true;
		error = null;

		try {
			const res = await fetch('/api/downloads');
			if (!res.ok) throw new Error('Failed to load downloads');
			downloads = await res.json();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	function stateColor(state: string): string {
		switch (state) {
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

	function formatProgress(progress: number): string {
		return `${(progress * 100).toFixed(1)}%`;
	}

	function formatDate(dateStr: string): string {
		const date = new Date(dateStr);
		return date.toLocaleDateString(undefined, {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}
</script>

<div class="container mx-auto p-4">
	<div class="mb-6 flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold">Downloads</h1>
			<p class="text-sm opacity-70">All YouTube and torrent downloads</p>
		</div>
		<button class="btn btn-sm btn-ghost" onclick={loadDownloads} disabled={loading}>
			{#if loading}
				<span class="loading loading-spinner loading-xs"></span>
			{:else}
				Refresh
			{/if}
		</button>
	</div>

	{#if error}
		<div class="alert alert-error mb-4">
			<span>{error}</span>
			<button class="btn btn-ghost btn-sm" onclick={() => (error = null)}>x</button>
		</div>
	{/if}

	{#if loading && downloads.length === 0}
		<div class="flex justify-center py-12">
			<span class="loading loading-spinner loading-lg"></span>
		</div>
	{:else if downloads.length === 0}
		<div class="rounded-lg bg-base-200 p-8 text-center">
			<p class="opacity-50">No downloads yet.</p>
		</div>
	{:else}
		<div class="overflow-x-auto rounded-lg border border-base-300">
			<table class="table table-sm">
				<thead>
					<tr class="bg-base-200">
						<th>Type</th>
						<th>Name</th>
						<th>State</th>
						<th>Progress</th>
						<th>Size</th>
						<th>Directory</th>
						<th>Date</th>
					</tr>
				</thead>
				<tbody>
					{#each downloads as dl (dl.id)}
						<tr class="hover:bg-base-200/50">
							<td>
								<span
									class={classNames('badge badge-sm', {
										'badge-secondary': dl.type === 'youtube',
										'badge-accent': dl.type === 'torrent'
									})}
								>
									{dl.type}
								</span>
							</td>
							<td class="max-w-xs truncate" title={dl.name}>
								{dl.name}
							</td>
							<td>
								<span class={classNames('badge badge-sm', stateColor(dl.state))}>
									{dl.state}
								</span>
							</td>
							<td>
								{#if dl.progress >= 1}
									<span class="text-success">Done</span>
								{:else}
									<div class="flex items-center gap-2">
										<progress
											class="progress progress-primary w-16"
											value={dl.progress * 100}
											max="100"
										></progress>
										<span class="text-xs">{formatProgress(dl.progress)}</span>
									</div>
								{/if}
							</td>
							<td>
								{#if dl.size > 0}
									{formatBytes(dl.size)}
								{:else}
									<span class="opacity-40">--</span>
								{/if}
							</td>
							<td class="max-w-xs truncate text-xs opacity-70" title={dl.outputPath ?? ''}>
								{#if dl.outputPath}
									{dl.outputPath}
								{:else}
									<span class="opacity-40">--</span>
								{/if}
							</td>
							<td class="text-xs opacity-70">
								{formatDate(dl.updatedAt)}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		<div class="mt-2 text-right text-sm opacity-50">
			{downloads.length} download{downloads.length !== 1 ? 's' : ''}
		</div>
	{/if}
</div>
