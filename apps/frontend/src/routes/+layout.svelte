<script lang="ts">
	import '../css/app.css';
	import 'ui-lib/services/i18n';
	import Navbar from 'ui-lib/components/core/Navbar.svelte';
	import SideDrawer from 'ui-lib/components/core/SideDrawer.svelte';
	import ThemeToggle from 'ui-lib/components/core/ThemeToggle.svelte';
	import NodeStatusBadge from 'ui-lib/components/core/NodeStatusBadge.svelte';
	import ToastOutlet from 'ui-lib/components/core/ToastOutlet.svelte';
	import SetupGate from 'ui-lib/components/setup/SetupGate.svelte';
	import { themeService } from 'ui-lib/services/theme.service';
	import { youtubeService } from 'ui-lib/services/youtube.service';
	import Modal from 'ui-lib/components/core/Modal.svelte';
	import SetupModalContent from 'ui-lib/components/setup/SetupModalContent.svelte';
	import { onMount } from 'svelte';
	import { afterNavigate, invalidateAll } from '$app/navigation';
	import { base } from '$app/paths';

	let { children } = $props();

	let setupModalOpen = $state(false);
	let drawerOpen = $state(false);

	const ytState = youtubeService.state;
	const YT_ACTIVE_STATES = ['pending', 'fetching', 'downloading', 'muxing'];
	let ytActiveCount = $derived(
		$ytState.downloads.filter((d: { state: string }) => YT_ACTIVE_STATES.includes(d.state)).length
	);

	afterNavigate(() => {
		drawerOpen = false;
	});

	onMount(() => {
		themeService.initialize('flix');
	});
</script>

<SideDrawer bind:open={drawerOpen}>
	{#snippet sidebar()}
		<div class="mb-4 text-lg font-bold">Mhaol<span class="text-primary">Media</span></div>
		<ul class="menu gap-1">
			<li><a href="{base}/media/movies">Movies</a></li>
			<li><a href="{base}/media/tv">TV</a></li>
			<li><a href="{base}/media/music">Music</a></li>
			<li><a href="{base}/media/videogames">Games</a></li>
			<li><a href="{base}/media/books">Books</a></li>
			<li><a href="{base}/media/photos">Photos</a></li>
			<li>
				<a href="{base}/media/youtube">
					YouTube
					{#if ytActiveCount > 0}
						<span class="badge badge-xs badge-primary">{ytActiveCount}</span>
					{/if}
				</a>
			</li>
			<li><a href="{base}/media/iptv">IPTV</a></li>
			<li><a href="{base}/media/search">Search</a></li>
			<li><a href="{base}/profiles">Profiles</a></li>
			<li><a href="{base}/import">Import</a></li>
			<li><a href="{base}/queue">Queue</a></li>
			<li><a href="{base}/options">Options</a></li>
		</ul>
		<div class="divider"></div>
		<div class="flex items-center gap-2">
			<NodeStatusBadge onclick={() => (setupModalOpen = true)} />
			<ThemeToggle />
		</div>
	{/snippet}

	<div class="flex h-screen flex-col">
		<Navbar brand={{ label: 'Mhaol', highlight: 'Media' }} classes="!bg-base-300">
			{#snippet center()}
				<div class="hidden items-center gap-1 lg:flex">
					<a href="{base}/media/movies" class="btn btn-outline btn-sm">Movies</a>
					<a href="{base}/media/tv" class="btn btn-outline btn-sm">TV</a>
					<a href="{base}/media/music" class="btn btn-outline btn-sm">Music</a>
					<a href="{base}/media/videogames" class="btn btn-outline btn-sm">Games</a>
					<a href="{base}/media/books" class="btn btn-outline btn-sm">Books</a>
					<a href="{base}/media/photos" class="btn btn-outline btn-sm">Photos</a>
					<a href="{base}/media/youtube" class="btn btn-outline btn-sm">
						YouTube
						{#if ytActiveCount > 0}
							<span class="badge badge-xs badge-primary ml-1">{ytActiveCount}</span>
						{/if}
					</a>
					<a href="{base}/media/iptv" class="btn btn-outline btn-sm">IPTV</a>
					<a href="{base}/media/search" class="btn btn-outline btn-sm">Search</a>
				</div>
			{/snippet}
			{#snippet end()}
				<button class="btn btn-ghost lg:hidden" aria-label="Open menu" onclick={() => (drawerOpen = true)}>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						fill="none"
						viewBox="0 0 24 24"
						stroke-width="1.5"
						stroke="currentColor"
						class="h-6 w-6"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5"
						/>
					</svg>
				</button>
				<div class="hidden items-center gap-1 lg:flex">
					<a href="{base}/profiles" class="btn btn-ghost btn-sm">Profiles</a>
					<a href="{base}/import" class="btn btn-ghost btn-sm">Import</a>
					<a href="{base}/queue" class="btn btn-ghost btn-sm">Queue</a>
					<a href="{base}/options" class="btn btn-ghost btn-sm">Options</a>
					<NodeStatusBadge onclick={() => (setupModalOpen = true)} />
					<ThemeToggle />
				</div>
			{/snippet}
		</Navbar>
		<SetupGate onready={() => invalidateAll()}>
			<main class="flex min-w-0 flex-1 overflow-hidden">
				<div class="relative flex min-w-0 flex-1 flex-col">
					{@render children?.()}
				</div>
			</main>
		</SetupGate>
	</div>
</SideDrawer>

<ToastOutlet />

<Modal open={setupModalOpen} maxWidth="max-w-md" onclose={() => (setupModalOpen = false)}>
	<SetupModalContent
		onconnected={() => (setupModalOpen = false)}
		ondisconnect={() => (setupModalOpen = false)}
	/>
</Modal>
