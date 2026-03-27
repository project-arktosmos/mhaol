<script lang="ts">
  import { onMount, getContext } from "svelte";
  import { goto } from "$app/navigation";
  import { base } from "$app/paths";
  import { fetchJson } from "ui-lib/transport/fetch-helpers";
  import {
    MEDIA_BAR_KEY,
    type MediaBarContext,
  } from "ui-lib/types/media-bar.type";

  const mediaBar = getContext<MediaBarContext>(MEDIA_BAR_KEY);
  mediaBar.configure({ title: "Music" });
  import {
    releaseGroupsToDisplay,
    artistsToDisplay,
  } from "addons/musicbrainz/transform";
  import type {
    MusicBrainzReleaseGroup,
    MusicBrainzArtist,
  } from "addons/musicbrainz/types";
  import { catalogItemToCardData } from "ui-lib/adapters/classes/catalog-card.adapter";
  import CatalogCard from "ui-lib/components/catalog/CatalogCard.svelte";
  import type { CatalogItem } from "ui-lib/types/catalog.type";
  import { favoritesService } from "ui-lib/services/favorites.service";
  import { pinsService } from "ui-lib/services/pins.service";
  import { albumStrategy } from "ui-lib/services/catalog-strategies/album.strategy";
  import { artistStrategy } from "ui-lib/services/catalog-strategies/artist.strategy";
  import Portal from "ui-lib/components/core/Portal.svelte";
  import Modal from "ui-lib/components/core/Modal.svelte";
  import MusicRecommendationsModalContent from "ui-lib/components/recommendations/MusicRecommendationsModalContent.svelte";

  const favs = favoritesService.state;
  const pins = pinsService.state;

  let albums = $state<CatalogItem[]>([]);
  let artists = $state<CatalogItem[]>([]);
  let albumsLoading = $state(false);
  let artistsLoading = $state(false);

  let pinnedItems = $state<CatalogItem[]>([]);
  let favoriteItems = $state<CatalogItem[]>([]);
  let pinnedLoading = $state(false);
  let favoritesLoading = $state(false);

  let recsModalOpen = $state(false);

  let pinnedAlbumIds = $derived(
    $pins.items
      .filter((p) => p.service === "musicbrainz-album")
      .map((p) => p.serviceId),
  );
  let pinnedArtistIds = $derived(
    $pins.items
      .filter((p) => p.service === "musicbrainz-artist")
      .map((p) => p.serviceId),
  );
  let favAlbumIds = $derived(
    $favs.items
      .filter((f) => f.service === "musicbrainz-album")
      .map((f) => f.serviceId),
  );
  let favArtistIds = $derived(
    $favs.items
      .filter((f) => f.service === "musicbrainz-artist")
      .map((f) => f.serviceId),
  );

  $effect(() => {
    const albumIds = pinnedAlbumIds;
    const artistIds = pinnedArtistIds;
    if (albumIds.length === 0 && artistIds.length === 0) {
      pinnedItems = [];
      return;
    }
    pinnedLoading = true;
    Promise.all([
      albumIds.length > 0 && albumStrategy.resolveByIds
        ? albumStrategy.resolveByIds(albumIds)
        : Promise.resolve([]),
      artistIds.length > 0 && artistStrategy.resolveByIds
        ? artistStrategy.resolveByIds(artistIds)
        : Promise.resolve([]),
    ])
      .then(([a, b]) => {
        pinnedItems = [...a, ...b];
        pinnedLoading = false;
      })
      .catch(() => {
        pinnedLoading = false;
      });
  });

  $effect(() => {
    const albumIds = favAlbumIds;
    const artistIds = favArtistIds;
    if (albumIds.length === 0 && artistIds.length === 0) {
      favoriteItems = [];
      return;
    }
    favoritesLoading = true;
    Promise.all([
      albumIds.length > 0 && albumStrategy.resolveByIds
        ? albumStrategy.resolveByIds(albumIds)
        : Promise.resolve([]),
      artistIds.length > 0 && artistStrategy.resolveByIds
        ? artistStrategy.resolveByIds(artistIds)
        : Promise.resolve([]),
    ])
      .then(([a, b]) => {
        favoriteItems = [...a, ...b];
        favoritesLoading = false;
      })
      .catch(() => {
        favoritesLoading = false;
      });
  });

  function toAlbumItems(data: MusicBrainzReleaseGroup[]): CatalogItem[] {
    return releaseGroupsToDisplay(data)
      .slice(0, 6)
      .map((a) => ({
        id: a.id,
        kind: "album" as const,
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
        source: "musicbrainz" as const,
        sourceId: a.id,
        createdAt: "",
        updatedAt: "",
        metadata: {
          musicbrainzId: a.id,
          primaryType: a.primaryType,
          secondaryTypes: a.secondaryTypes,
          artistCredits: a.artistCredits,
          firstReleaseYear: a.firstReleaseYear,
          coverArtUrl: a.coverArtUrl,
          releases: [],
        },
      }));
  }

  function toArtistItems(data: MusicBrainzArtist[]): CatalogItem[] {
    return artistsToDisplay(data)
      .slice(0, 6)
      .map((a) => ({
        id: a.id,
        kind: "artist" as const,
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
        source: "musicbrainz" as const,
        sourceId: a.id,
        createdAt: "",
        updatedAt: "",
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
          imageUrl: a.imageUrl,
        },
      }));
  }

  function cardWithOverlays(item: CatalogItem) {
    const base = catalogItemToCardData(item);
    const service =
      item.kind === "album" ? "musicbrainz-album" : "musicbrainz-artist";
    return {
      ...base,
      favorited: $favs.items.some(
        (f) => f.service === service && f.serviceId === item.sourceId,
      ),
      pinned: $pins.items.some(
        (p) => p.service === service && p.serviceId === item.sourceId,
      ),
    };
  }

  onMount(async () => {
    albumsLoading = true;
    artistsLoading = true;
    const [albumData, artistData] = await Promise.all([
      fetchJson<{ "release-groups": MusicBrainzReleaseGroup[] }>(
        "/api/musicbrainz/popular?genre=rock",
      ),
      fetchJson<{ artists: MusicBrainzArtist[] }>(
        "/api/musicbrainz/popular-artists?genre=rock",
      ),
    ]);
    albums = toAlbumItems(albumData?.["release-groups"] ?? []);
    albumsLoading = false;
    artists = toArtistItems(artistData?.artists ?? []);
    artistsLoading = false;
  });
