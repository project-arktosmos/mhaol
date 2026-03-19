<script lang="ts">
	import '../css/app.css';
	import 'frontend/services/i18n';
	import { onMount, onDestroy } from 'svelte';
	import { playerService } from 'frontend/services/player.service';
	import { identityService } from 'frontend/services/identity.service';
	import { peerLibraryService } from 'frontend/services/peer-library.service';
	import IdentitySidebar from 'frontend/components/core/IdentitySidebar.svelte';
	import AppNavbar from '../app-components/AppNavbar.svelte';
	import AppModalOutlet from '../app-components/AppModalOutlet.svelte';

	let { children } = $props();

	onMount(async () => {
		await playerService.initialize();
		await identityService.initialize();
		peerLibraryService.initialize();
	});

	onDestroy(() => {
		playerService.destroy();
	});
</script>

<div class="flex min-h-screen flex-col">
	<AppNavbar />
	<div class="flex flex-1">
		<main class="min-w-0 flex-1">
			{@render children?.()}
		</main>
		<IdentitySidebar />
	</div>
</div>

<AppModalOutlet />
