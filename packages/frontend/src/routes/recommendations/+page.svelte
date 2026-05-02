<script lang="ts">
	import { onMount } from 'svelte';
	import { base } from '$app/paths';
	import { goto } from '$app/navigation';
	import { page as pageStore } from '$app/state';
	import { listRecommendations, type Recommendation } from '$lib/recommendations.service';
	import {
		firkinsService,
		type Artist,
		type FirkinAddon,
		type ImageMeta,
		type FileEntry,
		type Review,
		type Trailer
	} from '$lib/firkins.service';
	import { userIdentityService } from '$lib/user-identity.service';

	const userIdentityState = userIdentityService.state;

	let rows = $state<Recommendation[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let lastLoadedAddress: string | null = null;
	let bookmarkingId = $state<string | null>(null);
	let bookmarkError = $state<string | null>(null);

	const addonFilter = $derived(pageStore.url.searchParams.get('addon') ?? '');
	const visibleRows = $derived(addonFilter ? rows.filter((r) => r.addon === addonFilter) : rows);

	$effect(() => {
		const address = $userIdentityState.identity?.address;
		if (!address) {
			rows = [];
			lastLoadedAddress = null;
			return;
		}
		if (lastLoadedAddress === address) return;
		lastLoadedAddress = address;
		void load(address);
	});

	async function load(address: string) {
		loading = true;
		error = null;
		try {
			rows = await listRecommendations(address);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			loading = false;
		}
	}

	function truncateCid(cid: string): string {
		if (cid.length <= 18) return cid;
		return `${cid.slice(0, 10)}…${cid.slice(-6)}`;
	}

	function formatScore(value: number): string {
		const rounded = Math.round(value * 10) / 10;
		return Number.isInteger(rounded) ? rounded.toFixed(0) : rounded.toFixed(1);
	}

	function formatVotes(count: number): string {
		if (count >= 1_000_000) return `${(count / 1_000_000).toFixed(1)}M votes`;
		if (count >= 1000) return `${(count / 1000).toFixed(1)}k votes`;
		return `${count} vote${count === 1 ? '' : 's'}`;
	}

	function copyHash(cid: string) {
		void navigator.clipboard?.writeText(cid).catch(() => {
			// silent — clipboard may be unavailable
		});
	}

	function buildUpstreamSourceFiles(addon: string, upstreamId: string): FileEntry[] {
		if (!upstreamId) return [];
		if (addon === 'musicbrainz') {
			return [
				{
					type: 'url',
					value: `https://musicbrainz.org/release-group/${upstreamId}`,
					title: 'MusicBrainz Release Group'
				}
			];
		}
		if (addon === 'tmdb-tv') {
			return [
				{
					type: 'url',
					value: `https://www.themoviedb.org/tv/${upstreamId}`,
					title: 'TMDB TV Show'
				}
			];
		}
		if (addon === 'tmdb-movie') {
			return [
				{
					type: 'url',
					value: `https://www.themoviedb.org/movie/${upstreamId}`,
					title: 'TMDB Movie'
				}
			];
		}
		return [];
	}

	function imagesFromRow(row: Recommendation): ImageMeta[] {
		return [row.posterUrl, row.backdropUrl]
			.filter((url): url is string => Boolean(url))
			.map((url) => ({ url, mimeType: 'image/jpeg', fileSize: 0, width: 0, height: 0 }));
	}

	async function fetchUpstreamMetadata(
		addon: string,
		upstreamId: string
	): Promise<{ artists: Artist[]; trailers: Trailer[]; reviews: Review[] }> {
		try {
			const res = await fetch(
				`${base}/api/catalog/${encodeURIComponent(addon)}/${encodeURIComponent(upstreamId)}/metadata`,
				{ cache: 'no-store' }
			);
			if (!res.ok) return { artists: [], trailers: [], reviews: [] };
			const body = (await res.json()) as {
				artists?: Artist[];
				trailers?: Trailer[];
				reviews?: Review[];
			};
			return {
				artists: Array.isArray(body.artists) ? body.artists : [],
				trailers: Array.isArray(body.trailers) ? body.trailers : [],
				reviews: Array.isArray(body.reviews) ? body.reviews : []
			};
		} catch {
			return { artists: [], trailers: [], reviews: [] };
		}
	}

	async function bookmark(row: Recommendation) {
		if (bookmarkingId) return;
		if (!row.upstreamId) {
			bookmarkError = 'recommendation is missing its upstream id';
			return;
		}
		bookmarkingId = row.firkinId;
		bookmarkError = null;
		try {
			const { artists, trailers, reviews } = await fetchUpstreamMetadata(row.addon, row.upstreamId);
			const created = await firkinsService.create({
				title: row.title,
				artists,
				description: row.description ?? '',
				images: imagesFromRow(row),
				files: buildUpstreamSourceFiles(row.addon, row.upstreamId),
				year: row.year,
				addon: row.addon as FirkinAddon,
				trailers,
				reviews: reviews.length > 0 ? reviews : (row.reviews ?? [])
			});
			// The detail page fires `loadRelated` + `ingestRecommendations`
			// on mount, so navigating there is what "pulls their
			// recommendations" — no need to duplicate that flow here.
			await goto(`${base}/catalog/${encodeURIComponent(created.id)}`);
		} catch (err) {
			bookmarkError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			bookmarkingId = null;
		}
	}

	onMount(() => {
		const address = $userIdentityState.identity?.address;
		if (address) {
			lastLoadedAddress = address;
			void load(address);
		}
	});
</script>

<svelte:head>
	<title>Mhaol Cloud — Recommendations</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<header class="flex flex-col gap-1">
		<div class="flex flex-wrap items-center gap-3">
			<h1 class="text-2xl font-semibold">Recommendations</h1>
			{#if addonFilter}
				<span class="badge gap-2 badge-outline">
					<code>{addonFilter}</code>
					<a href={`${base}/recommendations`} class="link" title="Clear addon filter">×</a>
				</span>
			{/if}
		</div>
		<p class="text-sm text-base-content/60">
			Items the catalog API has recommended to you, indexed by their virtual IPFS hash. Counts only
			update when you visit a real <code>/catalog/[ipfsHash]</code> detail page; virtual catalog pages
			don't contribute. Each source firkin contributes at most once per item. Bookmark a row to mint a
			real firkin and pull its own recommendations into this list.
		</p>
	</header>

	{#if !$userIdentityState.identity}
		<div class="alert alert-warning">
			<span>Sign in on the Profile page to see your recommendations.</span>
		</div>
	{:else}
		{#if error}
			<div class="alert alert-error"><span>{error}</span></div>
		{/if}
		{#if bookmarkError}
			<div class="alert alert-error"><span>{bookmarkError}</span></div>
		{/if}

		<section class="card border border-base-content/10 bg-base-200">
			<div class="card-body p-0">
				<div class="overflow-x-auto">
					<table class="table table-zebra">
						<thead>
							<tr>
								<th class="w-16">Poster</th>
								<th>Title</th>
								<th class="w-40">IPFS hash</th>
								<th class="w-20 text-right">Count</th>
								<th class="w-48">Rating</th>
								<th class="w-32"></th>
							</tr>
						</thead>
						<tbody>
							{#if loading && visibleRows.length === 0}
								<tr>
									<td colspan="6" class="text-center text-base-content/60">Loading…</td>
								</tr>
							{:else if visibleRows.length === 0}
								<tr>
									<td colspan="6" class="text-center text-base-content/60">
										{#if addonFilter}
											No recommendations for <code>{addonFilter}</code> yet.
										{:else}
											No recommendations yet — visit a movie, TV show, or album detail page to start
											collecting.
										{/if}
									</td>
								</tr>
							{:else}
								{#each visibleRows as row (row.firkinId)}
									<tr>
										<td>
											{#if row.posterUrl}
												<img
													src={row.posterUrl}
													alt={row.title}
													class="h-16 w-12 shrink-0 rounded object-cover"
													loading="lazy"
												/>
											{:else}
												<div
													class="flex h-16 w-12 shrink-0 items-center justify-center rounded bg-base-300 text-base-content/30"
												>
													<svg class="h-6 w-6" fill="currentColor" viewBox="0 0 24 24">
														<path
															d="M21 3H3c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h18c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H3V5h18v14z"
														/>
													</svg>
												</div>
											{/if}
										</td>
										<td>
											<div class="flex flex-col gap-0.5">
												<span class="font-medium">{row.title}</span>
												<div class="flex flex-wrap items-center gap-1 text-xs text-base-content/60">
													<span class="badge badge-ghost badge-sm">{row.addon}</span>
													{#if row.year}
														<span>{row.year}</span>
													{/if}
												</div>
											</div>
										</td>
										<td>
											<button
												type="button"
												class="font-mono text-xs text-base-content/70 hover:text-base-content"
												title={`${row.firkinId} (click to copy)`}
												onclick={() => copyHash(row.firkinId)}
											>
												{truncateCid(row.firkinId)}
											</button>
										</td>
										<td class="text-right font-mono text-sm">{row.count}</td>
										<td>
											{#if row.reviews && row.reviews.length > 0}
												<div class="flex flex-wrap items-center gap-1">
													{#each row.reviews as review (review.label)}
														<span
															class="badge gap-1 badge-outline font-mono badge-sm"
															title={review.voteCount !== undefined
																? `${review.label}: ${formatScore(review.score)} / ${formatScore(review.maxScore)} (${formatVotes(review.voteCount)})`
																: `${review.label}: ${formatScore(review.score)} / ${formatScore(review.maxScore)}`}
														>
															<span class="font-semibold">{review.label}</span>
															<span
																>{formatScore(review.score)} / {formatScore(review.maxScore)}</span
															>
														</span>
													{/each}
												</div>
											{:else}
												<span class="text-xs text-base-content/40">—</span>
											{/if}
										</td>
										<td class="text-right">
											<button
												type="button"
												class="btn gap-1 btn-xs btn-primary"
												onclick={() => bookmark(row)}
												disabled={bookmarkingId !== null || !row.upstreamId}
												title="Persist this recommendation as a firkin and pull its own recommendations"
											>
												<svg
													xmlns="http://www.w3.org/2000/svg"
													viewBox="0 0 24 24"
													fill="currentColor"
													class="h-3.5 w-3.5 shrink-0"
													aria-hidden="true"
												>
													<path d="M6 3h12a1 1 0 0 1 1 1v17l-7-4-7 4V4a1 1 0 0 1 1-1z" />
												</svg>
												<span>
													{bookmarkingId === row.firkinId ? 'Bookmarking…' : 'Bookmark'}
												</span>
											</button>
										</td>
									</tr>
								{/each}
							{/if}
						</tbody>
					</table>
				</div>
			</div>
		</section>
	{/if}
</div>
