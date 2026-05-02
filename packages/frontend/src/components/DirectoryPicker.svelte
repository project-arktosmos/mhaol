<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import { browseDirectory, type BrowseResponse } from '$lib/fs-browse.service';

	interface Props {
		value: string;
		disabled?: boolean;
		onChange: (path: string) => void;
	}

	let { value, disabled = false, onChange }: Props = $props();

	let browsing = $state<BrowseResponse | null>(null);
	let loading = $state(false);
	let error = $state<string | null>(null);

	onMount(() => {
		void load(value && value.trim() !== '' ? value : undefined);
	});

	async function load(path?: string) {
		loading = true;
		error = null;
		try {
			browsing = await browseDirectory(path);
			onChange(browsing.path);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			loading = false;
		}
	}

	function breadcrumb(path: string, separator: string): { label: string; path: string }[] {
		if (!path) return [];
		const isWindows = separator === '\\';
		if (isWindows) {
			const drive = /^([A-Za-z]:)/.exec(path)?.[1];
			const rest = drive ? path.slice(drive.length) : path;
			const parts = rest.split(/[\\/]+/).filter(Boolean);
			const out: { label: string; path: string }[] = [];
			if (drive) out.push({ label: `${drive}\\`, path: `${drive}\\` });
			let acc = drive ? `${drive}\\` : '';
			for (const p of parts) {
				acc = acc.endsWith('\\') ? `${acc}${p}` : `${acc}\\${p}`;
				out.push({ label: p, path: acc });
			}
			return out;
		}
		const parts = path.split('/').filter(Boolean);
		const out: { label: string; path: string }[] = [{ label: '/', path: '/' }];
		let acc = '';
		for (const p of parts) {
			acc = `${acc}/${p}`;
			out.push({ label: p, path: acc });
		}
		return out;
	}

	const crumbs = $derived(browsing ? breadcrumb(browsing.path, browsing.separator) : []);
</script>

<div class="flex flex-col gap-2 rounded-box border border-base-content/10 bg-base-100 p-3">
	<div class="flex items-center gap-2 text-xs">
		<button
			type="button"
			class="btn btn-ghost btn-xs"
			onclick={() => load(browsing?.home)}
			disabled={disabled || loading}
			title="Home"
		>
			Home
		</button>
		{#if browsing?.parent}
			<button
				type="button"
				class="btn btn-ghost btn-xs"
				onclick={() => load(browsing?.parent ?? undefined)}
				disabled={disabled || loading}
				title="Parent"
			>
				Up
			</button>
		{/if}
		<button
			type="button"
			class="btn btn-ghost btn-xs"
			onclick={() => load(browsing?.path)}
			disabled={disabled || loading}
			title="Refresh"
		>
			Refresh
		</button>
		{#if browsing && browsing.roots.length > 1}
			<div class="dropdown dropdown-end ml-auto">
				<button type="button" tabindex="0" class="btn btn-ghost btn-xs">Drives</button>
				<ul class="dropdown-content menu z-10 w-32 rounded-box bg-base-200 p-2 shadow">
					{#each browsing.roots as root (root.path)}
						<li>
							<button type="button" onclick={() => load(root.path)}>{root.name}</button>
						</li>
					{/each}
				</ul>
			</div>
		{/if}
	</div>

	{#if browsing}
		<div class="flex flex-wrap items-center gap-1 font-mono text-xs">
			{#each crumbs as crumb, i (crumb.path)}
				{#if i > 0 && browsing.separator !== '\\'}
					<span class="text-base-content/40">/</span>
				{:else if i > 0}
					<span class="text-base-content/40">\</span>
				{/if}
				<button
					type="button"
					class="rounded px-1 hover:bg-base-200"
					onclick={() => load(crumb.path)}
					disabled={disabled || loading}
				>
					{crumb.label}
				</button>
			{/each}
		</div>
	{/if}

	{#if error}
		<p class="text-sm text-error">{error}</p>
	{/if}

	<div class="max-h-64 overflow-y-auto rounded border border-base-content/10 bg-base-200">
		{#if loading && !browsing}
			<p class="p-3 text-sm text-base-content/60">Loading…</p>
		{:else if browsing && browsing.entries.length === 0}
			<p class="p-3 text-sm text-base-content/60">No subfolders here.</p>
		{:else if browsing}
			<ul class="divide-y divide-base-content/10">
				{#each browsing.entries as entry (entry.path)}
					<li>
						<button
							type="button"
							class={classNames(
								'flex w-full items-center gap-2 px-3 py-1.5 text-left text-sm hover:bg-base-300',
								{ 'opacity-50': disabled || loading }
							)}
							onclick={() => load(entry.path)}
							disabled={disabled || loading}
						>
							<span aria-hidden="true">📁</span>
							<span class="font-mono">{entry.name}</span>
						</button>
					</li>
				{/each}
			</ul>
		{/if}
	</div>

	<p class="text-xs text-base-content/60">
		Selected: <span class="font-mono">{value || '—'}</span>
	</p>
</div>
