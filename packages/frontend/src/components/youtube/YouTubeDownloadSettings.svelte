<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import { youtubeService } from '$services/youtube.service';
	import {
		AUDIO_QUALITY_OPTIONS,
		AUDIO_FORMAT_OPTIONS,
		VIDEO_QUALITY_OPTIONS,
		VIDEO_FORMAT_OPTIONS
	} from '$types/youtube.type';
	import type {
		AudioQuality,
		AudioFormat,
		DownloadMode,
		VideoQuality,
		VideoFormat
	} from '$types/youtube.type';

	const state = youtubeService.state;
	const settings = youtubeService.store;

	// Advanced config state
	let showAdvanced = false;
	let poToken = '';
	let cookies = '';
	let configSaving = false;

	// yt-dlp state
	let ytdlpDownloading = false;

	// Output path editing
	let outputPathInput = '';
	let editingPath = false;

	onMount(async () => {
		const config = await youtubeService.getConfig();
		if (config) {
			poToken = config.poToken || '';
			cookies = config.cookies || '';
		}
		outputPathInput = $state.outputPath;
	});

	$: if ($state.outputPath && !editingPath) {
		outputPathInput = $state.outputPath;
	}

	function handleModeChange(mode: DownloadMode) {
		youtubeService.setDownloadMode(mode);
	}

	function handleQualityChange(event: Event) {
		const target = event.target as HTMLSelectElement;
		youtubeService.setDefaultQuality(target.value as AudioQuality);
	}

	function handleFormatChange(event: Event) {
		const target = event.target as HTMLSelectElement;
		youtubeService.setDefaultFormat(target.value as AudioFormat);
	}

	function handleVideoQualityChange(event: Event) {
		const target = event.target as HTMLSelectElement;
		youtubeService.setDefaultVideoQuality(target.value as VideoQuality);
	}

	function handleVideoFormatChange(event: Event) {
		const target = event.target as HTMLSelectElement;
		youtubeService.setDefaultVideoFormat(target.value as VideoFormat);
	}

	async function handleSetOutputPath() {
		if (outputPathInput.trim()) {
			await youtubeService.setOutputPath(outputPathInput.trim());
			editingPath = false;
		}
	}

	async function handleSaveConfig() {
		configSaving = true;
		await youtubeService.setConfig({
			poToken: poToken.trim() || null,
			cookies: cookies.trim() || null
		});
		configSaving = false;
	}

	async function handleDownloadYtDlp() {
		ytdlpDownloading = true;
		await youtubeService.downloadYtDlp();
		ytdlpDownloading = false;
	}

	// Truncate path for display
	function truncatePath(path: string, maxLength: number = 40): string {
		if (path.length <= maxLength) return path;
		const parts = path.split('/');
		if (parts.length <= 2) return path;
		return `.../${parts.slice(-2).join('/')}`;
	}

	// Reactive yt-dlp status
	$: ytdlpStatus = $state.ytdlpStatus;
	$: ytdlpAvailable = ytdlpStatus?.available ?? false;
</script>

