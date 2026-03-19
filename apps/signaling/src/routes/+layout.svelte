<script lang="ts">
	import '../css/app.css';
	import 'frontend/services/i18n';
	import { themeService } from 'frontend/services/theme.service';
	import Navbar from 'ui-lib/components/core/Navbar.svelte';
	import ModalOutlet from 'ui-lib/components/core/ModalOutlet.svelte';
	import SettingsModalContent from 'ui-lib/components/settings/SettingsModalContent.svelte';

	let { children } = $props();

	const themeStore = themeService.store;

	const navItems = [{ id: 'settings', label: 'Settings' }];

	const modals = {
		settings: { component: SettingsModalContent, maxWidth: 'max-w-2xl' }
	};

	$effect(() => {
		document.documentElement.setAttribute('data-theme', $themeStore.theme);
	});
</script>

<div class="flex min-h-screen flex-col">
	<Navbar brand={{ label: 'Mhaol', highlight: 'Signaling' }} items={navItems} />
	<main class="min-w-0 flex-1">
		{@render children?.()}
	</main>
</div>

<ModalOutlet {modals} />
