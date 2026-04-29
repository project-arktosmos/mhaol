<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import DocumentCard from 'ui-lib/components/documents/DocumentCard.svelte';
	import {
		documentsService,
		DOCUMENT_SOURCES,
		FILE_TYPES,
		TYPES_BY_SOURCE,
		type Artist,
		type DocumentType,
		type DocumentSource,
		type FileEntry,
		type ImageMeta,
		type SubsLyrics
	} from '$lib/documents.service';
	import {
		fetchAlbumTrackTitles,
		fetchTmdbEpisodeTitles,
		formatSizeBytes,
		matchSubsLyricsForResult,
		matchTorrentsForResult,
		searchSource,
		searchSubsLyrics,
		searchTorrents,
		type SearchResultItem,
		type TorrentResultItem
	} from '$lib/search.service';
	import { computeCidV1Raw } from '$lib/cid';

	const docsStore = documentsService.state;

	let title = $state('');
	let description = $state('');
	let artists = $state<Artist[]>([]);
	let images = $state<ImageMeta[]>([]);
	let files = $state<FileEntry[]>([]);
	let year = $state<number | null>(null);
	let source = $state<DocumentSource>(DOCUMENT_SOURCES[0]);
	let type = $state<DocumentType>(TYPES_BY_SOURCE[DOCUMENT_SOURCES[0]][0]);
	let prefilledFiles = $state<FileEntry[]>([]);
	const availableTypes = $derived(TYPES_BY_SOURCE[source]);
	$effect(() => {
		if (!availableTypes.includes(type)) {
			type = availableTypes[0];
		}
	});
	let creating = $state(false);
	let createError = $state<string | null>(null);
	let deletingId = $state<string | null>(null);
	let showAdvanced = $state(false);
	let activeTab = $state<'covers' | 'results' | 'torrents' | 'subs-lyrics'>('covers');

	let searching = $state(false);
	let searchError = $state<string | null>(null);
	let searchResults = $state<SearchResultItem[]>([]);
	let selectedResultIndex = $state<number | null>(null);
	let torrentResults = $state<TorrentResultItem[]>([]);
	let torrentError = $state<string | null>(null);
	let subsLyricsResults = $state<SubsLyrics[]>([]);
	let subsLyricsError = $state<string | null>(null);
	let pickedSubsLyrics = $state<SubsLyrics[]>([]);
	let enrichingFiles = $state(false);
	let enrichError = $state<string | null>(null);
	const pickedSubsLyricsKey = $derived(
		new Set(pickedSubsLyrics.map((s) => `${s.source}::${s.externalId}`))
	);

	function subsLyricsKey(s: SubsLyrics): string {
		return `${s.source}::${s.externalId}`;
	}

	function isSubsLyricsPicked(s: SubsLyrics): boolean {
		return pickedSubsLyricsKey.has(subsLyricsKey(s));
	}

	function toggleSubsLyrics(s: SubsLyrics): void {
		const key = subsLyricsKey(s);
		if (pickedSubsLyrics.some((p) => subsLyricsKey(p) === key)) {
			pickedSubsLyrics = pickedSubsLyrics.filter((p) => subsLyricsKey(p) !== key);
		} else {
			pickedSubsLyrics = [...pickedSubsLyrics, s];
		}
	}

	function describeSubsLyrics(s: SubsLyrics): string {
		if (s.kind === 'lyrics') {
			const synced = (s.syncedLyrics?.length ?? 0) > 0 ? 'synced' : 'plain';
			return `${synced}${s.instrumental ? ' · instrumental' : ''}`;
		}
		const parts: string[] = [];
		if (s.format) parts.push(s.format);
		if (s.isHearingImpaired) parts.push('HI');
		return parts.join(' · ') || s.source;
	}
	const addedHashes = $derived(
		new Set(files.filter((f) => f.type === 'torrent magnet' && f.value).map((f) => f.value))
	);

	function addTorrentAsFile(t: TorrentResultItem) {
		if (!t.magnetLink) return;
		if (addedHashes.has(t.magnetLink)) return;
		files = [...files, { type: 'torrent magnet', value: t.magnetLink, title: t.title }];
	}

	async function pickResultTorrent(
		result: SearchResultItem,
		index: number,
		torrent: TorrentResultItem
	) {
		if (selectedResultIndex !== index) {
			await applyResult(result, index);
		}
		addTorrentAsFile(torrent);
		await commitCreate();
	}

	function resetForm() {
		title = '';
		description = '';
		artists = [];
		images = [];
		files = [];
		pickedSubsLyrics = [];
		year = null;
		source = DOCUMENT_SOURCES[0];
		type = TYPES_BY_SOURCE[DOCUMENT_SOURCES[0]][0];
		prefilledFiles = [];
	}

	function addArtist() {
		artists = [...artists, { name: '' }];
	}
	function removeArtist(i: number) {
		artists = artists.filter((_, idx) => idx !== i);
	}
	function addImage() {
		images = [...images, { url: '', mimeType: '', fileSize: 0, width: 0, height: 0 }];
	}
	function removeImage(i: number) {
		images = images.filter((_, idx) => idx !== i);
	}
	function addFile() {
		files = [...files, { type: FILE_TYPES[0], value: '' }];
	}
	function removeFile(i: number) {
		files = files.filter((_, idx) => idx !== i);
	}

	async function applyResult(result: SearchResultItem, index: number) {
		selectedResultIndex = index;
		title = result.title;
		description = result.description;
		artists = result.artists.map((a) => ({ ...a }));
		images = result.images.map((img) => ({ ...img }));
		files = [...prefilledFiles.map((f) => ({ ...f })), ...result.files.map((f) => ({ ...f }))];
		year = result.year;
		enrichError = null;

		const externalId = result.externalId;
		if (!externalId) return;

		const wantsEpisodes = source === 'tmdb' && type === 'tv show';
		const wantsTracks = source === 'musicbrainz' && type === 'album';
		if (!wantsEpisodes && !wantsTracks) return;

		enrichingFiles = true;
		try {
			const titles = wantsEpisodes
				? await fetchTmdbEpisodeTitles(externalId)
				: await fetchAlbumTrackTitles(externalId);
			files = [...files, ...titles.map((t) => ({ type: 'url' as const, value: '', title: t }))];
		} catch (err) {
			enrichError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			enrichingFiles = false;
		}
	}

	async function runSearch() {
		const trimmed = title.trim();
		if (!trimmed) {
			searchError = 'Enter a title to search';
			return;
		}
		searching = true;
		searchError = null;
		torrentError = null;
		subsLyricsError = null;
		subsLyricsResults = [];
		selectedResultIndex = null;
		const isMusic = type === 'album' || type === 'track';

		// For music, lyrics search runs in parallel against the title.
		// For movies/TV, subs need a TMDB id, so we wait for the source results
		// before fanning out per-id Wyzie requests.
		const subsPromise: Promise<SubsLyrics[]> = isMusic
			? searchSubsLyrics(type, trimmed)
			: Promise.resolve([]);

		const [sourceOutcome, torrentOutcome, subsOutcome] = await Promise.allSettled([
			searchSource(source, type, trimmed),
			searchTorrents(type, trimmed),
			subsPromise
		]);

		if (sourceOutcome.status === 'fulfilled') {
			searchResults = sourceOutcome.value;
		} else {
			searchResults = [];
			searchError =
				sourceOutcome.reason instanceof Error ? sourceOutcome.reason.message : 'Unknown error';
		}
		if (torrentOutcome.status === 'fulfilled') {
			torrentResults = torrentOutcome.value;
		} else {
			torrentResults = [];
			torrentError =
				torrentOutcome.reason instanceof Error ? torrentOutcome.reason.message : 'Unknown error';
		}
		if (subsOutcome.status === 'fulfilled') {
			subsLyricsResults = subsOutcome.value;
		} else {
			subsLyricsResults = [];
			subsLyricsError =
				subsOutcome.reason instanceof Error ? subsOutcome.reason.message : 'Unknown error';
		}

		if (!isMusic) {
			const ids = searchResults.map((r) => r.externalId).filter((id): id is string => Boolean(id));
			if (ids.length > 0) {
				try {
					subsLyricsResults = await searchSubsLyrics(type, trimmed, ids);
				} catch (err) {
					subsLyricsResults = [];
					subsLyricsError = err instanceof Error ? err.message : 'Unknown error';
				}
			}
		}

		searching = false;
	}

	const payloadJson = $derived(
		JSON.stringify(
			{
				title: title.trim(),
				description: description.trim(),
				artists,
				images,
				files,
				subsLyrics: pickedSubsLyrics,
				year,
				source,
				type
			},
			null,
			2
		)
	);

	let ipfsHash = $state('');
	let hashError = $state<string | null>(null);

	$effect(() => {
		const json = payloadJson;
		let cancelled = false;
		computeCidV1Raw(new TextEncoder().encode(json))
			.then((cid) => {
				if (cancelled) return;
				ipfsHash = cid;
				hashError = null;
			})
			.catch((err) => {
				if (cancelled) return;
				ipfsHash = '';
				hashError = err instanceof Error ? err.message : 'Unknown error';
			});
		return () => {
			cancelled = true;
		};
	});

	onMount(() => {
		const cleanup = documentsService.start();
		consumeUrlParams();
		return cleanup;
	});

	function consumeUrlParams() {
		const params = new URLSearchParams(window.location.search);
		const cidParam = params.get('cid');
		if (!cidParam) return;

		const sourceParam = params.get('source');
		if (sourceParam && (DOCUMENT_SOURCES as readonly string[]).includes(sourceParam)) {
			source = sourceParam as DocumentSource;
		}
		const typeParam = params.get('type');
		if (typeParam && (TYPES_BY_SOURCE[source] as readonly string[]).includes(typeParam)) {
			type = typeParam as DocumentType;
		}
		const titleParam = params.get('title')?.trim() ?? '';
		if (titleParam) title = titleParam;
		const yearParam = params.get('year');
		if (yearParam) {
			const y = parseInt(yearParam, 10);
			if (Number.isFinite(y) && y >= 1000 && y <= 9999) year = y;
		}
		const filenameParam = params.get('filename')?.trim();
		const ipfsEntry: FileEntry = {
			type: 'ipfs',
			value: cidParam,
			title: filenameParam || undefined
		};
		prefilledFiles = [ipfsEntry];
		files = [ipfsEntry];
		showAdvanced = true;

		window.history.replaceState(null, '', window.location.pathname + window.location.hash);

		if (title.trim()) {
			void runSearch();
		}
	}

	async function commitCreate(): Promise<boolean> {
		createError = null;
		const trimmedTitle = title.trim();
		if (!trimmedTitle) {
			createError = 'Title is required';
			return false;
		}
		creating = true;
		try {
			await documentsService.create({
				title: trimmedTitle,
				artists,
				description: description.trim(),
				images,
				files,
				subsLyrics: pickedSubsLyrics,
				year,
				type,
				source
			});
			resetForm();
			selectedResultIndex = null;
			searchResults = [];
			torrentResults = [];
			subsLyricsResults = [];
			searchError = null;
			torrentError = null;
			subsLyricsError = null;
			return true;
		} catch (err) {
			createError = err instanceof Error ? err.message : 'Unknown error';
			return false;
		} finally {
			creating = false;
		}
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		await commitCreate();
	}

	async function remove(id: string) {
		deletingId = id;
		try {
			await documentsService.remove(id);
		} catch (err) {
			documentsService.state.update((s) => ({
				...s,
				error: err instanceof Error ? err.message : 'Unknown error'
			}));
		} finally {
			deletingId = null;
		}
	}
