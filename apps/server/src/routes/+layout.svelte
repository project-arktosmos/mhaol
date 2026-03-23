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
  import { rosterService } from "ui-lib/services/roster.service";
  import type { PassportData } from "webrtc/types";

  setImageBaseUrl(apiUrl("/api/tmdb/image"));
  import PlayerOverlay from "ui-lib/components/player/PlayerOverlay.svelte";
  import type { SmartSearchTorrentResult } from "ui-lib/types/smart-search.type";
  import type { PlayableFile } from "ui-lib/types/player.type";

  let { children } = $props();

  type BrowseViewMode = "poster" | "backdrop" | "table";
  let browseViewModeValue = $state<BrowseViewMode>("poster");
  setContext("browseViewMode", {
    get value() {
      return browseViewModeValue;
    },
    set(mode: BrowseViewMode) {
      browseViewModeValue = mode;
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

  onMount(async () => {
    themeService.initialize("flix");
    await playerService.initialize();
    await identityService.initialize();
    rosterService.initialize("api");
    youtubeService.initialize();
    youtubeLibraryService.initialize();
    torrentService.initialize("server");

    // Connect to signaling as a client using CLIENT_WALLET identity
    try {
      const res = await fetch(apiUrl("/api/signaling/client-identity"));
      if (res.ok) {
        const data = await res.json();
        const passport: PassportData = data.passport;
        const serverRoom: string = data.serverRoom;

        const signFn = async (msg: string) => {
          const parts = msg.match(/^partykit-auth:(.+):(\d+)$/);
          if (!parts) throw new Error("Invalid auth message format");
          const authRes = await fetch(
            apiUrl(
              `/api/signaling/auth?room_id=${encodeURIComponent(parts[1])}&timestamp=${parts[2]}`,
            ),
          );
          if (!authRes.ok)
            throw new Error(`Auth signing failed: HTTP ${authRes.status}`);
          const authData = await authRes.json();
          return authData.signature;
        };

        // Connect to the server's personal room as a client
        signalingChatService.connectToRoom(
          DEFAULT_SIGNALING_URL,
          serverRoom,
          passport,
          signFn,
        );
      }
    } catch {
      // Signaling is optional for the admin frontend
    }
  });

  onDestroy(() => {
    playerService.destroy();
    torrentService.destroy();
    signalingChatService.destroy();
    rosterService.destroy();
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
