<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import AppHealthPanel from '../components/AppHealthPanel.svelte';
	import { appsHealthService } from '../lib/apps-health.service';

	const state = appsHealthService.state;

	onMount(() => {
		appsHealthService.start(5000);
	});

	onDestroy(() => {
		appsHealthService.stop();
	});

	const anyLoading = $derived($state.apps.some((a) => a.loading));
</script>

<svelte:head>
	<title>Mhaol — Apps</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<header class="flex items-center justify-between gap-4">
		<div>
			<h1 class="text-2xl font-bold">Mhaol Apps</h1>
			<p class="text-sm text-base-content/60">
				Health of the local Mhaol apps. Refreshes every 5 seconds.
			</p>
		</div>
		<button
			class="btn btn-outline btn-sm"
			onclick={() => appsHealthService.refresh()}
			disabled={anyLoading}
		>
			Refresh
		</button>
	</header>

	<div class="flex flex-col gap-4">
		{#each $state.apps as app (app.id)}
			<AppHealthPanel {app} />
		{/each}
	</div>
</div>
