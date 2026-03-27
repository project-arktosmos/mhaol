<script lang="ts">
	import type { TopRecommendedGame } from 'ui-lib/types/game-recommendations.type';
	import { gameRecommendationsService } from 'ui-lib/services/game-recommendations.service';
	import classNames from 'classnames';

	interface Props {
		selectedIndex?: number | null;
		labelMap?: Map<string, string>;
		onrowclick?: (index: number, game: TopRecommendedGame) => void;
	}

	let { selectedIndex = null, labelMap, onrowclick }: Props = $props();

	let games = $state<TopRecommendedGame[]>([]);
	let loading = $state(false);

	export async function refresh() {
		loading = true;
		try {
			games = await gameRecommendationsService.getTop();
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

{#if loading && games.length === 0}
	<div class="flex justify-center py-4">
		<span class="loading loading-sm loading-spinner"></span>
	</div>
{:else if games.length === 0}
	<p class="py-4 text-center text-xs text-base-content/50">No data yet</p>
{:else}
	{@const lvls = games[0]?.levels ?? []}
	<table class="table table-xs">
		<thead>
			<tr>
				<th>#</th>
				<th>Game</th>
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
			{#each games as game, i (game.gameId)}
				<tr
					class={classNames({
						'cursor-pointer': !!onrowclick,
						'bg-primary/20': selectedIndex === i,
						'hover:bg-base-200': selectedIndex !== i && !!onrowclick
					})}
					onclick={() => onrowclick?.(i, game)}
				>
					<td class="text-base-content/40">{i + 1}</td>
					<td class="max-w-48 truncate">{game.title ?? '—'}</td>
					{#if labelMap}
						<td class="text-center">{labelMap.get(String(game.gameId)) ?? ''}</td>
					{/if}
					{#each lvls as lvl}
						{@const cnt = game.levelCounts[lvl] ?? 0}
						{@const pct = game.levelPercentages[lvl] ?? 0}
						<td class="text-center">{cnt || ''}</td>
						<td class="text-center text-base-content/40">
							{cnt ? `${pct}%` : ''}
						</td>
					{/each}
					<td class="text-center font-semibold">{game.score}</td>
				</tr>
			{/each}
		</tbody>
	</table>
{/if}
