<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { apiUrl } from '$lib/api-base';
	import type { LibraryFs, LibraryFsEntry } from '$types/library.type';
	import type { YouTubeContent } from '$types/youtube.type';

	let fsData = $state<LibraryFs | null>(null);
	let contentMap = $state<Map<string, YouTubeContent>>(new Map());
	let loading = $state(true);
	let error = $state<string | null>(null);

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const units = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(1024));
		return `${(bytes / Math.pow(1024, i)).toFixed(2)} ${units[i]}`;
	}

	function formatDuration(seconds: number | null | undefined): string {
		if (!seconds) return '—';
		const h = Math.floor(seconds / 3600);
		const m = Math.floor((seconds % 3600) / 60);
		const s = seconds % 60;
		if (h > 0) return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
		return `${m}:${String(s).padStart(2, '0')}`;
	}

	function stemFromFilename(name: string): string {
		const dot = name.lastIndexOf('.');
		return dot !== -1 ? name.slice(0, dot) : name;
	}

	function sumSize(entries: LibraryFsEntry[]): number {
		return entries.reduce((acc, e) => acc + e.size, 0);
	}

	let audioTotal = $derived(fsData ? sumSize(fsData.audio) : 0);
	let videoTotal = $derived(fsData ? sumSize(fsData.video) : 0);
	let grandTotal = $derived(audioTotal + videoTotal);

	onMount(async () => {
		try {
			const [fsRes, mediaRes] = await Promise.all([
				fetch(apiUrl('/api/libraries/fs')),
				fetch(apiUrl('/api/media'))
			]);
			if (!fsRes.ok) {
				const body = await fsRes.json().catch(() => ({}));
				throw new Error((body as { error?: string }).error ?? `HTTP ${fsRes.status}`);
			}
			fsData = await fsRes.json();
			if (mediaRes.ok) {
				const items: YouTubeContent[] = await mediaRes.json();
				contentMap = new Map(items.map((c) => [c.youtubeId, c]));
			}
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	});
</script>

<div class="p-2">
	<div class="mb-6">
		<h1 class="text-2xl font-bold">Disk</h1>
		<p class="text-sm opacity-70">Library filesystem usage</p>
		{#if fsData}
			<div class="mt-1 flex items-center gap-2">
				<p class="truncate font-mono text-xs opacity-50">{fsData.path}</p>
				<button
					class="btn opacity-60 btn-ghost btn-xs"
					onclick={() => invoke('open_path', { path: fsData!.path })}
				>
					open
				</button>
			</div>
		{/if}
	</div>

	{#if loading}
		<div class="flex justify-center py-12">
			<span class="loading loading-md loading-spinner"></span>
		</div>
	{:else if error}
		<div class="alert alert-error">
			<span>{error}</span>
		</div>
	{:else if fsData}
		{@const sections: { label: string; entries: LibraryFsEntry[]; total: number }[] = [
			{ label: 'audio/', entries: fsData.audio, total: audioTotal },
			{ label: 'video/', entries: fsData.video, total: videoTotal }
		]}

		{#each sections as section (section.label)}
			<div class="card mb-4 bg-base-200">
				<div class="card-body p-4">
					<div class="mb-2 flex items-center justify-between">
						<h2 class="font-mono font-semibold">{section.label}</h2>
						<span class="badge badge-neutral">{formatBytes(section.total)}</span>
					</div>

					{#if section.entries.length === 0}
						<p class="text-sm opacity-50">No files</p>
					{:else}
						<div class="overflow-x-auto">
							<table class="table w-full table-sm">
								<thead>
									<tr>
										<th class="text-left">Title</th>
										<th class="text-left">Channel</th>
										<th class="text-left">Duration</th>
										<th class="text-left">File</th>
										<th class="text-right">Size</th>
									</tr>
								</thead>
								<tbody>
									{#each section.entries as entry (entry.name)}
										{@const meta = contentMap.get(stemFromFilename(entry.name))}
										<tr>
											<td class="max-w-xs truncate text-sm">{meta?.title ?? '—'}</td>
											<td class="max-w-40 truncate text-xs opacity-70"
												>{meta?.channelName ?? '—'}</td
											>
											<td class="font-mono text-xs opacity-70"
												>{formatDuration(meta?.durationSeconds)}</td
											>
											<td class="font-mono text-xs opacity-50">{entry.name}</td>
											<td class="text-right font-mono text-xs opacity-70"
												>{formatBytes(entry.size)}</td
											>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					{/if}
				</div>
			</div>
		{/each}

		<div class="card bg-base-300">
			<div class="card-body flex-row items-center justify-between p-4">
				<span class="font-semibold">Total</span>
				<span class="font-mono text-lg font-bold">{formatBytes(grandTotal)}</span>
			</div>
		</div>
	{/if}
</div>
