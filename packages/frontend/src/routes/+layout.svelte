<script lang="ts">
	import '../css/app.css';
	import '$services/i18n';
	import { onMount, onDestroy } from 'svelte';
	import { playerService } from '$services/player.service';
	import Navbar from '$components/core/Navbar.svelte';
	import PlayerModal from '$components/player/PlayerModal.svelte';

	let { children } = $props();

	const playerState = playerService.state;

	onMount(async () => {
		await playerService.initialize();
	});

	onDestroy(() => {
		playerService.destroy();
	});
</script>

<div class="flex min-h-screen flex-col">
	<Navbar />
	<main class="flex-1">
		{@render children?.()}
	</main>
</div>

{#if $playerState.currentFile}
	<PlayerModal
		file={$playerState.currentFile}
		connectionState={$playerState.connectionState}
		positionSecs={$playerState.positionSecs}
		durationSecs={$playerState.durationSecs}
		onclose={() => playerService.stop()}
	/>
{/if}