</script>

<Portal target={mediaBar.controlsTarget}>
  <button class="btn btn-ghost btn-sm" onclick={() => (recsModalOpen = true)}
    >Recs</button
  >
</Portal>

<div class="p-4">
  {#if pinnedLoading || pinnedItems.length > 0}
    <section class="mb-8">
      <h2 class="mb-3 text-lg font-semibold">Pinned</h2>
      {#if pinnedLoading}
        <div class="flex justify-center py-6">
          <span class="loading loading-sm loading-spinner"></span>
        </div>
      {:else}
        <div
          class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6"
        >
          {#each pinnedItems as item (item.id)}
            <CatalogCard
              card={cardWithOverlays(item)}
              onclick={() =>
                goto(
                  `${base}/media/music/${item.kind === "album" ? "album" : "artist"}/${item.sourceId}`,
                )}
            />
          {/each}
        </div>
      {/if}
    </section>
  {/if}

  {#if favoritesLoading || favoriteItems.length > 0}
    <section class="mb-8">
      <h2 class="mb-3 text-lg font-semibold">Favorites</h2>
      {#if favoritesLoading}
        <div class="flex justify-center py-6">
          <span class="loading loading-sm loading-spinner"></span>
        </div>
      {:else}
        <div
          class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6"
        >
          {#each favoriteItems as item (item.id)}
            <CatalogCard
              card={cardWithOverlays(item)}
              onclick={() =>
                goto(
                  `${base}/media/music/${item.kind === "album" ? "album" : "artist"}/${item.sourceId}`,
                )}
            />
          {/each}
        </div>
      {/if}
    </section>
  {/if}

  <section class="mb-8">
    <div class="mb-3 flex items-center justify-between">
      <h2 class="text-lg font-semibold">Albums</h2>
      <a href="{base}/media/music/album" class="btn btn-ghost btn-sm"
        >View all</a
      >
    </div>
    {#if albumsLoading}
      <div class="flex items-center justify-center py-12">
        <span class="loading loading-spinner loading-md"></span>
      </div>
    {:else if albums.length === 0}
      <p class="py-8 text-center text-sm opacity-50">No albums available</p>
    {:else}
      <div
        class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6"
      >
        {#each albums as album (album.id)}
          <CatalogCard
            card={cardWithOverlays(album)}
            onclick={() => goto(`${base}/media/music/album/${album.sourceId}`)}
          />
        {/each}
      </div>
    {/if}
  </section>

  <section>
    <div class="mb-3 flex items-center justify-between">
      <h2 class="text-lg font-semibold">Artists</h2>
      <a href="{base}/media/music/artist" class="btn btn-ghost btn-sm"
        >View all</a
      >
    </div>
    {#if artistsLoading}
      <div class="flex items-center justify-center py-12">
        <span class="loading loading-spinner loading-md"></span>
      </div>
    {:else if artists.length === 0}
      <p class="py-8 text-center text-sm opacity-50">No artists available</p>
    {:else}
      <div
        class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6"
      >
        {#each artists as artist (artist.id)}
          <CatalogCard
            card={cardWithOverlays(artist)}
            onclick={() =>
              goto(`${base}/media/music/artist/${artist.sourceId}`)}
          />
        {/each}
      </div>
    {/if}
  </section>
</div>

<Modal
  open={recsModalOpen}
  maxWidth="max-w-[90vw]"
  onclose={() => (recsModalOpen = false)}
>
  {#if recsModalOpen}
    <div class="p-4">
      <MusicRecommendationsModalContent
        {pinnedArtistIds}
        favoritedArtistIds={favArtistIds}
      />
    </div>
  {/if}
</Modal>
