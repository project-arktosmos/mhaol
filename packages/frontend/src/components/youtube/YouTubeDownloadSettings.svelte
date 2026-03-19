<script lang="ts">
	import classNames from 'classnames';
	import { youtubeService } from '$services/youtube.service';
	import { libraryService } from '$services/library.service';
	import type { Library } from '$types/library.type';
	import LibraryAddForm from '$components/libraries/LibraryAddForm.svelte';
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
	const libraries = libraryService.store;
	const libraryState = libraryService.state;

	// Advanced config state
	let showAdvanced = false;
	let poToken = '';
	let cookies = '';
	let configSaving = false;

	// Library selection
	let selectedLibraryId: string = '';
	let showInlineAddForm = false;
	let previousLibraryCount = 0;

	// Sync auth fields from settings store
	$: if ($settings.poToken !== undefined) poToken = $settings.poToken;
	$: if ($settings.cookies !== undefined) cookies = $settings.cookies;

	// Auto-select library matching current libraryId, or first library if none matches
	$: if ($libraries.length > 0) {
		if ($settings.libraryId) {
			const match = $libraries.find((lib: Library) => String(lib.id) === $settings.libraryId);
			if (match) {
				selectedLibraryId = String(match.id);
			}
		}
		if (!selectedLibraryId) {
			const first = $libraries[0];
			selectedLibraryId = String(first.id);
			youtubeService.setLibrary(String(first.id));
		}
	}

	// Detect when inline add form closes (cancel or successful add)
	$: if (showInlineAddForm && !$libraryState.showAddForm) {
		showInlineAddForm = false;
		// If a new library was added, auto-select it
		if ($libraries.length > previousLibraryCount) {
			const newest = $libraries[$libraries.length - 1];
			selectedLibraryId = String(newest.id);
			youtubeService.setLibrary(String(newest.id));
		}
	}

	$: previousLibraryCount = $libraries.length;

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

	async function handleLibrarySelect(event: Event) {
		const target = event.target as HTMLSelectElement;
		const libraryId = target.value;
		if (!libraryId) return;

		const library = $libraries.find((lib: Library) => String(lib.id) === libraryId);
		if (library) {
			selectedLibraryId = String(library.id);
			youtubeService.setLibrary(String(library.id));
		}
	}

	function handleShowAddForm() {
		showInlineAddForm = true;
		libraryService.openAddForm();
	}

	async function handleSaveConfig() {
		configSaving = true;
		await youtubeService.setConfig({
			poToken: poToken.trim() || null,
			cookies: cookies.trim() || null
		});
		configSaving = false;
	}

	// Reactive downloader status
	$: downloaderStatus = $state.downloaderStatus;
	$: downloaderAvailable = downloaderStatus?.available ?? false;
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
					class="select-bordered select w-full"
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
					class="select-bordered select w-full"
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
					class="select-bordered select w-full"
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
					class="select-bordered select w-full"
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

		<!-- Download Library -->
		<div class="form-control">
			<label class="label" for="library-select">
				<span class="label-text">Download Library</span>
			</label>

			{#if $libraries.length > 0}
				<div class="flex items-center gap-2">
					<select
						id="library-select"
						class="select-bordered select flex-1"
						value={selectedLibraryId}
						on:change={handleLibrarySelect}
					>
						<option value="" disabled>Select a library...</option>
						{#each $libraries as library (library.id)}
							<option value={String(library.id)}>
								{library.name}
							</option>
						{/each}
					</select>
					<button class="btn btn-ghost btn-sm" on:click={handleShowAddForm} title="Add new library">
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-4 w-4"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
							stroke-width="2"
						>
							<path stroke-linecap="round" stroke-linejoin="round" d="M12 4v16m8-8H4" />
						</svg>
					</button>
				</div>
			{:else}
				<div class="rounded-lg bg-base-300 p-4 text-center">
					<p class="mb-2 text-sm text-base-content/60">No libraries configured</p>
					<button class="btn btn-sm btn-primary" on:click={handleShowAddForm}>
						Create Library
					</button>
				</div>
			{/if}
		</div>

		<!-- Inline Library Add Form -->
		{#if showInlineAddForm && $libraryState.showAddForm}
			<LibraryAddForm />
		{/if}

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
				<button class="btn btn-sm btn-primary" on:click={handleSaveConfig} disabled={configSaving}>
					{#if configSaving}
						<span class="loading loading-xs loading-spinner"></span>
					{/if}
					Save Config
				</button>
			</div>
		{/if}
	</div>
</div>
