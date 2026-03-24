<script lang="ts">
	import '../css/app.css';
	import 'ui-lib/services/i18n';
	import Navbar from 'ui-lib/components/core/Navbar.svelte';
	import ThemeToggle from 'ui-lib/components/core/ThemeToggle.svelte';
	import SignalingStatusBadge from 'ui-lib/components/signaling/SignalingStatusBadge.svelte';
	import ToastOutlet from 'ui-lib/components/core/ToastOutlet.svelte';
	import { themeService } from 'ui-lib/services/theme.service';
	import { youtubeService } from 'ui-lib/services/youtube.service';
	import { onMount } from 'svelte';

	let { children } = $props();

	const ytState = youtubeService.state;
	const YT_ACTIVE_STATES = ['pending', 'fetching', 'downloading', 'muxing'];
	let ytActiveCount = $derived(
		$ytState.downloads.filter((d: { state: string }) => YT_ACTIVE_STATES.includes(d.state)).length
	);

	onMount(() => {
		themeService.initialize('flix');
	});
</script>

<div class="flex h-screen flex-col">
	<Navbar brand={{ label: 'Mhaol', highlight: 'Media' }} classes="!bg-base-300">
		{#snippet end()}
			<a href="/" class="btn btn-ghost btn-sm">Home</a>
			<a href="/movies" class="btn btn-ghost btn-sm">Movies</a>
			<a href="/tv" class="btn btn-ghost btn-sm">TV</a>
			<a href="/music" class="btn btn-ghost btn-sm">Music</a>
			<a href="/videogames" class="btn btn-ghost btn-sm">Games</a>
			<a href="/books" class="btn btn-ghost btn-sm">Books</a>
			<a href="/photos" class="btn btn-ghost btn-sm">Photos</a>
			<a href="/youtube" class="btn btn-ghost btn-sm">
				YouTube
				{#if ytActiveCount > 0}
					<span class="badge badge-xs badge-primary ml-1">{ytActiveCount}</span>
				{/if}
			</a>
			<a href="/connect" class="btn btn-ghost btn-sm">Connect</a>
			<a href="/roster" class="btn btn-ghost btn-sm">Roster</a>
			<a href="/import" class="btn btn-ghost btn-sm">Import</a>
			<a href="/options" class="btn btn-ghost btn-sm">Options</a>
			<SignalingStatusBadge />
			<ThemeToggle />
		{/snippet}
	</Navbar>
	<main class="flex min-w-0 flex-1 overflow-hidden">
		<div class="relative flex min-w-0 flex-1 flex-col">
			{@render children?.()}
		</div>
	</main>
</div>

<ToastOutlet />
