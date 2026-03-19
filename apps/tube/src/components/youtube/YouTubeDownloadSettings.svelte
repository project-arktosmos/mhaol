<script lang="ts">
	import classNames from 'classnames';
	import { youtubeService } from '$services/youtube.service';
	import { libraryService } from '$services/library.service';
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

	const ytState = youtubeService.state;
	const settings = youtubeService.store;
	const library = libraryService.library;

	let showAdvanced = $state(false);
	let poToken = $state('');
	let cookies = $state('');
	let configSaving = $state(false);

	// Sync auth fields from settings store
	$effect(() => {
		if ($settings.poToken !== undefined) poToken = $settings.poToken;
	});

	$effect(() => {
		if ($settings.cookies !== undefined) cookies = $settings.cookies;
	});

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

	async function handleSaveConfig() {
		configSaving = true;
		await youtubeService.setConfig({
			poToken: poToken.trim() || null,
			cookies: cookies.trim() || null
		});
		configSaving = false;
	}

	let downloaderStatus = $derived($ytState.downloaderStatus);
	let downloaderAvailable = $derived(downloaderStatus?.available ?? false);
</script>

<div class="card bg-base-200">
	<div class="card-body gap-4">
		<h2 class="card-title text-lg">Download Settings</h2>

		<!-- Downloader Status -->
		<div
			class={classNames('rounded-lg p-3', {
				'bg-success/10': downloaderAvailable,
				'bg-warning/10': !downloaderAvailable
			})}
		>
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-2">
					<div
						class={classNames('h-2 w-2 rounded-full', {
							'bg-success': downloaderAvailable,
							'bg-warning': !downloaderAvailable
						})}
					></div>
					<span class="text-sm font-medium">
						{#if downloaderAvailable}
							Downloader Ready
						{:else}
							Downloader Unavailable
						{/if}
					</span>
				</div>

				{#if downloaderAvailable && downloaderStatus?.version}
					<span class="text-xs text-base-content/60">{downloaderStatus.version}</span>
				{/if}
			</div>

			{#if !downloaderAvailable}
				<p class="mt-2 text-xs text-base-content/70">
					Could not connect to the download service. Make sure the backend is running.
				</p>
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
					onclick={() => handleModeChange('audio')}
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
					onclick={() => handleModeChange('video')}
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
					class="select-bordered select w-full"
					value={$settings.defaultQuality}
					onchange={handleQualityChange}
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
					class="select-bordered select w-full"
					value={$settings.defaultFormat}
					onchange={handleFormatChange}
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
					class="select-bordered select w-full"
					value={$settings.defaultVideoQuality}
					onchange={handleVideoQualityChange}
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
					class="select-bordered select w-full"
					value={$settings.defaultVideoFormat}
					onchange={handleVideoFormatChange}
				>
					{#each VIDEO_FORMAT_OPTIONS as option}
						<option value={option.value}>
							{option.label}
						</option>
					{/each}
				</select>
			</div>
		{/if}

		<!-- Download Library Path -->
		{#if $library}
			<div class="form-control">
				<span class="label">
					<span class="label-text">Download Location</span>
				</span>
				<div class="rounded-lg bg-base-300 px-3 py-2 font-mono text-sm text-base-content/70">
					{$library.path}
				</div>
			</div>
		{/if}

		<!-- Stats -->
		{#if $ytState.stats}
			<div class="divider my-1"></div>
			<div class="flex justify-between text-sm text-base-content/60">
				<span>Active: {$ytState.stats.activeDownloads}</span>
				<span>Completed: {$ytState.stats.completedDownloads}</span>
				<span>Failed: {$ytState.stats.failedDownloads}</span>
			</div>
		{/if}

		<!-- Advanced Settings (Collapsible) -->
		<div class="divider my-1"></div>
		<button
			class="flex w-full items-center justify-between text-sm text-base-content/70 hover:text-base-content"
			onclick={() => (showAdvanced = !showAdvanced)}
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
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
			</svg>
		</button>

		{#if showAdvanced}
			<div class="mt-2 flex flex-col gap-3 rounded-lg bg-base-300 p-3">
				<p class="text-xs text-base-content/60">
					You can provide authentication to bypass bot detection.
				</p>

				<!-- PO Token -->
				<div class="form-control">
					<label class="label py-1" for="po-token">
						<span class="label-text text-sm">PO Token</span>
					</label>
					<input
						id="po-token"
						type="text"
						class="input-bordered input input-sm w-full font-mono text-xs"
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
						class="textarea-bordered textarea w-full font-mono text-xs textarea-sm"
						placeholder="key1=value1; key2=value2"
						rows="2"
						bind:value={cookies}
					></textarea>
				</div>

				<!-- Save Button -->
				<button class="btn btn-sm btn-primary" onclick={handleSaveConfig} disabled={configSaving}>
					{#if configSaving}
						<span class="loading loading-xs loading-spinner"></span>
					{/if}
					Save Config
				</button>
			</div>
		{/if}
	</div>
</div>
