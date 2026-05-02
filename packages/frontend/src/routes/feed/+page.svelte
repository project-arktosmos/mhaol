<script lang="ts">
	import { base } from '$app/paths';
	import { Icon } from 'cloud-ui';
	import FirkinCard from '$components/firkins/FirkinCard.svelte';
	import {
		listRecommendations,
		recordRecommendationAction,
		type Recommendation,
		type RecommendationAction
	} from '$lib/recommendations.service';
	import {
		firkinsService,
		type Artist,
		type FirkinAddon,
		type FileEntry,
		type Trailer
	} from '$lib/firkins.service';
	import type { CloudFirkin } from '$types/firkin.type';
	import { userIdentityService } from '$lib/user-identity.service';

	const userIdentityState = userIdentityService.state;

	let queue = $state<Recommendation[]>([]);
	let loading = $state(false);
	let actioning = $state<RecommendationAction | null>(null);
	let error = $state<string | null>(null);
	let lastLoadedAddress: string | null = null;

	let current = $derived<Recommendation | null>(queue[0] ?? null);
	let cardFirkin = $derived<CloudFirkin | null>(current ? toCardFirkin(current) : null);
	let upcoming = $derived<Recommendation[]>(queue.slice(1, 21));

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
		} catch (err) {
			error = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			loading = false;
		}
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
	): Promise<{ artists: Artist[]; trailers: Trailer[] }> {
		try {
			const res = await fetch(
				`${base}/api/catalog/${encodeURIComponent(addon)}/${encodeURIComponent(upstreamId)}/metadata`,
				{ cache: 'no-store' }
			);
			if (!res.ok) return { artists: [], trailers: [] };
			const body = (await res.json()) as { artists?: Artist[]; trailers?: Trailer[] };
			return {
				artists: Array.isArray(body.artists) ? body.artists : [],
				trailers: Array.isArray(body.trailers) ? body.trailers : []
			};
		} catch {
			return { artists: [], trailers: [] };
		}
	}

	async function act(action: RecommendationAction) {
		const row = current;
		const address = $userIdentityState.identity?.address;
		if (!row || !address || actioning) return;
		actioning = action;
		error = null;
		try {
			if (action === 'bookmark') {
				if (!row.upstreamId) throw new Error('recommendation is missing its upstream id');
				const { artists, trailers } = await fetchUpstreamMetadata(row.addon, row.upstreamId);
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
					trailers
				});
			}
			await recordRecommendationAction({
				address,
				firkinId: row.firkinId,
				action
			});
			queue = queue.slice(1);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			actioning = null;
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
			its rating. Like or discard to advance; bookmark to mint a real firkin.
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
						Suggested {current?.count}× · {queue.length} item{queue.length === 1 ? '' : 's'} left
					</div>

					<div class="grid grid-cols-3 gap-2">
						<button
							type="button"
							class="btn gap-2 btn-outline btn-error"
							onclick={() => act('discard')}
							disabled={actioning !== null}
							title="Never show this recommendation again"
						>
							<Icon name="delapouite/trash-can" size={18} />
							<span>{actioning === 'discard' ? 'Discarding…' : 'Discard'}</span>
						</button>
						<button
							type="button"
							class="btn gap-2 btn-outline btn-secondary"
							onclick={() => act('like')}
							disabled={actioning !== null}
							title="Record a positive signal and move on"
						>
							<Icon name="zeromancer/heart-plus" size={18} />
							<span>{actioning === 'like' ? 'Liking…' : 'Like'}</span>
						</button>
						<button
							type="button"
							class="btn gap-2 btn-primary"
							onclick={() => act('bookmark')}
							disabled={actioning !== null || !current?.upstreamId}
							title="Persist this recommendation as a firkin"
						>
							<Icon name="lorc/bookmark" size={18} />
							<span>{actioning === 'bookmark' ? 'Bookmarking…' : 'Bookmark'}</span>
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
													{idx + 2}
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
