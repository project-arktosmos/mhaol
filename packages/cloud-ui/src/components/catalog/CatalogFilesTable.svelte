<script lang="ts">
	import classNames from 'classnames';
	import type { FirkinFile } from '../../types/firkin.js';

	interface Props {
		files: FirkinFile[];
	}
	let { files }: Props = $props();
</script>

<div class="card border border-base-content/10 bg-base-200 p-4">
	<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">
		Files ({files.length})
	</h2>
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
