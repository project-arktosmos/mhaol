<script lang="ts">
	import '../css/app.css';
	import classNames from 'classnames';
	import Identicon from '$components/core/Identicon.svelte';
	import Navbar from '$components/core/Navbar.svelte';
	import ThemeToggle from '$components/core/ThemeToggle.svelte';
	import ToastOutlet from '$components/core/ToastOutlet.svelte';
	import FirkinFilesPanel from '$components/firkins/FirkinFilesPanel.svelte';
	import NavbarAudioPlayer from '$components/player/NavbarAudioPlayer.svelte';
	import NavbarLyricsPanel from '$components/player/NavbarLyricsPanel.svelte';
	import PlayerVideo from '$components/player/PlayerVideo.svelte';
	import SubsLyricsFinder from '$components/player/SubsLyricsFinder.svelte';
	import { firkinPlaybackService } from '$services/firkin-playback.service';
	import { playerService } from '$services/player.service';
	import { themeService } from '$services/theme.service';
	import { setBrowserImageCacheResolver } from '$services/image-cache.service';
	import { cachedImageUrl } from '$lib/image-cache';
	import { mediaTrackerService } from '$lib/media-tracker.service';
	import { userIdentityService } from '$lib/user-identity.service';
	import { onMount, onDestroy } from 'svelte';
	import { base } from '$app/paths';
	import { NAV_ITEMS, type NavItem } from '$lib/generated/nav';

	setBrowserImageCacheResolver(cachedImageUrl);

	let { children } = $props();

	const playbackState = firkinPlaybackService.state;
	const playerState = playerService.state;
	const playerDisplayMode = playerService.displayMode;
	const identityState = userIdentityService.state;

	// `/profile` lives on the right side of the navbar (as the identity menu),
	// so hide it from the auto-generated central menu.
	const centralNavItems = NAV_ITEMS.filter((item) => item.href !== '/profile');

	const triggerClass = (item: NavItem) =>
		classNames('btn btn-outline btn-sm', { 'btn-disabled': !item.hasOwnPage });

	onMount(() => {
		themeService.initialize('flix');
		playerService.initialize();
		userIdentityService.initialize();
		mediaTrackerService.initialize();
	});

	onDestroy(() => {
		playerService.destroy();
	});
</script>

<div class="flex h-screen flex-col">
	<Navbar brand={{ label: 'Mhaol', highlight: 'Cloud' }} classes="!bg-base-300">
		{#snippet center()}
			<div class="flex flex-wrap items-center gap-1">
				{#each centralNavItems as item (item.href)}
					{#if item.children.length === 0}
						<a href="{base}{item.href}" class="btn btn-outline btn-sm">{item.label}</a>
					{:else}
						<div class="dropdown dropdown-bottom">
							{#if item.hasOwnPage}
								<a href="{base}{item.href}" class={triggerClass(item)}>{item.label}</a>
							{:else}
								<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
								<div tabindex="0" role="button" class={triggerClass(item)}>{item.label}</div>
							{/if}
							<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
							<ul
								tabindex="0"
								class="dropdown-content menu z-50 mt-1 min-w-48 rounded-box bg-base-200 p-2 shadow-lg"
							>
								{#each item.children as child (child.href)}
									<li><a href="{base}{child.href}">{child.label}</a></li>
								{/each}
							</ul>
						</div>
					{/if}
				{/each}
			</div>
		{/snippet}
		{#snippet end()}
			<div class="flex items-center gap-2">
				{#if $identityState.identity}
					<a
						href="{base}/profile"
						class="btn gap-2 font-mono normal-case btn-ghost btn-sm"
						title="Manage identity"
					>
						<Identicon
							address={$identityState.identity.address}
							title={`Identity: ${$identityState.identity.address}`}
							classes="h-5 w-5 shrink-0"
						/>
						{$identityState.identity.username}
					</a>
				{:else if $identityState.loading}
					<span class="loading loading-xs loading-spinner"></span>
				{/if}
				<ThemeToggle />
			</div>
		{/snippet}
	</Navbar>

	{#if $playerDisplayMode === 'navbar' && $playerState.currentFile && $playerState.directStreamUrl}
		<div
			class="fixed right-2 bottom-2 z-40 flex w-96 max-w-[calc(100vw-1rem)] flex-col overflow-hidden rounded-lg border border-base-300 bg-base-100 shadow-lg"
		>
			<NavbarLyricsPanel />
			<NavbarAudioPlayer />
		</div>
	{/if}

	<main class="flex min-w-0 flex-1 overflow-hidden">
		<div class="relative flex min-w-0 flex-1 flex-col overflow-y-auto">
			{@render children?.()}
		</div>
		<aside
			class="flex w-96 shrink-0 flex-col gap-2 overflow-y-auto border-l border-base-300 bg-base-200 p-2"
		>
			{#if $playbackState.firkin}
				<FirkinFilesPanel />
			{/if}
			{#if $playerDisplayMode === 'sidebar' || $playerDisplayMode === 'fullscreen'}
				<PlayerVideo
					file={$playerState.currentFile}
					connectionState={$playerState.connectionState}
					positionSecs={$playerState.positionSecs}
					durationSecs={$playerState.durationSecs}
					buffering={$playerState.buffering}
					poster={$playerState.currentFile?.thumbnailUrl}
					directStreamUrl={$playerState.directStreamUrl}
					directStreamMimeType={$playerState.directStreamMimeType}
				/>
			{/if}
			<SubsLyricsFinder />
		</aside>
	</main>
</div>

<ToastOutlet />
