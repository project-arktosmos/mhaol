<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import classNames from 'classnames';
	import { downloadsService } from 'frontend/services/downloads.service';
	import Modal from 'ui-lib/components/core/Modal.svelte';
	import type { UnifiedDownload } from 'frontend/types/download.type';
	import { formatBytes } from 'frontend/types/torrent.type';

	interface FileEntry {
		name: string;
		size: number;
		isDirectory: boolean;
	}

	interface TorrentFilesResponse {
		type: 'torrent';
		name: string;
		directory: string | null;
		files: FileEntry[];
	}

	interface YoutubeFilesResponse {
		type: 'youtube-video' | 'youtube-audio';
		thumbnailUrl: string | null;
		title: string;
		url: string;
		videoId: string;
		durationSeconds: number | null;
		outputPath: string | null;
	}

	type FilesResponse = TorrentFilesResponse | YoutubeFilesResponse;

	const downloadState = downloadsService.state;

	let filesModalOpen = $state(false);
	let filesModalLoading = $state(false);
	let filesModalData = $state<FilesResponse | null>(null);
	let filesModalDownload = $state<UnifiedDownload | null>(null);

	onMount(() => {
		downloadsService.startPolling();
	});

	onDestroy(() => {
		downloadsService.stopPolling();
	});

	async function openFiles(dl: UnifiedDownload) {
		filesModalDownload = dl;
		filesModalData = null;
		filesModalOpen = true;
		filesModalLoading = true;

		try {
			const res = await fetch(`/api/downloads/${encodeURIComponent(dl.id)}/files?type=${dl.type}`);
			if (!res.ok) throw new Error('Failed to load files');
			filesModalData = await res.json();
		} catch {
			filesModalData = null;
		} finally {
			filesModalLoading = false;
		}
	}

	function closeFilesModal() {
		filesModalOpen = false;
		filesModalData = null;
		filesModalDownload = null;
	}

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

	function formatDuration(seconds: number | null): string {
		if (!seconds) return '--';
		const m = Math.floor(seconds / 60);
		const s = Math.floor(seconds % 60);
		return `${m}:${s.toString().padStart(2, '0')}`;
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
					<th></th>
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
							<button class="btn btn-ghost btn-xs" onclick={() => openFiles(dl)}> Files </button>
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

<Modal open={filesModalOpen} maxWidth="max-w-lg" zIndex={60} onclose={closeFilesModal}>
	<h3 class="text-lg font-bold">{filesModalDownload?.name ?? 'Files'}</h3>

	{#if filesModalLoading}
		<div class="flex justify-center py-8">
			<span class="loading loading-md loading-spinner"></span>
		</div>
	{:else if filesModalData?.type === 'torrent'}
		{#if filesModalData.directory}
			<p class="mt-1 mb-3 truncate text-xs opacity-50" title={filesModalData.directory}>
				{filesModalData.directory}
			</p>
		{/if}
		{#if filesModalData.files.length === 0}
			<p class="py-4 opacity-50">No files found in this directory.</p>
		{:else}
			<div class="max-h-72 overflow-y-auto">
				<table class="table table-xs">
					<thead>
						<tr>
							<th>Name</th>
							<th class="text-right">Size</th>
						</tr>
					</thead>
					<tbody>
						{#each filesModalData.files as file (file.name)}
							<tr class="hover:bg-base-200/50">
								<td class="max-w-xs truncate" title={file.name}>
									{file.name}
								</td>
								<td class="text-right text-xs whitespace-nowrap opacity-70">
									{#if file.isDirectory}
										--
									{:else}
										{formatBytes(file.size)}
									{/if}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	{:else if filesModalData?.type === 'youtube-video' || filesModalData?.type === 'youtube-audio'}
		<div class="mt-3 flex flex-col gap-3">
			{#if filesModalData.thumbnailUrl}
				<img
					src={filesModalData.thumbnailUrl}
					alt={filesModalData.title}
					class="w-full rounded-lg"
				/>
			{/if}
			<div class="grid grid-cols-2 gap-2 text-sm">
				<span class="opacity-50">Type</span>
				<span>{filesModalData.type}</span>
				<span class="opacity-50">Duration</span>
				<span>{formatDuration(filesModalData.durationSeconds)}</span>
				{#if filesModalData.outputPath}
					<span class="opacity-50">File</span>
					<span class="truncate" title={filesModalData.outputPath}>
						{filesModalData.outputPath}
					</span>
				{/if}
			</div>
		</div>
	{:else}
		<p class="py-4 opacity-50">Could not load details.</p>
	{/if}
</Modal>
