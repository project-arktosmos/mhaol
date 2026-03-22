<script lang="ts">
  import { onMount } from "svelte";
  import { apiUrl } from "frontend/lib/api-base";
  import { tmdbBrowseService } from "frontend/services/tmdb-browse.service";
  import { browseDetailService } from "frontend/services/browse-detail.service";
  import { tvShowDetailsToDisplay } from "addons/tmdb/transform";
  import type {
    DisplayTMDBTvShow,
    DisplayTMDBTvShowDetails,
  } from "addons/tmdb/types";
  import SearchTab from "ui-lib/components/tmdb-browse/SearchTab.svelte";
  import PopularTab from "ui-lib/components/tmdb-browse/PopularTab.svelte";
  import TmdbBrowseGrid from "ui-lib/components/tmdb-browse/TmdbBrowseGrid.svelte";
  import TmdbPagination from "ui-lib/components/tmdb-browse/TmdbPagination.svelte";
  import classNames from "classnames";

  const browseState = tmdbBrowseService.state;

  let selectedBrowseTvShow: DisplayTMDBTvShow | null = $state(null);
  let browseTvShowDetails: DisplayTMDBTvShowDetails | null = $state(null);
  let browseDetailLoading = $state(false);

  async function handleSelectTvShow(tvShow: DisplayTMDBTvShow) {
    if (selectedBrowseTvShow?.id === tvShow.id) {
      selectedBrowseTvShow = null;
      browseTvShowDetails = null;
      browseDetailService.set({
        movie: null,
        tvShow: null,
        movieDetails: null,
        tvShowDetails: null,
        libraryItem: null,
        relatedData: null,
        loading: false,
        fetching: false,
        fetched: false,
        downloadStatus: null,
        fetchSteps: null,
      });
      return;
    }
    selectedBrowseTvShow = tvShow;
    browseTvShowDetails = null;
    fetchTvShowDetails(tvShow.id);
  }

  async function fetchTvShowDetails(tmdbId: number) {
    browseDetailLoading = true;
    try {
      const res = await fetch(apiUrl(`/api/tmdb/tv/${tmdbId}`));
      if (res.ok) {
        const raw = await res.json();
        browseTvShowDetails = tvShowDetailsToDisplay(raw);
      }
    } catch (e) {
      console.error("Failed to load TV show details:", e);
    } finally {
      browseDetailLoading = false;
    }
  }

  $effect(() => {
    browseDetailService.set({
      movie: null,
      tvShow: selectedBrowseTvShow,
      movieDetails: null,
      tvShowDetails: browseTvShowDetails,
      libraryItem: null,
      relatedData: null,
      loading: browseDetailLoading,
      fetching: false,
      fetched: false,
      downloadStatus: null,
      fetchSteps: null,
    });
  });

  $effect(() => {
    browseDetailService.registerCallbacks({
      onclose: () => {
        selectedBrowseTvShow = null;
        browseTvShowDetails = null;
      },
    });
  });

  onMount(() => {
    tmdbBrowseService.loadPopularTv();
    tmdbBrowseService.loadGenres();
    tmdbBrowseService.loadDiscoverTv();
  });
</script>

<div class="min-w-0 flex-1 overflow-y-auto p-4">
  <div class="container mx-auto">
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
        selectedTvShowId={selectedBrowseTvShow?.id ?? null}
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
        selectedTvShowId={selectedBrowseTvShow?.id ?? null}
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
            selectedTvShowId={selectedBrowseTvShow?.id ?? null}
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
