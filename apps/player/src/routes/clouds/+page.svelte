<script lang="ts">
  import classNames from "classnames";
  import { cloudsService } from "ui-lib/services/clouds.service";
  import { connectionConfigService } from "ui-lib/services/connection-config.service";
  import { nodeConnectionService } from "ui-lib/services/node-connection.service";
  import Modal from "ui-lib/components/core/Modal.svelte";
  import SetupModalContent from "ui-lib/components/setup/SetupModalContent.svelte";
  import type { CloudServer } from "ui-lib/types/cloud-server.type";

  const clouds = cloudsService.store;
  const activeConfig = connectionConfigService.store;
  const conn = nodeConnectionService.state;

  let addOpen = $state(false);
  let busyId = $state<string | null>(null);
  let connectError = $state<string | null>(null);

  function isActive(cloud: CloudServer): boolean {
    const c = $activeConfig;
    if (!c) return false;
    return (
      c.transportMode === cloud.transportMode &&
      c.serverUrl === cloud.serverUrl &&
      c.serverAddress === cloud.serverAddress
    );
  }

  function protocolLabel(mode: string): string {
    if (mode === "http") return "HTTP";
    if (mode === "ws") return "WebSocket";
    if (mode === "webrtc") return "WebRTC";
    return mode;
  }

  function endpointLabel(cloud: CloudServer): string {
    if (cloud.transportMode === "webrtc") return cloud.serverAddress || "—";
    return cloud.serverUrl || "—";
  }

  function formatDate(value?: number): string {
    if (!value) return "never";
    try {
      return new Date(value).toLocaleString();
    } catch {
      return String(value);
    }
  }

  async function connect(cloud: CloudServer) {
    busyId = cloud.id;
    connectError = null;
    const config = {
      transportMode: cloud.transportMode,
      serverUrl: cloud.serverUrl,
      serverAddress: cloud.serverAddress,
      signalingUrl: cloud.signalingUrl,
    };
    try {
      if (cloud.transportMode === "ws")
        await nodeConnectionService.connectWs(config);
      else if (cloud.transportMode === "webrtc")
        await nodeConnectionService.connectWebRtc(config);
      else await nodeConnectionService.connectHttp(config);
      connectionConfigService.save(config);
    } catch (err) {
      connectError = err instanceof Error ? err.message : "Connection failed";
    } finally {
      busyId = null;
    }
  }

  function disconnect() {
    nodeConnectionService.disconnect();
    connectionConfigService.clear();
  }

  function remove(cloud: CloudServer) {
    if (isActive(cloud)) disconnect();
    cloudsService.remove(cloud);
  }
</script>

<svelte:head>
  <title>Mhaol Player — Clouds</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
  <header class="flex items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-bold">Clouds</h1>
      <p class="text-sm text-base-content/60">
        Cloud servers you've connected to. Each entry is persisted in
        localStorage along with the RPC protocol used to connect. The player
        works without a connection — pick a cloud here when you want to query
        one.
      </p>
    </div>
    <button class="btn btn-outline btn-sm" onclick={() => (addOpen = true)}
      >+ Add cloud</button
    >
  </header>

  {#if connectError}
    <div class="alert alert-error">
      <span>{connectError}</span>
    </div>
  {/if}

  {#if $clouds.length === 0}
    <p class="text-sm text-base-content/60">
      No clouds saved yet. Click "Add cloud" to connect to one — the protocol
      and server you pick will be remembered here.
    </p>
  {:else}
    <div class="overflow-x-auto rounded-box border border-base-content/10">
      <table class="table table-sm">
        <thead>
          <tr>
            <th>Name</th>
            <th>Protocol</th>
            <th>Endpoint</th>
            <th>Signaling</th>
            <th>Last connected</th>
            <th>Status</th>
            <th class="w-48"></th>
          </tr>
        </thead>
        <tbody>
          {#each $clouds as cloud (cloud.id)}
            <tr class={classNames({ "bg-base-200": isActive(cloud) })}>
              <td class="font-medium">{cloud.name}</td>
              <td
                ><span class="badge badge-sm"
                  >{protocolLabel(cloud.transportMode)}</span
                ></td
              >
              <td class="font-mono text-xs">{endpointLabel(cloud)}</td>
              <td class="font-mono text-xs text-base-content/70"
                >{cloud.signalingUrl}</td
              >
              <td class="text-xs text-base-content/60"
                >{formatDate(cloud.lastConnectedAt)}</td
              >
              <td>
                {#if isActive(cloud) && $conn.phase === "ready"}
                  <span class="badge badge-sm badge-success">Active</span>
                {:else if isActive(cloud)}
                  <span class="badge badge-sm badge-info">{$conn.phase}</span>
                {:else}
                  <span class="badge badge-sm badge-ghost">Inactive</span>
                {/if}
              </td>
              <td class="text-right">
                <div class="flex flex-wrap justify-end gap-1">
                  {#if isActive(cloud) && ($conn.phase === "ready" || $conn.phase === "connecting")}
                    <button class="btn btn-outline btn-xs" onclick={disconnect}
                      >Disconnect</button
                    >
                  {:else}
                    <button
                      class={classNames("btn btn-outline btn-xs btn-primary", {
                        "btn-disabled": busyId === cloud.id,
                      })}
                      onclick={() => connect(cloud)}
                      disabled={busyId === cloud.id}
                    >
                      {busyId === cloud.id ? "Connecting…" : "Connect"}
                    </button>
                  {/if}
                  <button
                    class="btn text-error btn-ghost btn-xs"
                    onclick={() => remove(cloud)}
                  >
                    Remove
                  </button>
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<Modal open={addOpen} maxWidth="max-w-md" onclose={() => (addOpen = false)}>
  <SetupModalContent
    onconnected={() => (addOpen = false)}
    ondisconnect={() => (addOpen = false)}
  />
</Modal>
