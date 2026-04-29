<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import classNames from "classnames";
  import type {
    YouTubeDownloadProgress,
    YouTubeVideoInfo,
    DownloadMode,
  } from "addons/youtube/types";
  import { extractVideoId } from "addons/youtube/types";

  let url = $state("");
  let info = $state<YouTubeVideoInfo | null>(null);
  let infoLoading = $state(false);
  let infoError = $state<string | null>(null);
  let downloads = $state<YouTubeDownloadProgress[]>([]);
  let queueing = $state<DownloadMode | null>(null);
  let queueError = $state<string | null>(null);
  let connected = $state(false);

  let sse: EventSource | null = null;

  async function fetchInfo() {
    if (!url.trim()) return;
    infoLoading = true;
    infoError = null;
    info = null;
    try {
      const res = await fetch(
        `/api/ytdl/info/video?url=${encodeURIComponent(url.trim())}`,
      );
      if (!res.ok) {
        const body = await res.text();
        throw new Error(body || `HTTP ${res.status}`);
      }
      info = (await res.json()) as YouTubeVideoInfo;
    } catch (e) {
      infoError = e instanceof Error ? e.message : String(e);
    } finally {
      infoLoading = false;
    }
  }

  async function queueDownload(mode: DownloadMode) {
    if (!info) return;
    queueing = mode;
    queueError = null;
    try {
      const res = await fetch("/api/ytdl/downloads", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          url: url.trim(),
          videoId: info.videoId,
          title: info.title,
          mode,
          thumbnailUrl: info.thumbnailUrl,
          durationSeconds: info.duration,
        }),
      });
      if (!res.ok) {
        const body = await res.text();
        throw new Error(body || `HTTP ${res.status}`);
      }
    } catch (e) {
      queueError = e instanceof Error ? e.message : String(e);
    } finally {
      queueing = null;
    }
  }

  async function cancelDownload(id: string) {
    try {
      await fetch(`/api/ytdl/downloads/${id}`, { method: "DELETE" });
    } catch {
      // ignore — SSE will reconcile
    }
  }

  async function clearCompleted() {
    try {
      await fetch("/api/ytdl/downloads/completed", { method: "DELETE" });
      downloads = downloads.filter(
        (d) => !["completed", "failed", "cancelled"].includes(d.state),
      );
    } catch {
      // ignore
    }
  }

  function upsertDownload(progress: YouTubeDownloadProgress) {
    const idx = downloads.findIndex((d) => d.downloadId === progress.downloadId);
    if (idx >= 0) {
      downloads[idx] = progress;
      downloads = downloads;
    } else {
      downloads = [progress, ...downloads];
    }
  }

  function connectSSE() {
    sse = new EventSource("/api/ytdl/downloads/events");
    sse.addEventListener("connected", () => {
      connected = true;
    });
    sse.addEventListener("progress", (e) => {
      try {
        upsertDownload(JSON.parse(e.data));
      } catch {
        // ignore malformed event
      }
    });
    sse.addEventListener("error", () => {
      connected = false;
    });
  }

  function fmtBytes(n: number): string {
    if (!n) return "0 B";
    const units = ["B", "KB", "MB", "GB"];
    let i = 0;
    let v = n;
    while (v >= 1024 && i < units.length - 1) {
      v /= 1024;
      i++;
    }
    return `${v.toFixed(v >= 10 || i === 0 ? 0 : 1)} ${units[i]}`;
  }

  function fmtDuration(secs: number | null): string {
    if (!secs) return "";
    const h = Math.floor(secs / 3600);
    const m = Math.floor((secs % 3600) / 60);
    const s = Math.floor(secs % 60);
    return h > 0
      ? `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`
      : `${m}:${String(s).padStart(2, "0")}`;
  }

  const ACTIVE_STATES = ["pending", "fetching", "downloading", "muxing"];
  let activeDownloads = $derived(
    downloads.filter((d) => ACTIVE_STATES.includes(d.state)),
  );
  let finishedDownloads = $derived(
    downloads.filter((d) => !ACTIVE_STATES.includes(d.state)),
  );
  let canFetchInfo = $derived(!!extractVideoId(url));

  onMount(() => {
    connectSSE();
  });

  onDestroy(() => {
    sse?.close();
  });
</script>

