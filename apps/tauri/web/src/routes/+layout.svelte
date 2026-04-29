<script lang="ts">
	import '../css/app.css';
	import { themeService } from 'ui-lib/services/theme.service';
	import { onMount } from 'svelte';

	let { children } = $props();

	onMount(() => {
		themeService.initialize('flix');
	});
</script>

<div class="flex h-screen flex-col bg-base-100">
	<main class="flex min-w-0 flex-1 overflow-y-auto">
		<div class="relative flex min-w-0 flex-1 flex-col">
			{@render children?.()}
		</div>
	</main>
</div>
