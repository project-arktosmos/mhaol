<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { apiUrl } from "ui-lib/lib/api-base";
  import { tmdbBrowseService } from "ui-lib/services/tmdb-browse.service";
  import { smartPairService } from "ui-lib/services/smart-pair.service";
  import type { DisplayTMDBTvShow } from "addons/tmdb/types";
  import type {
    MediaCategory,
    MediaLinkSource,
  } from "ui-lib/types/media-card.type";
  import type { MediaList } from "ui-lib/types/media-list.type";
  import SearchTab from "ui-lib/components/tmdb-browse/SearchTab.svelte";
  import PopularTab from "ui-lib/components/tmdb-browse/PopularTab.svelte";
  import TmdbBrowseGrid from "ui-lib/components/tmdb-browse/TmdbBrowseGrid.svelte";
  import TmdbPagination from "ui-lib/components/tmdb-browse/TmdbPagination.svelte";
  import BrowseViewToggle from "ui-lib/components/browse/BrowseViewToggle.svelte";
  import TvShowMatchModal from "ui-lib/components/libraries/TvShowMatchModal.svelte";
  import classNames from "classnames";

  interface Props {
    data: {
      mediaTypes: Array<{ id: string; label: string }>;
      categories: MediaCategory[];
      linkSources: MediaLinkSource[];
      lists: MediaList[];
      libraries: Record<string, { name: string; type: string }>;
      error?: string;
    };
  }

  let { data }: Props = $props();

  const browseState = tmdbBrowseService.state;

  let pinnedTvShows = $state<DisplayTMDBTvShow[]>([]);
  let matchModalList: MediaList | null = $state(null);

  // Navigate to TMDB detail page (for browse results and pinned)
  function handleSelectTvShow(tvShow: DisplayTMDBTvShow) {
    goto(`/tv/${tvShow.id}`);
  }

  // Handle library show click: navigate if TMDB-linked, otherwise open match modal
  function handleSelectLibraryShow(tvShow: DisplayTMDBTvShow) {
    const list = listByDisplayId.get(tvShow.id);
    if (!list) return;

    const tmdbLink = list.links?.tmdb;
    if (tmdbLink) {
      goto(`/tv/${tmdbLink.serviceId}`);
    } else {
      matchModalList = list;
    }
  }

  async function handleMatch(tmdbId: number) {
    if (!matchModalList) return;
    const listId = matchModalList.id;

    const res = await fetch(apiUrl(`/api/media-lists/${listId}/tmdb`), {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ tmdbId }),
    });

    if (res.ok) {
      matchModalList = null;
      goto(`/tv/${tmdbId}`);
    }
  }

  // Filter to show-level TV lists (no parent)
  let tvShowLists = $derived(
    (data.lists ?? []).filter(
      (list) => list.parentListId === null && list.libraryType === "tv",
    ),
  );

  // Assign unique numeric IDs to each list (DisplayTMDBTvShow requires numeric id)
  // Use negative IDs to avoid collisions with real TMDB IDs
  let listIdMap = $derived(
    new Map(tvShowLists.map((list, i) => [list.id, -(i + 1)])),
  );

  // Map display numeric ID back to MediaList
  let listByDisplayId = $derived(
    new Map(tvShowLists.map((list) => [listIdMap.get(list.id)!, list])),
  );

  // Count child lists (seasons) per show
  let seasonCountByShowId = $derived.by(() => {
    const counts = new Map<string, number>();
    for (const list of data.lists ?? []) {
      if (list.parentListId && list.libraryType === "tv") {
        counts.set(
          list.parentListId,
          (counts.get(list.parentListId) ?? 0) + 1,
        );
      }
    }
    return counts;
  });

  // Count total episodes across all child lists per show
  let episodeCountByShowId = $derived.by(() => {
    const counts = new Map<string, number>();
    for (const list of data.lists ?? []) {
      if (list.parentListId && list.libraryType === "tv") {
        counts.set(
          list.parentListId,
          (counts.get(list.parentListId) ?? 0) + list.itemCount,
        );
      }
    }
    return counts;
  });

  function listToDisplayTvShow(list: MediaList): DisplayTMDBTvShow {
    const seasonCount = seasonCountByShowId.get(list.id) ?? null;
    const episodeCount =
      episodeCountByShowId.get(list.id) ?? list.itemCount ?? null;
    return {
      id: listIdMap.get(list.id)!,
      name: list.title,
      originalName: list.title,
      firstAirYear: "",
      lastAirYear: null,
      overview: "",
      posterUrl: null,
      backdropUrl: null,
      voteAverage: 0,
      voteCount: 0,
      genres: [],
      numberOfSeasons: seasonCount,
      numberOfEpisodes: episodeCount,
    };
  }

  // Group show lists by library for per-library grids
  let libraryGroups = $derived.by(() => {
    const grouped = new Map<string, MediaList[]>();
    for (const list of tvShowLists) {
      const existing = grouped.get(list.libraryId);
      if (existing) {
        existing.push(list);
      } else {
        grouped.set(list.libraryId, [list]);
      }
    }
    return Array.from(grouped.entries())
      .map(([libraryId, lists]) => ({
        libraryId,
        name: data.libraries[libraryId]?.name ?? libraryId,
        tvShows: lists.map(listToDisplayTvShow),
      }))
      .filter((g) => g.tvShows.length > 0);
  });

  onMount(() => {
    tmdbBrowseService.loadPopularTv();
    tmdbBrowseService.loadGenres();
    tmdbBrowseService.loadDiscoverTv();
    smartPairService.loadPinned().then((pinned) => {
      pinnedTvShows = pinned.tv;
    });
  });