<div class="flex h-full flex-col">
  <header
    class="flex flex-wrap items-center gap-3 border-b border-base-300 px-4 py-3"
  >
    <h1 class="text-lg font-bold">YouTube</h1>
    <span
      class={classNames("badge badge-sm", {
        "badge-success": connected,
        "badge-ghost": !connected,
      })}
    >
      {connected ? "yt-dlp connected" : "connecting…"}
    </span>
  </header>

  <div class="min-w-0 flex-1 overflow-y-auto p-4">
    <div class="mb-6 flex flex-col gap-2">
      <div class="join">
        <input
          type="text"
          class="input join-item input-bordered flex-1"
          placeholder="https://www.youtube.com/watch?v=…"
          bind:value={url}
          onkeydown={(e) => {
            if (e.key === "Enter" && canFetchInfo) fetchInfo();
          }}
        />
        <button
          class="btn join-item btn-primary"
          disabled={!canFetchInfo || infoLoading}
          onclick={fetchInfo}
        >
          {#if infoLoading}
            <span class="loading loading-sm loading-spinner"></span>
          {:else}
            Fetch info
          {/if}
        </button>
      </div>
      {#if infoError}
        <div class="alert alert-error">
          <span>{infoError}</span>
        </div>
      {/if}
    </div>

    {#if info}
      <div class="mb-6 flex gap-4 rounded-lg bg-base-200 p-4">
        {#if info.thumbnailUrl}
          <img
            src={info.thumbnailUrl}
            alt={info.title}
            class="h-24 w-40 shrink-0 rounded object-cover"
            loading="lazy"
          />
        {/if}
        <div class="flex min-w-0 flex-1 flex-col gap-2">
          <p class="font-semibold">{info.title}</p>
          <p class="text-sm opacity-70">
            {info.uploader ?? "Unknown uploader"}
            {#if info.duration}
              · {fmtDuration(info.duration)}
            {/if}
          </p>
          <div class="mt-auto flex flex-wrap gap-2">
            <button
              class="btn btn-sm btn-primary"
              disabled={queueing !== null}
              onclick={() => queueDownload("video")}
            >
              {queueing === "video" ? "Queueing…" : "Download video"}
            </button>
            <button
              class="btn btn-sm btn-secondary"
              disabled={queueing !== null}
              onclick={() => queueDownload("audio")}
            >
              {queueing === "audio" ? "Queueing…" : "Download audio"}
            </button>
            <button
              class="btn btn-sm"
              disabled={queueing !== null}
              onclick={() => queueDownload("both")}
            >
              {queueing === "both" ? "Queueing…" : "Both"}
            </button>
          </div>
          {#if queueError}
            <p class="text-sm text-error">{queueError}</p>
          {/if}
        </div>
      </div>
    {/if}

    {#if activeDownloads.length > 0}
      <section class="mb-6">
        <h2 class="mb-2 text-lg font-semibold">In progress</h2>
        <div class="flex flex-col gap-2">
          {#each activeDownloads as d (d.downloadId)}
            <div class="rounded-lg bg-base-200 p-3">
              <div class="flex items-center justify-between gap-2">
                <p class="truncate font-medium">{d.title || d.videoId}</p>
                <button
                  class="btn btn-ghost btn-xs"
                  onclick={() => cancelDownload(d.downloadId)}
                >
                  Cancel
                </button>
              </div>
              <div class="mt-1 flex items-center gap-2 text-sm opacity-70">
                <span>{d.state}</span>
                <span>·</span>
                <span>{d.mode}</span>
                <span>·</span>
                <span
                  >{fmtBytes(d.downloadedBytes)} / {fmtBytes(
                    d.totalBytes,
                  )}</span
                >
              </div>
              <progress
                class="progress mt-2 w-full"
                value={d.progress}
                max="1"
              ></progress>
            </div>
          {/each}
        </div>
      </section>
    {/if}

    {#if finishedDownloads.length > 0}
      <section>
        <div class="mb-2 flex items-center justify-between">
          <h2 class="text-lg font-semibold">Finished</h2>
          <button class="btn btn-ghost btn-sm" onclick={clearCompleted}
            >Clear</button
          >
        </div>
        <div class="flex flex-col gap-2">
          {#each finishedDownloads as d (d.downloadId)}
            <div
              class={classNames("rounded-lg p-3", {
                "bg-success/10": d.state === "completed",
                "bg-error/10": d.state === "failed",
                "bg-base-200":
                  d.state !== "completed" && d.state !== "failed",
              })}
            >
              <p class="truncate font-medium">{d.title || d.videoId}</p>
              <p class="text-sm opacity-70">
                {d.state}
                {#if d.error}— {d.error}{/if}
              </p>
            </div>
          {/each}
        </div>
      </section>
    {/if}

    {#if !info && downloads.length === 0}
      <p class="rounded-lg bg-base-200 p-8 text-center opacity-60">
        Paste a YouTube URL above to fetch info and queue a download.
      </p>
    {/if}
  </div>
</div>
