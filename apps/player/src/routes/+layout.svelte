<script lang="ts">
  import "../css/app.css";
  import "ui-lib/services/i18n";
  import "ui-lib/services/clouds.service";
  import classNames from "classnames";
  import Navbar from "ui-lib/components/core/Navbar.svelte";
  import ThemeToggle from "ui-lib/components/core/ThemeToggle.svelte";
  import NodeStatusBadge from "ui-lib/components/core/NodeStatusBadge.svelte";
  import ToastOutlet from "ui-lib/components/core/ToastOutlet.svelte";
  import PlayerVideo from "ui-lib/components/player/PlayerVideo.svelte";
  import { themeService } from "ui-lib/services/theme.service";
  import { connectionConfigService } from "ui-lib/services/connection-config.service";
  import { nodeConnectionService } from "ui-lib/services/node-connection.service";
  import { playerService } from "ui-lib/services/player.service";
  import Modal from "ui-lib/components/core/Modal.svelte";
  import SetupModalContent from "ui-lib/components/setup/SetupModalContent.svelte";
  import { onMount, onDestroy } from "svelte";
  import { invalidateAll } from "$app/navigation";
  import { base } from "$app/paths";
  import { NAV_ITEMS, type NavItem } from "$lib/generated/nav";

  let { children } = $props();
  let setupModalOpen = $state(false);

  const playerState = playerService.state;

  const triggerClass = (item: NavItem) =>
    classNames("btn btn-outline btn-sm", { "btn-disabled": !item.hasOwnPage });

  onMount(() => {
    themeService.initialize("flix");
    playerService.initialize();

    const config = connectionConfigService.get();
    if (!config) return;

    const promise =
      config.transportMode === "ws"
        ? nodeConnectionService.connectWs(config)
        : config.transportMode === "webrtc"
          ? nodeConnectionService.connectWebRtc(config)
          : nodeConnectionService.connectHttp(config);

    promise
      .then(() => invalidateAll())
      .catch(() => {
        // Connection is optional; user can retry from the navbar badge or /clouds.
      });
  });

  onDestroy(() => {
    playerService.destroy();
  });
</script>

<div class="flex h-screen flex-col">
  <Navbar
    brand={{ label: "Mhaol", highlight: "Player" }}
    classes="!bg-base-300"
  >
    {#snippet center()}
      <div class="flex flex-wrap items-center gap-1">
        {#each NAV_ITEMS as item (item.href)}
          {#if item.children.length === 0}
            <a href="{base}{item.href}" class="btn btn-outline btn-sm"
              >{item.label}</a
            >
          {:else}
            <div class="dropdown dropdown-hover dropdown-bottom">
              {#if item.hasOwnPage}
                <a href="{base}{item.href}" class={triggerClass(item)}
                  >{item.label}</a
                >
              {:else}
                <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
                <div tabindex="0" role="button" class={triggerClass(item)}>
                  {item.label}
                </div>
              {/if}
              <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
              <ul
                tabindex="0"
                class="dropdown-content menu z-50 mt-1 min-w-48 rounded-box bg-base-200 p-2 shadow-lg"
              >
                {#each item.children as child (child.href)}
                  <li><a href="{base}{child.href}">{child.label}</a></li>
                {/each}
              </ul>
            </div>
          {/if}
        {/each}
      </div>
    {/snippet}
    {#snippet end()}
      <div class="flex items-center gap-1">
        <NodeStatusBadge onclick={() => (setupModalOpen = true)} />
        <ThemeToggle />
      </div>
    {/snippet}
  </Navbar>

  <main class="flex min-w-0 flex-1 overflow-hidden">
    <div class="relative flex min-w-0 flex-1 flex-col">
      {@render children?.()}
    </div>
    {#if $playerState.currentFile}
      <aside
        class="flex w-96 shrink-0 flex-col gap-2 overflow-y-auto border-l border-base-300 bg-base-200 p-2"
      >
        <PlayerVideo
          file={$playerState.currentFile}
          connectionState={$playerState.connectionState}
          positionSecs={$playerState.positionSecs}
          durationSecs={$playerState.durationSecs}
          buffering={$playerState.buffering}
          poster={$playerState.currentFile.thumbnailUrl}
        />
      </aside>
    {/if}
  </main>
</div>

<ToastOutlet />

<Modal
  open={setupModalOpen}
  maxWidth="max-w-md"
  onclose={() => (setupModalOpen = false)}
>
  <SetupModalContent
    onconnected={() => (setupModalOpen = false)}
    ondisconnect={() => (setupModalOpen = false)}
  />
</Modal>
