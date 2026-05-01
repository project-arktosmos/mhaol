<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import classNames from 'classnames';
	import type {
		YouTubeDownloadProgress,
		YouTubeVideoInfo,
		DownloadMode,
		AudioFormat,
		AudioQuality,
		YouTubeStreamUrlResult,
		YouTubeStreamFormat
	} from 'addons/youtube/types';
	import { extractVideoId } from 'addons/youtube/types';
	import { playerService } from 'ui-lib/services/player.service';
	import type { PlayableFile } from 'ui-lib/types/player.type';

	interface SearchItem {
		videoId: string;
		type: string;
		url: string;
		title: string;
		thumbnail: string;
		duration: number;
		durationText: string;
		views: number;
		viewsText: string;
		uploadedDate: string;
		uploaderName: string;
		uploaderUrl: string;
		uploaderAvatar: string;
		uploaderVerified: boolean;
	}

	interface SearchResponse {
		items: SearchItem[];
		channels: unknown[];
		continuation: string | null;
	}

	let query = $state('');
	let searching = $state(false);
	let searchError = $state<string | null>(null);
	let searchResults = $state<SearchItem[]>([]);
	let searchContinuation = $state<string | null>(null);
	let loadingMore = $state(false);

	let url = $state('');
	let info = $state<YouTubeVideoInfo | null>(null);
	let infoLoading = $state(false);
	let infoError = $state<string | null>(null);
	let downloads = $state<YouTubeDownloadProgress[]>([]);
	let queueing = $state<string | null>(null);
	let queueError = $state<string | null>(null);
	let connected = $state(false);
	let playing = $state<string | null>(null);
	let playError = $state<string | null>(null);

	let sse: EventSource | null = null;

	async function runSearch() {
		const q = query.trim();
		if (!q) return;
		searching = true;
		searchError = null;
		searchResults = [];
		searchContinuation = null;
		try {
			const res = await fetch(`/api/ytdl/search?q=${encodeURIComponent(q)}`);
			if (!res.ok) {
				const body = await res.text();
				throw new Error(body || `HTTP ${res.status}`);
			}
			const data = (await res.json()) as SearchResponse;
			searchResults = data.items;
			searchContinuation = data.continuation;
		} catch (e) {
			searchError = e instanceof Error ? e.message : String(e);
		} finally {
			searching = false;
		}
	}

	async function loadMore() {
		if (!searchContinuation || loadingMore) return;
		loadingMore = true;
		try {
			const res = await fetch(
				`/api/ytdl/search?continuation=${encodeURIComponent(searchContinuation)}`
			);
			if (!res.ok) return;
			const data = (await res.json()) as SearchResponse;
			searchResults = [...searchResults, ...data.items];
			searchContinuation = data.continuation;
		} catch {
			// ignore
		} finally {
			loadingMore = false;
		}
	}

	function clearSearch() {
		query = '';
		searchResults = [];
		searchContinuation = null;
		searchError = null;
	}

	async function fetchInfo(target?: string) {
		const value = (target ?? url).trim();
		if (!value) return;
		url = value;
		infoLoading = true;
		infoError = null;
		info = null;
		try {
			const res = await fetch(`/api/ytdl/info/video?url=${encodeURIComponent(value)}`);
			if (!res.ok) {
				const body = await res.text();
				throw new Error(body || `HTTP ${res.status}`);
			}
			info = (await res.json()) as YouTubeVideoInfo;
		} catch (e) {
			infoError = e instanceof Error ? e.message : String(e);
		} finally {
			infoLoading = false;
		}
	}

	function selectSearchResult(item: SearchItem) {
		fetchInfo(`https://www.youtube.com/watch?v=${item.videoId}`);
	}

	function pickMuxed(result: YouTubeStreamUrlResult): YouTubeStreamFormat | null {
		const muxed = result.formats.filter((f) => !f.isAudioOnly && !f.isVideoOnly);
		if (muxed.length === 0) return null;
		muxed.sort((a, b) => {
			const heightDiff = (b.height ?? 0) - (a.height ?? 0);
			if (heightDiff !== 0) return heightDiff;
			return b.bitrate - a.bitrate;
		});
		return muxed[0];
	}

	function pickAudio(result: YouTubeStreamUrlResult): YouTubeStreamFormat | null {
		// Prefer the muxed (video+audio) format for audio-mode playback. The
		// audio-only adaptive streams YouTube serves are fragmented MP4
		// (`ftyp dash`) — browsers reject those in plain `<video src=...>` /
		// `<audio src=...>` with `MEDIA_ERR_SRC_NOT_SUPPORTED` because the
		// file shape needs MediaSource Extensions to play. The muxed format
		// is a self-contained MP4 that decodes everywhere; the player surface
		// hides the video frame and shows a music-note overlay, so the
		// user-facing experience is "audio only" either way.
		const muxed = pickMuxed(result);
		if (muxed) return muxed;
		// No muxed format available — fall back to audio-only as a last
		// resort. The player will surface any decode failure on screen.
		const audioOnly = result.formats.filter((f) => f.isAudioOnly);
		const mp4Audio = audioOnly.filter((f) => f.container === 'mp4');
		const sorted = (list: YouTubeStreamFormat[]) => [...list].sort((a, b) => b.bitrate - a.bitrate);
		return sorted(mp4Audio)[0] ?? sorted(audioOnly)[0] ?? null;
	}

	function pickFormat(
		result: YouTubeStreamUrlResult,
		mode: 'audio' | 'video'
	): YouTubeStreamFormat | null {
		return mode === 'audio' ? pickAudio(result) : pickMuxed(result);
	}

	async function streamItem(item: SearchItem, mode: 'audio' | 'video') {
		const dlUrl = `https://www.youtube.com/watch?v=${item.videoId}`;
		playing = item.videoId;
		playError = null;
		try {
			const res = await fetch(
				`/api/ytdl/info/stream-urls-browser?url=${encodeURIComponent(dlUrl)}`
			);
			if (!res.ok) {
				const body = await res.text();
				throw new Error(body || `HTTP ${res.status}`);
			}
			const result = (await res.json()) as YouTubeStreamUrlResult;
			const format = pickFormat(result, mode);
			if (!format) {
				throw new Error(
					mode === 'audio' ? 'No audio-only format available' : 'No muxed format available'
				);
			}
			const file: PlayableFile = {
				id: `youtube:${item.videoId}:${mode}`,
				type: 'youtube',
				name: item.title,
				outputPath: '',
				mode,
				format: null,
				videoFormat: null,
				thumbnailUrl: item.thumbnail || null,
				durationSeconds: item.duration || null,
				size: format.contentLength ?? 0,
				completedAt: ''
			};
			await playerService.playUrl(file, format.url, format.mimeType, 'sidebar');
		} catch (e) {
			playError = e instanceof Error ? e.message : String(e);
		} finally {
			playing = null;
		}
	}

	async function streamCurrentInfo(mode: 'audio' | 'video') {
		if (!info) return;
		await streamItem(
			{
				videoId: info.videoId,
				type: 'stream',
				url: `/watch?v=${info.videoId}`,
				title: info.title,
				thumbnail: info.thumbnailUrl ?? '',
				duration: info.duration,
				durationText: '',
				views: 0,
				viewsText: '',
				uploadedDate: '',
				uploaderName: info.uploader ?? '',
				uploaderUrl: '',
				uploaderAvatar: '',
				uploaderVerified: false
			},
			mode
		);
	}

	async function quickQueue(
		item: SearchItem,
		mode: DownloadMode,
		opts?: { format?: AudioFormat; quality?: AudioQuality }
	) {
		const dlUrl = `https://www.youtube.com/watch?v=${item.videoId}`;
		queueing = item.videoId;
		queueError = null;
		try {
			const res = await fetch('/api/ytdl/downloads', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					url: dlUrl,
					videoId: item.videoId,
					title: item.title,
					mode,
					format: opts?.format,
					quality: opts?.quality,
					thumbnailUrl: item.thumbnail,
					durationSeconds: item.duration
				})
			});
			if (!res.ok) {
				const body = await res.text();
				throw new Error(body || `HTTP ${res.status}`);
			}
		} catch (e) {
			queueError = e instanceof Error ? e.message : String(e);
		} finally {
			queueing = null;
		}
	}

	async function queueDownload(mode: DownloadMode) {
		if (!info) return;
		queueing = info.videoId;
		queueError = null;
		try {
			const res = await fetch('/api/ytdl/downloads', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					url: url.trim(),
					videoId: info.videoId,
					title: info.title,
					mode,
					thumbnailUrl: info.thumbnailUrl,
					durationSeconds: info.duration
				})
			});
			if (!res.ok) {
				const body = await res.text();
				throw new Error(body || `HTTP ${res.status}`);
			}
		} catch (e) {
			queueError = e instanceof Error ? e.message : String(e);
		} finally {
			queueing = null;
		}
	}

	async function cancelDownload(id: string) {
		try {
			await fetch(`/api/ytdl/downloads/${id}`, { method: 'DELETE' });
		} catch {
			// ignore — SSE will reconcile
		}
	}

	async function clearCompleted() {
		try {
			await fetch('/api/ytdl/downloads/completed', { method: 'DELETE' });
			downloads = downloads.filter((d) => !['completed', 'failed', 'cancelled'].includes(d.state));
		} catch {
			// ignore
		}
	}

	function upsertDownload(progress: YouTubeDownloadProgress) {
		const idx = downloads.findIndex((d) => d.downloadId === progress.downloadId);
		if (idx >= 0) {
			downloads[idx] = progress;
			downloads = downloads;
		} else {
			downloads = [progress, ...downloads];
		}
	}

	function connectSSE() {
		sse = new EventSource('/api/ytdl/downloads/events');
		sse.addEventListener('connected', () => {
			connected = true;
		});
		sse.addEventListener('progress', (e) => {
			try {
				upsertDownload(JSON.parse(e.data));
			} catch {
				// ignore malformed event
			}
		});
		sse.addEventListener('error', () => {
			connected = false;
		});
	}

	function fmtBytes(n: number): string {
		if (!n) return '0 B';
		const units = ['B', 'KB', 'MB', 'GB'];
		let i = 0;
		let v = n;
		while (v >= 1024 && i < units.length - 1) {
			v /= 1024;
			i++;
		}
		return `${v.toFixed(v >= 10 || i === 0 ? 0 : 1)} ${units[i]}`;
	}

	function fmtDuration(secs: number | null): string {
		if (!secs) return '';
		const h = Math.floor(secs / 3600);
		const m = Math.floor((secs % 3600) / 60);
		const s = Math.floor(secs % 60);
		return h > 0
			? `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
			: `${m}:${String(s).padStart(2, '0')}`;
	}

	const ACTIVE_STATES = ['pending', 'fetching', 'downloading', 'muxing'];
	let activeDownloads = $derived(downloads.filter((d) => ACTIVE_STATES.includes(d.state)));
	let finishedDownloads = $derived(downloads.filter((d) => !ACTIVE_STATES.includes(d.state)));
	let canFetchInfo = $derived(!!extractVideoId(url));

	onMount(() => {
		connectSSE();
	});

	onDestroy(() => {
		sse?.close();
	});
</script>

<div class="flex h-full flex-col">
	<header class="flex flex-wrap items-center gap-3 border-b border-base-300 px-4 py-3">
		<h1 class="text-lg font-bold">YouTube</h1>
		<span
			class={classNames('badge badge-sm', {
				'badge-success': connected,
				'badge-ghost': !connected
			})}
		>
			{connected ? 'yt-dlp connected' : 'connecting…'}
		</span>
		<div class="ml-auto flex items-center gap-2">
			<div class="join">
				<input
					type="text"
					class="input-bordered input input-sm join-item w-64"
					placeholder="Search YouTube…"
					bind:value={query}
					onkeydown={(e) => {
						if (e.key === 'Enter') runSearch();
					}}
				/>
				<button
					class="btn join-item btn-sm btn-primary"
					disabled={!query.trim() || searching}
					onclick={runSearch}
				>
					{#if searching}
						<span class="loading loading-xs loading-spinner"></span>
					{:else}
						Search
					{/if}
				</button>
				{#if searchResults.length > 0 || query}
					<button class="btn join-item btn-sm" onclick={clearSearch} disabled={searching}
						>Clear</button
					>
				{/if}
			</div>
		</div>
	</header>

	<div class="min-w-0 flex-1 overflow-y-auto p-4">
		<div class="mb-6 flex flex-col gap-2">
			<div class="join">
				<input
					type="text"
					class="input-bordered input join-item flex-1"
					placeholder="…or paste a URL: https://www.youtube.com/watch?v=…"
					bind:value={url}
					onkeydown={(e) => {
						if (e.key === 'Enter' && canFetchInfo) fetchInfo();
					}}
				/>
				<button
					class="btn join-item btn-primary"
					disabled={!canFetchInfo || infoLoading}
					onclick={() => fetchInfo()}
				>
					{#if infoLoading}
						<span class="loading loading-sm loading-spinner"></span>
					{:else}
						Fetch info
					{/if}
				</button>
			</div>
			{#if infoError}
				<div class="alert alert-error">
					<span>{infoError}</span>
				</div>
			{/if}
		</div>

		{#if searchError}
			<div class="mb-4 alert alert-error">
				<span>{searchError}</span>
			</div>
		{/if}

		{#if playError && searchResults.length > 0}
			<div class="mb-4 alert alert-error">
				<span>{playError}</span>
				<button class="btn btn-ghost btn-sm" onclick={() => (playError = null)}>Dismiss</button>
			</div>
		{/if}

		{#if searching && searchResults.length === 0}
			<div class="flex justify-center py-8">
				<span class="loading loading-lg loading-spinner"></span>
			</div>
		{:else if searchResults.length > 0}
			<section class="mb-6">
				<h2 class="mb-3 text-lg font-semibold">Search results</h2>
				<div class="grid grid-cols-1 gap-2 md:grid-cols-2 xl:grid-cols-3">
					{#each searchResults as item (item.videoId)}
						<div class="flex gap-3 rounded-lg bg-base-200 p-2">
							<button
								class="shrink-0"
								onclick={() => selectSearchResult(item)}
								aria-label="Select {item.title}"
							>
								{#if item.thumbnail}
									<img
										src={item.thumbnail}
										alt={item.title}
										class="h-20 w-32 rounded object-cover"
										loading="lazy"
									/>
								{:else}
									<div class="h-20 w-32 rounded bg-base-300"></div>
								{/if}
							</button>
							<div class="flex min-w-0 flex-1 flex-col gap-1">
								<button
									class="truncate text-left text-sm font-medium hover:underline"
									onclick={() => selectSearchResult(item)}
									title={item.title}
								>
									{item.title}
								</button>
								<p class="truncate text-xs opacity-60">
									{item.uploaderName}
									{#if item.durationText}· {item.durationText}{/if}
									{#if item.viewsText}· {item.viewsText}{/if}
								</p>
								<div class="mt-auto flex flex-wrap gap-1">
									<button
										class="btn btn-xs btn-primary"
										disabled={playing === item.videoId}
										onclick={() => streamItem(item, 'video')}
										title="Stream video in player"
									>
										{playing === item.videoId ? '…' : 'Video'}
									</button>
									<button
										class="btn btn-xs btn-secondary"
										disabled={playing === item.videoId}
										onclick={() => streamItem(item, 'audio')}
										title="Stream audio in player"
									>
										{playing === item.videoId ? '…' : 'Audio'}
									</button>
									<button
										class="btn btn-xs"
										disabled={queueing === item.videoId}
										onclick={() => quickQueue(item, 'video')}
										title="Download video"
									>
										⇣
									</button>
									<button
										class="btn btn-xs"
										disabled={queueing === item.videoId}
										onclick={() => quickQueue(item, 'audio', { format: 'mp3', quality: 'best' })}
										title="Download audio (MP3 320 kbps)"
									>
										⇣ MP3
									</button>
								</div>
							</div>
						</div>
					{/each}
				</div>
				{#if searchContinuation}
					<div class="mt-3 flex justify-center">
						<button class="btn btn-sm" disabled={loadingMore} onclick={loadMore}>
							{loadingMore ? 'Loading…' : 'Load more'}
						</button>
					</div>
				{/if}
			</section>
		{/if}

		{#if info}
			<div class="mb-6 flex gap-4 rounded-lg bg-base-200 p-4">
				{#if info.thumbnailUrl}
					<img
						src={info.thumbnailUrl}
						alt={info.title}
						class="h-24 w-40 shrink-0 rounded object-cover"
						loading="lazy"
					/>
				{/if}
				<div class="flex min-w-0 flex-1 flex-col gap-2">
					<p class="font-semibold">{info.title}</p>
					<p class="text-sm opacity-70">
						{info.uploader ?? 'Unknown uploader'}
						{#if info.duration}· {fmtDuration(info.duration)}{/if}
					</p>
					<div class="mt-auto flex flex-wrap gap-2">
						<button
							class="btn btn-sm btn-primary"
							disabled={playing !== null}
							onclick={() => streamCurrentInfo('video')}
						>
							{playing === info.videoId ? 'Loading…' : 'Stream video'}
						</button>
						<button
							class="btn btn-sm btn-secondary"
							disabled={playing !== null}
							onclick={() => streamCurrentInfo('audio')}
						>
							{playing === info.videoId ? 'Loading…' : 'Stream audio'}
						</button>
						<span class="divider m-0 divider-horizontal"></span>
						<button
							class="btn btn-sm"
							disabled={queueing !== null}
							onclick={() => queueDownload('video')}
						>
							Download video
						</button>
						<button
							class="btn btn-sm"
							disabled={queueing !== null}
							onclick={() => queueDownload('audio')}
						>
							Download audio
						</button>
						<button
							class="btn btn-sm"
							disabled={queueing !== null}
							onclick={() => queueDownload('both')}
						>
							Both
						</button>
					</div>
					{#if queueError}
						<p class="text-sm text-error">{queueError}</p>
					{/if}
					{#if playError}
						<p class="text-sm text-error">{playError}</p>
					{/if}
				</div>
			</div>
		{/if}

		{#if activeDownloads.length > 0}
			<section class="mb-6">
				<h2 class="mb-2 text-lg font-semibold">In progress</h2>
				<div class="flex flex-col gap-2">
					{#each activeDownloads as d (d.downloadId)}
						<div class="rounded-lg bg-base-200 p-3">
							<div class="flex items-center justify-between gap-2">
								<p class="truncate font-medium">{d.title || d.videoId}</p>
								<button class="btn btn-ghost btn-xs" onclick={() => cancelDownload(d.downloadId)}>
									Cancel
								</button>
							</div>
							<div class="mt-1 flex items-center gap-2 text-sm opacity-70">
								<span>{d.state}</span>
								<span>·</span>
								<span>{d.mode}</span>
								<span>·</span>
								<span>{fmtBytes(d.downloadedBytes)} / {fmtBytes(d.totalBytes)}</span>
							</div>
							<progress class="progress mt-2 w-full" value={d.progress} max="1"></progress>
						</div>
					{/each}
				</div>
			</section>
		{/if}

		{#if finishedDownloads.length > 0}
			<section>
				<div class="mb-2 flex items-center justify-between">
					<h2 class="text-lg font-semibold">Finished</h2>
					<button class="btn btn-ghost btn-sm" onclick={clearCompleted}>Clear</button>
				</div>
				<div class="flex flex-col gap-2">
					{#each finishedDownloads as d (d.downloadId)}
						<div
							class={classNames('rounded-lg p-3', {
								'bg-success/10': d.state === 'completed',
								'bg-error/10': d.state === 'failed',
								'bg-base-200': d.state !== 'completed' && d.state !== 'failed'
							})}
						>
							<p class="truncate font-medium">{d.title || d.videoId}</p>
							<p class="text-sm opacity-70">
								{d.state}
								{#if d.error}— {d.error}{/if}
							</p>
						</div>
					{/each}
				</div>
			</section>
		{/if}

		{#if !info && downloads.length === 0 && searchResults.length === 0}
			<p class="rounded-lg bg-base-200 p-8 text-center opacity-60">
				Search for videos above, or paste a URL to fetch info and queue a download.
			</p>
		{/if}
	</div>
</div>
