<script lang="ts">
  import "../css/app.css";
  import "ui-lib/services/i18n";
  import { onMount, onDestroy, setContext } from "svelte";
  import { playerService } from "ui-lib/services/player.service";
  import { identityService } from "ui-lib/services/identity.service";
  import { torrentService } from "ui-lib/services/torrent.service";
  import { themeService } from "ui-lib/services/theme.service";
  import { signalingChatService } from "ui-lib/services/signaling-chat.service";
  import { DEFAULT_SIGNALING_URL } from "ui-lib/lib/api-base";
  import { toastService } from "ui-lib/services/toast.service";
  import ThemeToggle from "ui-lib/components/core/ThemeToggle.svelte";
  import ToastOutlet from "ui-lib/components/core/ToastOutlet.svelte";
  import Navbar from "ui-lib/components/core/Navbar.svelte";
  import { invalidateAll } from "$app/navigation";
  import { youtubeService } from "ui-lib/services/youtube.service";
  import { youtubeLibraryService } from "ui-lib/services/youtube-library.service";
  import SmartSearchToast from "ui-lib/components/llm/SmartSearchToast.svelte";
  import { smartSearchService } from "ui-lib/services/smart-search.service";
  import { apiUrl } from "ui-lib/lib/api-base";
  import { setImageBaseUrl } from "addons/tmdb/transform";
  import { browseDetailService } from "ui-lib/services/browse-detail.service";
  import { signalingAdapter } from "ui-lib/adapters/classes/signaling.adapter";
  import { contactHandshakeService } from "webrtc/service";
  import type { PassportData, ContactHandshakeMessage } from "webrtc/types";
  import { get } from "svelte/store";

  setImageBaseUrl(apiUrl("/api/tmdb/image"));
  import BrowseDetailPanel from "ui-lib/components/browse/BrowseDetailPanel.svelte";
  import Modal from "ui-lib/components/core/Modal.svelte";
  import type { SmartSearchTorrentResult } from "ui-lib/types/smart-search.type";
  import type { PlayableFile } from "ui-lib/types/player.type";

  let { children } = $props();

  const chatStore = signalingChatService.state;

  let prevPhase: string | null = null;

  let browseViewModeValue = $state<"poster" | "backdrop">("poster");
  setContext("browseViewMode", {
    get value() {
      return browseViewModeValue;
    },
  });

  const playerState = playerService.state;
  const browseDetail = browseDetailService.state;

  let isLargeScreen = $state(false);

  $effect(() => {
    const mql = window.matchMedia("(min-width: 1024px)");
    isLargeScreen = mql.matches;
    const handler = (e: MediaQueryListEvent) => {
      isLargeScreen = e.matches;
    };
    mql.addEventListener("change", handler);
    return () => mql.removeEventListener("change", handler);
  });


  const ytState = youtubeService.state;
  const YT_ACTIVE_STATES = ["pending", "fetching", "downloading", "muxing"];
  let ytActiveCount = $derived(
    $ytState.downloads.filter((d: { state: string }) =>
      YT_ACTIVE_STATES.includes(d.state),
    ).length,
  );

  let hasBrowseSelection = $derived($browseDetail.domain !== null);

  function handleSidebarAction(action: string) {
    const cbs = browseDetailService.getCallbacks();
    const cb = cbs[action as keyof typeof cbs];
    if (typeof cb === 'function') (cb as () => void)();
  }

  async function handleSmartSearchStream(candidate: SmartSearchTorrentResult) {
    smartSearchService.hide();
    const infoHash = await smartSearchService.startStream(candidate);
    if (!infoHash) return;
    invalidateAll();
    playerService.setDisplayMode("sidebar");

    let ready = false;
    const unsubscribe = torrentService.state.subscribe(() => {
      if (!ready) return;
      const torrent = torrentService.findByHash(infoHash);
      if (!torrent) return;

      smartSearchService.updateStreamingProgress(torrent.progress);

      if (torrent.progress >= 0.02 || torrent.state === "seeding") {
        unsubscribe();
        smartSearchService.clearStreaming();

        const file: PlayableFile = {
          id: `torrent:${infoHash}`,
          type: "torrent",
          name: torrent.name,
          outputPath: torrent.outputPath ?? "",
          mode: "video",
          format: null,
          videoFormat: null,
          thumbnailUrl: null,
          durationSeconds: null,
          size: torrent.size,
          completedAt: "",
          streamUrl: `/api/torrent/torrents/${infoHash}/stream`,
        };
        playerService.playStream(file);
      }
    });
    ready = true;
  }

  let unsubChat: (() => void) | undefined;

  onMount(async () => {
    themeService.initialize("flix");
    await playerService.initialize();
    await identityService.initialize();

    // Initialize contact handshake with local passport
    const identities = get(identityService.state).identities;
    if (identities.length > 0 && identities[0].passport) {
      const passport: PassportData = JSON.parse(identities[0].passport);
      contactHandshakeService.initialize({
        passport,
        adapter: {
          sendToPeer: (peerId, envelope) => signalingChatService.sendToPeer(peerId, envelope),
          disconnectPeer: (peerId) => signalingChatService.disconnectPeer(peerId),
          connectToPeer: (peerId) => signalingChatService.connectToPeer(peerId),
          getPeerConnectionStatus: (peerId) =>
            signalingChatService.getPeerConnectionStatus(peerId),
        },
        callbacks: {
          onRequestReceived: (request) => {
            toastService.addWithActions(
              `Contact request from ${request.name} (${signalingAdapter.shortAddress(request.address)})`,
              [
                {
                  label: "Accept",
                  onclick: () => contactHandshakeService.acceptRequest(request.address),
                },
                {
                  label: "Reject",
                  onclick: () => contactHandshakeService.rejectRequest(request.address),
                },
              ],
              "info",
            );
          },
          onRequestAccepted: (contact) => {
            toastService.success(`${contact.name} accepted your contact request`);
          },
          onError: (message) => {
            toastService.error(message);
          },
        },
      });

      signalingChatService.addPeerChannelOpenListener((peerId) =>
        contactHandshakeService.handleChannelOpen(peerId),
      );
      signalingChatService.onContactMessage = (peerId, msg) =>
        contactHandshakeService.handleMessage(peerId, msg as ContactHandshakeMessage);
    }

    youtubeService.initialize();
    youtubeLibraryService.initialize();
    torrentService.initialize("server");
    signalingChatService.connect(DEFAULT_SIGNALING_URL, "default");

    unsubChat = chatStore.subscribe((s) => {
      if (prevPhase === null) {
        prevPhase = s.phase;
        return;
      }
      if (s.phase === prevPhase) return;

      switch (s.phase) {
        case "connecting":
          toastService.info("Connecting to signaling server...");
          break;
        case "connected":
          toastService.success("Connected to signaling server");
          break;
        case "error":
          toastService.error(s.error || "Connection error");
          break;
        case "disconnected":
          if (prevPhase === "connected") {
            toastService.warning("Disconnected from signaling server");
          }
          break;
      }
      prevPhase = s.phase;
    });
  });

  onDestroy(() => {
    playerService.destroy();
    torrentService.destroy();
    signalingChatService.destroy();
    contactHandshakeService.destroy();
    unsubChat?.();
  });
