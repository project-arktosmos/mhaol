<script lang="ts">
	import { base } from '$app/paths';
	import { Icon } from 'cloud-ui';
	import FirkinCard from '$components/firkins/FirkinCard.svelte';
	import {
		listRecommendations,
		recordRecommendationAction,
		setRecommendationRating,
		type Recommendation
	} from '$lib/recommendations.service';
	import {
		firkinsService,
		type Artist,
		type FirkinAddon,
		type FileEntry,
		type Review,
		type Trailer
	} from '$lib/firkins.service';
	import type { CloudFirkin } from '$types/firkin.type';
	import { userIdentityService } from '$lib/user-identity.service';

	const userIdentityState = userIdentityService.state;

	let queue = $state<Recommendation[]>([]);
	let cursor = $state(0);
	let loading = $state(false);
	let discarding = $state(false);
	let bookmarking = $state(false);
	let rating = $state(false);
	let error = $state<string | null>(null);
	let lastLoadedAddress: string | null = null;

	let busy = $derived(discarding || bookmarking || rating);
	let current = $derived<Recommendation | null>(queue[cursor] ?? null);
	let cardFirkin = $derived<CloudFirkin | null>(current ? toCardFirkin(current) : null);
	let upcoming = $derived<Recommendation[]>(queue.slice(cursor + 1, cursor + 21));
	let activeStars = $derived(current?.userRating ? Math.round(current.userRating / 20) : 0);
	let canPrev = $derived(cursor > 0);
	let canNext = $derived(cursor + 1 < queue.length);

	function ratingLabel(row: Recommendation): string {
		const reviews = row.reviews ?? [];
		if (reviews.length === 0) return '—';
		const total = reviews.reduce(
			(sum, r) => (r.maxScore > 0 ? sum + r.score / r.maxScore : sum),
			0
		);
		const avg = (total / reviews.length) * 10;
		const rounded = Math.round(avg * 10) / 10;
		return `${Number.isInteger(rounded) ? rounded.toFixed(0) : rounded.toFixed(1)} / 10`;
	}

	$effect(() => {
		const address = $userIdentityState.identity?.address;
		if (!address) {
			queue = [];
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
			queue = await listRecommendations(address, { excludeActioned: true });
			cursor = 0;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			loading = false;
		}
	}

	function dropCurrent() {
		queue = queue.filter((_, i) => i !== cursor);
		if (cursor >= queue.length) cursor = Math.max(0, queue.length - 1);
	}

	function go(delta: number) {
		const next = cursor + delta;
		if (next < 0 || next >= queue.length) return;
		cursor = next;
	}

	function toCardFirkin(row: Recommendation): CloudFirkin {
		return {
			id: row.firkinId,
			cid: row.firkinId,
			title: row.title,
			artists: [],
			description: row.description ?? '',
			images: row.posterUrl
				? [
						{
							url: row.posterUrl,
							mimeType: 'image/jpeg',
							fileSize: 0,
							width: 0,
							height: 0
						}
					]
				: [],
			files: [],
			year: row.year,
			addon: row.addon,
			creator: '',
			created_at: row.created_at,
			updated_at: row.updated_at,
			reviews: row.reviews
		};
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

	async function applyRating(score: number) {
		const row = current;
		const address = $userIdentityState.identity?.address;
		if (!row || !address) return;
		const res = await setRecommendationRating({
			address,
			firkinId: row.firkinId,
			rating: score
		});
		queue = queue.map((r, i) => (i === cursor ? { ...r, userRating: res.userRating } : r));
	}

	// Rating intentionally does NOT drop the card — the user may still
	// want to bookmark after rating. The server-side filter takes the item
	// out on the next fetch.
	async function rate(stars: number) {
		if (busy) return;
		const score = Math.max(1, Math.min(5, stars)) * 20;
		rating = true;
		error = null;
		try {
			await applyRating(score);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			rating = false;
		}
	}

	async function discard() {
		if (busy || !current) return;
		discarding = true;
		error = null;
		try {
			await applyRating(0);
			dropCurrent();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			discarding = false;
		}
	}

	async function bookmark() {
		const row = current;
		const address = $userIdentityState.identity?.address;
		if (!row || !address || busy) return;
		if (!row.upstreamId) {
			error = 'recommendation is missing its upstream id';
			return;
		}
		bookmarking = true;
		error = null;
		try {
			const { artists, trailers, reviews } = await fetchUpstreamMetadata(row.addon, row.upstreamId);
			await firkinsService.create({
				title: row.title,
				artists,
				description: row.description ?? '',
				images: row.posterUrl
					? [
							{
								url: row.posterUrl,
								mimeType: 'image/jpeg',
								fileSize: 0,
								width: 0,
								height: 0
							}
						]
					: [],
				files: buildUpstreamSourceFiles(row.addon, row.upstreamId),
				year: row.year,
				addon: row.addon as FirkinAddon,
				trailers,
				reviews: reviews.length > 0 ? reviews : (row.reviews ?? [])
			});
			await recordRecommendationAction({
				address,
				firkinId: row.firkinId,
				action: 'bookmark'
			});
			dropCurrent();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			bookmarking = false;
		}
	}
</script>

<svelte:head>
	<title>Mhaol Cloud — Feed</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<header class="flex flex-col gap-1">
		<h1 class="text-2xl font-semibold">Feed</h1>
		<p class="text-sm text-base-content/60">
			One recommendation at a time, sorted by how often it's been suggested and (as a tiebreaker)
			its rating. Rate with stars; discard to set rating 0 and drop it; bookmark to mint a real
			firkin; previous and next walk the queue without acting on it.
		</p>
	</header>

	{#if !$userIdentityState.identity}
		<div class="alert w-full max-w-md alert-warning">
			<span>Sign in on the Profile page to see your feed.</span>
		</div>
	{:else}
		{#if error}
			<div class="alert w-full max-w-md alert-error"><span>{error}</span></div>
		{/if}

		{#if loading && queue.length === 0}
			<div class="text-base-content/60">Loading…</div>
		{:else if !cardFirkin}
			<div class="alert w-full max-w-md alert-info">
				<span>
					No recommendations to show. Bookmark items from the catalog to start collecting.
				</span>
			</div>
		{:else}
			<div class="flex flex-col items-start gap-6 lg:flex-row">
				<div class="flex w-full max-w-md flex-col gap-3">
					<FirkinCard firkin={cardFirkin} />

					<div class="text-xs text-base-content/60">
						Suggested {current?.count}× · item {cursor + 1} of {queue.length}
					</div>

					<div class="flex items-center justify-between gap-2">
						<div class="flex items-center gap-1" role="radiogroup" aria-label="Your rating">
							{#each [1, 2, 3, 4, 5] as star (star)}
								<button
									type="button"
									class={star <= activeStars
										? 'text-warning'
										: 'text-base-content/30 hover:text-base-content/60'}
									onclick={() => rate(star)}
									disabled={busy}
									role="radio"
									aria-checked={star === activeStars}
									aria-label={`${star} star${star === 1 ? '' : 's'}`}
									title={`Rate ${star * 20} / 100`}
								>
									<Icon name="lorc/flat-star" size={24} />
								</button>
							{/each}
						</div>
						<span class="text-xs text-base-content/60">
							{current?.userRating ? `${current.userRating} / 100` : 'Not rated'}
						</span>
					</div>

					<div class="grid grid-cols-2 gap-2">
						<button
							type="button"
							class="btn gap-2 btn-outline btn-error"
							onclick={discard}
							disabled={busy}
							title="Set rating to 0 and remove from the feed"
						>
							<Icon name="delapouite/trash-can" size={18} />
							<span>{discarding ? 'Discarding…' : 'Discard'}</span>
						</button>
						<button
							type="button"
							class="btn gap-2 btn-primary"
							onclick={bookmark}
							disabled={busy || !current?.upstreamId}
							title="Persist this recommendation as a firkin"
						>
							<Icon name="lorc/bookmark" size={18} />
							<span>{bookmarking ? 'Bookmarking…' : 'Bookmark'}</span>
						</button>
					</div>

					<div class="grid grid-cols-2 gap-2">
						<button
							type="button"
							class="btn gap-2 btn-outline"
							onclick={() => go(-1)}
							disabled={busy || !canPrev}
							title="Go to the previous recommendation"
						>
							<Icon name="delapouite/player-previous" size={18} />
							<span>Previous</span>
						</button>
						<button
							type="button"
							class="btn gap-2 btn-outline"
							onclick={() => go(1)}
							disabled={busy || !canNext}
							title="Skip to the next recommendation"
						>
							<span>Next</span>
							<Icon name="delapouite/player-next" size={18} />
						</button>
					</div>
				</div>

				<section class="card min-w-0 flex-1 border border-base-content/10 bg-base-200">
					<div class="card-body gap-2 p-0">
						<header class="px-4 pt-4">
							<h2 class="text-sm font-semibold tracking-wide text-base-content/70 uppercase">
								Up next
							</h2>
						</header>
						<div class="overflow-x-auto">
							<table class="table table-sm">
								<thead>
									<tr>
										<th class="w-10 text-right">#</th>
										<th class="w-12"></th>
										<th>Title</th>
										<th class="w-16 text-right">Count</th>
										<th class="w-24">Rating</th>
									</tr>
								</thead>
								<tbody>
									{#if upcoming.length === 0}
										<tr>
											<td colspan="5" class="text-center text-xs text-base-content/50">
												Nothing else queued.
											</td>
										</tr>
									{:else}
										{#each upcoming as row, idx (row.firkinId)}
											<tr>
												<td class="text-right font-mono text-xs text-base-content/60">
													{cursor + idx + 2}
												</td>
												<td>
													{#if row.posterUrl}
														<img
															src={row.posterUrl}
															alt={row.title}
															class="h-12 w-8 shrink-0 rounded object-cover"
															loading="lazy"
														/>
													{:else}
														<div
															class="h-12 w-8 shrink-0 rounded bg-base-300"
															aria-hidden="true"
														></div>
													{/if}
												</td>
												<td class="min-w-0">
													<div class="flex flex-col gap-0.5">
														<span class="truncate text-sm font-medium">{row.title}</span>
														<div
															class="flex flex-wrap items-center gap-1 text-xs text-base-content/60"
														>
															<span class="badge badge-ghost badge-xs">{row.addon}</span>
															{#if row.year}
																<span>{row.year}</span>
															{/if}
														</div>
													</div>
												</td>
												<td class="text-right font-mono text-xs">{row.count}</td>
												<td class="font-mono text-xs">{ratingLabel(row)}</td>
											</tr>
										{/each}
									{/if}
								</tbody>
							</table>
						</div>
					</div>
				</section>
			</div>
		{/if}
	{/if}
</div>
