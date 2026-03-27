<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { base } from "$app/paths";
  import classNames from "classnames";
  import CatalogBrowsePage from "ui-lib/components/catalog/CatalogBrowsePage.svelte";
  import { catalogService } from "ui-lib/services/catalog.service";
  import { gameStrategy } from "ui-lib/services/catalog-strategies/game.strategy";
  import type { CatalogItem } from "ui-lib/types/catalog.type";
  import { isGame } from "ui-lib/types/catalog.type";
  import {
    RA_CONSOLES,
    CONSOLE_WASM_STATUS,
  } from "addons/retroachievements/types";
  import { CONSOLE_IMAGES } from "assets/game-consoles";
  import { favoritesService } from "ui-lib/services/favorites.service";
  import { pinsService } from "ui-lib/services/pins.service";
  import { getContext } from "svelte";
  import {
    MEDIA_BAR_KEY,
    type MediaBarContext,
  } from "ui-lib/types/media-bar.type";
  import Portal from "ui-lib/components/core/Portal.svelte";
  import Modal from "ui-lib/components/core/Modal.svelte";
  import GameRecommendationsModalContent from "ui-lib/components/recommendations/GameRecommendationsModalContent.svelte";

  const mediaBar = getContext<MediaBarContext>(MEDIA_BAR_KEY);

  const browseState = catalogService.state;
  const favs = favoritesService.state;
  const pins = pinsService.state;

  let recsModalOpen = $state(false);
  let selectedConsoleId = $state(5);

  let pinnedGameIds = $derived(
    $pins.items
      .filter((p) => p.service === "retroachievements")
      .map((p) => Number(p.serviceId)),
  );
  let favoritedGameIds = $derived(
    $favs.items
      .filter((f) => f.service === "retroachievements")
      .map((f) => Number(f.serviceId)),
  );

  onMount(() => {
    catalogService.registerStrategy(gameStrategy);
    catalogService.activate("game");
    catalogService.setFilter("console", String(selectedConsoleId));
  });

  function handleConsoleChange(consoleId: number) {
    selectedConsoleId = consoleId;
    catalogService.setFilter("console", String(consoleId));
  }

  function handleSelectItem(item: CatalogItem) {
    if (isGame(item)) {
      goto(`${base}/media/videogames/${item.sourceId}`);
    }
  }

  function cardOverlays(item: CatalogItem) {
    return {
      favorited: $favs.items.some(
        (f) =>
          f.service === "retroachievements" && f.serviceId === item.sourceId,
      ),
      pinned: $pins.items.some(
        (p) =>
          p.service === "retroachievements" && p.serviceId === item.sourceId,
      ),
    };
  }
</script>

<Portal target={mediaBar.controlsTarget}>
  <button class="btn btn-ghost btn-sm" onclick={() => (recsModalOpen = true)}
    >Recs</button
  >
</Portal>

<CatalogBrowsePage
  browseState={$browseState}
  title="Videogames"
  strategy={gameStrategy}
  {cardOverlays}
  onsearch={(q) => catalogService.search(q)}
  ontabchange={(tab) => catalogService.loadTab(tab)}
  onpagechange={(p) => catalogService.loadPage(p)}
  onselectitem={handleSelectItem}
>
  {#snippet filterBar()}
    <div
      class="grid grid-cols-3 gap-3 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-9"
    >
      {#each RA_CONSOLES as console}
        <button
          class={classNames(
            "relative flex flex-col items-center gap-1.5 rounded-lg p-2 transition-colors",
            {
              "bg-primary/15 ring-2 ring-primary":
                selectedConsoleId === console.id,
              "hover:bg-base-200": selectedConsoleId !== console.id,
            },
          )}
          onclick={() => handleConsoleChange(console.id)}
        >
          <span
            class={classNames("absolute right-1 top-1 h-2 w-2 rounded-full", {
              "bg-success": CONSOLE_WASM_STATUS[console.id] === "yes",
              "bg-warning": CONSOLE_WASM_STATUS[console.id] === "experimental",
              "bg-error": CONSOLE_WASM_STATUS[console.id] === "no",
            })}
            title={CONSOLE_WASM_STATUS[console.id] === "yes"
              ? "WASM emulator available"
              : CONSOLE_WASM_STATUS[console.id] === "experimental"
                ? "WASM emulator (experimental)"
                : "No WASM emulator"}
          ></span>
          {#if CONSOLE_IMAGES[console.id]}
            <img
              src={CONSOLE_IMAGES[console.id]}
              alt={console.name}
              class="h-10 w-10 object-contain"
            />
          {:else}
            <div
              class="flex h-10 w-10 items-center justify-center rounded bg-base-300 text-xs text-base-content/50"
            >
              ?
            </div>
          {/if}
          <span class="text-center text-xs font-medium leading-tight"
            >{console.name}</span
          >
        </button>
      {/each}
    </div>
  {/snippet}
</CatalogBrowsePage>

<Modal
  open={recsModalOpen}
  maxWidth="max-w-[90vw]"
  onclose={() => (recsModalOpen = false)}
>
  {#if recsModalOpen}
    <div class="p-4">
      <GameRecommendationsModalContent {pinnedGameIds} {favoritedGameIds} />
    </div>
  {/if}
</Modal>
