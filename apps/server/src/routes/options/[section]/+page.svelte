<script lang="ts">
  import { page } from "$app/stores";
  import TorrentModalContent from "ui-lib/components/torrent/TorrentModalContent.svelte";
  import DownloadsModalContent from "ui-lib/components/downloads/DownloadsModalContent.svelte";
  import IdentityModalContent from "ui-lib/components/identity/IdentityModalContent.svelte";
  import VideoSettingsModalContent from "ui-lib/components/settings/VideoSettingsModalContent.svelte";
  import ShareModalContent from "ui-lib/components/share/ShareModalContent.svelte";
  import AddonsModalContent from "ui-lib/components/addons/AddonsModalContent.svelte";
  import LlmModelsModalContent from "ui-lib/components/llm/LlmModelsModalContent.svelte";
  import DatabaseModalContent from "ui-lib/components/database/DatabaseModalContent.svelte";
  import SignalingInfoContent from "ui-lib/components/signaling/SignalingInfoContent.svelte";
  import LibraryModalContent from "ui-lib/components/libraries/LibraryModalContent.svelte";
  import TubeSettingsContent from "ui-lib/components/settings/TubeSettingsContent.svelte";
  import DiskContent from "ui-lib/components/settings/DiskContent.svelte";
  import SmartSearchModalContent from "ui-lib/components/llm/SmartSearchModalContent.svelte";

  const sectionComponents: Record<string, { component: any; props?: Record<string, any> }> = {
    libraries: { component: LibraryModalContent, props: { fixedMediaTypes: ["video"] } },
    torrent: { component: TorrentModalContent },
    downloads: { component: DownloadsModalContent },
    identity: { component: IdentityModalContent },
    addons: { component: AddonsModalContent },
    llm: { component: LlmModelsModalContent },
    share: { component: ShareModalContent },
    signaling: { component: SignalingInfoContent },
    "smart-search": { component: SmartSearchModalContent },
    settings: { component: VideoSettingsModalContent },
    database: { component: DatabaseModalContent },
    "yt-settings": { component: TubeSettingsContent },
    "yt-disk": { component: DiskContent },
  };

  let section = $derived($page.params.section);
  let entry = $derived(sectionComponents[section]);
</script>

{#if entry}
  <entry.component {...(entry.props ?? {})} />
{:else}
  <p class="text-base-content/60">Unknown section: {section}</p>
{/if}
