<script lang="ts">
	import classNames from 'classnames';
	import { createEventDispatcher } from 'svelte';
	import type { TaggerStatus } from '$services/image-tagger.service';

	interface Props {
		taggerReady: boolean;
		taggerInitializing: boolean;
		taggerStatus: TaggerStatus;
		taggerProgress: number;
		taggerError: string | null;
		taggingCount: number;
		totalImages: number;
		untaggedCount: number;
	}

	let {
		taggerReady,
		taggerInitializing,
		taggerStatus,
		taggerProgress,
		taggerError,
		taggingCount,
		totalImages,
		untaggedCount
	}: Props = $props();

	const dispatch = createEventDispatcher<{
		tagAll: void;
		tagUntagged: void;
	}>();

	let statusLabel = $derived.by(() => {
		if (taggerError) return `Error: ${taggerError}`;
		switch (taggerStatus) {
			case 'downloading':
				return `Downloading model... ${taggerProgress}%`;
			case 'loading':
				return 'Loading model into memory...';
			case 'ready':
				return 'SigLIP Ready';
			default:
				return 'SigLIP — loads on first tag';
		}
	});

	let badgeVariant = $derived.by(() => {
		if (taggerError) return 'badge-error';
		switch (taggerStatus) {
			case 'downloading':
			case 'loading':
				return 'badge-warning';
			case 'ready':
				return 'badge-success';
			default:
				return 'badge-ghost';
		}
	});

	let showProgress = $derived(taggerStatus === 'downloading' || taggerStatus === 'loading');
</script>

<div class="flex flex-col gap-3">
	<div class="flex flex-wrap items-center gap-4">
		<div class="flex items-center gap-2">
			<span class={classNames('badge badge-sm', badgeVariant)}>
				{#if taggerStatus === 'downloading' || taggerStatus === 'loading'}
					<span class="loading loading-spinner loading-xs mr-1"></span>
				{/if}
				{statusLabel}
			</span>
		</div>

		{#if totalImages > 0}
			<div class="flex gap-2">
				{#if untaggedCount > 0}
					<button
						class="btn btn-primary btn-sm"
						disabled={taggingCount > 0}
						onclick={() => dispatch('tagUntagged')}
					>
						{#if taggingCount > 0}
							<span class="loading loading-spinner loading-xs"></span>
							Tagging ({taggingCount})...
						{:else}
							Tag Untagged ({untaggedCount})
						{/if}
					</button>
				{/if}
				<button
					class="btn btn-secondary btn-sm"
					disabled={taggingCount > 0}
					onclick={() => dispatch('tagAll')}
				>
					Re-tag All ({totalImages})
				</button>
			</div>
		{/if}
	</div>

	{#if showProgress}
		<div class="w-full max-w-md">
			<progress
				class="progress progress-warning w-full"
				value={taggerProgress}
				max="100"
			></progress>
		</div>
	{/if}
</div>
