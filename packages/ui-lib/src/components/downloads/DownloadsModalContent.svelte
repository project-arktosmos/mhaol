<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import classNames from 'classnames';
	import { downloadsService } from 'ui-lib/services/downloads.service';
	import type { UnifiedDownload } from 'ui-lib/types/download.type';
	import { formatBytes } from 'ui-lib/types/torrent.type';

	const downloadState = downloadsService.state;

	onMount(() => {
		downloadsService.startPolling();
	});

	onDestroy(() => {
		downloadsService.stopPolling();
	});

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

	const PAUSABLE_STATES = new Set(['downloading', 'checking', 'initializing']);

	function canPause(dl: UnifiedDownload): boolean {
		return dl.type === 'torrent' && PAUSABLE_STATES.has(dl.state);
	}

	function canResume(dl: UnifiedDownload): boolean {
		return dl.type === 'torrent' && dl.state === 'paused';
	}

	async function pauseDownload(dl: UnifiedDownload) {
		await downloadsService.pauseDownload(dl);
	}

	async function resumeDownload(dl: UnifiedDownload) {
		await downloadsService.resumeDownload(dl);
	}

	async function removeDownload(dl: UnifiedDownload) {
		await downloadsService.removeDownload(dl, false);
	}

	let confirmDeleteId = $state<string | null>(null);

	async function removeWithFiles(dl: UnifiedDownload) {
		confirmDeleteId = null;
		await downloadsService.removeDownload(dl, true);
	}
</script>

<div class="mb-4">
	<h3 class="text-lg font-bold">Downloads</h3>
	<p class="text-sm opacity-70">All YouTube and torrent downloads</p>
</div>

{#if $downloadState.loading && $downloadState.downloads.length === 0}
	<div class="flex justify-center py-12">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else if $downloadState.error && $downloadState.downloads.length === 0}
	<div class="mb-4 alert alert-error">
		<span>{$downloadState.error}</span>
	</div>
{:else if $downloadState.downloads.length === 0}
	<div class="rounded-lg bg-base-200 p-8 text-center">
		<p class="opacity-50">No downloads yet.</p>
	</div>
{:else}
	<div class="max-h-[60vh] overflow-x-auto overflow-y-auto rounded-lg border border-base-300">
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
					<th>Actions</th>
				</tr>
			</thead>
			<tbody>
				{#each $downloadState.downloads as dl (dl.id)}
					<tr class="hover:bg-base-200/50">
						<td>
							<span
								class={classNames('badge badge-sm', {
									'badge-secondary': dl.type.startsWith('youtube'),
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
										class="progress w-16 progress-primary"
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
						<td>
							<div class="flex gap-1">
								{#if canPause(dl)}
									<button
										class="btn btn-ghost btn-xs"
										title="Pause"
										onclick={() => pauseDownload(dl)}
									>
										<svg
											xmlns="http://www.w3.org/2000/svg"
											class="h-3.5 w-3.5"
											viewBox="0 0 20 20"
											fill="currentColor"
										>
											<path
												fill-rule="evenodd"
												d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zM7 8a1 1 0 012 0v4a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v4a1 1 0 102 0V8a1 1 0 00-1-1z"
												clip-rule="evenodd"
											/>
										</svg>
									</button>
								{:else if canResume(dl)}
									<button
										class="btn btn-ghost btn-xs"
										title="Resume"
										onclick={() => resumeDownload(dl)}
									>
										<svg
											xmlns="http://www.w3.org/2000/svg"
											class="h-3.5 w-3.5"
											viewBox="0 0 20 20"
											fill="currentColor"
										>
											<path
												fill-rule="evenodd"
												d="M10 18a8 8 0 100-16 8 8 0 000 16zM9.555 7.168A1 1 0 008 8v4a1 1 0 001.555.832l3-2a1 1 0 000-1.664l-3-2z"
												clip-rule="evenodd"
											/>
										</svg>
									</button>
								{/if}
								<button
									class="btn btn-ghost btn-xs"
									title="Remove"
									onclick={() => removeDownload(dl)}
								>
									<svg
										xmlns="http://www.w3.org/2000/svg"
										class="h-3.5 w-3.5"
										viewBox="0 0 20 20"
										fill="currentColor"
									>
										<path
											fill-rule="evenodd"
											d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
											clip-rule="evenodd"
										/>
									</svg>
								</button>
								{#if dl.type === 'torrent'}
									{#if confirmDeleteId === dl.id}
										<span class="text-xs text-error">Delete files?</span>
										<button class="btn btn-xs btn-error" onclick={() => removeWithFiles(dl)}>
											Yes
										</button>
										<button class="btn btn-ghost btn-xs" onclick={() => (confirmDeleteId = null)}>
											No
										</button>
									{:else}
										<button
											class="btn text-error btn-ghost btn-xs"
											title="Remove and delete files"
											onclick={() => (confirmDeleteId = dl.id)}
										>
											<svg
												xmlns="http://www.w3.org/2000/svg"
												class="h-3.5 w-3.5"
												viewBox="0 0 20 20"
												fill="currentColor"
											>
												<path
													fill-rule="evenodd"
													d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z"
													clip-rule="evenodd"
												/>
											</svg>
										</button>
									{/if}
								{/if}
							</div>
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>

	<div class="mt-2 text-right text-sm opacity-50">
		{$downloadState.downloads.length} download{$downloadState.downloads.length !== 1 ? 's' : ''}
	</div>
{/if}
