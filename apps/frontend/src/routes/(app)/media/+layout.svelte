<script lang="ts">
	import { setContext } from 'svelte';
	import { page } from '$app/stores';
	import classNames from 'classnames';
	import { MEDIA_BAR_KEY, type MediaBarContext } from 'ui-lib/types/media-bar.type';

	let title = $state('');
	let count = $state<number | null>(null);
	let countLabel = $state('items');
	let controlsTarget: HTMLDivElement | undefined = $state();
	let tabsTarget: HTMLDivElement | undefined = $state();
	let filterBarTarget: HTMLDivElement | undefined = $state();

	let prevUrl = $state($page.url.pathname);

	$effect(() => {
		const current = $page.url.pathname;
		if (current !== prevUrl) {
			prevUrl = current;
			title = '';
			count = null;
			countLabel = 'items';
		}
	});

	setContext<MediaBarContext>(MEDIA_BAR_KEY, {
		configure(config) {
			title = config.title;
			count = config.count ?? null;
			countLabel = config.countLabel ?? 'items';
		},
		get controlsTarget() {
			return controlsTarget;
		},
		get tabsTarget() {
			return tabsTarget;
		},
		get filterBarTarget() {
			return filterBarTarget;
		}
	});

	let { children } = $props();
</script>

<div class="flex h-full flex-col">
	<div
		class={classNames(
			'flex flex-wrap items-center gap-3 border-b border-base-300 px-4 py-3',
			{ hidden: !title }
		)}
	>
		<h1 class="text-lg font-bold">{title}</h1>
		{#if count !== null}
			<span class="badge badge-ghost">{count} {countLabel}</span>
		{/if}
		<div bind:this={controlsTarget} class="ml-auto flex items-center gap-2"></div>
	</div>
	<div bind:this={tabsTarget}></div>
	<div bind:this={filterBarTarget}></div>
	<div class="min-w-0 flex-1 overflow-y-auto">
		{@render children()}
	</div>
</div>