<div class="card bg-base-200">
	<div class="card-body gap-4">
		<h2 class="card-title text-lg">Download Settings</h2>

		<!-- yt-dlp Status -->
		<div
			class={classNames('rounded-lg p-3', {
				'bg-success/10': ytdlpAvailable,
				'bg-warning/10': !ytdlpAvailable
			})}
		>
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-2">
					<div
						class={classNames('h-2 w-2 rounded-full', {
							'bg-success': ytdlpAvailable,
							'bg-warning': !ytdlpAvailable
						})}
					></div>
					<span class="text-sm font-medium">
						{#if ytdlpAvailable}
							yt-dlp Ready
						{:else}
							yt-dlp Not Installed
						{/if}
					</span>
				</div>

				{#if ytdlpAvailable && ytdlpStatus?.version}
					<span class="text-xs text-base-content/60">{ytdlpStatus.version}</span>
				{/if}
			</div>

			{#if !ytdlpAvailable}
				<p class="mt-2 text-xs text-base-content/70">
					Install yt-dlp for reliable downloads. Without it, downloads will not work.
				</p>
				<button
					class="btn btn-primary btn-sm mt-2"
					on:click={handleDownloadYtDlp}
					disabled={ytdlpDownloading}
				>
					{#if ytdlpDownloading}
						<span class="loading loading-spinner loading-xs"></span>
						Downloading...
					{:else}
						Download yt-dlp
					{/if}
				</button>
			{/if}
		</div>

		<!-- Download Mode Toggle -->
		<div class="form-control">
			<span class="label">
				<span class="label-text">Download Mode</span>
			</span>
			<div class="join w-full">
				<button
					class={classNames('btn join-item flex-1', {
						'btn-primary': $settings.downloadMode === 'audio',
						'btn-ghost': $settings.downloadMode !== 'audio'
					})}
					on:click={() => handleModeChange('audio')}
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-4 w-4"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"
						/>
					</svg>
					Audio
				</button>
				<button
					class={classNames('btn join-item flex-1', {
						'btn-primary': $settings.downloadMode === 'video',
						'btn-ghost': $settings.downloadMode !== 'video'
					})}
					on:click={() => handleModeChange('video')}
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-4 w-4"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z"
						/>
					</svg>
					Video
				</button>
			</div>
		</div>

		{#if $settings.downloadMode === 'audio'}
			<!-- Audio Quality -->
			<div class="form-control">
				<label class="label" for="quality-select">
					<span class="label-text">Audio Quality</span>
				</label>
				<select
					id="quality-select"
					class="select select-bordered w-full"
					value={$settings.defaultQuality}
					on:change={handleQualityChange}
				>
					{#each AUDIO_QUALITY_OPTIONS as option}
						<option value={option.value}>
							{option.label} - {option.description}
						</option>
					{/each}
				</select>
			</div>

			<!-- Audio Format -->
			<div class="form-control">
				<label class="label" for="format-select">
					<span class="label-text">Audio Format</span>
				</label>
				<select
					id="format-select"
					class="select select-bordered w-full"
					value={$settings.defaultFormat}
					on:change={handleFormatChange}
				>
					{#each AUDIO_FORMAT_OPTIONS as option}
						<option value={option.value}>
							{option.label}
						</option>
					{/each}
				</select>
			</div>
		{:else}
			<!-- Video Quality -->
			<div class="form-control">
				<label class="label" for="video-quality-select">
					<span class="label-text">Video Quality</span>
				</label>
				<select
					id="video-quality-select"
					class="select select-bordered w-full"
					value={$settings.defaultVideoQuality}
					on:change={handleVideoQualityChange}
				>
					{#each VIDEO_QUALITY_OPTIONS as option}
						<option value={option.value}>
							{option.label} - {option.description}
						</option>
					{/each}
				</select>
			</div>

			<!-- Video Format -->
			<div class="form-control">
				<label class="label" for="video-format-select">
					<span class="label-text">Video Format</span>
				</label>
				<select
					id="video-format-select"
					class="select select-bordered w-full"
					value={$settings.defaultVideoFormat}
					on:change={handleVideoFormatChange}
				>
					{#each VIDEO_FORMAT_OPTIONS as option}
						<option value={option.value}>
							{option.label}
						</option>
					{/each}
				</select>
			</div>
		{/if}

		<!-- Output Folder -->
		<div class="form-control">
			<label class="label" for="output-folder">
				<span class="label-text">Output Folder</span>
			</label>
			<div class="flex items-center gap-2">
				<input
					id="output-folder"
					type="text"
					class="input input-bordered flex-1"
					bind:value={outputPathInput}
					on:focus={() => (editingPath = true)}
					placeholder="/path/to/downloads"
					title={$state.outputPath}
				/>
				<button class="btn btn-outline btn-sm" on:click={handleSetOutputPath}> Set </button>
			</div>
		</div>

		<!-- Stats -->
		{#if $state.stats}
			<div class="divider my-1"></div>
			<div class="flex justify-between text-sm text-base-content/60">
				<span>Active: {$state.stats.activeDownloads}</span>
				<span>Completed: {$state.stats.completedDownloads}</span>
				<span>Failed: {$state.stats.failedDownloads}</span>
			</div>
		{/if}

		<!-- Advanced Settings (Collapsible) -->
		<div class="divider my-1"></div>
		<button
			class="flex w-full items-center justify-between text-sm text-base-content/70 hover:text-base-content"
			on:click={() => (showAdvanced = !showAdvanced)}
		>
			<span>Advanced (Auth Config)</span>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="h-4 w-4 transition-transform"
				class:rotate-180={showAdvanced}
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M19 9l-7 7-7-7"
				/>
			</svg>
		</button>

		{#if showAdvanced}
			<div class="mt-2 flex flex-col gap-3 rounded-lg bg-base-300 p-3">
				<p class="text-xs text-base-content/60">
					You can provide authentication to bypass bot detection. See the
					<a
						href="https://github.com/yt-dlp/yt-dlp/wiki/Extractors#po-token-guide"
						target="_blank"
						rel="noopener noreferrer"
						class="link link-primary"
					>
						PO Token Guide
					</a>
					for instructions.
				</p>

				<!-- PO Token -->
				<div class="form-control">
					<label class="label py-1" for="po-token">
						<span class="label-text text-sm">PO Token</span>
					</label>
					<input
						id="po-token"
						type="text"
						class="input input-bordered input-sm w-full font-mono text-xs"
						placeholder="Enter PO token..."
						bind:value={poToken}
					/>
				</div>

				<!-- Cookies -->
				<div class="form-control">
					<label class="label py-1" for="cookies">
						<span class="label-text text-sm">Cookies</span>
					</label>
					<textarea
						id="cookies"
						class="textarea textarea-bordered textarea-sm w-full font-mono text-xs"
						placeholder="key1=value1; key2=value2"
						rows="2"
						bind:value={cookies}
					></textarea>
				</div>

				<!-- Save Button -->
				<button
					class="btn btn-primary btn-sm"
					on:click={handleSaveConfig}
					disabled={configSaving}
				>
					{#if configSaving}
						<span class="loading loading-spinner loading-xs"></span>
					{/if}
					Save Config
				</button>
			</div>
		{/if}
	</div>
</div>
