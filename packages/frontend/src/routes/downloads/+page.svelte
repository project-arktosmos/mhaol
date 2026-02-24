<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import type { UnifiedDownload } from '$types/download.type';
	import { formatBytes } from '$types/torrent.type';

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
		type: 'youtube';
		thumbnailUrl: string | null;
		title: string;
		url: string;
		videoId: string;
		mode: string;
		quality: string;
		format: string;
		durationSeconds: number | null;
		outputPath: string | null;
	}

	type FilesResponse = TorrentFilesResponse | YoutubeFilesResponse;

	let downloads = $state<UnifiedDownload[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);

	let modalOpen = $state(false);
	let modalLoading = $state(false);
	let modalData = $state<FilesResponse | null>(null);
	let modalDownload = $state<UnifiedDownload | null>(null);

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

	async function openFiles(dl: UnifiedDownload) {
		modalDownload = dl;
		modalData = null;
		modalOpen = true;
		modalLoading = true;

		try {
			const res = await fetch(
				`/api/downloads/${encodeURIComponent(dl.id)}/files?type=${dl.type}`
			);
			if (!res.ok) throw new Error('Failed to load files');
			modalData = await res.json();
		} catch {
			modalData = null;
		} finally {
			modalLoading = false;
		}
	}

	function closeModal() {
		modalOpen = false;
		modalData = null;
		modalDownload = null;
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

	function formatDuration(seconds: number | null): string {
		if (!seconds) return '--';
		const m = Math.floor(seconds / 60);
		const s = Math.floor(seconds % 60);
		return `${m}:${s.toString().padStart(2, '0')}`;
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
						<th></th>
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
							<td>
								<button class="btn btn-ghost btn-xs" onclick={() => openFiles(dl)}>
									Files
								</button>
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

{#if modalOpen}
	<div class="modal modal-open">
		<div class="modal-box max-w-lg">
			<button
				class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2"
				onclick={closeModal}
			>
				x
			</button>
			<h3 class="text-lg font-bold">{modalDownload?.name ?? 'Files'}</h3>

			{#if modalLoading}
				<div class="flex justify-center py-8">
					<span class="loading loading-spinner loading-md"></span>
				</div>
			{:else if modalData?.type === 'torrent'}
				{#if modalData.directory}
					<p class="mb-3 mt-1 truncate text-xs opacity-50" title={modalData.directory}>
						{modalData.directory}
					</p>
				{/if}
				{#if modalData.files.length === 0}
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
								{#each modalData.files as file (file.name)}
									<tr class="hover:bg-base-200/50">
										<td class="max-w-xs truncate" title={file.name}>
											{file.name}
										</td>
										<td class="whitespace-nowrap text-right text-xs opacity-70">
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
			{:else if modalData?.type === 'youtube'}
				<div class="mt-3 flex flex-col gap-3">
					{#if modalData.thumbnailUrl}
						<img
							src={modalData.thumbnailUrl}
							alt={modalData.title}
							class="w-full rounded-lg"
						/>
					{/if}
					<div class="grid grid-cols-2 gap-2 text-sm">
						<span class="opacity-50">Mode</span>
						<span>{modalData.mode}</span>
						<span class="opacity-50">Quality</span>
						<span>{modalData.quality}</span>
						<span class="opacity-50">Format</span>
						<span>{modalData.format}</span>
						<span class="opacity-50">Duration</span>
						<span>{formatDuration(modalData.durationSeconds)}</span>
						{#if modalData.outputPath}
							<span class="opacity-50">File</span>
							<span class="truncate" title={modalData.outputPath}>
								{modalData.outputPath}
							</span>
						{/if}
					</div>
				</div>
			{:else}
				<p class="py-4 opacity-50">Could not load details.</p>
			{/if}
		</div>
		<div class="modal-backdrop" onclick={closeModal}></div>
	</div>
{/if}
