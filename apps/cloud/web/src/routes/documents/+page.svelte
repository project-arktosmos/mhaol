<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import {
		documentsService,
		DOCUMENT_SOURCES,
		FILE_TYPES,
		TYPES_BY_SOURCE,
		type Artist,
		type DocumentType,
		type DocumentSource,
		type FileEntry,
		type ImageMeta
	} from '$lib/documents.service';
	import {
		fetchAlbumTrackTitles,
		fetchTmdbEpisodeTitles,
		searchSource,
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
	const availableTypes = $derived(TYPES_BY_SOURCE[source]);
	$effect(() => {
		if (!availableTypes.includes(type)) {
			type = availableTypes[0];
		}
	});
	let creating = $state(false);
	let createError = $state<string | null>(null);
	let deletingId = $state<string | null>(null);

	let searching = $state(false);
	let searchError = $state<string | null>(null);
	let searchResults = $state<SearchResultItem[]>([]);
	let selectedResultIndex = $state<number | null>(null);
	let torrentResults = $state<TorrentResultItem[]>([]);
	let torrentError = $state<string | null>(null);
	let enrichingFiles = $state(false);
	let enrichError = $state<string | null>(null);
	const addedHashes = $derived(
		new Set(files.filter((f) => f.type === 'torrent magnet' && f.value).map((f) => f.value))
	);

	function addTorrentAsFile(t: TorrentResultItem) {
		if (!t.magnetLink) return;
		if (addedHashes.has(t.magnetLink)) return;
		files = [...files, { type: 'torrent magnet', value: t.magnetLink, title: t.title }];
	}

	function resetForm() {
		title = '';
		description = '';
		artists = [];
		images = [];
		files = [];
		year = null;
		source = DOCUMENT_SOURCES[0];
		type = TYPES_BY_SOURCE[DOCUMENT_SOURCES[0]][0];
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
		files = result.files.map((f) => ({ ...f }));
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
		selectedResultIndex = null;
		const [sourceOutcome, torrentOutcome] = await Promise.allSettled([
			searchSource(source, type, trimmed),
			searchTorrents(type, trimmed)
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
		documentsService.refresh();
	});

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		createError = null;
		const trimmedTitle = title.trim();
		if (!trimmedTitle) {
			createError = 'Title is required';
			return;
		}
		creating = true;
		try {
			await documentsService.create({
				title: trimmedTitle,
				artists,
				description: description.trim(),
				images,
				files,
				year,
				type,
				source
			});
			resetForm();
			selectedResultIndex = null;
		} catch (err) {
			createError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			creating = false;
		}
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

	function formatDate(value: string): string {
		try {
			return new Date(value).toLocaleString();
		} catch {
			return value;
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
										<p class="text-xs text-error">Could not load episodes/tracks: {enrichError}</p>
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

	<div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
		<section class="card border border-base-content/10 bg-base-200 p-4">
			<h2 class="mb-3 text-lg font-semibold">Search results</h2>
			<p class="mb-3 text-xs text-base-content/60">
				Results from <code>{source}</code> for type <code>{type}</code>.
			</p>
			{#if searchError}
				<div class="mb-3 alert alert-error">
					<span>{searchError}</span>
				</div>
			{/if}
			{#if searching}
				<p class="text-sm text-base-content/60">Searching…</p>
			{:else if searchResults.length === 0}
				<p class="text-sm text-base-content/60">No results yet — type a title and click Search.</p>
			{:else}
				<p class="mb-2 text-xs text-base-content/60">Click a result to fill in the form above.</p>
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
									<td class="font-mono text-xs text-base-content/70">{result.externalId ?? ''}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/if}
		</section>

		<section class="card border border-base-content/10 bg-base-200 p-4">
			<h2 class="mb-3 text-lg font-semibold">Torrents (PirateBay)</h2>
			<p class="mb-3 text-xs text-base-content/60">
				Torrents matching the title in the category for <code>{type}</code>.
			</p>
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
				<p class="mb-2 text-xs text-base-content/60">
					Click a torrent to add it as a file to the form above.
				</p>
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
									<td class="max-w-md text-xs break-all text-base-content/70"
										>{torrent.title}</td
									>
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
		</section>
	</div>

	<section class="card border border-base-content/10 bg-base-200 p-4">
		<h2 class="mb-3 text-lg font-semibold">Create payload preview</h2>
		<p class="mb-3 text-xs text-base-content/60">
			JSON body that will be POSTed to <code>/api/documents</code> when you hit Create. The IPFS hash
			is the CIDv1 (raw, sha2-256) of these bytes and updates as you type.
		</p>
		<div class="flex flex-col gap-3">
			<label class="flex flex-col gap-1">
				<span class="text-xs font-semibold text-base-content/60 uppercase">JSON</span>
				<textarea
					class="textarea-bordered textarea h-40 w-full font-mono text-xs"
					readonly
					disabled
					value={payloadJson}
				></textarea>
			</label>
			<label class="flex flex-col gap-1">
				<span class="text-xs font-semibold text-base-content/60 uppercase">IPFS hash (CIDv1)</span>
				<input
					type="text"
					class="input-bordered input input-sm w-full font-mono text-xs"
					readonly
					disabled
					value={hashError ?? ipfsHash}
				/>
			</label>
		</div>
	</section>

	<section class="flex flex-col gap-3">
		<h2 class="text-lg font-semibold">Existing documents</h2>
		{#if $docsStore.loading && $docsStore.documents.length === 0}
			<p class="text-sm text-base-content/60">Loading…</p>
		{:else if $docsStore.documents.length === 0}
			<p class="text-sm text-base-content/60">No documents yet.</p>
		{:else}
			<div class="overflow-x-auto rounded-box border border-base-content/10">
				<table class="table table-sm">
					<thead>
						<tr>
							<th>ID</th>
							<th>Type</th>
							<th>Source</th>
							<th>Title</th>
							<th>Year</th>
							<th>Artists</th>
							<th>Images</th>
							<th>Files</th>
							<th>Description</th>
							<th>Created</th>
							<th class="w-24"></th>
						</tr>
					</thead>
					<tbody>
						{#each $docsStore.documents as doc (doc.id)}
							<tr>
								<td class="font-mono text-xs text-base-content/70">{doc.id}</td>
								<td class="text-xs">{doc.type}</td>
								<td class="text-xs">{doc.source}</td>
								<td class="font-medium">{doc.title}</td>
								<td class="text-xs">{doc.year ?? ''}</td>
								<td class="text-xs">{(doc.artists ?? []).map((a) => a.name).join(', ')}</td>
								<td class="text-xs">{(doc.images ?? []).length}</td>
								<td class="text-xs">{(doc.files ?? []).length}</td>
								<td class="max-w-md text-xs whitespace-pre-wrap text-base-content/80"
									>{doc.description}</td
								>
								<td class="text-xs text-base-content/60">{formatDate(doc.created_at)}</td>
								<td class="text-right">
									<button
										class="btn text-error btn-ghost btn-xs"
										onclick={() => remove(doc.id)}
										disabled={deletingId === doc.id}
									>
										{deletingId === doc.id ? 'Removing…' : 'Remove'}
									</button>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</section>
</div>
