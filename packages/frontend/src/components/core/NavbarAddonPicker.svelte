<script lang="ts">
	import classNames from 'classnames';
	import { onMount } from 'svelte';
	import { base } from '$app/paths';
	import { page } from '$app/state';
	import { listSources, type CatalogSource } from '$lib/catalog.service';

	let { classes = 'flex flex-wrap items-center gap-1' }: { classes?: string } = $props();

	let sources = $state<CatalogSource[]>([]);

	onMount(() => {
		void (async () => {
			try {
				sources = await listSources();
			} catch {
				sources = [];
			}
		})();
	});

	const rootPath = `${base}/`;
	const isOnRoot = $derived(page.url.pathname === rootPath);
	const addonParam = $derived(page.url.searchParams.get('addon') ?? '');
	const allActive = $derived(isOnRoot && (addonParam === '' || addonParam === 'all'));
	const activeAddon = $derived.by(() => {
		if (!isOnRoot || allActive) return null;
		return sources.some((s) => s.id === addonParam) ? addonParam : null;
	});
</script>

{#if sources.length > 0}
	<div class={classes}>
		<a
			href={rootPath}
			data-sveltekit-noscroll
			class={classNames('btn btn-sm', {
				'btn-primary': allActive,
				'btn-outline': !allActive
			})}
			title="All addons"
		>
			All
		</a>
		{#each sources as source (source.id)}
			{@const active = activeAddon === source.id}
			<a
				href="{rootPath}?addon={encodeURIComponent(source.id)}"
				data-sveltekit-noscroll
				class={classNames('btn btn-sm', {
					'btn-primary': active,
					'btn-outline': !active
				})}
				title={source.kind}
			>
				{source.label}
			</a>
		{/each}
	</div>
{/if}