</script>

<div class="flex h-screen flex-col">
  <Navbar
    brand={{ label: "Mhaol", highlight: "Server" }}
    classes="!bg-base-300"
  >
    {#snippet end()}
      <a href="/movies" class="btn btn-ghost btn-sm">Movies</a>
      <a href="/tv" class="btn btn-ghost btn-sm">TV</a>
      <a href="/music" class="btn btn-ghost btn-sm">Music</a>
      <a href="/videogames" class="btn btn-ghost btn-sm">Games</a>
      <a href="/photos" class="btn btn-ghost btn-sm">Photos</a>
      <a href="/youtube" class="btn btn-ghost btn-sm">
        YouTube
        {#if ytActiveCount > 0}
          <span class="badge badge-xs badge-primary ml-1"
            >{ytActiveCount}</span
          >
        {/if}
      </a>
      <button
        class="btn btn-circle btn-ghost btn-sm"
        onclick={() =>
          (browseViewModeValue =
            browseViewModeValue === "poster" ? "backdrop" : "poster")}
        aria-label="Toggle view mode"
        title={browseViewModeValue === "poster"
          ? "Switch to backdrop view"
          : "Switch to poster view"}
      >
        {#if browseViewModeValue === "poster"}
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-5 w-5"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width="1.5"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M2.25 15.75l5.159-5.159a2.25 2.25 0 013.182 0l5.159 5.159m-1.5-1.5l1.409-1.409a2.25 2.25 0 013.182 0l2.909 2.909M3.75 21h16.5A2.25 2.25 0 0022.5 18.75V5.25A2.25 2.25 0 0020.25 3H3.75A2.25 2.25 0 001.5 5.25v13.5A2.25 2.25 0 003.75 21z"
            />
          </svg>
        {:else}
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-5 w-5"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width="1.5"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M3.75 6A2.25 2.25 0 016 3.75h2.25A2.25 2.25 0 0110.5 6v2.25a2.25 2.25 0 01-2.25 2.25H6a2.25 2.25 0 01-2.25-2.25V6zM3.75 15.75A2.25 2.25 0 016 13.5h2.25a2.25 2.25 0 012.25 2.25V18a2.25 2.25 0 01-2.25 2.25H6A2.25 2.25 0 013.75 18v-2.25zM13.5 6a2.25 2.25 0 012.25-2.25H18A2.25 2.25 0 0120.25 6v2.25A2.25 2.25 0 0118 10.5h-2.25a2.25 2.25 0 01-2.25-2.25V6zM13.5 15.75a2.25 2.25 0 012.25-2.25H18a2.25 2.25 0 012.25 2.25V18A2.25 2.25 0 0118 20.25h-2.25A2.25 2.25 0 0113.5 18v-2.25z"
            />
          </svg>
        {/if}
      </button>
      <a
        href="/options"
        class="btn btn-circle btn-ghost btn-sm"
        aria-label="Options"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="1.5"
          stroke="currentColor"
          class="h-5 w-5"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.325.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 0 1 1.37.49l1.296 2.247a1.125 1.125 0 0 1-.26 1.431l-1.003.827c-.293.241-.438.613-.43.992a7.723 7.723 0 0 1 0 .255c-.008.378.137.75.43.991l1.004.827c.424.35.534.955.26 1.43l-1.298 2.247a1.125 1.125 0 0 1-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.47 6.47 0 0 1-.22.128c-.331.183-.581.495-.644.869l-.213 1.281c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.019-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 0 1-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 0 1-1.369-.49l-1.297-2.247a1.125 1.125 0 0 1 .26-1.431l1.004-.827c.292-.24.437-.613.43-.991a6.932 6.932 0 0 1 0-.255c.007-.38-.138-.751-.43-.992l-1.004-.827a1.125 1.125 0 0 1-.26-1.43l1.297-2.247a1.125 1.125 0 0 1 1.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.086.22-.128.332-.183.582-.495.644-.869l.214-1.28Z"
          />
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z"
          />
        </svg>
      </a>
      <ThemeToggle />
    {/snippet}
  </Navbar>
  <main class="flex min-w-0 flex-1 overflow-hidden">
    {@render children?.()}
    <div
      class="hidden w-85 shrink-0 overflow-y-auto border-l border-base-300 bg-base-200 lg:block"
    >
      <BrowseDetailPanel />
    </div>
  </main>
</div>

{#if !isLargeScreen}
  <Modal
    open={hasBrowseSelection || !!$playerState.currentFile}
    maxWidth="max-w-lg"
    onclose={() => handleSidebarAction("onclose")}
  >
    <BrowseDetailPanel />
  </Modal>
{/if}
<SmartSearchToast
  onlibrarychange={() => invalidateAll()}
  onstream={handleSmartSearchStream}
/>
<ToastOutlet />
