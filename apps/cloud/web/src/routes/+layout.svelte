<script lang="ts">
	import '../css/app.css';
	import 'ui-lib/services/i18n';
	import classNames from 'classnames';
	import Navbar from 'ui-lib/components/core/Navbar.svelte';
	import ThemeToggle from 'ui-lib/components/core/ThemeToggle.svelte';
	import ToastOutlet from 'ui-lib/components/core/ToastOutlet.svelte';
	import FirkinFilesPanel from 'ui-lib/components/firkins/FirkinFilesPanel.svelte';
	import PlayerVideo from 'ui-lib/components/player/PlayerVideo.svelte';
	import SubsLyricsFinder from 'ui-lib/components/player/SubsLyricsFinder.svelte';
	import { firkinPlaybackService } from 'ui-lib/services/firkin-playback.service';
	import { firkinStreamService } from 'ui-lib/services/firkin-stream.service';
	import { playerService } from 'ui-lib/services/player.service';
	import { themeService } from 'ui-lib/services/theme.service';
	import { setBrowserImageCacheResolver } from 'ui-lib/services/image-cache.service';
	import { cachedImageUrl } from '$lib/image-cache';
	import { onMount, onDestroy } from 'svelte';
	import { base } from '$app/paths';
	import { NAV_ITEMS, type NavItem } from '$lib/generated/nav';

	setBrowserImageCacheResolver(cachedImageUrl);

	let { children } = $props();

	const playbackState = firkinPlaybackService.state;
	const playerState = playerService.state;

	const triggerClass = (item: NavItem) =>
		classNames('btn btn-outline btn-sm', { 'btn-disabled': !item.hasOwnPage });

	onMount(() => {
		themeService.initialize('flix');
		playerService.initialize();
	});

	onDestroy(() => {
		playerService.destroy();
	});
</script>

<div class="flex h-screen flex-col">
	<Navbar brand={{ label: 'Mhaol', highlight: 'Cloud' }} classes="!bg-base-300">
		{#snippet center()}
			<div class="flex flex-wrap items-center gap-1">
				{#each NAV_ITEMS as item (item.href)}
					{#if item.children.length === 0}
						<a href="{base}{item.href}" class="btn btn-outline btn-sm">{item.label}</a>
					{:else}
						<div class="dropdown-hover dropdown dropdown-bottom">
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
			<div class="flex items-center gap-1">
				<ThemeToggle />
			</div>
		{/snippet}
	</Navbar>

	<main class="flex min-w-0 flex-1 overflow-hidden">
		<div class="relative flex min-w-0 flex-1 flex-col overflow-y-auto">
			{@render children?.()}
		</div>
		<aside
			class="flex w-96 shrink-0 flex-col gap-2 overflow-y-auto border-l border-base-300 bg-base-200 p-2"
		>
			{#if $playbackState.firkin}
				<FirkinFilesPanel onPlayFile={(file) => firkinStreamService.play(file)} />
			{/if}
			<PlayerVideo
				file={$playerState.currentFile}
				connectionState={$playerState.connectionState}
				positionSecs={$playerState.positionSecs}
				durationSecs={$playerState.durationSecs}
				buffering={$playerState.buffering}
				poster={$playerState.currentFile?.thumbnailUrl}
				directStreamUrl={$playerState.directStreamUrl}
			/>
			<SubsLyricsFinder />
		</aside>
	</main>
</div>

<ToastOutlet />
