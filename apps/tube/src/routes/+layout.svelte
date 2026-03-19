<script lang="ts">
	import { onMount } from 'svelte';
	import '../css/app.css';
	import 'frontend/services/i18n';
	import { themeService } from 'frontend/services/theme.service';
	import { youtubeService } from 'frontend/services/youtube.service';
	import { youtubeLibraryService } from 'frontend/services/youtube-library.service';
	import { modalRouterService } from 'frontend/services/modal-router.service';
	import Navbar from 'ui-lib/components/core/Navbar.svelte';
	import ModalOutlet from 'ui-lib/components/core/ModalOutlet.svelte';
	import RightPanel from 'ui-lib/components/youtube/RightPanel.svelte';
	import TubeSettingsContent from 'ui-lib/components/settings/TubeSettingsContent.svelte';
	import DiskContent from 'ui-lib/components/settings/DiskContent.svelte';
	import YouTubeDownloadQueue from 'ui-lib/components/youtube/YouTubeDownloadQueue.svelte';

	let { children } = $props();

	const themeStore = themeService.store;
	const ytState = youtubeService.state;

	const ACTIVE_STATES = ['pending', 'fetching', 'downloading', 'muxing'];
	let activeCount = $derived(
		$ytState.downloads.filter((d: { state: string }) => ACTIVE_STATES.includes(d.state)).length
	);

	const modals = {
		settings: { component: TubeSettingsContent, maxWidth: 'max-w-2xl' },
		disk: { component: DiskContent, maxWidth: 'max-w-4xl' },
		'download-queue': { component: YouTubeDownloadQueue, maxWidth: 'max-w-lg' }
	};

	onMount(() => {
		youtubeService.initialize();
		youtubeLibraryService.initialize();
	});

	$effect(() => {
		document.documentElement.setAttribute('data-theme', $themeStore.theme);
	});
</script>

<div class="flex h-screen flex-col overflow-hidden">
	<Navbar brand={{ label: 'Mhaol', highlight: 'Tube' }}>
		{#snippet children()}
			<div class="flex items-center gap-2">
				<button
					class="btn px-4 btn-ghost btn-sm"
					onclick={() => modalRouterService.openNavbar('settings')}
					aria-label="Settings"
					title="Settings"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-5 w-5"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
						/>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
						/>
					</svg>
				</button>
				<button
					class="btn px-4 btn-ghost btn-sm"
					onclick={() => modalRouterService.openNavbar('disk')}
					aria-label="Disk usage"
					title="Disk usage"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-5 w-5"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4"
						/>
					</svg>
				</button>
				<div class="indicator">
					{#if activeCount > 0}
						<span class="indicator-item badge badge-xs badge-primary">{activeCount}</span>
					{/if}
					<button
						class="btn px-4 btn-ghost btn-sm"
						onclick={() => modalRouterService.openNavbar('download-queue')}
						aria-label="Download queue"
						title="Download queue"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-5 w-5"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
							/>
						</svg>
					</button>
				</div>
			</div>
		{/snippet}
	</Navbar>
	<ModalOutlet {modals} />
	<div class="flex min-h-0 flex-1">
		<main class="min-w-0 flex-1 overflow-y-auto bg-base-300">
			{@render children?.()}
		</main>
		<RightPanel />
	</div>
</div>
