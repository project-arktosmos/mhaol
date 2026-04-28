<script lang="ts">
	import '../css/app.css';
	import 'ui-lib/services/i18n';
	import Navbar from 'ui-lib/components/core/Navbar.svelte';
	import ThemeToggle from 'ui-lib/components/core/ThemeToggle.svelte';
	import ToastOutlet from 'ui-lib/components/core/ToastOutlet.svelte';
	import { themeService } from 'ui-lib/services/theme.service';
	import { onMount } from 'svelte';

	let { children } = $props();

	onMount(() => {
		themeService.initialize('flix');
	});
</script>

<div class="flex h-screen flex-col">
	<Navbar brand={{ label: 'Mhaol', highlight: 'Cloud' }} classes="!bg-base-300">
		{#snippet end()}
			<div class="flex items-center gap-1">
				<ThemeToggle />
			</div>
		{/snippet}
	</Navbar>

	<main class="flex min-w-0 flex-1 overflow-y-auto">
		<div class="relative flex min-w-0 flex-1 flex-col">
			{@render children?.()}
		</div>
	</main>
</div>

<ToastOutlet />
