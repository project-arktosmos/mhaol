<script lang="ts">
	import classNames from 'classnames';
	import type { CatalogBrowseState } from 'ui-lib/types/catalog.type';
	import type { FilterKind } from 'ui-lib/data/media-registry';

	interface ConsoleEntry {
		id: number;
		name: string;
	}

	interface Props {
		filterKind: FilterKind;
		browseState: CatalogBrowseState;
		onfilter: (filterId: string, value: string) => void;
		consoles?: ConsoleEntry[];
		consoleWasmStatus?: Record<number, string>;
		consoleImages?: Record<number, string>;
		selectedConsoleId?: number;
		onconsolechange?: (consoleId: number) => void;
	}

	let {
		filterKind,
		browseState,
		onfilter,
		consoles,
		consoleWasmStatus,
		consoleImages,
		selectedConsoleId,
		onconsolechange
	}: Props = $props();
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
{:else if filterKind === 'console-selector'}
	{#if consoles && consoleWasmStatus && consoleImages && onconsolechange}
		<div class="grid grid-cols-3 gap-3 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-9">
			{#each consoles as console}
				<button
					class={classNames(
						'relative flex flex-col items-center gap-1.5 rounded-lg p-2 transition-colors',
						{
							'bg-primary/15 ring-2 ring-primary': selectedConsoleId === console.id,
							'hover:bg-base-200': selectedConsoleId !== console.id
						}
					)}
					onclick={() => onconsolechange(console.id)}
				>
					<span
						class={classNames('absolute top-1 right-1 h-2 w-2 rounded-full', {
							'bg-success': consoleWasmStatus[console.id] === 'yes',
							'bg-warning': consoleWasmStatus[console.id] === 'experimental',
							'bg-error': consoleWasmStatus[console.id] === 'no'
						})}
						title={consoleWasmStatus[console.id] === 'yes'
							? 'WASM emulator available'
							: consoleWasmStatus[console.id] === 'experimental'
								? 'WASM emulator (experimental)'
								: 'No WASM emulator'}
					></span>
					{#if consoleImages[console.id]}
						<img
							src={consoleImages[console.id]}
							alt={console.name}
							class="h-10 w-10 object-contain"
						/>
					{:else}
						<div
							class="flex h-10 w-10 items-center justify-center rounded bg-base-300 text-xs text-base-content/50"
						>
							?
						</div>
					{/if}
					<span class="text-center text-xs leading-tight font-medium">{console.name}</span>
				</button>
			{/each}
		</div>
	{/if}
{/if}
