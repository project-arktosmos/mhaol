<script lang="ts">
	import '../css/app.css';
	import 'frontend/services/i18n';
	import { onMount } from 'svelte';
	import { themeService } from 'frontend/services/theme.service';
	import LandingNavbar from 'ui-lib/components/landing/LandingNavbar.svelte';

	let { children } = $props();

	onMount(() => {
		themeService.initialize();
	});
</script>

<div class="min-h-screen bg-base-100">
	<LandingNavbar />
	{@render children?.()}
</div>
