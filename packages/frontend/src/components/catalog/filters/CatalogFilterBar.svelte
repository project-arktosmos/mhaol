<script lang="ts">
	import classNames from 'classnames';
	import type { CatalogBrowseState } from '$types/catalog.type';
	import type { FilterKind } from '$data/media-registry';

	interface Props {
		filterKind: FilterKind;
		browseState: CatalogBrowseState;
		onfilter: (filterId: string, value: string) => void;
	}

	let { filterKind, browseState, onfilter }: Props = $props();
</script>

{#if filterKind === 'genre-buttons'}
	{#if browseState.filterOptions.genre}
		<div class="flex flex-wrap gap-1">
			{#each browseState.filterOptions.genre as option}
				<button
					class={classNames('btn btn-xs', {
						'btn-primary': (browseState.filters.genre || '') === option.id,
						'btn-ghost': (browseState.filters.genre || '') !== option.id
					})}
					onclick={() => onfilter('genre', option.id)}
				>
					{option.label}
				</button>
			{/each}
		</div>
	{/if}
{/if}
