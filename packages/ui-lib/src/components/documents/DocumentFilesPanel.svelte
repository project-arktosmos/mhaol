<script lang="ts">
	import { documentPlaybackService } from 'ui-lib/services/document-playback.service';

	const state = documentPlaybackService.state;
</script>

{#if $state.document}
	<section class="flex flex-col gap-2 rounded-box border border-base-content/10 bg-base-300 p-3">
		<header class="flex items-center justify-between gap-2">
			<h3 class="truncate text-sm font-semibold" title={$state.document.title}>
				{$state.document.title}
			</h3>
			<button
				type="button"
				class="btn btn-ghost btn-xs"
				onclick={() => documentPlaybackService.clear()}
				aria-label="Close playback panel"
			>
				×
			</button>
		</header>
		{#if $state.files.length === 0}
			<p class="text-xs text-base-content/60">No IPFS files for this document yet.</p>
		{:else}
			<ul class="flex flex-col gap-1">
				{#each $state.files as file (file.value)}
					<li
						class="flex flex-col gap-0.5 rounded border border-base-content/10 bg-base-100 px-2 py-1"
					>
						<span
							class="truncate text-xs font-medium"
							title={file.title ?? file.value}
						>
							{file.title ?? file.value}
						</span>
						<span
							class="truncate font-mono text-[10px] text-base-content/60"
							title={file.value}
						>
							{file.value}
						</span>
					</li>
				{/each}
			</ul>
		{/if}
	</section>
{/if}
