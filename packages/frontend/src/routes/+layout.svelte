<script lang="ts">
	import '../css/app.css';
	import '$services/i18n';
	import { onMount, onDestroy } from 'svelte';
	import { playerService } from '$services/player.service';
	import { identityService } from '$services/identity.service';
	import Navbar from '$components/core/Navbar.svelte';
	import IdentitySidebar from '$components/core/IdentitySidebar.svelte';
	import DownloadsModal from '$components/downloads/DownloadsModal.svelte';
	import TorrentModal from '$components/torrent/TorrentModal.svelte';
	import YouTubeModal from '$components/youtube/YouTubeModal.svelte';
	import LibraryModal from '$components/libraries/LibraryModal.svelte';

	let { children } = $props();

	onMount(async () => {
		await playerService.initialize();
		await identityService.initialize();
	});

	onDestroy(() => {
		playerService.destroy();
	});
</script>

<div class="flex min-h-screen flex-col">
	<Navbar />
	<div class="flex flex-1">
		<main class="min-w-0 flex-1">
			{@render children?.()}
		</main>
		<IdentitySidebar />
	</div>
</div>

<DownloadsModal />
<TorrentModal />
<YouTubeModal />
<LibraryModal />