</script>

<svelte:head>
	<title>Mhaol Cloud — Documents</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<header class="flex items-center justify-between gap-4">
		<div>
			<h1 class="text-2xl font-bold">Documents</h1>
			<p class="text-sm text-base-content/60">
				Documents stored in the cloud's SurrealDB. Each entry has a title, artists, description,
				images, and a list of files (ipfs CIDs, torrent magnets, or direct URLs).
			</p>
		</div>
		<button
			class="btn btn-outline btn-sm"
			onclick={() => documentsService.refresh()}
			disabled={$docsStore.loading}
		>
			Refresh
		</button>
	</header>

	{#if $docsStore.error}
		<div class="alert alert-error">
			<span>{$docsStore.error}</span>
		</div>
	{/if}

	<section class="card border border-base-content/10 bg-base-200 p-4">
		<h2 class="mb-3 text-lg font-semibold">Add a document</h2>
		<form class="flex flex-col gap-3" onsubmit={submit}>
			<div class="overflow-x-auto rounded-box border border-base-content/10">
				<table class="table table-sm">
					<tbody>
						<tr>
							<th class="w-32 align-middle">Source</th>
							<td>
								<select
									class="select-bordered select w-full select-sm"
									bind:value={source}
									disabled={creating}
								>
									{#each DOCUMENT_SOURCES as option (option)}
										<option value={option}>{option}</option>
									{/each}
								</select>
							</td>
						</tr>
						<tr>
							<th class="w-32 align-middle">Type</th>
							<td>
								<select
									class="select-bordered select w-full select-sm"
									bind:value={type}
									disabled={creating}
								>
									{#each availableTypes as option (option)}
										<option value={option}>{option}</option>
									{/each}
								</select>
							</td>
						</tr>
						<tr>
							<th class="w-32 align-middle">Title</th>
							<td>
								<div class="flex items-center gap-2">
									<input
										type="text"
										class="input-bordered input input-sm w-full"
										placeholder="Project brief"
										bind:value={title}
										disabled={creating}
									/>
									<button
										type="button"
										class={classNames('btn btn-outline btn-sm', {
											'btn-disabled': searching || creating
										})}
										onclick={runSearch}
										disabled={searching || creating}
									>
										{searching ? 'Searching…' : 'Search'}
									</button>
								</div>
							</td>
						</tr>
						<tr>
							<td colspan="2" class="bg-base-100/50">
								<button
									type="button"
									class="btn btn-ghost btn-xs"
									onclick={() => (showAdvanced = !showAdvanced)}
								>
									{showAdvanced ? '▾ Hide' : '▸ Show'} more fields (year, artists, images, files, description)
								</button>
							</td>
						</tr>
						{#if showAdvanced}
							<tr>
								<th class="w-32 align-middle">Year</th>
								<td>
									<input
										type="number"
										class="input-bordered input input-sm w-32"
										placeholder="e.g. 1999"
										min="1000"
										max="9999"
										bind:value={year}
										disabled={creating}
									/>
								</td>
							</tr>
							<tr>
								<th class="w-32 align-top">Artists</th>
								<td>
									<div class="flex flex-col gap-2">
										{#each artists as _, i (i)}
											<div class="flex items-center gap-2">
												<input
													type="text"
													class="input-bordered input input-sm w-1/3"
													placeholder="Name"
													bind:value={artists[i].name}
													disabled={creating}
												/>
												<input
													type="text"
													class="input-bordered input input-sm w-1/3"
													placeholder="URL"
													bind:value={artists[i].url}
													disabled={creating}
												/>
												<input
													type="text"
													class="input-bordered input input-sm w-1/3"
													placeholder="Image URL"
													bind:value={artists[i].imageUrl}
													disabled={creating}
												/>
												<button
													type="button"
													class="btn text-error btn-ghost btn-xs"
													onclick={() => removeArtist(i)}
													disabled={creating}
													aria-label="Remove artist"
												>
													×
												</button>
											</div>
										{/each}
										<div>
											<button
												type="button"
												class="btn btn-outline btn-xs"
												onclick={addArtist}
												disabled={creating}
											>
												+ Add artist
											</button>
										</div>
									</div>
								</td>
							</tr>
							<tr>
								<th class="w-32 align-top">Images</th>
								<td>
									<div class="flex flex-col gap-2">
										{#each images as _, i (i)}
											<div class="flex flex-wrap items-center gap-2">
												<input
													type="text"
													class="input-bordered input input-sm min-w-48 flex-1"
													placeholder="URL"
													bind:value={images[i].url}
													disabled={creating}
												/>
												<input
													type="text"
													class="input-bordered input input-sm w-32"
													placeholder="Mime type"
													bind:value={images[i].mimeType}
													disabled={creating}
												/>
												<input
													type="number"
													class="input-bordered input input-sm w-28"
													placeholder="Size (B)"
													bind:value={images[i].fileSize}
													disabled={creating}
												/>
												<input
													type="number"
													class="input-bordered input input-sm w-20"
													placeholder="W"
													bind:value={images[i].width}
													disabled={creating}
												/>
												<input
													type="number"
													class="input-bordered input input-sm w-20"
													placeholder="H"
													bind:value={images[i].height}
													disabled={creating}
												/>
												<button
													type="button"
													class="btn text-error btn-ghost btn-xs"
													onclick={() => removeImage(i)}
													disabled={creating}
													aria-label="Remove image"
												>
													×
												</button>
											</div>
										{/each}
										<div>
											<button
												type="button"
												class="btn btn-outline btn-xs"
												onclick={addImage}
												disabled={creating}
											>
												+ Add image
											</button>
										</div>
									</div>
								</td>
							</tr>
							<tr>
								<th class="w-32 align-top">Files</th>
								<td>
									<div class="flex flex-col gap-2">
										{#if enrichingFiles}
											<p class="text-xs text-base-content/60">Loading episodes/tracks…</p>
										{/if}
										{#if enrichError}
											<p class="text-xs text-error">
												Could not load episodes/tracks: {enrichError}
											</p>
										{/if}
										{#each files as _, i (i)}
											<div class="flex flex-wrap items-center gap-2">
												<select
													class="select-bordered select w-40 select-sm"
													bind:value={files[i].type}
													disabled={creating}
												>
													{#each FILE_TYPES as option (option)}
														<option value={option}>{option}</option>
													{/each}
												</select>
												<input
													type="text"
													class="input-bordered input input-sm min-w-48 flex-1"
													placeholder="Value (CID, magnet:?…, https://…)"
													bind:value={files[i].value}
													disabled={creating}
												/>
												<input
													type="text"
													class="input-bordered input input-sm w-48"
													placeholder="Title (optional)"
													bind:value={files[i].title}
													disabled={creating}
												/>
												<button
													type="button"
													class="btn text-error btn-ghost btn-xs"
													onclick={() => removeFile(i)}
													disabled={creating}
													aria-label="Remove file"
												>
													×
												</button>
											</div>
										{/each}
										<div>
											<button
												type="button"
												class="btn btn-outline btn-xs"
												onclick={addFile}
												disabled={creating}
											>
												+ Add file
											</button>
										</div>
									</div>
								</td>
							</tr>
							<tr>
								<th class="w-32 align-top">Description</th>
								<td>
									<input
										type="text"
										class="input-bordered input input-sm w-full"
										placeholder="Short summary of the document"
										bind:value={description}
										disabled={creating}
									/>
								</td>
							</tr>
							<tr>
								<th class="w-32 align-top">JSON</th>
								<td>
									<div class="flex flex-col gap-2">
										<textarea
											class="textarea-bordered textarea h-40 w-full font-mono text-xs"
											readonly
											disabled
											value={payloadJson}
										></textarea>
										<input
											type="text"
											class="input-bordered input input-sm w-full font-mono text-xs"
											readonly
											disabled
											value={hashError ?? ipfsHash}
										/>
									</div>
								</td>
							</tr>
						{/if}
					</tbody>
				</table>
			</div>
			<div>
				<button
					type="submit"
					class={classNames('btn btn-sm btn-primary', { 'btn-disabled': creating })}
					disabled={creating}
				>
					{creating ? 'Creating…' : 'Create'}
				</button>
			</div>
		</form>
		{#if createError}
			<p class="mt-2 text-sm text-error">{createError}</p>
		{/if}
	</section>

	{#if searchResults.length > 0 || torrentResults.length > 0 || subsLyricsResults.length > 0 || searching || searchError || torrentError || subsLyricsError}
		<section class="card border border-base-content/10 bg-base-200 p-4">
			<div class="mb-3 flex items-center justify-between">
				<h2 class="text-lg font-semibold">Results</h2>
			</div>
			<div role="tablist" class="tabs-bordered mb-3 tabs">
				<button
					type="button"
					role="tab"
					class={classNames('tab', { 'tab-active': activeTab === 'covers' })}
					onclick={() => (activeTab = 'covers')}
				>
					Covers
				</button>
				<button
					type="button"
					role="tab"
					class={classNames('tab', { 'tab-active': activeTab === 'results' })}
					onclick={() => (activeTab = 'results')}
				>
					Search results
				</button>
				<button
					type="button"
					role="tab"
					class={classNames('tab', { 'tab-active': activeTab === 'torrents' })}
					onclick={() => (activeTab = 'torrents')}
				>
					Torrents
				</button>
				<button
					type="button"
					role="tab"
					class={classNames('tab', { 'tab-active': activeTab === 'subs-lyrics' })}
					onclick={() => (activeTab = 'subs-lyrics')}
				>
					Subs/Lyrics{subsLyricsResults.length > 0 ? ` (${subsLyricsResults.length})` : ''}
				</button>
			</div>

			{#if activeTab === 'covers'}
				{#if searchResults.length === 0}
					<p class="text-sm text-base-content/60">
						No results yet — type a title and click Search.
					</p>
				{:else}
					<div class="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-3">
						{#each searchResults as result, i (result.externalId ?? i)}
							{@const cover = result.images[0]?.url}
							{@const matches = matchTorrentsForResult(result, torrentResults)}
							{@const subsMatches = matchSubsLyricsForResult(result, subsLyricsResults)}
							<div
								class={classNames(
									'flex overflow-hidden rounded-box border bg-base-100 transition',
									{
										'border-primary': selectedResultIndex === i,
										'border-base-content/10': selectedResultIndex !== i
									}
								)}
							>
								<button
									type="button"
									class="flex w-32 shrink-0 flex-col bg-base-300 text-left hover:bg-base-200"
									onclick={() => applyResult(result, i)}
									aria-label={`Use "${result.title}" to fill form`}
								>
									<div class="aspect-[2/3] w-full">
										{#if cover}
											<img
												src={cover}
												alt={result.title}
												class="h-full w-full object-cover"
												loading="lazy"
											/>
										{:else}
											<div
												class="flex h-full w-full items-center justify-center text-xs text-base-content/40"
											>
												No image
											</div>
										{/if}
									</div>
									<div class="flex flex-col gap-1 p-2">
										<span class="line-clamp-2 text-sm font-medium">{result.title}</span>
										{#if result.year}
											<span class="text-xs text-base-content/60">{result.year}</span>
										{/if}
									</div>
								</button>
								<div class="flex flex-1 flex-col border-l border-base-content/10 p-2">
									<span class="mb-1 text-xs font-semibold text-base-content/60 uppercase">
										Torrents{matches.length > 0 ? ` (${matches.length})` : ''}
									</span>
									{#if matches.length === 0}
										<p class="text-xs text-base-content/50">No matching torrents.</p>
									{:else}
										<div class="flex max-h-48 flex-col gap-1 overflow-y-auto">
											{#each matches as torrent (torrent.infoHash)}
												<button
													type="button"
													class={classNames(
														'flex flex-wrap items-center gap-2 rounded border border-base-content/10 px-2 py-1 text-left text-xs hover:bg-base-200',
														{ 'opacity-60': addedHashes.has(torrent.magnetLink) }
													)}
													onclick={() => pickResultTorrent(result, i, torrent)}
													title={torrent.title}
												>
													<span class="font-medium">{torrent.quality ?? '—'}</span>
													<span class="text-success">↑{torrent.seeders}</span>
													<span class="text-warning">↓{torrent.leechers}</span>
													<span class="text-base-content/60"
														>{formatSizeBytes(torrent.sizeBytes)}</span
													>
													{#if addedHashes.has(torrent.magnetLink)}
														<span class="ml-auto">✓</span>
													{/if}
												</button>
											{/each}
										</div>
									{/if}
									<span class="mt-2 mb-1 text-xs font-semibold text-base-content/60 uppercase">
										Subs/Lyrics{subsMatches.length > 0 ? ` (${subsMatches.length})` : ''}
									</span>
									{#if subsMatches.length === 0}
										<p class="text-xs text-base-content/50">No matching subs/lyrics.</p>
									{:else}
										<div class="flex max-h-48 flex-col gap-1 overflow-y-auto">
											{#each subsMatches as sub (subsLyricsKey(sub))}
												{@const picked = isSubsLyricsPicked(sub)}
												<button
													type="button"
													class={classNames(
														'flex flex-wrap items-center gap-2 rounded border border-base-content/10 px-2 py-1 text-left text-xs hover:bg-base-200',
														{ 'opacity-60': picked }
													)}
													onclick={() => toggleSubsLyrics(sub)}
													title={sub.display ?? `${sub.source}:${sub.externalId}`}
												>
													<span class="font-medium">
														{sub.kind === 'lyrics' ? 'LRC' : (sub.language ?? sub.format ?? '?')}
													</span>
													<span class="text-base-content/60">{describeSubsLyrics(sub)}</span>
													{#if picked}
														<span class="ml-auto">✓</span>
													{/if}
												</button>
											{/each}
										</div>
									{/if}
								</div>
							</div>
						{/each}
					</div>
				{/if}
			{:else if activeTab === 'results'}
				{#if searchError}
					<div class="mb-3 alert alert-error">
						<span>{searchError}</span>
					</div>
				{/if}
				{#if searching}
					<p class="text-sm text-base-content/60">Searching…</p>
				{:else if searchResults.length === 0}
					<p class="text-sm text-base-content/60">
						No results yet — type a title and click Search.
					</p>
				{:else}
					<div class="overflow-x-auto rounded-box border border-base-content/10">
						<table class="table table-sm">
							<thead>
								<tr>
									<th>Title</th>
									<th>Year</th>
									<th>Artists</th>
									<th>Images</th>
									<th>Files</th>
									<th>Description</th>
									<th>External ID</th>
								</tr>
							</thead>
							<tbody>
								{#each searchResults as result, i (result.externalId ?? i)}
									<tr
										class={classNames('cursor-pointer hover:bg-base-300', {
											'bg-base-300': selectedResultIndex === i
										})}
										onclick={() => applyResult(result, i)}
									>
										<td class="font-medium">{result.title}</td>
										<td class="text-xs">{result.year ?? ''}</td>
										<td class="text-xs">{result.artists.map((a) => a.name).join(', ')}</td>
										<td class="text-xs">{result.images.length}</td>
										<td class="text-xs">{result.files.length}</td>
										<td class="max-w-md text-xs whitespace-pre-wrap text-base-content/80"
											>{result.description}</td
										>
										<td class="font-mono text-xs text-base-content/70">{result.externalId ?? ''}</td
										>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}
			{:else if activeTab === 'subs-lyrics'}
				{#if subsLyricsError}
					<div class="mb-3 alert alert-error">
						<span>{subsLyricsError}</span>
					</div>
				{/if}
				{#if searching}
					<p class="text-sm text-base-content/60">Searching…</p>
				{:else if subsLyricsResults.length === 0}
					<p class="text-sm text-base-content/60">
						No subs/lyrics yet — pick a music or movie/TV type and search.
					</p>
				{:else}
					<div class="overflow-x-auto rounded-box border border-base-content/10">
						<table class="table table-sm">
							<thead>
								<tr>
									<th>Kind</th>
									<th>Source</th>
									<th>Title / Lang</th>
									<th>Format</th>
									<th>Detail</th>
									<th class="w-16">Picked</th>
								</tr>
							</thead>
							<tbody>
								{#each subsLyricsResults as sub (subsLyricsKey(sub))}
									{@const picked = isSubsLyricsPicked(sub)}
									<tr
										class={classNames('cursor-pointer hover:bg-base-300', {
											'bg-base-300': picked
										})}
										onclick={() => toggleSubsLyrics(sub)}
									>
										<td class="text-xs">{sub.kind}</td>
										<td class="text-xs">{sub.source}</td>
										<td class="text-xs"
											>{sub.kind === 'lyrics'
												? `${sub.artistName ?? ''} — ${sub.trackName ?? ''}`
												: (sub.display ?? sub.language ?? '')}</td
										>
										<td class="text-xs">{sub.format ?? ''}</td>
										<td class="text-xs text-base-content/70">{describeSubsLyrics(sub)}</td>
										<td class="text-center text-xs">{picked ? '✓' : ''}</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}
			{:else if activeTab === 'torrents'}
				{#if torrentError}
					<div class="mb-3 alert alert-error">
						<span>{torrentError}</span>
					</div>
				{/if}
				{#if searching}
					<p class="text-sm text-base-content/60">Searching…</p>
				{:else if torrentResults.length === 0}
					<p class="text-sm text-base-content/60">No torrents yet — hit Search.</p>
				{:else}
					<div class="overflow-x-auto rounded-box border border-base-content/10">
						<table class="table table-sm">
							<thead>
								<tr>
									<th>Title</th>
									<th>Year</th>
									<th>Quality</th>
									<th>Name</th>
									<th>Stats</th>
									<th class="w-16">Added</th>
								</tr>
							</thead>
							<tbody>
								{#each torrentResults as torrent (torrent.infoHash)}
									<tr
										class={classNames('cursor-pointer hover:bg-base-300', {
											'opacity-60': addedHashes.has(torrent.magnetLink)
										})}
										onclick={() => addTorrentAsFile(torrent)}
									>
										<td class="font-medium">{torrent.parsedTitle}</td>
										<td class="text-xs">{torrent.year ?? ''}</td>
										<td class="text-xs">{torrent.quality ?? ''}</td>
										<td class="max-w-md text-xs break-all text-base-content/70">{torrent.title}</td>
										<td class="text-xs text-base-content/70">{torrent.description}</td>
										<td class="text-center text-xs"
											>{addedHashes.has(torrent.magnetLink) ? '✓' : ''}</td
										>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}
			{/if}
		</section>
	{/if}

	<section class="flex flex-col gap-3">
		<h2 class="text-lg font-semibold">Existing documents</h2>
		{#if $docsStore.loading && $docsStore.documents.length === 0}
			<p class="text-sm text-base-content/60">Loading…</p>
		{:else if $docsStore.documents.length === 0}
			<p class="text-sm text-base-content/60">No documents yet.</p>
		{:else}
			<div
				class="grid grid-cols-1 gap-4 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5"
			>
				{#each $docsStore.documents as doc (doc.id)}
					<DocumentCard document={doc} onRemove={remove} removing={deletingId === doc.id} />
				{/each}
			</div>
		{/if}
	</section>
</div>
