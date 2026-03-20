<script lang="ts">
	import '../css/app.css';
	import 'frontend/services/i18n';
	import { onMount } from 'svelte';
	import { identityService } from 'frontend/services/identity.service';
	import { themeService } from 'frontend/services/theme.service';
	import ThemeToggle from 'ui-lib/components/core/ThemeToggle.svelte';
	import Navbar from 'ui-lib/components/core/Navbar.svelte';
	import ModalOutlet from 'ui-lib/components/core/ModalOutlet.svelte';
	import IdentityModalContent from 'ui-lib/components/identity/IdentityModalContent.svelte';
	import PluginsModalContent from 'ui-lib/components/plugins/PluginsModalContent.svelte';
	import AddonsModalContent from 'ui-lib/components/addons/AddonsModalContent.svelte';

	let { children } = $props();

	const navItems = [
		{ id: 'identity', label: 'Identity' },
		{ id: 'plugins', label: 'Plugins' },
		{ id: 'addons', label: 'Addons' }
	];

	const modals = {
		identity: { component: IdentityModalContent, maxWidth: 'max-w-3xl' },
		plugins: { component: PluginsModalContent, maxWidth: 'max-w-4xl' },
		addons: { component: AddonsModalContent, maxWidth: 'max-w-4xl' }
	};

	onMount(async () => {
		themeService.initialize('photos');
		await identityService.initialize();
	});
</script>

<div class="flex min-h-screen flex-col">
	<Navbar brand={{ label: 'Mhaol', highlight: 'Photos' }} items={navItems}>
		{#snippet end()}
			<ThemeToggle />
		{/snippet}
	</Navbar>
	<main class="flex min-w-0 flex-1 overflow-hidden">
		{@render children?.()}
	</main>
</div>

<ModalOutlet {modals} />
