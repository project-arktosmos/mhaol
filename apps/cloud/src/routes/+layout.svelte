<script lang="ts">
	import '../css/app.css';
	import 'frontend/services/i18n';
	import { onMount } from 'svelte';
	import { themeService } from 'frontend/services/theme.service';
	import { cloudLibraryService } from 'frontend/services/cloud-library.service';
	import { cloudPeerService } from 'frontend/services/cloud-peer.service';
	import Navbar from 'ui-lib/components/core/Navbar.svelte';
	import ModalOutlet from 'ui-lib/components/core/ModalOutlet.svelte';
	import SignalingModalContent from 'ui-lib/components/signaling/SignalingModalContent.svelte';
	import SettingsModalContent from 'ui-lib/components/settings/SettingsModalContent.svelte';
	import CloudLibraryModalContent from 'ui-lib/components/cloud/CloudLibraryModalContent.svelte';

	let { children } = $props();

	const themeStore = themeService.store;

	const navItems = [
		{ id: 'libraries', label: 'Libraries' },
		{ id: 'signaling', label: 'Signaling' },
		{ id: 'settings', label: 'Settings' }
	];

	const modals = {
		libraries: { component: CloudLibraryModalContent, maxWidth: 'max-w-3xl' },
		signaling: { component: SignalingModalContent, maxWidth: 'max-w-5xl' },
		settings: { component: SettingsModalContent, maxWidth: 'max-w-2xl' }
	};

	onMount(async () => {
		await cloudLibraryService.initialize();
		cloudPeerService.initialize();
	});

	$effect(() => {
		document.documentElement.setAttribute('data-theme', $themeStore.theme);
	});
</script>

<div class="flex min-h-screen flex-col">
	<Navbar brand={{ label: 'Mhaol', highlight: 'Cloud' }} items={navItems} />
	<main class="min-w-0 flex-1">
		{@render children?.()}
	</main>
</div>

<ModalOutlet {modals} />