</script>

<div class="relative min-w-0 flex-1 overflow-y-auto p-4">
  <div class="absolute right-3 top-3 z-10">
    <BrowseViewToggle />
  </div>
  <div class="container mx-auto">
    {#if pinnedTvShows.length > 0}
      <section class="mb-8">
        <h2 class="mb-3 text-lg font-semibold">Pinned</h2>
        <TmdbBrowseGrid
          tvShows={pinnedTvShows}
          onselectTvShow={handleSelectTvShow}
        />
      </section>
    {/if}

    {#each libraryGroups as group (group.libraryId)}
      <section class="mb-8">
        <h2 class="mb-3 text-lg font-semibold">{group.name}</h2>
        <TmdbBrowseGrid
          tvShows={group.tvShows}
          onselectTvShow={handleSelectLibraryShow}
        />
      </section>
    {/each}

    <section class="mb-8">
      <h2 class="mb-3 text-lg font-semibold">Search TV Shows</h2>
      <SearchTab
        movies={[]}
        tvShows={$browseState.searchTv}
        moviesPage={1}
        tvPage={$browseState.searchTvPage}
        moviesTotalPages={1}
        tvTotalPages={$browseState.searchTvTotalPages}
        query={$browseState.searchQuery}
        loadingTv={$browseState.loading["searchTv"] ?? false}
        error={$browseState.error}
        mediaType="tv"
        selectedTvShowId={null}
        onselectTvShow={handleSelectTvShow}
        onsearchMovies={() => {}}
        onsearchTv={(q, p) => tmdbBrowseService.searchTv(q, p)}
      />
    </section>

    <section class="mb-8">
      <h2 class="mb-3 text-lg font-semibold">Popular TV Shows</h2>
      <PopularTab
        movies={[]}
        tvShows={$browseState.popularTv}
        moviesPage={1}
        tvPage={$browseState.popularTvPage}
        moviesTotalPages={1}
        tvTotalPages={$browseState.popularTvTotalPages}
        loadingTv={$browseState.loading["popularTv"] ?? false}
        error={$browseState.error}
        mediaType="tv"
        selectedTvShowId={null}
        onselectTvShow={handleSelectTvShow}
        onloadMovies={() => {}}
        onloadTv={(p) => tmdbBrowseService.loadPopularTv(p)}
      />
    </section>

    <section class="mb-8">
      <h2 class="mb-3 text-lg font-semibold">Discover TV Shows</h2>
      {#if $browseState.tvGenres.length > 0}
        <div
          class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
        >
          {#each $browseState.tvGenres as genre (genre.id)}
            <button
              class={classNames("btn btn-sm h-auto min-h-12 flex-col py-2", {
                "btn-primary": $browseState.selectedGenreId === genre.id,
                "btn-ghost bg-base-200":
                  $browseState.selectedGenreId !== genre.id,
              })}
              onclick={() => {
                const genreId =
                  $browseState.selectedGenreId === genre.id ? null : genre.id;
                tmdbBrowseService.loadDiscoverTv(1, genreId);
              }}
            >
              {genre.name}
            </button>
          {/each}
        </div>
      {/if}
      {#if $browseState.loading["discoverTv"]}
        <div class="flex justify-center p-8">
          <span class="loading loading-lg loading-spinner"></span>
        </div>
      {:else if $browseState.discoverTv.length > 0}
        <div class="mt-4">
          <TmdbBrowseGrid
            tvShows={$browseState.discoverTv}
            selectedTvShowId={null}
            onselectTvShow={handleSelectTvShow}
          />
          <TmdbPagination
            page={$browseState.discoverTvPage}
            totalPages={$browseState.discoverTvTotalPages}
            loading={$browseState.loading["discoverTv"] ?? false}
            onpage={(p) =>
              tmdbBrowseService.loadDiscoverTv(p, $browseState.selectedGenreId)}
          />
        </div>
      {/if}
    </section>
  </div>
</div>

{#if matchModalList}
  <TvShowMatchModal
    showName={matchModalList.title}
    onmatch={handleMatch}
    onclose={() => (matchModalList = null)}
  />
{/if}
