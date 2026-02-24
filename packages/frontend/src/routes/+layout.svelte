<script lang="ts">
	import '../css/app.css';
	import '$services/i18n';
	import Navbar from '$components/core/Navbar.svelte';

	let { children } = $props();
</script>

<div class="flex min-h-screen flex-col">
	<Navbar />
	<main class="flex-1">
		{@render children?.()}
	</main>
</div>
