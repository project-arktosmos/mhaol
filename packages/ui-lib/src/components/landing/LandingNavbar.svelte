<script lang="ts">
	import { base } from '$app/paths';
	import ThemeToggle from 'ui-lib/components/core/ThemeToggle.svelte';
</script>

<nav class="navbar sticky top-0 z-50 bg-base-100 shadow-sm">
	<div class="flex-1">
		<a href="{base}/" class="btn text-xl font-bold btn-ghost"
			>Mhaol <span class="text-primary">Ecosystem</span></a
		>
	</div>
	<div class="flex-none">
		<ThemeToggle />
	</div>
</nav>
