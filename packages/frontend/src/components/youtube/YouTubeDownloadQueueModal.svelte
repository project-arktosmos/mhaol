<script lang="ts">
	import YouTubeDownloadQueue from '$components/youtube/YouTubeDownloadQueue.svelte';

	interface Props {
		open: boolean;
		onClose: () => void;
	}

	let { open, onClose }: Props = $props();

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) onClose();
	}
</script>

{#if open}
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		onclick={handleBackdropClick}
	>
		<div class="mx-4 flex max-h-[80vh] w-full max-w-lg flex-col">
			<div class="mb-3 flex items-center justify-between">
				<h2 class="text-lg font-semibold text-base-content">Download Queue</h2>
				<button class="btn btn-circle btn-ghost btn-sm" onclick={onClose} aria-label="Close">
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
							d="M6 18L18 6M6 6l12 12"
						/>
					</svg>
				</button>
			</div>
			<div class="overflow-y-auto">
				<YouTubeDownloadQueue />
			</div>
		</div>
	</div>
{/if}
