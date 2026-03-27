<script lang="ts">
	import type { TopRecommendedBook } from 'ui-lib/types/book-recommendations.type';
	import { bookRecommendationsService } from 'ui-lib/services/book-recommendations.service';
	import classNames from 'classnames';

	interface Props {
		selectedIndex?: number | null;
		labelMap?: Map<string, string>;
		onrowclick?: (index: number, book: TopRecommendedBook) => void;
	}

	let { selectedIndex = null, labelMap, onrowclick }: Props = $props();

	let books = $state<TopRecommendedBook[]>([]);
	let loading = $state(false);

	export async function refresh() {
		loading = true;
		try {
			books = await bookRecommendationsService.getTop();
		} catch {
			/* best-effort */
		} finally {
			loading = false;
		}
	}

	export function isLoading(): boolean {
		return loading;
	}
</script>

{#if loading && books.length === 0}
	<div class="flex justify-center py-4">
		<span class="loading loading-sm loading-spinner"></span>
	</div>
{:else if books.length === 0}
	<p class="py-4 text-center text-xs text-base-content/50">No data yet</p>
{:else}
	{@const lvls = books[0]?.levels ?? []}
	<table class="table table-xs">
		<thead>
			<tr>
				<th>#</th>
				<th>Book</th>
				{#if labelMap}
					<th class="text-center"></th>
				{/if}
				{#each lvls as lvl}
					<th class="text-center" colspan="2">L{lvl}</th>
				{/each}
				<th class="text-center">Score</th>
			</tr>
		</thead>
		<tbody>
			{#each books as book, i (book.key)}
				<tr
					class={classNames({
						'cursor-pointer': !!onrowclick,
						'bg-primary/20': selectedIndex === i,
						'hover:bg-base-200': selectedIndex !== i && !!onrowclick
					})}
					onclick={() => onrowclick?.(i, book)}
				>
					<td class="text-base-content/40">{i + 1}</td>
					<td class="max-w-48 truncate">{book.title ?? '—'}</td>
					{#if labelMap}
						<td class="text-center">{labelMap.get(book.key) ?? ''}</td>
					{/if}
					{#each lvls as lvl}
						{@const cnt = book.levelCounts[lvl] ?? 0}
						{@const pct = book.levelPercentages[lvl] ?? 0}
						<td class="text-center">{cnt || ''}</td>
						<td class="text-center text-base-content/40">
							{cnt ? `${pct}%` : ''}
						</td>
					{/each}
					<td class="text-center font-semibold">{book.score}</td>
				</tr>
			{/each}
		</tbody>
	</table>
{/if}
