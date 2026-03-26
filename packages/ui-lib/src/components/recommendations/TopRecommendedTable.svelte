<script lang="ts">
	import type { TopRecommendedMovie } from 'ui-lib/types/recommendations.type';
	import { recommendationsService } from 'ui-lib/services/recommendations.service';
	import classNames from 'classnames';

	interface Props {
		mediaType?: string;
		selectedIndex?: number | null;
		onrowclick?: (index: number, movie: TopRecommendedMovie) => void;
	}

	let { mediaType, selectedIndex = null, onrowclick }: Props = $props();

	let movies = $state<TopRecommendedMovie[]>([]);
	let loading = $state(false);

	export async function refresh() {
		loading = true;
		try {
			movies = await recommendationsService.getTopMovies(mediaType);
		} catch {
			/* best-effort */
		} finally {
			loading = false;
		}
	}

	export function getMovies(): TopRecommendedMovie[] {
		return movies;
	}

	export function isLoading(): boolean {
		return loading;
	}
</script>

{#if loading && movies.length === 0}
	<div class="flex justify-center py-4">
		<span class="loading loading-sm loading-spinner"></span>
	</div>
{:else if movies.length === 0}
	<p class="py-4 text-center text-xs text-base-content/50">No data yet</p>
{:else}
	{@const lvls = movies[0]?.levels ?? []}
	<table class="table table-xs">
		<thead>
			<tr>
				<th>#</th>
				<th>Title</th>
				{#each lvls as lvl}
					<th class="text-center" colspan="2">L{lvl}</th>
				{/each}
				<th class="text-center">Score</th>
			</tr>
		</thead>
		<tbody>
			{#each movies as movie, i (movie.tmdbId)}
				<tr
					class={classNames({
						'cursor-pointer': !!onrowclick,
						'bg-primary/20': selectedIndex === i,
						'hover:bg-base-200': selectedIndex !== i && !!onrowclick
					})}
					onclick={() => onrowclick?.(i, movie)}
				>
					<td class="text-base-content/40">{i + 1}</td>
					<td class="max-w-48 truncate">{movie.title ?? '—'}</td>
					{#each lvls as lvl}
						{@const cnt = movie.levelCounts[lvl] ?? 0}
						{@const pct = movie.levelPercentages[lvl] ?? 0}
						<td class="text-center">{cnt || ''}</td>
						<td class="text-center text-base-content/40">
							{cnt ? `${pct}%` : ''}
						</td>
					{/each}
					<td class="text-center font-semibold">{movie.score}</td>
				</tr>
			{/each}
		</tbody>
	</table>
{/if}
