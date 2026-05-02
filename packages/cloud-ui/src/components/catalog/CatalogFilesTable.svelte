<script lang="ts">
	import classNames from 'classnames';
	import type { FirkinFile } from '../../types/firkin.js';

	interface Props {
		files: FirkinFile[];
		collapsible?: boolean;
		open?: boolean;
		onToggle?: () => void;
	}
	let { files, collapsible = false, open = true, onToggle }: Props = $props();

	const heading = $derived(`Files (${files.length})`);
</script>

<div class="flex flex-col gap-2">
	<div class="flex items-center justify-between gap-2">
		{#if collapsible}
			<button
				type="button"
				class="flex flex-1 items-center gap-2 text-left"
				onclick={() => onToggle?.()}
				aria-expanded={open}
			>
				<span class="text-base-content/60" aria-hidden="true">{open ? '▼' : '▶'}</span>
				<h2 class="text-sm font-semibold text-base-content/70 uppercase">{heading}</h2>
			</button>
		{:else}
			<h2 class="text-sm font-semibold text-base-content/70 uppercase">{heading}</h2>
		{/if}
	</div>
	{#if !collapsible || open}
		<div class="mt-2">
			{#if files.length === 0}
				<p class="text-sm text-base-content/60">No files attached.</p>
			{:else}
				<div class="overflow-x-auto rounded-box border border-base-content/10">
					<table class="table table-sm">
						<thead>
							<tr>
								<th class="w-24">Type</th>
								<th>Title</th>
								<th>Value</th>
							</tr>
						</thead>
						<tbody>
							{#each files as file, i (i)}
								<tr>
									<td class={classNames('text-xs font-semibold')}>
										<span class="badge badge-outline badge-sm">{file.type}</span>
									</td>
									<td class="text-xs [overflow-wrap:anywhere]">
										{file.title ?? ''}
									</td>
									<td class="font-mono text-xs break-all">{file.value}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/if}
		</div>
	{/if}
</div>
