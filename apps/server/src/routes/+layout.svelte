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
  import SignalingStatusBadge from "ui-lib/components/signaling/SignalingStatusBadge.svelte";
  import { invalidateAll } from "$app/navigation";

  import { youtubeService } from "ui-lib/services/youtube.service";
  import { youtubeLibraryService } from "ui-lib/services/youtube-library.service";
  import SmartSearchToast from "ui-lib/components/llm/SmartSearchToast.svelte";
  import { smartSearchService } from "ui-lib/services/smart-search.service";
  import { apiUrl } from "ui-lib/lib/api-base";
  import { setImageBaseUrl } from "addons/tmdb/transform";
  // browseDetailService still used by movies page library tab (will be removed in cleanup)
  import { rosterService } from "ui-lib/services/roster.service";
  import { signalingAdapter } from "ui-lib/adapters/classes/signaling.adapter";
  import { contactHandshakeService } from "webrtc/service";
  import type { PassportData, ContactHandshakeMessage, Endorsement } from "webrtc/types";
  import { get } from "svelte/store";
  import { getAddress } from "viem";
  import { serverCatalogService } from "ui-lib/services/server-catalog.service";
  import { p2pStreamService } from "ui-lib/services/p2p-stream.service";
  import { movieDetailsToDisplay } from "addons/tmdb/transform";
  import type { MediaItem } from "ui-lib/types/media-card.type";
  import type { CatalogMovie } from "ui-lib/types/server-catalog.type";

  setImageBaseUrl(apiUrl("/api/tmdb/image"));
  import PlayerOverlay from "ui-lib/components/player/PlayerOverlay.svelte";
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
    toggle() {
      browseViewModeValue = browseViewModeValue === "poster" ? "backdrop" : "poster";
    },
  });

  const playerState = playerService.state;

  const ytState = youtubeService.state;
  const YT_ACTIVE_STATES = ["pending", "fetching", "downloading", "muxing"];
  let ytActiveCount = $derived(
    $ytState.downloads.filter((d: { state: string }) =>
      YT_ACTIVE_STATES.includes(d.state),
    ).length,
  );

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
    rosterService.initialize("api");

    // Initialize contact handshake with local passport
    const identities = get(identityService.state).identities;
    if (identities.length > 0 && identities[0].passport) {
      const passport: PassportData = JSON.parse(identities[0].passport);
      contactHandshakeService.initialize({
        passport,
        adapter: {
          sendToPeer: (peerId, envelope) =>
            signalingChatService.sendToPeer(peerId, envelope),
          disconnectPeer: (peerId) =>
            signalingChatService.disconnectPeer(peerId),
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
                  onclick: async () => {
                    // Fetch endorsement from backend before accepting
                    let endorsement: Endorsement | undefined;
                    try {
                      const res = await fetch(apiUrl("/api/signaling/endorse"), {
                        method: "POST",
                        headers: { "Content-Type": "application/json" },
                        body: JSON.stringify({ passportRaw: request.passport.raw }),
                      });
                      if (res.ok) {
                        endorsement = await res.json();
                      }
                    } catch {
                      // Continue without endorsement
                    }

                    contactHandshakeService.acceptRequest(request.address, endorsement);
                    rosterService.addEntry({
                      name: request.name,
                      address: request.address,
                      passport: JSON.stringify(request.passport),
                      endorsement: endorsement ? JSON.stringify(endorsement) : undefined,
                    });
                  },
                },
                {
                  label: "Reject",
                  onclick: () =>
                    contactHandshakeService.rejectRequest(request.address),
                },
              ],
              "info",
            );
          },
          onRequestAccepted: (contact) => {
            rosterService.addEntry({
              name: contact.name,
              address: contact.address,
              passport: JSON.stringify(contact.passport),
              endorsement: contact.endorsement ? JSON.stringify(contact.endorsement) : undefined,
            });
          },
          onConnectionReady: async (peerId) => {
            try {
              const res = await fetch(apiUrl("/api/media"));
              if (!res.ok) return;
              const data = (await res.json()) as {
                libraries: Record<string, { name: string; type: string }>;
                itemsByType: Record<string, MediaItem[]>;
              };
              const movieLibIds = new Set(
                Object.entries(data.libraries)
                  .filter(([, lib]) => lib.type === "movies")
                  .map(([id]) => id),
              );
              const movieItems = (data.itemsByType?.video ?? []).filter(
                (item) => movieLibIds.has(item.libraryId),
              );

              // Fetch TMDB metadata for each item that has a tmdb link
              const origin = window.location.origin;
              const abs = (url: string | null) =>
                url && url.startsWith("/") ? `${origin}${url}` : url;

              const catalog: CatalogMovie[] = await Promise.all(
                movieItems.map(async (item) => {
                  const tmdbLink = item.links?.tmdb;
                  let tmdb = null;
                  if (tmdbLink) {
                    try {
                      const r = await fetch(
                        apiUrl(`/api/tmdb/movies/${tmdbLink.serviceId}`),
                      );
                      if (r.ok) {
                        const details = movieDetailsToDisplay(await r.json());
                        tmdb = {
                          ...details,
                          posterUrl: abs(details.posterUrl),
                          backdropUrl: abs(details.backdropUrl),
                          images: details.images.map((img) => ({
                            ...img,
                            thumbnailUrl: abs(img.thumbnailUrl) ?? "",
                            fullUrl: abs(img.fullUrl) ?? "",
                          })),
                          cast: details.cast.map((c) => ({
                            ...c,
                            profileUrl: abs(c.profileUrl),
                          })),
                        };
                      }
                    } catch {
                      // best-effort
                    }
                  }
                  return { item, tmdb };
                }),
              );
              serverCatalogService.sendMovieCatalog(peerId, catalog);
            } catch {
              // Silently fail — catalog is best-effort
            }
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
        contactHandshakeService.handleMessage(
          peerId,
          msg as ContactHandshakeMessage,
        );
    }

    serverCatalogService.initialize();
    serverCatalogService.onStreamRequest = async (peerId, itemPath) => {
      console.log("[ServerCatalog] Stream request received:", {
        peerId,
        itemPath,
      });
      try {
        const streamConfig = p2pStreamService.getSessionConfig();
        const res = await fetch(apiUrl("/api/player/sessions"), {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            file_path: itemPath,
            mode: "video",
            video_codec: streamConfig.video_codec,
            video_quality: streamConfig.video_quality,
          }),
        });
        if (!res.ok) {
          const body = await res.json().catch(() => ({}));
          serverCatalogService.sendStreamError(
            peerId,
            (body as { error?: string }).error ?? `HTTP ${res.status}`,
          );
          return;
        }
        const session = (await res.json()) as {
          session_id: string;
          room_id: string;
          signaling_url: string;
        };
        serverCatalogService.sendStreamSession(
          peerId,
          session.session_id,
          session.room_id,
          session.signaling_url,
        );
      } catch (err) {
        serverCatalogService.sendStreamError(
          peerId,
          err instanceof Error ? err.message : "Failed to create session",
        );
      }
    };

    youtubeService.initialize();
    youtubeLibraryService.initialize();
    torrentService.initialize("server");
    // Connect to signaling with real identity via backend signing
    const identities2 = get(identityService.state).identities;
    if (identities2.length > 0 && identities2[0].passport) {
      const sigPassport: PassportData = JSON.parse(identities2[0].passport);
      const serverAddress = JSON.parse(sigPassport.raw).address;
      const personalRoom = getAddress(serverAddress as `0x${string}`);

      const signFn = async (msg: string) => {
        const parts = msg.match(/^partykit-auth:(.+):(\d+)$/);
        if (!parts) throw new Error("Invalid auth message format");
        const res = await fetch(
          apiUrl(
            `/api/signaling/auth?room_id=${encodeURIComponent(parts[1])}&timestamp=${parts[2]}`,
          ),
        );
        if (!res.ok) throw new Error(`Auth signing failed: HTTP ${res.status}`);
        const data = await res.json();
        return data.signature;
      };

      // Connect to handshakes room for contact exchange
      signalingChatService.connectToRoom(
        DEFAULT_SIGNALING_URL,
        "handshakes",
        sigPassport,
        signFn,
      );

      // Connect to own personal room
      signalingChatService.connectToRoom(
        DEFAULT_SIGNALING_URL,
        personalRoom,
        sigPassport,
        signFn,
      );
    }

    unsubChat = chatStore.subscribe((s) => {
      // Derive aggregate phase from the handshakes room
      const handshakesRoom = s.rooms["handshakes"];
      const phase = handshakesRoom?.phase ?? "disconnected";

      if (prevPhase === null) {
        prevPhase = phase;
        return;
      }
      if (phase === prevPhase) return;

      switch (phase) {
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
      prevPhase = phase;
    });
  });

  onDestroy(() => {
    playerService.destroy();
    torrentService.destroy();
    signalingChatService.destroy();
    contactHandshakeService.destroy();
    rosterService.destroy();
    serverCatalogService.destroy();
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
      <a href="/books" class="btn btn-ghost btn-sm">Books</a>
      <a href="/photos" class="btn btn-ghost btn-sm">Photos</a>
      <a href="/youtube" class="btn btn-ghost btn-sm">
        YouTube
        {#if ytActiveCount > 0}
          <span class="badge badge-xs badge-primary ml-1">{ytActiveCount}</span>
        {/if}
      </a>
      <a href="/roster" class="btn btn-ghost btn-sm">Roster</a>
      <a href="/import" class="btn btn-ghost btn-sm">Import</a>
      <a href="/options" class="btn btn-ghost btn-sm">Options</a>
      <SignalingStatusBadge />
      <ThemeToggle />
    {/snippet}
  </Navbar>
  <main class="flex min-w-0 flex-1 overflow-hidden">
    <div class="relative flex min-w-0 flex-1 flex-col">
      {@render children?.()}
    </div>
  </main>
</div>

<PlayerOverlay />
<SmartSearchToast
  onlibrarychange={() => invalidateAll()}
  onstream={handleSmartSearchStream}
/>
<ToastOutlet />
