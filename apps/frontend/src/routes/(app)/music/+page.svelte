<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { fetchJson } from 'ui-lib/transport/fetch-helpers';
	import { releaseGroupsToDisplay, artistsToDisplay } from 'addons/musicbrainz/transform';
	import type { MusicBrainzReleaseGroup, MusicBrainzArtist } from 'addons/musicbrainz/types';
	import { catalogItemToCardData } from 'ui-lib/adapters/classes/catalog-card.adapter';
	import CatalogCard from 'ui-lib/components/catalog/CatalogCard.svelte';
	import type { CatalogItem } from 'ui-lib/types/catalog.type';
	import { favoritesService } from 'ui-lib/services/favorites.service';
	import { pinsService } from 'ui-lib/services/pins.service';

	const favs = favoritesService.state;
	const pins = pinsService.state;

	let albums = $state<CatalogItem[]>([]);
	let artists = $state<CatalogItem[]>([]);
	let albumsLoading = $state(false);
	let artistsLoading = $state(false);

	function toAlbumItems(data: MusicBrainzReleaseGroup[]): CatalogItem[] {
		return releaseGroupsToDisplay(data).slice(0, 6).map((a) => ({
			id: a.id,
			kind: 'album' as const,
			title: a.title,
			sortTitle: a.title.toLowerCase(),
			year: a.firstReleaseYear || null,
			overview: null,
			posterUrl: a.coverArtUrl,
			backdropUrl: null,
			voteAverage: null,
			voteCount: null,
			parentId: null,
			position: null,
			source: 'musicbrainz' as const,
			sourceId: a.id,
			createdAt: '',
			updatedAt: '',
			metadata: {
				musicbrainzId: a.id,
				primaryType: a.primaryType,
				secondaryTypes: a.secondaryTypes,
				artistCredits: a.artistCredits,
				firstReleaseYear: a.firstReleaseYear,
				coverArtUrl: a.coverArtUrl,
				releases: []
			}
		}));
	}

	function toArtistItems(data: MusicBrainzArtist[]): CatalogItem[] {
		return artistsToDisplay(data).slice(0, 6).map((a) => ({
			id: a.id,
			kind: 'artist' as const,
			title: a.name,
			sortTitle: a.sortName.toLowerCase(),
			year: a.beginYear || null,
			overview: null,
			posterUrl: a.imageUrl,
			backdropUrl: null,
			voteAverage: null,
			voteCount: null,
			parentId: null,
			position: null,
			source: 'musicbrainz' as const,
			sourceId: a.id,
			createdAt: '',
			updatedAt: '',
			metadata: {
				musicbrainzId: a.id,
				sortName: a.sortName,
				type: a.type,
				country: a.country,
				disambiguation: a.disambiguation,
				beginYear: a.beginYear,
				endYear: a.endYear,
				ended: a.ended,
				tags: a.tags,
				imageUrl: a.imageUrl
			}
		}));
	}

	function cardWithOverlays(item: CatalogItem) {
		const base = catalogItemToCardData(item);
		const service = item.kind === 'album' ? 'musicbrainz-album' : 'musicbrainz-artist';
		return {
			...base,
			favorited: $favs.items.some((f) => f.service === service && f.serviceId === item.sourceId),
			pinned: $pins.items.some((p) => p.service === service && p.serviceId === item.sourceId)
		};
	}

	onMount(async () => {
		albumsLoading = true;
		artistsLoading = true;
		const [albumData, artistData] = await Promise.all([
			fetchJson<{ 'release-groups': MusicBrainzReleaseGroup[] }>('/api/musicbrainz/popular?genre=rock'),
			fetchJson<{ artists: MusicBrainzArtist[] }>('/api/musicbrainz/popular-artists?genre=rock')
		]);
		albums = toAlbumItems(albumData?.['release-groups'] ?? []);
		albumsLoading = false;
		artists = toArtistItems(artistData?.artists ?? []);
		artistsLoading = false;
	});
</script>

<div class="flex min-w-0 flex-1 flex-col overflow-y-auto">
	<div class="flex items-center justify-between gap-4 border-b border-base-300 px-4 py-3">
		<h1 class="text-lg font-bold">Music</h1>
	</div>
	<div class="p-4">
		<section class="mb-8">
			<div class="mb-3 flex items-center justify-between">
				<h2 class="text-lg font-semibold">Albums</h2>
				<a href="{base}/music/album" class="btn btn-ghost btn-sm">View all</a>
			</div>
			{#if albumsLoading}
				<div class="flex items-center justify-center py-12">
					<span class="loading loading-spinner loading-md"></span>
				</div>
			{:else if albums.length === 0}
				<p class="py-8 text-center text-sm opacity-50">No albums available</p>
			{:else}
				<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6">
					{#each albums as album (album.id)}
						<CatalogCard
							card={cardWithOverlays(album)}
							onclick={() => goto(`${base}/music/album/${album.sourceId}`)}
						/>
					{/each}
				</div>
			{/if}
		</section>

		<section>
			<div class="mb-3 flex items-center justify-between">
				<h2 class="text-lg font-semibold">Artists</h2>
				<a href="{base}/music/artist" class="btn btn-ghost btn-sm">View all</a>
			</div>
			{#if artistsLoading}
				<div class="flex items-center justify-center py-12">
					<span class="loading loading-spinner loading-md"></span>
				</div>
			{:else if artists.length === 0}
				<p class="py-8 text-center text-sm opacity-50">No artists available</p>
			{:else}
				<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6">
					{#each artists as artist (artist.id)}
						<CatalogCard
							card={cardWithOverlays(artist)}
							onclick={() => goto(`${base}/music/artist/${artist.sourceId}`)}
						/>
					{/each}
				</div>
			{/if}
		</section>
	</div>
</div>
